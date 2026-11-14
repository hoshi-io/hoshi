//! TEMPORARY: CI smoke-test hooks (.github/workflows/smoke.yml). Inert unless the env vars
//! are set. Delete this file and its call sites once macOS/iOS are verified.
//!
//! HOSHI_SMOKE_VIDEO=<url>   frontend opens /watch/smoke/1 and plays <url>
//! HOSHI_SMOKE_HEADLESS=1    runs a burst of headless fetches and logs the outcome

use std::io::Write;
use std::path::PathBuf;
use std::sync::{Arc, OnceLock};
use std::time::Duration;

use hoshi_core::headless::{HeadlessBrowser, HeadlessOptions};
use hoshi_core::playback::PlaybackEvent;
use serde::Serialize;
use tracing::info;

// Results also go to <app data>/smoke.log: iOS truncates chunked stdout, so the log
// stream alone can't be trusted. CI pulls this file out of the app container.
static LOG_PATH: OnceLock<PathBuf> = OnceLock::new();

fn record(line: String) {
    info!("{line}");
    if let Some(path) = LOG_PATH.get() {
        if let Ok(mut f) = std::fs::OpenOptions::new().create(true).append(true).open(path) {
            let _ = writeln!(f, "{line}");
        }
    }
}

/// Lets the frontend report its side of the smoke test into the same file.
#[tauri::command]
pub fn smoke_log(msg: String) {
    record(format!("smoke frontend {msg}"));
}

#[derive(Serialize)]
pub struct SmokeConfig {
    video: Option<String>,
}

#[tauri::command]
pub fn smoke_config() -> SmokeConfig {
    SmokeConfig { video: std::env::var("HOSHI_SMOKE_VIDEO").ok() }
}

pub fn spawn(state: Arc<hoshi_core::AppState>, headless: Arc<dyn HeadlessBrowser>, data_dir: PathBuf) {
    if std::env::var_os("HOSHI_SMOKE_VIDEO").is_none() && std::env::var_os("HOSHI_SMOKE_HEADLESS").is_none() {
        return;
    }
    let _ = std::fs::create_dir_all(&data_dir);
    let path = data_dir.join("smoke.log");
    let _ = std::fs::remove_file(&path);
    let _ = LOG_PATH.set(path);
    record("smoke started".into());

    if std::env::var_os("HOSHI_SMOKE_VIDEO").is_some() {
        let mut rx = state.playback.subscribe_events();
        tauri::async_runtime::spawn(async move {
            let mut last_logged = -10.0;
            while let Ok(ev) = rx.recv().await {
                match ev {
                    PlaybackEvent::Position(p) if p - last_logged >= 2.0 => {
                        record(format!("smoke playback position={p:.1}"));
                        last_logged = p;
                    }
                    PlaybackEvent::Error(e) => record(format!("smoke playback error {e}")),
                    _ => {}
                }
            }
        });
    }

    if std::env::var_os("HOSHI_SMOKE_HEADLESS").is_some() {
        tauri::async_runtime::spawn(async move {
            // Let the main webview finish loading; mobile results come back through it.
            tokio::time::sleep(Duration::from_secs(10)).await;

            let fetch = |n: usize| {
                let headless = headless.clone();
                async move {
                    let options = HeadlessOptions {
                        javascript: Some("document.title".into()),
                        ..Default::default()
                    };
                    match headless.fetch("https://example.com/", options).await {
                        Ok(r) => record(format!("smoke headless ok n={n} title={:?} html_len={}", r.result, r.html.len())),
                        Err(e) => record(format!("smoke headless failed n={n} error={e:?}")),
                    }
                }
            };

            // Back to back first, then two at once: the overlap is where races show up.
            for n in 1..=5 {
                fetch(n).await;
            }
            tokio::join!(fetch(6), fetch(7));
            record("smoke headless done".into());
        });
    }
}
