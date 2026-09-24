//! Per-platform GL/webview compositing surface setup.
//!
//! Each platform gets its own module because the compositing mechanism is
//! platform-native (GTK widget tree on Linux, NSView layering on macOS,
//! DirectComposition on Windows, Android view hierarchy on Android) — there
//! is no cross-platform abstraction here by design, only a shared `attach`
//! entry point signature.

#[cfg(target_os = "linux")]
pub mod linux;

#[cfg(target_os = "linux")]
pub use linux::attach;

/// Wakes up a render-context creation attempt that's waiting on mpv to
/// finish initializing. `attach()` manages one of these via the app handle
/// so that `commands::playback::initialize_player` — which has no other way
/// to reach into the GTK-side render setup — can call `notify_one()` after
/// `state.playback.initialize()` succeeds.
///
/// This exists because the GL surface can realize (and attempt to build a
/// render context) before the webview ever calls `initialize_player` — the
/// two happen on completely independent timelines. Without this, that first
/// attempt fails once, nothing retries it, and video never renders even
/// after initialization later succeeds.
pub struct PlayerReadyNotify(pub std::sync::Arc<tokio::sync::Notify>);

// TODO: macos, windows, android — same `attach(window, state)` shape,
// different native compositing underneath.