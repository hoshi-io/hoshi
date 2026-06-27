use serde::{Deserialize, Serialize};
use crate::content::models::{ContentType, FullContent, RelationType};
use crate::tracker::provider::TrackerMedia;

#[derive(Debug, Clone, Deserialize)]
pub struct SearchParams {
    pub r#type: Option<String>,
    pub nsfw: Option<bool>,
    pub status: Option<String>,
    pub query: Option<String>,
    pub limit: Option<i32>,
    pub offset: Option<i32>,
    pub sort: Option<String>,
    pub genre: Option<String>,
    pub format: Option<String>,
    pub tracker: Option<String>,
    pub extension_filters: Option<String>,
    pub page: Option<u32>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ContentListResponse {
    pub data: Vec<TrackerMedia>,
    pub total: usize,
    pub limit: i32,
    pub offset: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AnimeSection {
    pub trending:          Vec<FullContent>,
    pub popular:           Vec<FullContent>,
    pub top_rated:         Vec<FullContent>,
    pub seasonal:          Vec<FullContent>,
    pub upcoming:          Vec<FullContent>,
    pub recently_finished: Vec<FullContent>,
    pub top_action:        Vec<FullContent>,
}

impl AnimeSection {
    pub fn filter_nsfw(&mut self) {
        self.trending.retain(|m| !m.content.nsfw);
        self.popular.retain(|m| !m.content.nsfw);
        self.top_rated.retain(|m| !m.content.nsfw);
        self.seasonal.retain(|m| !m.content.nsfw);
        self.upcoming.retain(|m| !m.content.nsfw);
        self.recently_finished.retain(|m| !m.content.nsfw);
        self.top_action.retain(|m| !m.content.nsfw);
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MangaSection {
    pub trending:          Vec<FullContent>,
    pub popular:           Vec<FullContent>,
    pub top_rated:         Vec<FullContent>,
    pub seasonal:          Vec<FullContent>,
    pub recently_finished: Vec<FullContent>,
}

impl MangaSection {
    pub fn filter_nsfw(&mut self) {
        self.trending.retain(|m| !m.content.nsfw);
        self.popular.retain(|m| !m.content.nsfw);
        self.top_rated.retain(|m| !m.content.nsfw);
        self.seasonal.retain(|m| !m.content.nsfw);
        self.recently_finished.retain(|m| !m.content.nsfw);
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NovelSection {
    pub trending:          Vec<FullContent>,
    pub popular:           Vec<FullContent>,
    pub top_rated:         Vec<FullContent>,
    pub recently_finished: Vec<FullContent>,
}

impl NovelSection {
    pub fn filter_nsfw(&mut self) {
        self.trending.retain(|m| !m.content.nsfw);
        self.popular.retain(|m| !m.content.nsfw);
        self.top_rated.retain(|m| !m.content.nsfw);
        self.recently_finished.retain(|m| !m.content.nsfw);
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HomeView {
    pub anime:     AnimeSection,
    pub manga:     MangaSection,
    pub novel:     NovelSection,
    pub cached_at: i64,
}

impl HomeView {
    pub fn filter_nsfw(&mut self) {
        self.anime.filter_nsfw();
        self.manga.filter_nsfw();
        self.novel.filter_nsfw();
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateExtensionMappingRequest {
    pub extension_name: String,
    pub extension_id: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateTrackerMappingRequest {
    pub tracker_name: String,
    pub tracker_id: String,
}

pub fn parse_content_type(t: &str) -> ContentType {
    match t {
        "manga" => ContentType::Manga,
        "novel" => ContentType::Novel,
        _       => ContentType::Anime,
    }
}

#[derive(Debug, Deserialize)]
pub struct AniSkipResponse {
    pub results: Vec<AniSkipResult>,
}

#[derive(Debug, Deserialize)]
pub struct AniSkipResult {
    pub interval: AniSkipInterval,
    #[serde(rename = "skipType")]
    pub skip_type: String,
}

#[derive(Debug, Deserialize)]
pub struct AniSkipInterval {
    #[serde(rename = "startTime")]
    pub start_time: f64,
    #[serde(rename = "endTime")]
    pub end_time: f64,
}

#[derive(Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct RelationNode {
    pub cid: String,
    pub title: String,
    pub cover_image: Option<String>,
}

#[derive(Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct RelationEdge {
    pub source_cid: String,
    pub target_cid: String,
    pub relation_type: RelationType,
}

#[derive(Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct RelationGraph {
    pub nodes: Vec<RelationNode>,
    pub edges: Vec<RelationEdge>,
}