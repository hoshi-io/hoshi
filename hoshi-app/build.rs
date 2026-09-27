fn main() {
    tauri_build::build();

    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("windows") {
        let mpv_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("mpv-windows");
        println!("cargo:rustc-link-search=native={}", mpv_dir.display());
    }
}