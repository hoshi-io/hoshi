use std::sync::Arc;

use hoshi_core::error::CoreError;
use hoshi_core::playback::{Chapter, EpisodeChapter, ExternalSubtitle, LoadMode, LoadSpec, NowPlaying, Track};
use hoshi_core::AppState;
use serde::Deserialize;
use tauri::State;

#[cfg(target_os = "linux")]
use crate::player_surface::PlayerReadyNotify;

#[tauri::command]
pub async fn initialize_player(
    state: State<'_, Arc<AppState>>,
    #[cfg(target_os = "linux")] ready_notify: State<'_, PlayerReadyNotify>,
) -> Result<(), CoreError> {
    state.playback.initialize().await?;
    #[cfg(target_os = "linux")]
    ready_notify.0.notify_one();
    Ok(())
}

#[tauri::command]
pub async fn shutdown_player(state: State<'_, Arc<AppState>>) -> Result<(), CoreError> {
    state.playback.shutdown().await
}

#[tauri::command]
pub async fn stop_playback(state: State<'_, Arc<AppState>>) -> Result<(), CoreError> {
    state.playback.stop().await
}

/// A single `Key: Value` HTTP header, as an object rather than a
/// `[key, value]` tuple so the JS side isn't juggling arrays-of-arrays.
#[derive(Deserialize)]
pub struct HeaderInput {
    pub key: String,
    pub value: String,
}

#[derive(Deserialize)]
pub struct SubtitleInput {
    pub url: String,
    pub title: Option<String>,
    pub lang: Option<String>,
}

#[derive(Deserialize)]
pub struct ChapterInput {
    pub start: f64,
    pub end: f64,
    pub title: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NowPlayingInput {
    pub cid: String,
    pub episode: i64,
    pub title: String,
    pub episode_title: String,
    pub cover_image: Option<String>,
    pub nsfw: bool,
    pub total_episodes: i64,
}

#[tauri::command]
pub async fn load_stream(
    state: State<'_, Arc<AppState>>,
    session: State<'_, crate::TauriSession>,
    url: String,
    mode: Option<String>,
    headers: Option<Vec<HeaderInput>>,
    subtitles: Option<Vec<SubtitleInput>>,
    chapters: Option<Vec<ChapterInput>>,
    start_position: Option<f64>,
    now_playing: Option<NowPlayingInput>,
) -> Result<(), CoreError> {
    let user_id = crate::require_auth(&session).await?;

    let mode = match mode.as_deref() {
        Some("append") => LoadMode::Append,
        Some("append-play") => LoadMode::AppendPlay,
        _ => LoadMode::Replace,
    };

    let spec = LoadSpec {
        url,
        headers: headers
            .unwrap_or_default()
            .into_iter()
            .map(|h| (h.key, h.value))
            .collect(),
        subtitles: subtitles
            .unwrap_or_default()
            .into_iter()
            .map(|s| ExternalSubtitle { url: s.url, title: s.title, lang: s.lang })
            .collect(),
        chapters: chapters
            .unwrap_or_default()
            .into_iter()
            .map(|c| EpisodeChapter { start: c.start, end: c.end, title: c.title })
            .collect(),
        start_position,
        now_playing: now_playing.map(|np| NowPlaying {
            cid: np.cid,
            episode: np.episode,
            title: np.title,
            episode_title: np.episode_title,
            cover_image: np.cover_image,
            nsfw: np.nsfw,
            total_episodes: np.total_episodes,
            user_id,
        }),
    };

    state.playback.load(spec, mode).await
}

#[tauri::command]
pub async fn set_lang_preferences(
    state: State<'_, Arc<AppState>>,
    sub_lang: Option<String>,
    dub_lang: Option<String>,
) -> Result<(), CoreError> {
    state.playback.set_lang_preferences(sub_lang, dub_lang).await
}

#[tauri::command]
pub async fn toggle_pause(state: State<'_, Arc<AppState>>) -> Result<bool, CoreError> {
    state.playback.toggle_pause().await
}

#[tauri::command]
pub async fn set_paused(state: State<'_, Arc<AppState>>, paused: bool) -> Result<(), CoreError> {
    state.playback.set_paused(paused).await
}

#[tauri::command]
pub async fn set_volume(state: State<'_, Arc<AppState>>, volume: f64) -> Result<(), CoreError> {
    state.playback.set_volume(volume).await
}

#[tauri::command]
pub async fn get_volume(state: State<'_, Arc<AppState>>) -> Result<f64, CoreError> {
    state.playback.volume().await
}

#[tauri::command]
pub async fn set_muted(state: State<'_, Arc<AppState>>, muted: bool) -> Result<(), CoreError> {
    state.playback.set_muted(muted).await
}

#[tauri::command]
pub async fn seek(state: State<'_, Arc<AppState>>, target: f64, relative: bool) -> Result<(), CoreError> {
    state.playback.seek(target, relative).await
}

#[tauri::command]
pub async fn get_position(state: State<'_, Arc<AppState>>) -> Result<f64, CoreError> {
    state.playback.position().await
}

#[tauri::command]
pub async fn get_duration(state: State<'_, Arc<AppState>>) -> Result<f64, CoreError> {
    state.playback.duration().await
}

#[tauri::command]
pub async fn set_speed(state: State<'_, Arc<AppState>>, speed: f64) -> Result<(), CoreError> {
    state.playback.set_speed(speed).await
}

#[tauri::command]
pub async fn get_chapters(state: State<'_, Arc<AppState>>) -> Result<Vec<Chapter>, CoreError> {
    state.playback.chapters().await
}

#[tauri::command]
pub async fn set_chapter(state: State<'_, Arc<AppState>>, index: i64) -> Result<(), CoreError> {
    state.playback.set_chapter(index).await
}

#[tauri::command]
pub async fn get_tracks(state: State<'_, Arc<AppState>>) -> Result<Vec<Track>, CoreError> {
    state.playback.tracks().await
}

#[tauri::command]
pub async fn set_audio_track(state: State<'_, Arc<AppState>>, id: Option<i64>) -> Result<(), CoreError> {
    state.playback.set_audio_track(id).await
}

#[tauri::command]
pub async fn set_video_track(state: State<'_, Arc<AppState>>, id: Option<i64>) -> Result<(), CoreError> {
    state.playback.set_video_track(id).await
}

#[tauri::command]
pub async fn set_subtitle_track(state: State<'_, Arc<AppState>>, id: Option<i64>) -> Result<(), CoreError> {
    state.playback.set_subtitle_track(id).await
}