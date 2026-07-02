pub mod anilist;
pub mod mal;
pub mod kitsu;
pub mod simkl;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use reqwest::Client;
use crate::content::models::{Character, ContentType, Metadata, StaffMember};
use crate::error::{CoreError, CoreResult};
use crate::schedule::types::AiringEpisode;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TrackerMedia {
    pub tracker_id: String,
    pub tracker_url: Option<String>,
    pub cross_ids: HashMap<String, String>,
    pub content_type: ContentType,
    pub title: String,
    pub alt_titles: Vec<String>,
    #[serde(default)]
    pub title_i18n: HashMap<String, String>,
    pub synopsis: Option<String>,
    pub cover_image: Option<String>,
    pub banner_image: Option<String>,
    pub episode_count: Option<i32>,
    pub chapter_count: Option<i32>,
    pub status: Option<String>,
    pub genres: Vec<String>,
    pub tags: Vec<String>,
    pub nsfw: bool,
    pub release_date: Option<String>,
    pub end_date: Option<String>,
    pub rating: Option<f32>,
    pub trailer_url: Option<String>,
    pub format: Option<String>,
    pub studio: Option<String>,
    pub characters: Vec<Character>,
    pub staff: Vec<StaffMember>,
    pub relations: Vec<TrackerRelation>,
    pub episode_duration: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrackerRelation {
    pub relation_type: String,
    pub media: TrackerMedia,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserListEntry {
    pub tracker_media_id: String,
    pub title: String,
    pub poster: Option<String>,
    pub content_type: ContentType,
    pub format: Option<String>,
    pub status: Option<String>,
    pub progress: i32,
    pub score: Option<f64>,
    pub start_date: Option<String>,
    pub end_date: Option<String>,
    pub repeat_count: i32,
    pub notes: Option<String>,
    pub is_private: bool,
    pub total_episodes: Option<i32>,
    pub total_chapters: Option<i32>,
    pub media: Option<TrackerMedia>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenData {
    pub access_token: String,
    pub refresh_token: Option<String>,
    pub token_type: String,
    pub expires_at: String,
    pub tracker_user_id: String,
    pub display_name:    Option<String>,
    pub avatar_url:      Option<String>,
    pub profile_url:     Option<String>,
    pub score_format: Option<String>
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateEntryParams {
    pub media_id: String,
    pub status: Option<String>,
    pub progress: Option<i32>,
    pub score: Option<f64>,
    pub start_date: Option<String>,
    pub end_date: Option<String>,
    pub repeat_count: Option<i32>,
    pub notes: Option<String>,
    pub is_private: Option<bool>,
    pub score_format: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TrackerAuthConfig {
    pub oauth_flow: String,
    pub auth_url: String,
    pub token_url: Option<String>,
    pub client_id: Option<String>,
    pub scopes: Vec<String>,
}

#[async_trait]
pub trait TrackerProvider: Send + Sync {
    fn name(&self) -> &'static str;
    fn display_name(&self) -> &'static str;
    fn icon_url(&self) -> &'static str;
    fn supported_types(&self) -> Vec<ContentType>;
    fn auth_config(&self) -> TrackerAuthConfig;

    async fn validate_and_store_token(
        &self,
        access_token: &str,
        token_type: &str,
    ) -> CoreResult<TokenData>;

    async fn search(
        &self,
        query: Option<&str>,
        content_type: ContentType,
        limit: usize,
        page: usize,
        sort: Option<&str>,
        genre: Option<&str>,
        format: Option<&str>,
        nsfw: Option<bool>,
        status: Option<&str>,
    ) -> CoreResult<Vec<TrackerMedia>>;

    async fn get_by_id(&self, tracker_id: &str) -> CoreResult<Option<TrackerMedia>>;

    async fn get_home(&self) -> CoreResult<HashMap<String, Vec<TrackerMedia>>> {
        Ok(HashMap::new())
    }

    async fn get_user_list(
        &self,
        access_token: &str,
        tracker_user_id: &str,
        score_format: Option<&str>,
    ) -> CoreResult<Vec<UserListEntry>>;

    async fn update_entry(
        &self,
        access_token: &str,
        params: UpdateEntryParams,
    ) -> CoreResult<()>;

    async fn delete_entry(
        &self,
        access_token: &str,
        media_id: &str,
    ) -> CoreResult<bool>;

    fn to_core_metadata(&self, cid: &str, media: &TrackerMedia) -> Metadata;

    async fn fetch_airing_schedule(
        &self,
        _from_ts: i64,
        _to_ts: i64,
    ) -> CoreResult<Vec<AiringEpisode>> {
        Err(CoreError::NotFound("Airing schedule not supported".into()))
    }

    async fn refresh_score_format(&self, _access_token: &str) -> CoreResult<Option<String>> {
        Ok(None)
    }
}

pub struct TrackerRegistry {
    providers: HashMap<String, Arc<dyn TrackerProvider>>,
}

impl TrackerRegistry {
    pub fn new() -> Self {
        Self { providers: HashMap::new() }
    }

    pub fn register(&mut self, provider: Arc<dyn TrackerProvider>) {
        self.providers.insert(provider.name().to_string(), provider);
    }

    pub fn get(&self, name: &str) -> Option<Arc<dyn TrackerProvider>> {
        self.providers.get(name).cloned()
    }

    pub fn all(&self) -> Vec<Arc<dyn TrackerProvider>> {
        self.providers.values().cloned().collect()
    }

    pub fn names(&self) -> Vec<String> {
        self.providers.keys().cloned().collect()
    }
}

pub fn build_registry(client: Client) -> TrackerRegistry {
    let mut registry = TrackerRegistry::new();

    registry.register(Arc::new(anilist::AniListProvider::new(client.clone())));
    registry.register(Arc::new(kitsu::KitsuProvider::new(client.clone())));
    registry.register(Arc::new(mal::MalProvider::new(client.clone())));
    registry.register(Arc::new(simkl::SimklProvider::new(client.clone())));
    registry
}