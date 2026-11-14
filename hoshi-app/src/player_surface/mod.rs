#[cfg(target_os = "windows")]
pub mod windows;

#[cfg(target_os = "windows")]
pub use windows::{attach, EmbedWid};

#[cfg(target_os = "linux")]
pub mod linux;

#[cfg(target_os = "linux")]
pub use linux::attach;
#[cfg(any(target_os = "linux", target_os = "android", target_os = "ios", target_os = "macos"))]
pub struct PlayerReadyNotify(pub std::sync::Arc<tokio::sync::Notify>);
#[cfg(target_os = "android")]
pub mod android;

#[cfg(target_os = "android")]
pub use android::{attach, flush_pending_surface};

#[cfg(target_os = "ios")]
pub mod ios;

#[cfg(target_os = "ios")]
pub use ios::{attach, flush_pending_surface};

#[cfg(mobile)]
pub mod plugin;

#[cfg(target_os = "macos")]
pub mod macos;
#[cfg(target_os = "macos")]
pub use macos::attach;
