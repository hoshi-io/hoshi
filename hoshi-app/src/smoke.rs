//! TEMPORARY: CI smoke-test hooks (.github/workflows/smoke.yml). Inert unless the env vars
//! are set. Delete this file and its call sites once macOS/iOS are verified.
//!
//! HOSHI_SMOKE_VIDEO=<url>   frontend opens /watch/smoke/1 and plays <url>
//! HOSHI_SMOKE_HEADLESS=1    runs a burst of headless fetches and logs the outcome

use std::sync::Arc;
use std::time::Duration;

use hoshi_core::headless::{HeadlessBrowser, HeadlessOptions};
use hoshi_core::playback::PlaybackEvent;
use serde::Serialize;
use tracing::{error, info};

#[derive(Serialize)]
pub struct SmokeConfig {
    video: Option<String>,
}

#[tauri::command]
pub fn smoke_config() -> SmokeConfig {
    SmokeConfig { video: std::env::var("HOSHI_SMOKE_VIDEO").ok() }
}

pub fn spawn(state: Arc<hoshi_core::AppState>, headless: Arc<dyn HeadlessBrowser>) {
    if std::env::var_os("HOSHI_SMOKE_VIDEO").is_some() {
        let mut rx = state.playback.subscribe_events();
        tauri::async_runtime::spawn(async move {
            let mut last_logged = -10.0;
            while let Ok(ev) = rx.recv().await {
                match ev {
                    PlaybackEvent::Position(p) if p - last_logged >= 2.0 => {
                        info!(position = p, "smoke playback");
                        last_logged = p;
                    }
                    PlaybackEvent::Error(e) => error!(error = %e, "smoke playback error"),
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
                        Ok(r) => info!(n, title = ?r.result, html_len = r.html.len(), "smoke headless ok"),
                        Err(e) => error!(n, error = ?e, "smoke headless failed"),
                    }
                }
            };

            // Back to back first, then two at once: the overlap is where races show up.
            for n in 1..=5 {
                fetch(n).await;
            }
            tokio::join!(fetch(6), fetch(7));
            info!("smoke headless done");
        });
    }
}
