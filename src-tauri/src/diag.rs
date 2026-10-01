//! Typed, redacted error categories for ambient diagnostics (D-021, FR-18).
//!
//! Library errors are untrusted text. A `reqwest::Error` prints its request
//! URL, an `io::Error` can carry an OS message about a private path, a
//! `serde_json::Error` quotes the unexpected value, and qmonster's mux errors
//! wrap tmux/herdr stderr. None of that `Display` text may reach normal stderr
//! or a UI error string. Instead every ambient failure is reduced to:
//!
//! - a `&'static str` context chosen by qhud (never a runtime string), and
//! - one [`ErrorKind`] from a fixed allowlist, plus integer metadata
//!   (an HTTP status, a JSON line/column, a count) where that helps.
//!
//! Explicit operator-requested JSON commands (`--dump`, `--codex-usage`, ...)
//! print their payloads on stdout and are not routed through this module.

use std::fmt;

/// The complete allowlist of ambient error categories. Adding a variant is a
/// contract change: update the runbook's category table in all languages.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ErrorKind {
    /// A deadline elapsed (request, process exchange, OS timeout).
    Timeout,
    /// The connection could not be made or broke mid-exchange.
    Network,
    /// The server rejected the credential (HTTP 401/403).
    Authentication,
    /// The server asked the client to slow down (HTTP 429).
    RateLimited,
    /// A file or directory does not exist (or a binary is not on `PATH`).
    NotFound,
    /// The OS denied access to a file, directory, or binary.
    PermissionDenied,
    /// Any other local filesystem or stream I/O failure.
    Io,
    /// A child process could not be started, or its command exchange failed.
    Spawn,
    /// A service answered but is not usable right now (no server, HTTP 5xx,
    /// a mux command that exited non-zero).
    Unavailable,
    /// The peer answered outside the expected protocol (unexpected status,
    /// redirect, RPC error).
    Protocol,
    /// A body or file could not be decoded into the expected shape.
    Parse,
    /// Local configuration is invalid or could not be applied.
    Configuration,
    /// A desktop or OS facility (tray, opener, window system) failed.
    Platform,
}

impl ErrorKind {
    pub fn label(self) -> &'static str {
        match self {
            Self::Timeout => "timeout",
            Self::Network => "network",
            Self::Authentication => "authentication",
            Self::RateLimited => "rate-limited",
            Self::NotFound => "not-found",
            Self::PermissionDenied => "permission-denied",
            Self::Io => "io",
            Self::Spawn => "spawn",
            Self::Unavailable => "unavailable",
            Self::Protocol => "protocol",
            Self::Parse => "parse",
            Self::Configuration => "configuration",
            Self::Platform => "platform",
        }
    }
}

impl fmt::Display for ErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.label())
    }
}

/// `"<context> failed (<kind>)"`. The context is `'static` on purpose: it
/// cannot be built from a path, URL, label, or upstream message at runtime.
pub fn failure(context: &'static str, kind: ErrorKind) -> String {
    format!("{context} failed ({kind})")
}

/// Filesystem and stream errors. Only the OS error *kind* is read; the
/// message (which can name a private path) is discarded.
pub fn io(error: &std::io::Error) -> ErrorKind {
    use std::io::ErrorKind as Io;
    match error.kind() {
        Io::NotFound => ErrorKind::NotFound,
        Io::PermissionDenied => ErrorKind::PermissionDenied,
        Io::TimedOut | Io::WouldBlock => ErrorKind::Timeout,
        Io::ConnectionRefused
        | Io::ConnectionReset
        | Io::ConnectionAborted
        | Io::NotConnected
        | Io::AddrNotAvailable
        | Io::BrokenPipe
        | Io::UnexpectedEof => ErrorKind::Network,
        Io::InvalidData => ErrorKind::Parse,
        _ => ErrorKind::Io,
    }
}

/// Process-start errors: a missing binary or denied execution keep their
/// precise kind; everything else is `spawn`.
pub fn spawn(error: &std::io::Error) -> ErrorKind {
    match io(error) {
        kind @ (ErrorKind::NotFound | ErrorKind::PermissionDenied | ErrorKind::Timeout) => kind,
        _ => ErrorKind::Spawn,
    }
}

/// HTTP status classes. The numeric status itself is safe to print beside it.
pub fn http_status(status: u16) -> ErrorKind {
    match status {
        401 | 403 => ErrorKind::Authentication,
        429 => ErrorKind::RateLimited,
        408 | 504 => ErrorKind::Timeout,
        500..=599 => ErrorKind::Unavailable,
        _ => ErrorKind::Protocol,
    }
}

/// HTTP client errors. `reqwest::Error`'s `Display` includes the request
/// URL, so only its predicates and status code are consulted.
pub fn reqwest(error: &reqwest::Error) -> ErrorKind {
    if error.is_timeout() {
        ErrorKind::Timeout
    } else if error.is_builder() {
        ErrorKind::Configuration
    } else if error.is_connect() {
        ErrorKind::Network
    } else if let Some(status) = error.status() {
        http_status(status.as_u16())
    } else if error.is_redirect() {
        ErrorKind::Protocol
    } else if error.is_decode() {
        ErrorKind::Parse
    } else {
        // is_request / is_body and anything future: the exchange broke.
        ErrorKind::Network
    }
}

/// A rejected JSON document, reduced to a coarse category and position.
/// Serde type errors can quote the unexpected value, and provider bodies
/// carry account UUIDs and emails, so the raw error text is never kept.
pub fn json(error: &serde_json::Error) -> String {
    let category = match error.classify() {
        serde_json::error::Category::Io => "I/O error",
        serde_json::error::Category::Syntax => "JSON syntax error",
        serde_json::error::Category::Data => "schema mismatch",
        serde_json::error::Category::Eof => "incomplete JSON",
    };
    format!(
        "{category} at line {}, column {}",
        error.line(),
        error.column()
    )
}

/// qmonster mux errors wrap tmux/herdr stderr and pane targets, so only the
/// variant is read.
pub fn mux(error: &qmonster::tmux::PollingError) -> ErrorKind {
    use qmonster::tmux::PollingError;
    match error {
        PollingError::Command(_) => ErrorKind::Spawn,
        PollingError::NonZero(_) => ErrorKind::Unavailable,
    }
}

/// Mux source construction returns `anyhow::Error` whose text embeds the
/// underlying failure. Only typed causes in the chain are inspected.
pub fn mux_build(error: &(dyn std::error::Error + 'static)) -> ErrorKind {
    let mut cause: Option<&(dyn std::error::Error + 'static)> = Some(error);
    while let Some(current) = cause {
        if let Some(polling) = current.downcast_ref::<qmonster::tmux::PollingError>() {
            return mux(polling);
        }
        if let Some(io_error) = current.downcast_ref::<std::io::Error>() {
            return spawn(io_error);
        }
        cause = current.source();
    }
    ErrorKind::Unavailable
}

/// qmonster configuration load errors: I/O keeps its OS kind; TOML parse
/// errors (which quote the offending line) become `configuration`.
pub fn qmonster_config(error: &qmonster::app::config::ConfigError) -> ErrorKind {
    use qmonster::app::config::ConfigError;
    match error {
        ConfigError::Io(io_error) => io(io_error),
        ConfigError::Parse(_) | ConfigError::Serialize(_) => ErrorKind::Configuration,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Values that must never survive any formatter in this module: an
    /// email, token-like text, a POSIX and a Windows path, an account and
    /// workspace ID, and a provider-controlled pane label.
    const CANARIES: &[&str] = &[
        "person@example.com",
        "sk-ant-oat01-CANARYTOKEN",
        "eyJhbGciOiJIUzI1NiJ9.canary",
        "/home/person/.codex/auth.json",
        "C:\\Users\\person\\.claude",
        "acct-7f3c9d2e",
        "workspace-123",
        "pane:%42 claude@work",
    ];

    fn canary_text() -> String {
        CANARIES.join(" ")
    }

    fn assert_clean(formatted: &str) {
        for canary in CANARIES {
            assert!(
                !formatted.contains(canary),
                "{canary:?} survived: {formatted}"
            );
        }
        assert!(
            !formatted.contains('@'),
            "email marker survived: {formatted}"
        );
        assert!(
            !formatted.contains('/'),
            "path marker survived: {formatted}"
        );
        assert!(
            !formatted.contains('\\'),
            "path marker survived: {formatted}"
        );
    }

    #[test]
    fn every_category_label_is_a_fixed_lowercase_token() {
        let all = [
            ErrorKind::Timeout,
            ErrorKind::Network,
            ErrorKind::Authentication,
            ErrorKind::RateLimited,
            ErrorKind::NotFound,
            ErrorKind::PermissionDenied,
            ErrorKind::Io,
            ErrorKind::Spawn,
            ErrorKind::Unavailable,
            ErrorKind::Protocol,
            ErrorKind::Parse,
            ErrorKind::Configuration,
            ErrorKind::Platform,
        ];
        for kind in all {
            let label = kind.label();
            assert!(!label.is_empty());
            assert!(
                label.chars().all(|c| c.is_ascii_lowercase() || c == '-'),
                "{label}"
            );
            assert_eq!(kind.to_string(), label);
        }
    }

    #[test]
    fn io_errors_keep_only_their_os_kind() {
        use std::io::{Error, ErrorKind as Io};
        let cases = [
            (Io::NotFound, ErrorKind::NotFound),
            (Io::PermissionDenied, ErrorKind::PermissionDenied),
            (Io::TimedOut, ErrorKind::Timeout),
            (Io::ConnectionRefused, ErrorKind::Network),
            (Io::InvalidData, ErrorKind::Parse),
            (Io::Other, ErrorKind::Io),
        ];
        for (os_kind, expected) in cases {
            let error = Error::new(os_kind, canary_text());
            assert!(error.to_string().contains("person@example.com"));
            assert_eq!(io(&error), expected);
            assert_clean(&failure("write store", io(&error)));
        }
    }

    #[test]
    fn spawn_errors_name_the_kind_not_the_binary() {
        let missing =
            std::process::Command::new("/nonexistent/person@example.com/sk-ant-oat01-CANARYTOKEN")
                .spawn()
                .expect_err("a binary under /nonexistent cannot start");
        assert_eq!(spawn(&missing), ErrorKind::NotFound);
        assert_clean(&failure("codex app-server spawn", spawn(&missing)));

        let other = std::io::Error::other(canary_text());
        assert_eq!(spawn(&other), ErrorKind::Spawn);
    }

    #[test]
    fn http_statuses_map_to_classes() {
        assert_eq!(http_status(401), ErrorKind::Authentication);
        assert_eq!(http_status(403), ErrorKind::Authentication);
        assert_eq!(http_status(429), ErrorKind::RateLimited);
        assert_eq!(http_status(504), ErrorKind::Timeout);
        assert_eq!(http_status(503), ErrorKind::Unavailable);
        assert_eq!(http_status(404), ErrorKind::Protocol);
    }

    fn canary_url(port: u16) -> String {
        // Userinfo, a token-like path segment, and an account query: all
        // places a reqwest Display string would echo back.
        format!(
            "https://127.0.0.1:{port}/person@example.com/sk-ant-oat01-CANARYTOKEN\
             ?account=acct-7f3c9d2e&workspace=workspace-123"
        )
    }

    #[test]
    fn reqwest_builder_errors_drop_the_url() {
        let error = reqwest::Client::new()
            .get("https://person@example.com:not-a-port/sk-ant-oat01-CANARYTOKEN")
            .build()
            .expect_err("a non-numeric port is not a URL");
        assert_eq!(reqwest(&error), ErrorKind::Configuration);
        assert_clean(&failure("request", reqwest(&error)));
    }

    #[test]
    fn reqwest_transport_errors_drop_the_url() {
        // Loopback only: a port that was just released refuses the
        // connection, and a listener that never answers times out. No
        // provider endpoint is contacted.
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();
        runtime.block_on(async {
            // Windows retries a refused loopback SYN for about two seconds
            // before reporting it, so the refusal case gets a generous
            // deadline; only the silent listener should time out.
            let patient = reqwest::Client::builder()
                .timeout(std::time::Duration::from_secs(15))
                .build()
                .unwrap();
            let client = reqwest::Client::builder()
                .timeout(std::time::Duration::from_millis(200))
                .build()
                .unwrap();

            let closed = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
            let closed_port = closed.local_addr().unwrap().port();
            drop(closed);
            let refused = patient
                .get(canary_url(closed_port))
                .send()
                .await
                .expect_err("nothing listens on a released port");
            assert!(
                refused.to_string().contains("127.0.0.1"),
                "reqwest's own Display carries the URL: {refused}"
            );
            assert_eq!(reqwest(&refused), ErrorKind::Network);
            assert_clean(&failure("request", reqwest(&refused)));

            let silent = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
            let silent_port = silent.local_addr().unwrap().port();
            let timed_out = client
                .get(canary_url(silent_port))
                .send()
                .await
                .expect_err("a listener that never answers times out");
            assert_eq!(reqwest(&timed_out), ErrorKind::Timeout);
            assert_clean(&failure("request", reqwest(&timed_out)));
            drop(silent);
        });
    }

    #[test]
    fn json_errors_report_position_without_values() {
        let body = format!(r#"{{"accessToken": {{"nested": ["{}"]}}"#, canary_text());
        let syntax = serde_json::from_str::<serde_json::Value>(&body).unwrap_err();
        assert_clean(&json(&syntax));

        #[derive(Debug, serde::Deserialize)]
        #[allow(dead_code)]
        struct Shape {
            count: u32,
        }
        let data = serde_json::from_str::<Shape>(
            &serde_json::json!({ "count": canary_text() }).to_string(),
        )
        .unwrap_err();
        assert!(data.to_string().contains("person@example.com"));
        let formatted = json(&data);
        assert!(formatted.starts_with("schema mismatch at line 1, column "));
        assert_clean(&formatted);
    }

    #[test]
    fn mux_errors_drop_tmux_and_herdr_text() {
        use qmonster::tmux::PollingError;
        let command = PollingError::Command(canary_text());
        let non_zero = PollingError::NonZero(canary_text());
        assert_eq!(mux(&command), ErrorKind::Spawn);
        assert_eq!(mux(&non_zero), ErrorKind::Unavailable);
        assert_clean(&failure("herdr source", mux(&command)));

        // build_tmux_source returns anyhow::Error, whose Deref target is the
        // original error; a source chain stands in for it here.
        #[derive(Debug)]
        struct Wrapper(Option<Box<dyn std::error::Error + 'static>>, String);
        impl std::fmt::Display for Wrapper {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.write_str(&self.1)
            }
        }
        impl std::error::Error for Wrapper {
            fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
                self.0.as_deref()
            }
        }
        let wrapped = Wrapper(
            Some(Box::new(PollingError::NonZero(canary_text()))),
            canary_text(),
        );
        assert_eq!(mux_build(&wrapped), ErrorKind::Unavailable);
        let wrapped_io = Wrapper(
            Some(Box::new(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                canary_text(),
            ))),
            canary_text(),
        );
        assert_eq!(mux_build(&wrapped_io), ErrorKind::NotFound);
        let direct = PollingError::Command(canary_text());
        assert_eq!(mux_build(&direct), ErrorKind::Spawn);
        let opaque = Wrapper(None, canary_text());
        assert_eq!(mux_build(&opaque), ErrorKind::Unavailable);
        assert_clean(&failure("tmux source build", mux_build(&opaque)));
    }

    #[test]
    fn qmonster_config_errors_drop_the_offending_line() {
        use qmonster::app::config::ConfigError;
        let toml_error = toml::from_str::<toml::Value>(&format!("bad = \"{}", canary_text()))
            .expect_err("an unterminated string is invalid TOML");
        let parse = ConfigError::Parse(toml_error);
        assert_eq!(qmonster_config(&parse), ErrorKind::Configuration);
        assert_clean(&failure("qmonster config load", qmonster_config(&parse)));

        let io_error = ConfigError::Io(std::io::Error::new(
            std::io::ErrorKind::PermissionDenied,
            canary_text(),
        ));
        assert_eq!(qmonster_config(&io_error), ErrorKind::PermissionDenied);
    }
}
