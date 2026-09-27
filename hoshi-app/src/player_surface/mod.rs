#[cfg(target_os = "windows")]
pub mod windows;

#[cfg(target_os = "windows")]
pub use windows::{attach, EmbedWid};

#[cfg(target_os = "linux")]
pub mod linux;

#[cfg(target_os = "linux")]
pub use linux::attach;
#[cfg(any(target_os = "linux", target_os = "android"))]
pub struct PlayerReadyNotify(pub std::sync::Arc<tokio::sync::Notify>);
#[cfg(target_os = "android")]
pub mod android;
#[cfg(target_os = "android")]
pub mod android_plugin;

#[cfg(target_os = "android")]
pub use android::attach;