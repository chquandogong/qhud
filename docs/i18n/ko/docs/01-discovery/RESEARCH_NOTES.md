<!-- qhud:languages -->
<p align="center">
  <strong>한국어</strong> · <a href="../../../../01-discovery/RESEARCH_NOTES.md">English</a> · <a href="../../../zh-CN/docs/01-discovery/RESEARCH_NOTES.md">简体中文</a>
</p>
<!-- /qhud:languages -->

<!-- qhud:anchor -->
<a id="research_notes"></a>

# 조사 노트

> 상태: 완료 · 날짜: 2026-08-05 · 담당: chquandogong

<!-- qhud:anchor -->
<a id="window-layering-on-linux-desktops"></a>

## Linux 데스크톱의 창 계층

- GNOME Mutter의 layer-shell 프로토콜 요청은 **열린 상태이며 미구현**입니다. 타사 앱은 GNOME에서 네이티브 Wayland 위젯 계층을 사용할 수 없습니다: <https://gitlab.gnome.org/GNOME/mutter/-/work_items/973>
- `wlr-layer-shell` 프로토콜(wlroots, KWin, Smithay 등 GNOME 이외에서 구현): <https://wayland.app/protocols/wlr-layer-shell-unstable-v1>
- 따라서 GNOME에서는 Mutter가 따르는 XWayland EWMH 상태(`_NET_WM_STATE_BELOW` + `_NET_WM_STATE_STICKY`)를 통해서만 데스크톱 위젯 계층을 사용할 수 있습니다(실기 검증 — FEASIBILITY_REPORT).

<!-- qhud:anchor -->
<a id="tauri-capabilities"></a>

## Tauri 기능

- `always_on_bottom` 창 옵션 및 `set_always_on_bottom`: <https://github.com/tauri-apps/tauri/commit/c1ec0f155118527361dd5645d920becbc8afd569>
- 기반 tao 구현(Linux의 GTK `set_keep_below`): <https://github.com/tauri-apps/tao/pull/522>
- 창 위치/순서 관리는 **Wayland 백엔드에서 오류 없이 아무 동작도 하지 않습니다**. qhud가 `GDK_BACKEND=x11`을 강제하는 이유입니다: <https://github.com/tauri-apps/tauri/issues/14913>

<!-- qhud:anchor -->
<a id="qmonster-reuse-surface-rev-aa2bd39"></a>

## qmonster 재사용 범위 (rev aa2bd39)

- `qmonster::app::bootstrap::Context::new(config, source, notifier, sink)` — `PaneSource` + `NotifyBackend`에 대한 제네릭 공개 생성자입니다.
- `qmonster::store::sink::NoopSink` — qhud에 필요한 쓰기 없는 감사 sink가 이미 업스트림에 있습니다.
- `qmonster::app::event_loop::run_once_with_target` — 관찰 한 주기를 실행하고 `Vec<PaneReport>`를 반환합니다. qhud는 이를 위젯 JSON(schema v1)에 매핑합니다.
- `build.rs`는 `.git`이 없으면 `v{CARGO_PKG_VERSION}-nogit`를 사용하므로 cargo Git 의존성으로 안전하게 사용할 수 있습니다.
- 계기 의미: 압박 정도는 0..1 비율(`context_pressure`, `quota_5h_pressure`, `quota_weekly_pressure`)이며 초기화 시각은 Unix 초(`quota_*_resets_at`)입니다.
