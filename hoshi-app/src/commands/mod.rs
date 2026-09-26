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
pub mod playback;
pub mod dev;
#[cfg(feature = "discord-rpc")]
pub mod discord;

pub fn generate_handlers() -> impl Fn(tauri::ipc::Invoke) -> bool {
    tauri::generate_handler![
        i18n::load_locale,
        logs::get_system_logs, logs::list_log_files, logs::get_log_file, logs::delete_log_file,
        auth::login, auth::register, auth::logout, auth::get_current_profile,
        users::get_all_users, users::get_me, users::update_me, users::delete_me, users::change_password, users::upload_avatar, users::delete_avatar,
        content::merge_content, content::search_local_content, content::list_episode_servers, content::get_trending, content::get_home_content, content::get_content, content::get_content_by_cid, content::update_content, content::search, content::get_relation_tree, content::get_content_items, content::play_content_by_number, content::add_tracker_mapping, content::add_extension_source, content::update_extension_mapping, content::update_tracker_mapping, content::delete_tracker_mapping, content::search_extension,
        schedule::get_schedule,
        list::get_list, list::get_single_entry, list::upsert_entry, list::delete_entry, list::get_stats, list::get_entry_history, list::get_activity_feed,
        extensions::get_extensions, extensions::get_extension_filters, extensions::get_extension_settings, extensions::install_extension, extensions::install_sora_extension, extensions::install_lnreader_extension, extensions::update_extension, extensions::uninstall_extension, extensions::update_extension_settings,
        config::get_user_config, config::patch_user_config,
        progress::get_content_progress, progress::get_continue_watching, progress::update_anime_progress, progress::update_chapter_progress,
        intergations::list_trackers, intergations::add_integration, intergations::remove_integration, intergations::set_sync_enabled,
        playback::initialize_player, playback::shutdown_player, playback::load_stream, playback::toggle_pause, playback::set_paused, playback::set_volume, playback::get_volume, playback::set_muted, playback::seek, playback::get_position, playback::get_duration, playback::set_speed, playback::get_chapters, playback::set_chapter, playback::get_tracks, playback::set_audio_track, playback::set_video_track, playback::set_subtitle_track, playback::stop_playback, playback::set_lang_preferences, playback::set_player_options,
        dev::list_dev_extensions, dev::create_dev_extension, dev::read_extension_source, dev::write_extension_source, dev::read_manifest_raw, dev::write_manifest_raw, dev::run_extension_function, dev::delete_dev_extension,

        #[cfg(feature = "discord-rpc")]
        discord::set_activity,
        #[cfg(feature = "discord-rpc")]
        discord::clear_activity,
        #[cfg(mobile)]
        crate::headless::headless_plugin::notify_done,
        #[cfg(mobile)]
        crate::orientation::orientation_plugin::lock_orientation,
        #[cfg(mobile)]
        crate::orientation::orientation_plugin::unlock_orientation,
        #[cfg(mobile)]
        crate::orientation::orientation_plugin::get_current::orientation,
        #[cfg(mobile)]
        crate::intent::intent_plugin::launch_intent,
        #[cfg(mobile)]
        crate::immersive::immersive_plugin::enter_fullscreen,
        #[cfg(mobile)]
        crate::immersive::immersive_plugin::exit_fullscreen,
    ]
}