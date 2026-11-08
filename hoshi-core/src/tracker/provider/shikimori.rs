use super::{
    RateLimit, TokenData, TrackerAuthConfig, TrackerMedia, TrackerProvider, TrackerRelation,
    UpdateEntryParams, UserListEntry,
};
use crate::content::models::{Character, ContentType, EpisodeData, Metadata, StaffMember, Status};
use crate::error::{CoreError, CoreResult};
use async_trait::async_trait;
use chrono::Utc;
use reqwest::{Client, Method, RequestBuilder, Response, StatusCode};
use serde_json::{json, Map, Value};
use std::collections::HashMap;
use std::sync::Mutex;
use std::time::Duration;
use tokio::sync::OnceCell;

const SITE: &str = "https://shikimori.io";
// The secret is embedded on purpose: Shikimori's token + refresh endpoints require it.
const CLIENT_ID: &str = "R8Wo4AcWPITRmMRkro5rfUB1-JFS3qV3LXHrn15di7M";
const CLIENT_SECRET: &str = "UtivAuzuxlIniu-CRdAxf_qc3ez_gE8DK67xaklzEDA";
const USER_AGENT: &str = "hoshi";

const PAGE_SIZE: usize = 50;

const COMMON_FIELDS: &str = "id malId name english japanese russian synonyms description url \
    kind status score isCensored \
    airedOn { year month day date } releasedOn { year month day date } \
    poster { main2xUrl mainUrl } genres { name kind }";
const ANIME_FIELDS: &str = "episodes episodesAired duration rating studios { name }";
const MANGA_FIELDS: &str = "chapters volumes";
const DETAIL_FIELDS: &str = "characterRoles { rolesEn character { name poster { mainUrl } } } \
    personRoles { rolesEn person { name poster { mainUrl } } } \
    related { relationKind \
      anime { id malId name kind poster { mainUrl } } \
      manga { id malId name kind poster { mainUrl } } }";
const ANIME_DETAIL_FIELDS: &str = "videos { kind url }";

fn selection(is_manga: bool, detailed: bool) -> String {
    let mut s = format!(
        "{} {}",
        COMMON_FIELDS,
        if is_manga { MANGA_FIELDS } else { ANIME_FIELDS }
    );
    if detailed {
        s.push(' ');
        s.push_str(DETAIL_FIELDS);
        if !is_manga {
            s.push(' ');
            s.push_str(ANIME_DETAIL_FIELDS);
        }
    }
    s
}

fn id_str(v: &Value) -> Option<String> {
    match v {
        Value::String(s) if !s.is_empty() => Some(s.clone()),
        Value::Number(n) => Some(n.to_string()),
        _ => None,
    }
}

fn str_of(v: &Value, key: &str) -> Option<String> {
    v.get(key)
        .and_then(|x| x.as_str())
        .filter(|s| !s.is_empty())
        .map(String::from)
}

fn int_of(v: &Value, key: &str) -> Option<i64> {
    v.get(key).and_then(|x| x.as_i64())
}

fn strip_markup(s: &str) -> String {
    let chars: Vec<char> = s.chars().collect();
    let mut out = String::with_capacity(s.len());
    let mut i = 0;
    while i < chars.len() {
        if chars[i] == '[' {
            let mut j = i + 1;
            if j < chars.len() && chars[j] == '/' {
                j += 1;
            }
            let name_start = j;
            while j < chars.len() && (chars[j].is_ascii_alphanumeric() || chars[j] == '_') {
                j += 1;
            }
            if j > name_start && j < chars.len() && (chars[j] == ']' || chars[j] == '=') {
                if let Some(rel) = chars[j..].iter().position(|c| *c == ']') {
                    i = j + rel + 1;
                    continue;
                }
            }
        }
        out.push(chars[i]);
        i += 1;
    }
    out.trim().to_string()
}

struct ShGenre {
    id: String,
    name: String,
}

pub struct ShikimoriProvider {
    client: Client,
    anime_genres: OnceCell<Vec<ShGenre>>,
    manga_genres: OnceCell<Vec<ShGenre>>,
    user_ids: Mutex<HashMap<String, String>>,
}

impl ShikimoriProvider {
    pub fn new(client: Client) -> Self {
        Self {
            client,
            anime_genres: OnceCell::new(),
            manga_genres: OnceCell::new(),
            user_ids: Mutex::new(HashMap::new()),
        }
    }

    /// Sends a request; on 429 waits for `Retry-After` (max 10s) and retries up to twice.
    async fn send(req: RequestBuilder) -> CoreResult<Response> {
        let mut attempt = 0;
        loop {
            let this = req
                .try_clone()
                .ok_or_else(|| CoreError::Internal("Shikimori: request not cloneable".into()))?;
            let res = this
                .send()
                .await
                .map_err(|e| CoreError::Network(e.to_string()))?;

            if res.status() == StatusCode::TOO_MANY_REQUESTS && attempt < 2 {
                let wait = res
                    .headers()
                    .get(reqwest::header::RETRY_AFTER)
                    .and_then(|v| v.to_str().ok())
                    .and_then(|v| v.parse::<u64>().ok())
                    .unwrap_or(2)
                    .min(10);
                tokio::time::sleep(Duration::from_secs(wait)).await;
                attempt += 1;
                continue;
            }
            return Ok(res);
        }
    }

    /// The sync queue classifies errors by variant (AuthError / RateLimited / Network).
    fn ensure_ok(res: Response) -> CoreResult<Response> {
        let status = res.status();
        if status.is_success() {
            return Ok(res);
        }
        Err(match status {
            StatusCode::UNAUTHORIZED => {
                CoreError::AuthError("error.tracker.invalid_credentials".into())
            }
            StatusCode::FORBIDDEN => {
                CoreError::AuthError("error.tracker.insufficient_scope".into())
            }
            StatusCode::NOT_FOUND => CoreError::NotFound("error.tracker.not_found".into()),
            StatusCode::TOO_MANY_REQUESTS => {
                CoreError::RateLimited("error.tracker.rate_limited".into())
            }
            _ => {
                tracing::warn!(status = status.as_u16(), "Shikimori returned an error status");
                CoreError::Internal("error.tracker.http_error".into())
            }
        })
    }

    async fn graphql(&self, token: Option<&str>, query: &str, variables: Value) -> CoreResult<Value> {
        let mut req = self
            .client
            .post(format!("{SITE}/api/graphql"))
            .header("User-Agent", USER_AGENT)
            .json(&json!({ "query": query, "variables": variables }));
        if let Some(t) = token {
            req = req.bearer_auth(t);
        }

        let res = Self::ensure_ok(Self::send(req).await?)?;
        let body: Value = res
            .json()
            .await
            .map_err(|e| CoreError::Parse(e.to_string()))?;

        if let Some(errs) = body
            .get("errors")
            .and_then(|e| e.as_array())
            .filter(|a| !a.is_empty())
        {
            return Err(CoreError::Internal(format!(
                "Shikimori GraphQL error: {}",
                errs[0]
            )));
        }
        Ok(body)
    }

    fn rest(&self, method: Method, path: &str, token: &str) -> RequestBuilder {
        self.client
            .request(method, format!("{SITE}/api/v2{path}"))
            .header("User-Agent", USER_AGENT)
            .bearer_auth(token)
    }

    async fn current_user_id(&self, token: &str) -> CoreResult<String> {
        let cached = self.user_ids.lock().unwrap().get(token).cloned();
        if let Some(id) = cached {
            return Ok(id);
        }

        let res = self
            .graphql(Some(token), "query { currentUser { id } }", json!({}))
            .await?;
        let id = res
            .pointer("/data/currentUser/id")
            .and_then(id_str)
            .ok_or_else(|| {
                CoreError::AuthError("error.tracker.invalid_credentials".into())
            })?;

        {
            let mut map = self.user_ids.lock().unwrap();
            if map.len() > 16 {
                map.clear();
            }
            map.insert(token.to_string(), id.clone());
        }
        Ok(id)
    }

    async fn find_user_rate_id(
        &self,
        token: &str,
        is_manga: bool,
        target_id: &str,
    ) -> CoreResult<Option<String>> {
        let user_id = self.current_user_id(token).await?;
        let target_type = if is_manga { "Manga" } else { "Anime" };

        let req = self.rest(Method::GET, "/user_rates", token).query(&[
            ("user_id", user_id.as_str()),
            ("target_id", target_id),
            ("target_type", target_type),
        ]);
        let res = Self::ensure_ok(Self::send(req).await?)?;
        let list: Value = res
            .json()
            .await
            .map_err(|e| CoreError::Parse(e.to_string()))?;

        Ok(list
            .as_array()
            .and_then(|a| a.first())
            .and_then(|r| r.get("id"))
            .and_then(id_str))
    }

    async fn genre_table(&self, is_manga: bool) -> CoreResult<&Vec<ShGenre>> {
        let cell = if is_manga {
            &self.manga_genres
        } else {
            &self.anime_genres
        };
        cell.get_or_try_init(|| async {
            let entry_type = if is_manga { "Manga" } else { "Anime" };
            let res = self
                .graphql(
                    None,
                    "query($t: GenreEntryTypeEnum!) { genres(entryType: $t) { id name } }",
                    json!({ "t": entry_type }),
                )
                .await?;
            let list = res
                .pointer("/data/genres")
                .and_then(|v| v.as_array())
                .ok_or_else(|| CoreError::NotFound("Shikimori: no genres".into()))?;
            let table: Vec<ShGenre> = list
                .iter()
                .filter_map(|g| {
                    Some(ShGenre {
                        id: id_str(g.get("id")?)?,
                        name: str_of(g, "name")?,
                    })
                })
                .collect();
            CoreResult::Ok(table)
        })
            .await
    }

    /// Accepts ids or names (comma separated); unknown names are dropped.
    async fn resolve_genre_ids(&self, is_manga: bool, genre: &str) -> Option<String> {
        let wanted: Vec<&str> = genre
            .split(',')
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .collect();
        if wanted.is_empty() {
            return None;
        }

        let mut ids: Vec<String> = Vec::new();
        let mut table: Option<&Vec<ShGenre>> = None;
        for g in wanted {
            if g.chars().all(|c| c.is_ascii_digit()) {
                ids.push(g.to_string());
                continue;
            }
            if table.is_none() {
                table = self.genre_table(is_manga).await.ok();
            }
            if let Some(found) = table.and_then(|t| t.iter().find(|x| x.name.eq_ignore_ascii_case(g))) {
                ids.push(found.id.clone());
            }
        }

        if ids.is_empty() {
            None
        } else {
            Some(ids.join(","))
        }
    }

    fn snake(s: &str) -> String {
        s.trim().to_lowercase().replace(['-', ' '], "_")
    }

    /// "anime:123" / "manga:123" / bare "123" (anime). Returns (is_manga, id).
    fn parse_media_id(id: &str) -> (bool, &str) {
        match id.split_once(':') {
            Some(("manga", rest)) => (true, rest),
            Some(("anime", rest)) => (false, rest),
            _ => (false, id),
        }
    }

    /// Entry ids in Shikimori URLs can carry a letter prefix; the API wants the number.
    fn numeric(id: &str) -> &str {
        id.trim_start_matches(|c: char| c.is_ascii_alphabetic())
    }

    fn parse_date(obj: Option<&Value>) -> Option<String> {
        let o = obj?;
        if let Some(full) = str_of(o, "date") {
            return Some(full.get(..10).unwrap_or(&full).to_string());
        }
        let year = int_of(o, "year")?;
        let month = int_of(o, "month").unwrap_or(1);
        let day = int_of(o, "day").unwrap_or(1);
        Some(format!("{:04}-{:02}-{:02}", year, month, day))
    }

    fn normalize_status(s: &str) -> Status {
        match s {
            "released" => Status::Completed,
            "anons" => Status::Planned,
            "paused" => Status::Hiatus,
            "discontinued" => Status::Cancelled,
            // ongoing, unknown
            _ => Status::Ongoing,
        }
    }

    /// Shikimori list status -> the app's AniList-style vocabulary.
    fn status_to_app(s: &str) -> &'static str {
        match s {
            "watching" => "CURRENT",
            "completed" => "COMPLETED",
            "on_hold" => "PAUSED",
            "dropped" => "DROPPED",
            "rewatching" => "REPEATING",
            // planned
            _ => "PLANNING",
        }
    }

    /// Accepts the app's vocabulary plus MAL-style aliases.
    fn status_to_shiki(s: &str) -> Option<&'static str> {
        match Self::snake(s).as_str() {
            "current" | "watching" | "reading" => Some("watching"),
            "planning" | "planned" | "plan_to_watch" | "plan_to_read" | "plantowatch" => {
                Some("planned")
            }
            "completed" | "complete" | "finished" => Some("completed"),
            "paused" | "on_hold" | "hold" => Some("on_hold"),
            "dropped" => Some("dropped"),
            "repeating" | "rewatching" | "rereading" => Some("rewatching"),
            _ => None,
        }
    }

    fn map_sort(sort: Option<&str>) -> Option<&'static str> {
        match Self::snake(sort?).as_str() {
            "popularity" | "popular" | "trending" => Some("popularity"),
            "score" | "rating" | "average_score" | "top" | "rank" => Some("ranked"),
            "title" | "name" => Some("name"),
            "newest" | "latest" | "start_date" | "release_date" => Some("aired_on"),
            _ => None,
        }
    }

    fn map_format(f: &str) -> String {
        match Self::snake(f).as_str() {
            "lightnovel" | "ln" => "light_novel".to_string(),
            "oneshot" => "one_shot".to_string(),
            other => other.to_string(),
        }
    }

    fn map_search_status(s: &str, is_manga: bool) -> Option<&'static str> {
        match Self::snake(s).as_str() {
            "finished" | "completed" | "complete" | "finished_airing" | "released" => {
                Some("released")
            }
            "releasing" | "airing" | "currently_airing" | "ongoing" | "publishing"
            | "currently_publishing" => Some("ongoing"),
            "not_yet_released" | "not_yet_aired" | "not_yet_published" | "upcoming"
            | "planned" | "anons" => Some("anons"),
            "hiatus" | "on_hiatus" | "paused" if is_manga => Some("paused"),
            "cancelled" | "discontinued" if is_manga => Some("discontinued"),
            _ => None,
        }
    }

    /// Works for the light (list/search) and the detailed (get_by_id) selections,
    /// and for the small stubs inside `related`.
    fn media_to_tracker_media(data: &Value, is_manga: bool) -> Option<TrackerMedia> {
        let id = data.get("id").and_then(id_str)?;
        let kind = str_of(data, "kind");
        let content_type = if !is_manga {
            ContentType::Anime
        } else if matches!(kind.as_deref(), Some("light_novel") | Some("novel")) {
            ContentType::Novel
        } else {
            ContentType::Manga
        };
        let prefix = if is_manga { "manga" } else { "anime" };

        let mut cross_ids = HashMap::new();
        if let Some(mal) = data.get("malId").and_then(id_str) {
            cross_ids.insert("mal".to_string(), format!("{prefix}:{mal}"));
        }

        let title = str_of(data, "name").unwrap_or_else(|| "Unknown".to_string());
        let english = str_of(data, "english");
        let native = str_of(data, "japanese");

        let mut title_i18n = HashMap::new();
        title_i18n.insert("romaji".to_string(), title.clone());
        if let Some(e) = &english {
            title_i18n.insert("english".to_string(), e.clone());
        }
        if let Some(n) = &native {
            title_i18n.insert("native".to_string(), n.clone());
        }

        let synonyms = data
            .get("synonyms")
            .and_then(|v| v.as_array())
            .into_iter()
            .flatten()
            .filter_map(|v| v.as_str())
            .map(String::from);
        let mut alt_titles: Vec<String> = Vec::new();
        for t in english
            .iter()
            .cloned()
            .chain(native.iter().cloned())
            .chain(str_of(data, "russian"))
            .chain(synonyms)
        {
            if !t.is_empty() && t != title && !alt_titles.contains(&t) {
                alt_titles.push(t);
            }
        }

        let cover_image = data
            .get("poster")
            .and_then(|p| str_of(p, "main2xUrl").or_else(|| str_of(p, "mainUrl")));

        let (mut genres, mut tags) = (Vec::new(), Vec::new());
        for g in data
            .get("genres")
            .and_then(|v| v.as_array())
            .into_iter()
            .flatten()
        {
            let Some(name) = str_of(g, "name") else { continue };
            if str_of(g, "kind").as_deref() == Some("theme") {
                tags.push(name);
            } else {
                genres.push(name);
            }
        }

        // In Shikimori's API `isCensored` marks adult (hentai/yaoi/yuri) entries.
        let nsfw = data.get("isCensored").and_then(|v| v.as_bool()).unwrap_or(false)
            || str_of(data, "rating").as_deref() == Some("rx");

        let rating = data
            .get("score")
            .and_then(|v| v.as_f64())
            .filter(|s| *s > 0.0)
            .map(|s| s as f32);

        let studio = data
            .get("studios")
            .and_then(|v| v.as_array())
            .and_then(|a| a.first())
            .and_then(|s| str_of(s, "name"));

        let trailer_url = data
            .get("videos")
            .and_then(|v| v.as_array())
            .and_then(|a| {
                a.iter()
                    .find(|v| str_of(v, "kind").as_deref() == Some("pv"))
            })
            .and_then(|v| str_of(v, "url"));

        let first_role = |r: &Value| -> String {
            r.get("rolesEn")
                .and_then(|x| x.as_array())
                .map(|a| {
                    a.iter()
                        .filter_map(|v| v.as_str())
                        .collect::<Vec<_>>()
                        .join(", ")
                })
                .unwrap_or_default()
        };

        let characters: Vec<Character> = data
            .get("characterRoles")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .take(10)
                    .filter_map(|r| {
                        let c = r.get("character")?;
                        Some(Character {
                            name: str_of(c, "name")?,
                            role: first_role(r),
                            actor: None,
                            image: c.get("poster").and_then(|p| str_of(p, "mainUrl")),
                        })
                    })
                    .collect()
            })
            .unwrap_or_default();

        let staff: Vec<StaffMember> = data
            .get("personRoles")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .take(8)
                    .filter_map(|r| {
                        let p = r.get("person")?;
                        Some(StaffMember {
                            name: str_of(p, "name")?,
                            role: first_role(r),
                            image: p.get("poster").and_then(|x| str_of(x, "mainUrl")),
                        })
                    })
                    .collect()
            })
            .unwrap_or_default();

        let mut relations: Vec<TrackerRelation> = Vec::new();
        for r in data
            .get("related")
            .and_then(|v| v.as_array())
            .into_iter()
            .flatten()
        {
            let relation_type = str_of(r, "relationKind")
                .unwrap_or_default()
                .to_uppercase();
            let anime = r.get("anime").filter(|v| !v.is_null());
            let manga = r.get("manga").filter(|v| !v.is_null());
            let (node, rel_is_manga) = match (anime, manga) {
                (Some(a), _) => (a, false),
                (None, Some(m)) => (m, true),
                _ => continue,
            };
            if let Some(media) = Self::media_to_tracker_media(node, rel_is_manga) {
                relations.push(TrackerRelation { relation_type, media });
            }
        }

        Some(TrackerMedia {
            tracker_id: format!("{prefix}:{id}"),
            tracker_url: str_of(data, "url"),
            cross_ids,
            content_type,
            title,
            alt_titles,
            title_i18n,
            synopsis: str_of(data, "description")
                .map(|d| strip_markup(&d))
                .filter(|d| !d.is_empty()),
            cover_image,
            banner_image: None,
            episode_count: int_of(data, "episodes").filter(|n| *n > 0).map(|n| n as i32),
            chapter_count: int_of(data, "chapters").filter(|n| *n > 0).map(|n| n as i32),
            status: str_of(data, "status"),
            genres,
            tags,
            nsfw,
            release_date: Self::parse_date(data.get("airedOn")),
            end_date: Self::parse_date(data.get("releasedOn")),
            rating,
            trailer_url,
            format: kind,
            studio,
            characters,
            staff,
            relations,
            episode_duration: int_of(data, "duration").filter(|n| *n > 0).map(|n| n as i32),
        })
    }

    fn user_rate_to_entry(ur: &Value, is_manga: bool) -> Option<UserListEntry> {
        let node = ur
            .get(if is_manga { "manga" } else { "anime" })
            .filter(|v| !v.is_null())?;
        let media = Self::media_to_tracker_media(node, is_manga)?;

        let status = ur
            .get("status")
            .and_then(|v| v.as_str())
            .map(|s| Self::status_to_app(s).to_string());
        let progress = int_of(ur, if is_manga { "chapters" } else { "episodes" }).unwrap_or(0) as i32;
        // Shikimori uses 0 for "not rated".
        let score = int_of(ur, "score").filter(|s| *s > 0).map(|s| s as f64);

        // `episodes` is 0 while the total is unknown; fall back to what has aired.
        let total_episodes = if is_manga {
            None
        } else {
            media.episode_count.or_else(|| {
                int_of(node, "episodesAired")
                    .filter(|n| *n > 0)
                    .map(|n| n as i32)
            })
        };

        Some(UserListEntry {
            tracker_media_id: media.tracker_id.clone(),
            title: media.title.clone(),
            poster: media.cover_image.clone(),
            content_type: media.content_type.clone(),
            format: media.format.clone(),
            status,
            progress,
            score,
            // Shikimori's UserRate has no start/finish dates and no private flag.
            start_date: None,
            end_date: None,
            repeat_count: int_of(ur, "rewatches").unwrap_or(0) as i32,
            notes: str_of(ur, "text"),
            is_private: false,
            total_episodes,
            total_chapters: media.chapter_count,
            media: Some(media),
        })
    }

    async fn fetch_user_rates(
        &self,
        token: &str,
        user_id: &str,
        is_manga: bool,
    ) -> CoreResult<Vec<UserListEntry>> {
        let media_sel = if is_manga {
            format!("manga {{ {} }}", selection(true, false))
        } else {
            format!("anime {{ {} }}", selection(false, false))
        };
        let query = format!(
            "query($page: PositiveInt, $userId: ID, $targetType: UserRateTargetTypeEnum) {{ \
               userRates(page: $page, limit: {PAGE_SIZE}, userId: $userId, targetType: $targetType) {{ \
                 id status score episodes chapters rewatches text {media_sel} \
               }} \
             }}"
        );
        let target_type = if is_manga { "Manga" } else { "Anime" };

        let mut out = Vec::new();
        let mut page = 1;
        loop {
            let res = self
                .graphql(
                    Some(token),
                    &query,
                    json!({ "page": page, "userId": user_id, "targetType": target_type }),
                )
                .await?;
            let rates = res
                .pointer("/data/userRates")
                .and_then(|v| v.as_array())
                .cloned()
                .unwrap_or_default();

            let n = rates.len();
            out.extend(rates.iter().filter_map(|r| Self::user_rate_to_entry(r, is_manga)));

            if n < PAGE_SIZE || page >= 1000 {
                break;
            }
            page += 1;
            // 5 rps / 90 rpm limit.
            tokio::time::sleep(Duration::from_millis(700)).await;
        }
        Ok(out)
    }
}

#[async_trait]
impl TrackerProvider for ShikimoriProvider {
    fn name(&self) -> &'static str {
        "shikimori"
    }
    fn display_name(&self) -> &'static str {
        "Shikimori"
    }
    fn icon_url(&self) -> &'static str {
        "https://shikimori.io/favicon.ico"
    }

    fn supported_types(&self) -> Vec<ContentType> {
        vec![ContentType::Anime, ContentType::Manga, ContentType::Novel]
    }

    /// Shikimori allows 5 rps / 90 rpm. A sync job costs ~2 requests (lookup + write),
    /// so one job every 1.5s stays under the per-minute cap.
    fn rate_limit(&self) -> RateLimit {
        RateLimit {
            min_interval: Duration::from_millis(1500),
        }
    }

    fn auth_config(&self) -> TrackerAuthConfig {
        TrackerAuthConfig {
            // Authorization code + client secret (no PKCE): the frontend must send the
            // code in `access_token`, like the pkce flow, but without a code_verifier.
            oauth_flow: "code".to_string(),
            auth_url: format!("{SITE}/oauth/authorize"),
            token_url: Some(format!("{SITE}/oauth/token")),
            client_id: Some(CLIENT_ID.to_string()),
            scopes: vec!["user_rates".to_string()],
        }
    }

    fn token_request_extras(&self) -> Vec<(&'static str, String)> {
        vec![("client_secret", CLIENT_SECRET.to_string())]
    }

    fn user_agent(&self) -> Option<&'static str> {
        Some(USER_AGENT)
    }

    async fn validate_and_store_token(
        &self,
        access_token: &str,
        token_type: &str,
    ) -> CoreResult<TokenData> {
        let res = self
            .graphql(
                Some(access_token),
                "query { currentUser { id nickname avatarUrl url } }",
                json!({}),
            )
            .await?;

        let user = res
            .pointer("/data/currentUser")
            .filter(|v| !v.is_null())
            .ok_or_else(|| CoreError::AuthError("error.tracker.invalid_credentials".into()))?;
        let id = user
            .get("id")
            .and_then(id_str)
            .ok_or_else(|| CoreError::AuthError("error.tracker.invalid_credentials".into()))?;

        Ok(TokenData {
            access_token: access_token.to_string(),
            refresh_token: None,
            token_type: token_type.to_string(),
            // Shikimori access tokens last one day; the real value comes from `expires_in`.
            expires_at: Utc::now()
                .checked_add_signed(chrono::Duration::days(1))
                .unwrap_or_else(Utc::now)
                .to_rfc3339(),
            tracker_user_id: id,
            display_name: str_of(user, "nickname"),
            avatar_url: str_of(user, "avatarUrl"),
            profile_url: str_of(user, "url"),
            score_format: None,
        })
    }

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
        _tags: Option<&[String]>,
    ) -> CoreResult<Vec<TrackerMedia>> {
        let is_manga = !matches!(content_type, ContentType::Anime);

        let mut vars = Map::new();
        vars.insert("limit".into(), json!(limit.clamp(1, PAGE_SIZE)));
        vars.insert("page".into(), json!(page.max(1)));

        let has_query = match query.map(str::trim).filter(|s| !s.is_empty()) {
            Some(q) => {
                vars.insert("search".into(), json!(q));
                true
            }
            None => false,
        };

        match Self::map_sort(sort) {
            Some(o) => {
                vars.insert("order".into(), json!(o));
            }
            // Browsing without a query: most popular first instead of the API's default rank.
            None if !has_query => {
                vars.insert("order".into(), json!("popularity"));
            }
            None => {}
        }

        let kind = match format.map(str::trim).filter(|s| !s.is_empty()) {
            Some(f) => Some(Self::map_format(f)),
            None if matches!(content_type, ContentType::Novel) => Some("light_novel,novel".to_string()),
            None if matches!(content_type, ContentType::Manga) => {
                Some("!light_novel,!novel".to_string())
            }
            None => None,
        };
        if let Some(k) = kind {
            vars.insert("kind".into(), json!(k));
        }

        if let Some(s) = status.and_then(|s| Self::map_search_status(s, is_manga)) {
            vars.insert("status".into(), json!(s));
        }

        if let Some(g) = genre {
            if let Some(ids) = self.resolve_genre_ids(is_manga, g).await {
                vars.insert("genre".into(), json!(ids));
            }
        }

        // "NSFW only": `censored: false` lets adult entries through, anime can also filter by
        // rating; the final retain below keeps adult entries only (manga has no rating filter).
        if nsfw == Some(true) {
            vars.insert("censored".into(), json!(false));
            if !is_manga {
                vars.insert("rating".into(), json!("rx"));
            }
        }

        let sel = selection(is_manga, false);
        let (gql, key) = if is_manga {
            (
                format!(
                    "query($search: String, $page: PositiveInt, $limit: PositiveInt, $order: OrderEnum, \
                     $kind: MangaKindString, $status: MangaStatusString, $genre: String, $censored: Boolean) {{ \
                       mangas(search: $search, page: $page, limit: $limit, order: $order, kind: $kind, \
                              status: $status, genre: $genre, censored: $censored) {{ {sel} }} \
                     }}"
                ),
                "mangas",
            )
        } else {
            (
                format!(
                    "query($search: String, $page: PositiveInt, $limit: PositiveInt, $order: OrderEnum, \
                     $kind: AnimeKindString, $status: AnimeStatusString, $genre: String, \
                     $censored: Boolean, $rating: RatingString) {{ \
                       animes(search: $search, page: $page, limit: $limit, order: $order, kind: $kind, \
                              status: $status, genre: $genre, censored: $censored, rating: $rating) {{ {sel} }} \
                     }}"
                ),
                "animes",
            )
        };

        let res = self.graphql(None, &gql, Value::Object(vars)).await?;
        let list = res
            .pointer(&format!("/data/{key}"))
            .and_then(|v| v.as_array())
            .ok_or_else(|| CoreError::NotFound("No results from Shikimori".into()))?;

        let mut results: Vec<TrackerMedia> = list
            .iter()
            .filter_map(|m| Self::media_to_tracker_media(m, is_manga))
            .collect();
        if nsfw == Some(true) {
            results.retain(|m| m.nsfw);
        }
        Ok(results)
    }

    async fn get_by_id(&self, tracker_id: &str) -> CoreResult<Option<TrackerMedia>> {
        let (is_manga, raw) = Self::parse_media_id(tracker_id);
        let id = Self::numeric(raw);
        if id.is_empty() {
            return Ok(None);
        }

        let sel = selection(is_manga, true);
        let (gql, key) = if is_manga {
            (
                format!("query($ids: String) {{ mangas(ids: $ids, limit: 1) {{ {sel} }} }}"),
                "mangas",
            )
        } else {
            (
                format!("query($ids: String) {{ animes(ids: $ids, limit: 1) {{ {sel} }} }}"),
                "animes",
            )
        };

        let res = self.graphql(None, &gql, json!({ "ids": id })).await?;
        Ok(res
            .pointer(&format!("/data/{key}"))
            .and_then(|v| v.as_array())
            .and_then(|a| a.first())
            .and_then(|m| Self::media_to_tracker_media(m, is_manga)))
    }

    async fn get_user_list(
        &self,
        access_token: &str,
        tracker_user_id: &str,
        _score_format: Option<&str>,
    ) -> CoreResult<Vec<UserListEntry>> {
        let mut all = self
            .fetch_user_rates(access_token, tracker_user_id, false)
            .await?;
        all.extend(
            self.fetch_user_rates(access_token, tracker_user_id, true)
                .await?,
        );
        Ok(all)
    }

    /// GraphQL has no mutations, so writes go through REST v2 (`/api/v2/user_rates`).
    async fn update_entry(&self, access_token: &str, params: UpdateEntryParams) -> CoreResult<()> {
        let (is_manga, raw) = Self::parse_media_id(&params.media_id);
        let target_id = Self::numeric(raw).to_string();
        let target_type = if is_manga { "Manga" } else { "Anime" };

        let mut attrs = Map::new();
        if let Some(st) = &params.status {
            let mapped = Self::status_to_shiki(st)
                .ok_or_else(|| CoreError::BadRequest("error.tracker.invalid_status".into()))?;
            attrs.insert("status".into(), json!(mapped));
        }
        if let Some(p) = params.progress {
            let key = if is_manga { "chapters" } else { "episodes" };
            attrs.insert(key.to_string(), json!(p.max(0)));
        }
        if let Some(sc) = params.score {
            attrs.insert("score".into(), json!((sc.round() as i64).clamp(0, 10)));
        }
        if let Some(r) = params.repeat_count {
            attrs.insert("rewatches".into(), json!(r.max(0)));
        }
        if let Some(n) = params.notes {
            attrs.insert("text".into(), json!(n));
        }
        // Shikimori has no start/finish dates or private flag on user rates.

        match self.find_user_rate_id(access_token, is_manga, &target_id).await? {
            Some(rate_id) => {
                if attrs.is_empty() {
                    return Ok(());
                }
                let req = self
                    .rest(Method::PATCH, &format!("/user_rates/{rate_id}"), access_token)
                    .json(&json!({ "user_rate": attrs }));
                Self::ensure_ok(Self::send(req).await?)?;
            }
            None => {
                let user_id = self.current_user_id(access_token).await?;
                attrs.insert("user_id".into(), json!(user_id));
                attrs.insert("target_id".into(), json!(target_id));
                attrs.insert("target_type".into(), json!(target_type));
                let req = self
                    .rest(Method::POST, "/user_rates", access_token)
                    .json(&json!({ "user_rate": attrs }));
                Self::ensure_ok(Self::send(req).await?)?;
            }
        }
        Ok(())
    }

    async fn delete_entry(&self, access_token: &str, media_id: &str) -> CoreResult<bool> {
        let (is_manga, raw) = Self::parse_media_id(media_id);
        let target_id = Self::numeric(raw);

        let Some(rate_id) = self.find_user_rate_id(access_token, is_manga, target_id).await? else {
            // Nothing to delete: already gone remotely.
            return Ok(false);
        };

        let req = self.rest(
            Method::DELETE,
            &format!("/user_rates/{rate_id}"),
            access_token,
        );
        let res = Self::send(req).await?;
        if res.status() == StatusCode::NOT_FOUND {
            return Ok(false); // already gone
        }
        Self::ensure_ok(res)?;
        Ok(true)
    }

    fn to_core_metadata(&self, cid: &str, media: &TrackerMedia) -> Metadata {
        let now = Utc::now().timestamp();
        let count = match media.content_type {
            ContentType::Anime => media.episode_count.unwrap_or(0),
            _ => media.chapter_count.unwrap_or(0),
        };

        Metadata {
            id: None,
            cid: cid.to_string(),
            source_name: self.name().to_string(),
            source_id: Some(media.tracker_id.clone()),
            subtype: media.format.clone(),
            title: media.title.clone(),
            alt_titles: media.alt_titles.clone(),
            title_i18n: media.title_i18n.clone(),
            synopsis: media.synopsis.clone(),
            cover_image: media.cover_image.clone(),
            banner_image: media.banner_image.clone(),
            eps_or_chapters: EpisodeData::Count(count),
            status: media.status.as_deref().map(Self::normalize_status),
            genres: media.genres.clone(),
            release_date: media.release_date.clone(),
            end_date: media.end_date.clone(),
            rating: media.rating,
            trailer_url: media.trailer_url.clone(),
            characters: media.characters.clone(),
            studio: media.studio.clone(),
            staff: media.staff.clone(),
            external_ids: json!(media.cross_ids),
            episode_duration: media.episode_duration,
            created_at: now,
            updated_at: now,
        }
    }
}