use std::fmt::Write;
use std::sync::Once;

use libmpv2::Mpv;
use sysinfo::{CpuRefreshKind, System};
use tracing::info;

use crate::AppState;

/// `1.2.2 (a1b2c3d4e5)`, `-dirty` when core had uncommitted changes at build time.
pub fn core_version() -> String {
    format!("{} ({})", env!("CARGO_PKG_VERSION"), env!("HOSHI_CORE_COMMIT"))
}

/// The full block. `shell` lines come first (label, value).
pub fn report(state: &AppState, shell: &[(&str, String)]) -> String {
    let mut lines: Vec<(&str, String)> = shell.to_vec();
    lines.push(("core", core_version()));
    lines.extend(system_lines());
    lines.push(("headless", if state.extension_manager.headless_available() { "available" } else { "unavailable" }.into()));
    lines.push(("extensions", extensions_line(state)));

    let mut out = String::from("Diagnostics");
    for (label, value) in lines {
        let _ = write!(out, "\n  {label:<11} {value}");
    }
    out
}

pub fn log_startup(state: &AppState, shell: &[(&str, String)]) {
    info!("{}", report(state, shell));
}

fn system_lines() -> Vec<(&'static str, String)> {
    let unknown = || "unknown".to_string();
    let os = System::long_os_version().unwrap_or_else(|_| unknown());
    let kernel = System::kernel_version().unwrap_or_else(|_| unknown());
    let mut lines = vec![("os", format!("{os}, kernel {kernel}, {}", std::env::consts::ARCH))];

    #[cfg(target_os = "linux")]
    {
        let var = |k: &str| std::env::var(k).ok().filter(|v| !v.is_empty());
        let session = var("XDG_SESSION_TYPE").unwrap_or_else(unknown);
        let desktop = var("XDG_CURRENT_DESKTOP").unwrap_or_else(unknown);
        lines.push(("session", format!("{session}, {desktop}")));
    }

    match System::new() {
        Ok(mut sys) => {
            sys.refresh_cpu_list(CpuRefreshKind::nothing());
            sys.refresh_memory();
            let brand = sys.cpus().first().map(|c| c.brand().trim().to_string()).filter(|b| !b.is_empty());
            lines.push(("cpu", format!("{}, {} threads", brand.unwrap_or_else(unknown), sys.cpus().len())));
            lines.push(("memory", format!("{:.1} GiB", sys.total_memory() as f64 / (1u64 << 30) as f64)));
        }
        Err(_) => lines.push(("cpu", unknown())),
    }
    lines
}

fn extensions_line(state: &AppState) -> String {
    let mut exts: Vec<String> = state.extension_manager.list_extensions().into_iter()
        .map(|e| format!("{} {}", e.name, e.version))
        .collect();
    exts.sort();
    if exts.is_empty() {
        return "none".into();
    }
    format!("{}: {}", exts.len(), exts.join(", "))
}

/// mpv is loaded on first playback, so its versions are logged then, once per run.
pub(crate) fn log_mpv_once(mpv: &Mpv) {
    static ONCE: Once = Once::new();
    ONCE.call_once(|| {
        let get = |p: &str| mpv.get_property::<String>(p).unwrap_or_else(|_| "unknown".into());
        info!(mpv = %get("mpv-version"), ffmpeg = %get("ffmpeg-version"), "mpv loaded");
    });
}
