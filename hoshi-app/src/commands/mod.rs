pub mod auth;
pub mod users;
pub mod content;
pub mod extensions;
pub mod intergations;
pub mod schedule;
pub mod list;
pub mod config;
pub mod progress;
pub mod logs;
pub mod i18n;

#[cfg(not(mobile))]
pub mod mpv;
#[cfg(feature = "discord-rpc")]
pub mod discord;