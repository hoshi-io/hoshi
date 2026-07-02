use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

use crate::content::models::{
    Character, ContentType, EpisodeData, Metadata, StaffMember, Status,
};
use crate::error::{CoreError, CoreResult};

use super::{
    TokenData, TrackerAuthConfig, TrackerMedia, TrackerProvider, UpdateEntryParams,
    UserListEntry,
};

const SIMKL_AUTHORIZE_URL: &str = "https://simkl.com/oauth/authorize";
const SIMKL_TOKEN_URL: &str = "https://api.simkl.com/oauth/token";
const SIMKL_API_BASE: &str = "https://api.simkl.com";
const SIMKL_CLIENT_ID: &str = "d8385263a0cd0e60acd779d9db61310f41c8f99e40571af596ef79c7de1d4b2e";
const SIMKL_APP_NAME: &str = "hoshi";
const SIMKL_APP_VERSION: &str = "1.1.1";

#[derive(Debug, Clone, Deserialize)]
struct SimklIds {
    #[serde(alias = "simkl_id", default)]
    simkl: Option<Value>,
    #[serde(default)]
    slug: Option<String>,
    #[serde(default)]
    imdb: Option<String>,
    #[serde(default)]
    tmdb: Option<Value>,
    #[serde(default)]
    tvdb: Option<Value>,
    #[serde(default)]
    mal: Option<Value>,
    #[serde(default)]
    anidb: Option<Value>,
}

impl SimklIds {
    fn helper_val_to_string(val: &Value) -> String {
        match val {
            Value::String(s) => s.clone(),
            Value::Number(n) => n.to_string(),
            other => {
                let s = other.to_string();
                if s == "null" { "".to_string() } else { s }
            }
        }
    }

    fn simkl_id_string(&self) -> String {
        self.simkl
            .as_ref()
            .map(Self::helper_val_to_string)
            .unwrap_or_default()
    }

    fn to_cross_ids(&self) -> HashMap<String, String> {
        let mut map = HashMap::new();
        let mut insert = |key: &str, val: &Option<Value>| {
            if let Some(v) = val {
                let s = Self::helper_val_to_string(v);
                if !s.is_empty() && s != "null" {
                    map.insert(key.to_string(), s);
                }
            }
        };

        insert("imdb", &self.imdb.clone().map(Value::String));
        insert("tmdb", &self.tmdb);
        insert("tvdb", &self.tvdb);
        insert("mal", &self.mal);
        insert("anidb", &self.anidb);
        map
    }
}

#[derive(Debug, Clone, Deserialize)]
struct SimklRatingBlock {
    #[serde(default)]
    rating: Option<f32>,
}

#[derive(Debug, Clone, Deserialize)]
struct SimklRatings {
    #[serde(default)]
    simkl: Option<SimklRatingBlock>,
}

#[derive(Debug, Clone, Deserialize)]
struct SimklStudio {
    #[serde(default)]
    name: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
struct SimklMediaObject {
    ids: SimklIds,
    title: String,
    #[serde(default)]
    poster: Option<String>,
    #[serde(default)]
    fanart: Option<String>,
    #[serde(default)]
    overview: Option<String>,
    #[serde(default)]
    genres: Option<Vec<String>>,
    #[serde(default)]
    status: Option<String>,
    #[serde(default)]
    runtime: Option<i32>,
    // "Originating TV station / streamer" per SIMKL's docs — NOT the
    // production studio. Kept only as a fallback for content types (e.g.
    // plain TV shows) that don't populate `studios`.
    #[serde(default)]
    network: Option<String>,
    // Actual production studios (e.g. "MAPPA", "Sunrise") live here.
    #[serde(default)]
    studios: Option<Vec<SimklStudio>>,
    #[serde(default)]
    ratings: Option<SimklRatings>,
    #[serde(default)]
    total_episodes: Option<i32>,
    #[serde(default)]
    anime_type: Option<String>, // tv, special, ova, movie, music video, ona
    #[serde(default)]
    en_title: Option<String>,
    #[serde(default)]
    first_aired: Option<String>,
    #[serde(default)]
    ended: Option<bool>,
    #[serde(default)]
    year: Option<i32>,
}

impl SimklMediaObject {
    fn into_tracker_media(self) -> TrackerMedia {
        let mut alt_titles = Vec::new();
        if let Some(en) = &self.en_title {
            if en != &self.title {
                alt_titles.push(en.clone());
            }
        }

        TrackerMedia {
            tracker_id: self.ids.simkl_id_string(),
            tracker_url: self
                .ids
                .slug
                .as_ref()
                .map(|slug| format!("https://simkl.com/anime/{slug}")),
            cross_ids: self.ids.to_cross_ids(),
            content_type: ContentType::Anime,
            title: self.title,
            alt_titles,
            title_i18n: HashMap::new(),
            synopsis: self.overview,
            cover_image: self
                .poster
                .map(|p| format!("https://simkl.in/posters/{p}_m.webp")),
            banner_image: self
                .fanart
                .map(|f| format!("https://simkl.in/fanart/{f}_w.webp")),
            episode_count: self.total_episodes,
            chapter_count: None,
            status: self.status,
            genres: self.genres.unwrap_or_default(),
            tags: Vec::new(),
            nsfw: false,
            release_date: self.first_aired.or_else(|| self.year.map(|y| y.to_string())),
            end_date: None,
            rating: self.ratings.and_then(|r| r.simkl).and_then(|s| s.rating),
            trailer_url: None,
            format: self.anime_type,
            studio: self
                .studios
                .as_ref()
                .and_then(|studios| studios.iter().find_map(|s| s.name.clone()))
                .or(self.network),
            characters: Vec::new(),
            staff: Vec::new(),
            relations: Vec::new(),
            episode_duration: self.runtime,
        }
    }
}

#[derive(Debug, Deserialize)]
struct SimklAllItemsResponse {
    #[serde(default)]
    anime: Vec<SimklListItem>,
}

#[derive(Debug, Deserialize)]
struct SimklListItem {
    #[serde(default)]
    status: Option<String>,
    #[serde(default)]
    user_rating: Option<i64>,
    #[serde(default)]
    last_watched_at: Option<String>,
    #[serde(default)]
    watched_episodes_count: Option<i32>,
    #[serde(default)]
    total_episodes_count: Option<i32>,
    #[serde(default)]
    anime_type: Option<String>,
    #[serde(default)]
    private: Option<bool>,
    // The media object for anime[] entries is nested under "show" per SIMKL's
    // docs (the same key used for TV shows) — NOT under "anime".
    show: Option<SimklMediaObject>,
}

impl SimklListItem {
    fn into_user_list_entry(self) -> Option<UserListEntry> {
        let media_obj = self.show?;
        let tracker_media_id = media_obj.ids.simkl_id_string();
        let title = media_obj.title.clone();
        let poster = media_obj
            .poster
            .clone()
            .map(|p| format!("https://simkl.in/posters/{p}_m.webp"));
        let total_episodes = media_obj.total_episodes;
        let mut media = media_obj.into_tracker_media();
        // anime_type lives on the list item itself for anime[] entries, not
        // inside the nested "show" object, so backfill it here.
        if media.format.is_none() {
            media.format = self.anime_type.clone();
        }

        Some(UserListEntry {
            tracker_media_id,
            title,
            poster,
            content_type: ContentType::Anime,
            format: media.format.clone(),
            status: self.status,
            progress: self.watched_episodes_count.unwrap_or(0),
            score: self.user_rating.map(|r| r as f64),
            start_date: None,
            end_date: self.last_watched_at,
            repeat_count: 0,
            notes: None,
            is_private: self.private.unwrap_or(false),
            total_episodes: self.total_episodes_count.or(total_episodes),
            total_chapters: None,
            media: Some(media),
        })
    }
}

#[derive(Debug, Serialize)]
struct SimklIdRefIds {
    simkl: String,
}

#[derive(Debug, Serialize)]
struct SimklIdRef {
    ids: SimklIdRefIds,
}

impl SimklIdRef {
    fn new(media_id: &str) -> Self {
        Self {
            ids: SimklIdRefIds {
                simkl: media_id.to_string(),
            },
        }
    }
}

#[derive(Debug, Serialize)]
struct SimklEpisodeRef {
    number: i32,
}

#[derive(Debug, Serialize)]
struct SimklHistoryShowEntry {
    ids: SimklIdRefIds,
    #[serde(skip_serializing_if = "Option::is_none")]
    status: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    rating: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    episodes: Option<Vec<SimklEpisodeRef>>,
}

#[derive(Debug, Serialize)]
struct SimklHistoryRequest {
    anime: Vec<SimklHistoryShowEntry>,
}

#[derive(Debug, Serialize)]
struct SimklRemoveRequest {
    anime: Vec<SimklIdRef>,
}

/// Normalizes the app's generic `Status` enum — serialized lowercase as
/// `planned | ongoing | completed | cancelled | hiatus` — into the exact
/// watchlist status strings SIMKL's API accepts:
/// `watching | plantowatch | hold | completed | dropped`.
/// Sending anything else makes `/sync/history` (and `/sync/add-to-list`)
/// return `400 wrong_parameter`.
fn normalize_simkl_status(status: &str) -> String {
    match status.to_ascii_lowercase().replace([' ', '-'], "_").as_str() {
        // App's generic Status enum (Planned/Ongoing/Completed/Cancelled/Hiatus)
        "planned" => "plantowatch",
        "ongoing" => "watching",
        "completed" => "completed",
        "cancelled" => "dropped",
        "hiatus" => "hold",
        // Defensive aliases in case a Simkl-native or other-tracker string
        // ever flows through directly.
        "watching" | "current" => "watching",
        "finished" => "completed",
        "plantowatch" | "plan_to_watch" | "planning" => "plantowatch",
        "hold" | "on_hold" | "paused" => "hold",
        "dropped" => "dropped",
        other => other,
    }
        .to_string()
}

fn map_simkl_status(status: &str) -> Option<Status> {
    match status.to_ascii_lowercase().as_str() {
        "tba" => Some(Status::Planned),
        "ended" => Some(Status::Completed),
        "airing" => Some(Status::Ongoing),
        _ => None,
    }
}

pub struct SimklProvider {
    client: Client,
}

impl SimklProvider {
    pub fn new(client: Client) -> Self {
        Self { client }
    }

    fn authenticated_request(
        &self,
        method: reqwest::Method,
        path: &str,
        access_token: &str,
    ) -> reqwest::RequestBuilder {
        self.client
            .request(method, format!("{SIMKL_API_BASE}{path}"))
            .header("Authorization", format!("Bearer {access_token}"))
            .header("simkl-api-key", SIMKL_CLIENT_ID)
            .header(
                "User-Agent",
                format!("{}/{}", SIMKL_APP_NAME, SIMKL_APP_VERSION),
            )
    }

    async fn post_sync<B: Serialize + ?Sized>(
        &self,
        access_token: &str,
        path: &str,
        body: &B,
    ) -> CoreResult<String> {
        let resp = self
            .authenticated_request(reqwest::Method::POST, path, access_token)
            .json(body)
            .send()
            .await
            .map_err(|e| CoreError::Network(e.to_string()))?;

        let status = resp.status();
        let raw = resp
            .text()
            .await
            .map_err(|e| CoreError::Network(e.to_string()))?;

        if !status.is_success() {
            return Err(CoreError::Network(format!(
                "SIMKL request to {path} failed ({status}): {raw}"
            )));
        }

        Ok(raw)
    }
}

#[async_trait]
impl TrackerProvider for SimklProvider {
    fn name(&self) -> &'static str {
        "simkl"
    }

    fn display_name(&self) -> &'static str {
        "SIMKL"
    }

    fn icon_url(&self) -> &'static str {
        "https://eu.simkl.in/img_favicon/v2/favicon-192x192.png"
    }

    fn supported_types(&self) -> Vec<ContentType> {
        vec![ContentType::Anime]
    }

    fn auth_config(&self) -> TrackerAuthConfig {
        TrackerAuthConfig {
            oauth_flow: "pkce".to_string(),
            auth_url: SIMKL_AUTHORIZE_URL.to_string(),
            token_url: Some(SIMKL_TOKEN_URL.to_string()),
            client_id: Some(SIMKL_CLIENT_ID.to_string()),
            scopes: vec![],
        }
    }

    async fn validate_and_store_token(
        &self,
        access_token: &str,
        token_type: &str,
    ) -> CoreResult<TokenData> {
        let settings_resp = self
            .authenticated_request(reqwest::Method::GET, "/users/settings", access_token)
            .send()
            .await
            .map_err(|e| CoreError::Network(e.to_string()))?;

        let settings_status = settings_resp.status();
        let settings_raw = settings_resp
            .text()
            .await
            .map_err(|e| CoreError::Network(e.to_string()))?;

        if !settings_status.is_success() {
            return Err(CoreError::AuthError(format!(
                "failed to validate SIMKL token ({settings_status}): {settings_raw}"
            )));
        }

        let settings: Value = serde_json::from_str(&settings_raw)
            .map_err(|e| CoreError::Parse(format!("failed to parse SIMKL user settings: {e}")))?;

        let account_id = settings
            .get("account")
            .and_then(|a| a.get("id"))
            .and_then(|v| v.as_i64().map(|n| n.to_string()).or_else(|| v.as_str().map(str::to_string)));

        let display_name = settings
            .get("user")
            .and_then(|u| u.get("name"))
            .and_then(|v| v.as_str())
            .map(str::to_string);

        let tracker_user_id = account_id
            .clone()
            .or_else(|| display_name.clone())
            .unwrap_or_default();

        let avatar_url = settings.get("user").and_then(|u| u.get("avatar")).and_then(|v| {
            v.as_str()
                .map(str::to_string)
                .or_else(|| v.get("icon").and_then(|i| i.as_str()).map(str::to_string))
                .or_else(|| v.get("full").and_then(|i| i.as_str()).map(str::to_string))
        });

        let profile_url = None;

        let expires_at = chrono::Utc::now() + chrono::Duration::days(365 * 5);

        Ok(TokenData {
            access_token: access_token.to_string(),
            refresh_token: None,
            token_type: token_type.to_string(),
            expires_at: expires_at.to_rfc3339(),
            tracker_user_id,
            display_name,
            avatar_url,
            profile_url,
            score_format: None,
        })
    }

    async fn search(
        &self,
        query: Option<&str>,
        _content_type: ContentType,
        limit: usize,
        page: usize,
        _sort: Option<&str>,
        _genre: Option<&str>,
        _format: Option<&str>,
        _nsfw: Option<bool>,
        _status: Option<&str>,
    ) -> CoreResult<Vec<TrackerMedia>> {
        let query = query.unwrap_or_default();

        let resp = self
            .client
            .get(format!("{SIMKL_API_BASE}/search/anime"))
            .query(&[("q", query), ("client_id", SIMKL_CLIENT_ID)])
            .header(
                "User-Agent",
                format!("{}/{}", SIMKL_APP_NAME, SIMKL_APP_VERSION),
            )
            .send()
            .await
            .map_err(|e| CoreError::Network(e.to_string()))?;

        let status = resp.status();
        let raw = resp
            .text()
            .await
            .map_err(|e| CoreError::Network(e.to_string()))?;

        if !status.is_success() {
            return Err(CoreError::Network(format!(
                "SIMKL search failed ({status}): {raw}"
            )));
        }

        let results: Vec<SimklMediaObject> = serde_json::from_str(&raw)
            .map_err(|e| CoreError::Parse(format!("failed to parse SIMKL search results: {e}")))?;

        let start = page.saturating_sub(1).saturating_mul(limit.max(1));
        let media = results
            .into_iter()
            .skip(start)
            .take(limit.max(1))
            .map(SimklMediaObject::into_tracker_media)
            .collect();

        Ok(media)
    }

    async fn get_by_id(&self, tracker_id: &str) -> CoreResult<Option<TrackerMedia>> {
        let resp = self
            .client
            .get(format!("{SIMKL_API_BASE}/anime/{tracker_id}"))
            .query(&[("extended", "full"), ("client_id", SIMKL_CLIENT_ID)])
            .header(
                "User-Agent",
                format!("{}/{}", SIMKL_APP_NAME, SIMKL_APP_VERSION),
            )
            .send()
            .await
            .map_err(|e| CoreError::Network(e.to_string()))?;

        if resp.status() == reqwest::StatusCode::NOT_FOUND {
            return Ok(None);
        }

        let status = resp.status();
        let raw = resp
            .text()
            .await
            .map_err(|e| CoreError::Network(e.to_string()))?;

        if !status.is_success() {
            return Err(CoreError::Network(format!(
                "SIMKL get_by_id failed ({status}) on /anime/{tracker_id}: {raw}"
            )));
        }

        let obj: SimklMediaObject = serde_json::from_str(&raw)
            .map_err(|e| CoreError::Parse(format!("failed to parse SIMKL media object: {e}")))?;

        Ok(Some(obj.into_tracker_media()))
    }

    async fn get_user_list(
        &self,
        access_token: &str,
        _tracker_user_id: &str,
        _score_format: Option<&str>,
    ) -> CoreResult<Vec<UserListEntry>> {
        let resp = self
            .authenticated_request(
                reqwest::Method::GET,
                "/sync/all-items/anime?extended=full",
                access_token,
            )
            .send()
            .await
            .map_err(|e| CoreError::Network(e.to_string()))?;

        let status = resp.status();
        let raw = resp
            .text()
            .await
            .map_err(|e| CoreError::Network(e.to_string()))?;

        if !status.is_success() {
            return Err(CoreError::Network(format!(
                "SIMKL get_user_list failed ({status}) on /sync/all-items/anime: {raw}"
            )));
        }

        let parsed: SimklAllItemsResponse = serde_json::from_str(&raw).map_err(|e| {
            CoreError::Parse(format!(
                "failed to parse SIMKL /sync/all-items/anime response: {e}"
            ))
        })?;

        Ok(parsed
            .anime
            .into_iter()
            .filter_map(SimklListItem::into_user_list_entry)
            .collect())
    }

    async fn update_entry(&self, access_token: &str, params: UpdateEntryParams) -> CoreResult<()> {
        // SIMKL's docs are explicit: don't chain /sync/add-to-list after
        // /sync/history — history already moves the item to the right
        // watchlist status (and may downgrade e.g. "completed" -> "watching"
        // for a still-airing show), so a follow-up add-to-list call can
        // clobber that decision. Send status + rating + progress together
        // in one /sync/history call instead.
        let status = params.status.as_deref().map(normalize_simkl_status);

        let episodes = params.progress.map(|progress| {
            (1..=progress.max(0))
                .map(|n| SimklEpisodeRef { number: n })
                .collect::<Vec<_>>()
        });

        let rating = params.score.map(|s| s.round() as i64);

        if status.is_none() && rating.is_none() && episodes.is_none() {
            return Ok(());
        }

        let body = SimklHistoryRequest {
            anime: vec![SimklHistoryShowEntry {
                ids: SimklIdRefIds {
                    simkl: params.media_id.clone(),
                },
                status,
                rating,
                episodes,
            }],
        };
        self.post_sync(access_token, "/sync/history", &body).await?;

        Ok(())
    }

    async fn delete_entry(&self, access_token: &str, media_id: &str) -> CoreResult<bool> {
        let body = SimklRemoveRequest {
            anime: vec![SimklIdRef::new(media_id)],
        };

        let raw = self
            .post_sync(access_token, "/sync/history/remove", &body)
            .await?;

        let parsed: Value = serde_json::from_str(&raw)
            .map_err(|e| CoreError::Parse(format!("failed to parse SIMKL remove response: {e}")))?;

        let deleted_count: i64 = parsed
            .get("deleted")
            .and_then(|d| d.as_object())
            .map(|obj| obj.values().filter_map(|v| v.as_i64()).sum())
            .unwrap_or(0);

        Ok(deleted_count > 0)
    }

    fn to_core_metadata(&self, cid: &str, media: &TrackerMedia) -> Metadata {
        let now = chrono::Utc::now().timestamp();

        let external_ids = serde_json::to_value(&media.cross_ids).unwrap_or(Value::Null);

        let eps_or_chapters = EpisodeData::Count(media.episode_count.unwrap_or(0));

        let characters: Vec<Character> = media
            .characters
            .iter()
            .map(|c| Character {
                name: c.name.clone(),
                role: c.role.clone(),
                actor: c.actor.clone(),
                image: c.image.clone(),
            })
            .collect();

        let staff: Vec<StaffMember> = media
            .staff
            .iter()
            .map(|s| StaffMember {
                name: s.name.clone(),
                role: s.role.clone(),
                image: s.image.clone(),
            })
            .collect();

        Metadata {
            id: None,
            cid: cid.to_string(),
            source_name: self.name().to_string(),
            source_id: Some(media.tracker_id.clone()),
            subtype: media.format.clone(), // anime_type: tv/movie/ova/ona/special/music video
            title: media.title.clone(),
            alt_titles: media.alt_titles.clone(),
            title_i18n: media.title_i18n.clone(),
            synopsis: media.synopsis.clone(),
            cover_image: media.cover_image.clone(),
            banner_image: media.banner_image.clone(),
            eps_or_chapters,
            status: media.status.as_deref().and_then(map_simkl_status),
            genres: media.genres.clone(),
            release_date: media.release_date.clone(),
            end_date: media.end_date.clone(),
            rating: media.rating,
            trailer_url: media.trailer_url.clone(),
            characters,
            studio: media.studio.clone(),
            staff,
            external_ids,
            episode_duration: media.episode_duration,
            created_at: now,
            updated_at: now,
        }
    }
}