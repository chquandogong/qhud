fn main() {
    // tauri::generate_context! embeds ../ui at compile time, but as a
    // proc-macro it does not register those files with cargo's change
    // tracking — without this line, editing ui/* silently ships stale
    // assets in the next build.
    println!("cargo:rerun-if-changed=../ui");
    tauri_build::build()
}
