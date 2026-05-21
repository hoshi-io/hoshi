use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use crate::content::models::ContentUnit;
use crate::impl_from_row;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AnimeProgress {
    pub id: i64,
    pub user_id: i32,
    pub cid: String,
    pub episode: i32,
    pub timestamp_seconds: i32,
    pub episode_duration_seconds: Option<i32>,
    pub completed: bool,
    pub last_accessed: i64,
}
impl_from_row!(AnimeProgress { id, user_id, cid, episode, timestamp_seconds, episode_duration_seconds, completed, last_accessed });

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ChapterProgress {
    pub id: i64,
    pub user_id: i32,
    pub cid: String,
    pub chapter: i32,
    pub completed: bool,
    pub last_accessed: i64,
}
impl_from_row!(ChapterProgress { id, user_id, cid, chapter, completed, last_accessed });

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateAnimeProgressBody {
    pub cid: String,
    pub episode: i32,
    pub timestamp_seconds: i32,
    pub episode_duration_seconds: Option<i32>,
    pub completed: Option<bool>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateChapterProgressBody {
    pub cid: String,
    pub chapter: i32,
    pub completed: Option<bool>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ContinueItem {
    pub cid: String,
    pub content_type: String,
    pub title: String,
    #[serde(default)]
    pub title_i18n: HashMap<String, String>,
    pub cover_image: Option<String>,
    pub nsfw: bool,
    pub episode: Option<i32>,
    pub timestamp_seconds: Option<i32>,
    pub episode_duration_seconds: Option<i32>,
    pub chapter: Option<i32>,
    pub unit: Option<ContentUnit>,
    pub last_accessed: i64,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ContinueWatchingResponse {
    pub items: Vec<ContinueItem>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProgressResponse {
    pub success: bool,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ContentProgressResponse {
    pub cid: String,
    pub anime_progress: Vec<AnimeProgress>,
    pub chapter_progress: Vec<ChapterProgress>,
}