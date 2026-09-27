//! Windows compositing "surface": mpv draws directly into the root
//! window's client area via `wid`, underneath WebView2, which is made
//! transparent so mpv's frame shows through wherever the page doesn't
//! paint.
//!
//! Unlike Linux, there's no render context, no supervisor loop, and no GL
//! surface to manage here — mpv owns its own D3D11 swapchain once
//! initialized. The only job `attach()` has is capturing the window's
//! HWND *before* the frontend ever calls `initialize_player`, since mpv
//! needs `wid` set prior to `mpv_initialize()` (see
//! `hoshi_core::playback::build_mpv`).

use tauri::Manager;
use tracing::info;

/// The root window's HWND, stashed during `attach()` so that
/// `commands::playback::initialize_player` can pass it to
/// `PlaybackHandle::initialize()` as mpv's `wid`.
pub struct EmbedWid(pub i64);

pub fn attach(window: &tauri::WebviewWindow) -> Result<(), Box<dyn std::error::Error>> {
    let hwnd = window.hwnd()?;
    let wid = hwnd.0 as isize as i64;

    window.app_handle().manage(EmbedWid(wid));

    // Only the webview needs to be see-through — the window itself stays
    // a normal opaque window (its `backgroundColor` config gives it a
    // solid black backdrop). A transparent webview reveals whatever this
    // *same* window has already painted (mpv's frame, or black before mpv
    // starts) rather than the desktop behind the whole app.
    if let Err(e) = window.with_webview(|webview| unsafe {
        use webview2_com::Microsoft::Web::WebView2::Win32::{
            ICoreWebView2Controller2, COREWEBVIEW2_COLOR,
        };
        use windows::core::Interface;

        let controller2: ICoreWebView2Controller2 = match webview.controller().cast() {
            Ok(c) => c,
            Err(e) => {
                tracing::warn!(error = ?e, "failed to cast to ICoreWebView2Controller2");
                return;
            }
        };

        if let Err(e) = controller2.SetDefaultBackgroundColor(COREWEBVIEW2_COLOR {
            R: 0,
            G: 0,
            B: 0,
            A: 0,
        }) {
            tracing::warn!(error = ?e, "failed to set webview transparent background");
        }
    }) {
        tracing::warn!(error = ?e, "with_webview failed");
    }

    info!(wid, "stashed root HWND for mpv wid-based embedding");
    Ok(())
}

