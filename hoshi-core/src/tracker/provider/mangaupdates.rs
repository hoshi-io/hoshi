use super::{
    http_status_error, RateLimit, TokenData, TrackerAuthConfig, TrackerMedia, TrackerProvider,
    TrackerRelation, UpdateEntryParams, UserListEntry,
};
use crate::content::models::{ContentType, EpisodeData, Metadata, StaffMember, Status};
use crate::core_err;
use crate::error::CoreResult;
use async_trait::async_trait;
use chrono::Utc;
use regex::Regex;
use reqwest::{Client, RequestBuilder, Response, StatusCode};
use serde::de::DeserializeOwned;
use serde::Deserialize;
use serde_json::{json, Value};
use std::collections::HashMap;
use std::sync::LazyLock;
use std::time::Duration;
use tracing::warn;

const API: &str = "https://api.mangaupdates.com/v1";
const LIST_PAGE: usize = 100;
/// Every comic type; searching "manga" means all of these (novels are their own content type).
const COMIC_TYPES: [&str; 15] = [
    "Manga", "Manhwa", "Manhua", "OEL", "Doujinshi", "Filipino", "Indonesian", "Thai",
    "Vietnamese", "Malaysian", "Nordic", "French", "Spanish", "German", "Russian",
];
const ADULT_GENRES: [&str; 3] = ["Adult", "Hentai", "Smut"];

/// MangaUpdates (api.mangaupdates.com). Tracker ids are the base36 ids used in
/// website URLs and by MangaBaka's cross ids; the API wants them as decimal integers.
pub struct MangaUpdatesProvider {
    client: Client,
}

impl MangaUpdatesProvider {
    pub fn new(client: Client) -> Self {
        Self { client }
    }

    /// Writes are throttled per user ("Five second update delay", HTTP 412): wait it out once.
    async fn send(&self, req: RequestBuilder) -> CoreResult<Response> {
        let retry = req.try_clone();
        let res = req
            .send()
            .await
            .map_err(|e| core_err!(Network, "error.tracker.request_failed", e))?;

        let wait = match res.status() {
            StatusCode::PRECONDITION_FAILED => 6,
            StatusCode::TOO_MANY_REQUESTS => 10,
            _ => return Ok(res),
        };
        let Some(retry) = retry else { return Ok(res) };
        tokio::time::sleep(Duration::from_secs(wait)).await;
        retry
            .send()
            .await
            .map_err(|e| core_err!(Network, "error.tracker.request_failed", e))
    }

    async fn ensure_ok(res: Response) -> CoreResult<Response> {
        let status = res.status();
        if status.is_success() {
            return Ok(res);
        }
        if status == StatusCode::PRECONDITION_FAILED {
            return Err(core_err!(RateLimited, "error.tracker.rate_limited"));
        }
        let body = res.text().await.unwrap_or_default();
        Err(http_status_error("mangaupdates", status, &body))
    }

    async fn parse<T: DeserializeOwned>(res: Response) -> CoreResult<T> {
        res.json::<T>()
            .await
            .map_err(|e| core_err!(Parse, "error.tracker.parse_failed", e))
    }

    async fn get_json<T: DeserializeOwned>(&self, req: RequestBuilder) -> CoreResult<T> {
        Self::parse(Self::ensure_ok(self.send(req).await?).await?).await
    }

    fn to_base36(mut n: i64) -> String {
        const DIGITS: &[u8] = b"0123456789abcdefghijklmnopqrstuvwxyz";
        if n <= 0 {
            return "0".into();
        }
        let mut out = Vec::new();
        while n > 0 {
            out.push(DIGITS[(n % 36) as usize]);
            n /= 36;
        }
        out.reverse();
        String::from_utf8(out).unwrap_or_default()
    }

    /// Accepts the base36 id (`pb8uwds`) or the API's decimal id (`55099564912`).
    fn parse_id(raw: &str) -> CoreResult<i64> {
        let s = raw.rsplit(':').next().unwrap_or(raw).trim().to_ascii_lowercase();
        let parsed = if s.len() >= 9 && s.bytes().all(|b| b.is_ascii_digit()) {
            s.parse::<i64>().ok()
        } else {
            i64::from_str_radix(&s, 36).ok()
        };
        parsed.ok_or_else(|| core_err!(BadRequest, "error.tracker.invalid_id", raw))
    }

    fn snake(s: &str) -> String {
        s.trim().to_lowercase().replace(['-', ' '], "_")
    }

    /// Descriptions are markdown with site links; keep the text only.
    fn clean_description(s: &str) -> Option<String> {
        static LINK: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"\[([^\]]*)\]\([^)]*\)").unwrap());
        static EMPHASIS: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"\*\*|__").unwrap());
        static BREAK: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"[ \t]+\n").unwrap());
        let s = LINK.replace_all(s, "$1");
        let s = EMPHASIS.replace_all(&s, "");
        let s = BREAK.replace_all(&s, "\n");
        let s = s.trim();
        (!s.is_empty()).then(|| s.to_string())
    }

    /// `"115 Volumes (Ongoing)"` -> `ongoing`; falls back to the `completed` flag.
    fn parse_status(status: Option<&str>, completed: Option<bool>) -> Option<String> {
        let s = status.unwrap_or("").to_lowercase();
        let state = if s.contains("hiatus") {
            "hiatus"
        } else if s.contains("discontinued") || s.contains("cancel") {
            "cancelled"
        } else if s.contains("complete") || completed == Some(true) {
            "completed"
        } else if s.contains("ongoing") || completed == Some(false) {
            "ongoing"
        } else {
            return None;
        };
        Some(state.to_string())
    }

    fn normalize_status(s: &str) -> Status {
        match s {
            "completed" => Status::Completed,
            "hiatus" => Status::Hiatus,
            "cancelled" => Status::Cancelled,
            _ => Status::Ongoing,
        }
    }

    fn content_type_of(series_type: Option<&str>) -> ContentType {
        if series_type == Some("Novel") {
            ContentType::Novel
        } else {
            ContentType::Manga
        }
    }

    fn type_from_format(format: &str) -> Option<&'static str> {
        Some(match Self::snake(format).as_str() {
            "manga" => "Manga",
            "manhwa" => "Manhwa",
            "manhua" => "Manhua",
            "novel" | "light_novel" | "ln" => "Novel",
            "oel" => "OEL",
            "doujinshi" => "Doujinshi",
            _ => return None,
        })
    }

    fn map_sort(sort: &str) -> Option<&'static str> {
        // "rank" makes the API throw; reading-list size is the stable popularity signal.
        Some(match Self::snake(sort).as_str() {
            "popularity" | "popular" | "rank" | "popularity_desc" => "list_reading",
            "trending" | "trending_desc" => "week_pos",
            "score" | "rating" | "top" | "average_score" | "score_desc" => "rating",
            "newest" | "start_date" | "release_date" | "year" => "year",
            "title" | "title_romaji" => "title",
            "latest" | "updated" => "date_added",
            _ => return None,
        })
    }

    fn relation_type(t: &str) -> &'static str {
        match t {
            "Prequel" => "PREQUEL",
            "Sequel" => "SEQUEL",
            "Spin-Off" => "SPIN_OFF",
            "Side Story" => "SIDE_STORY",
            "Main Story" => "PARENT",
            "Adapted From" => "SOURCE",
            "Alternate Version" => "ALTERNATIVE",
            _ => "OTHER",
        }
    }

    /// List types -> the MAL-style vocabulary the app already knows.
    fn list_type_to_generic(t: &str) -> &'static str {
        match t {
            "read" => "reading",
            "complete" => "completed",
            "unfinished" => "dropped",
            "hold" => "on_hold",
            _ => "plan_to_read", // wish
        }
    }

    /// Accepts the MAL-, AniList- and MangaUpdates-style vocabularies.
    fn list_type_from_generic(s: &str) -> CoreResult<&'static str> {
        match Self::snake(s).as_str() {
            "reading" | "current" | "rereading" | "repeating" | "read" => Ok("read"),
            "completed" | "complete" => Ok("complete"),
            "dropped" | "unfinished" => Ok("unfinished"),
            "on_hold" | "paused" | "hold" => Ok("hold"),
            "plan_to_read" | "planning" | "planned" | "wish" => Ok("wish"),
            _ => Err(core_err!(Validation, "error.tracker.invalid_status")),
        }
    }

    fn stub(id: i64, title: String) -> TrackerMedia {
        TrackerMedia {
            tracker_id: Self::to_base36(id),
            tracker_url: None,
            cross_ids: HashMap::new(),
            content_type: ContentType::Manga,
            title,
            alt_titles: vec![],
            title_i18n: HashMap::new(),
            synopsis: None,
            cover_image: None,
            banner_image: None,
            episode_count: None,
            chapter_count: None,
            status: None,
            genres: vec![],
            tags: vec![],
            nsfw: false,
            release_date: None,
            end_date: None,
            rating: None,
            trailer_url: None,
            format: None,
            studio: None,
            characters: vec![],
            staff: vec![],
            relations: vec![],
            episode_duration: None,
        }
    }

    /// Works for both the full series model and the leaner search/list one.
    fn series_to_tracker_media(s: &MuSeries) -> TrackerMedia {
        let genres: Vec<String> = s.genres.iter().map(|g| g.genre.clone()).collect();
        let nsfw = genres.iter().any(|g| ADULT_GENRES.contains(&g.as_str()));

        // Categories are user-voted; keep the ones the community agrees on.
        let mut cats: Vec<&MuCategory> = s.categories.iter().filter(|c| c.votes.unwrap_or(0) > 0).collect();
        cats.sort_by(|a, b| b.votes.unwrap_or(0).cmp(&a.votes.unwrap_or(0)));
        let tags = cats.into_iter().take(30).map(|c| c.category.clone()).collect();

        let mut staff: Vec<StaffMember> = Vec::new();
        for a in &s.authors {
            let role = a.kind.clone().unwrap_or_else(|| "Author".into());
            match staff.iter_mut().find(|m| m.name == a.name) {
                Some(m) if m.role != role => m.role = "Author & Artist".into(),
                Some(_) => {}
                None => staff.push(StaffMember { name: a.name.clone(), role, image: None }),
            }
        }

        let relations = s
            .related_series
            .iter()
            .filter_map(|r| {
                let id = r.related_series_id?;
                Some(TrackerRelation {
                    relation_type: Self::relation_type(r.relation_type.as_deref().unwrap_or("")).to_string(),
                    media: Self::stub(id, r.related_series_name.clone().unwrap_or_default()),
                })
            })
            .collect();

        let status = Self::parse_status(s.status.as_deref(), s.completed);
        let mut title_i18n = HashMap::new();
        title_i18n.insert("romaji".to_string(), s.title.clone());

        TrackerMedia {
            tracker_id: Self::to_base36(s.series_id),
            tracker_url: s.url.clone(),
            cross_ids: HashMap::new(),
            content_type: Self::content_type_of(s.kind.as_deref()),
            title: s.title.clone(),
            alt_titles: s
                .associated
                .iter()
                .map(|a| a.title.clone())
                .filter(|t| !t.is_empty() && *t != s.title)
                .collect(),
            title_i18n,
            synopsis: s.description.as_deref().and_then(Self::clean_description),
            cover_image: s.image.as_ref().and_then(|i| i.url.as_ref()).and_then(|u| u.original.clone()),
            banner_image: None,
            episode_count: None,
            // The latest chapter is only a total once the series is finished.
            chapter_count: s
                .latest_chapter
                .filter(|_| status.as_deref() == Some("completed"))
                .map(|c| c as i32),
            status,
            genres,
            tags,
            nsfw,
            release_date: s.year.as_deref().map(str::trim).filter(|y| y.len() == 4).map(|y| format!("{y}-01-01")),
            end_date: None,
            rating: s.bayesian_rating.filter(|r| *r > 0.0).map(|r| r as f32),
            trailer_url: None,
            format: s.kind.as_deref().map(Self::snake),
            studio: None,
            characters: vec![],
            staff,
            relations,
            episode_duration: None,
        }
    }

    async fn lists(&self, token: &str) -> CoreResult<Vec<MuList>> {
        self.get_json(self.client.get(format!("{API}/lists")).bearer_auth(token)).await
    }

    /// The user's built-in list of a type (custom lists also carry a type; prefer the built-in one).
    fn list_id_for(lists: &[MuList], kind: &str) -> CoreResult<i64> {
        lists
            .iter()
            .filter(|l| l.kind.as_deref() == Some(kind))
            .min_by_key(|l| l.custom.unwrap_or(false))
            .map(|l| l.list_id)
            .ok_or_else(|| core_err!(NotFound, "error.tracker.not_found", kind))
    }

    async fn list_entry(&self, token: &str, id: i64) -> CoreResult<Option<MuListRecord>> {
        let req = self.client.get(format!("{API}/lists/series/{id}")).bearer_auth(token);
        let res = self.send(req).await?;
        if res.status() == StatusCode::NOT_FOUND {
            return Ok(None);
        }
        Self::parse(Self::ensure_ok(res).await?).await.map(Some)
    }
}

#[async_trait]
impl TrackerProvider for MangaUpdatesProvider {
    fn name(&self) -> &'static str {
        "mangaupdates"
    }
    fn display_name(&self) -> &'static str {
        "MangaUpdates"
    }
    fn icon_url(&self) -> &'static str {
        "https://www.google.com/s2/favicons?domain=mangaupdates.com&sz=64"
    }

    fn rate_limit(&self) -> RateLimit {
        // No published limit; their policy asks for spaced-out requests.
        RateLimit { min_interval: Duration::from_millis(1000) }
    }

    fn supported_types(&self) -> Vec<ContentType> {
        vec![ContentType::Manga, ContentType::Novel]
    }

    fn auth_config(&self) -> TrackerAuthConfig {
        TrackerAuthConfig {
            oauth_flow: "password".to_string(),
            auth_url: format!("{API}/account/login"),
            token_url: None,
            client_id: None,
            scopes: vec![],
        }
    }

    async fn password_login(&self, username: &str, password: &str) -> CoreResult<Option<String>> {
        let req = self
            .client
            .put(format!("{API}/account/login"))
            .json(&json!({ "username": username, "password": password }));
        let res = self.send(req).await?;
        if matches!(res.status(), StatusCode::UNAUTHORIZED | StatusCode::BAD_REQUEST) {
            return Err(core_err!(AuthError, "error.tracker.invalid_credentials"));
        }
        let login: MuLogin = Self::parse(Self::ensure_ok(res).await?).await?;
        login
            .context
            .and_then(|c| c.session_token)
            .filter(|t| !t.is_empty())
            .map(Some)
            .ok_or_else(|| core_err!(AuthError, "error.tracker.invalid_credentials", login.reason))
    }

    async fn validate_and_store_token(&self, access_token: &str, token_type: &str) -> CoreResult<TokenData> {
        let p: MuProfile = self
            .get_json(self.client.get(format!("{API}/account/profile")).bearer_auth(access_token))
            .await?;

        Ok(TokenData {
            access_token: access_token.to_string(),
            refresh_token: None,
            token_type: token_type.to_string(),
            // No documented expiry or refresh; a 401 later sends the user to reconnect.
            expires_at: Utc::now()
                .checked_add_signed(chrono::Duration::days(365))
                .unwrap_or_else(Utc::now)
                .to_rfc3339(),
            tracker_user_id: p.user_id.to_string(),
            display_name: Some(p.username),
            avatar_url: p.avatar.and_then(|a| a.url),
            profile_url: p.url,
            score_format: Some("POINT_10_DECIMAL".to_string()),
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
        tags: Option<&[String]>,
    ) -> CoreResult<Vec<TrackerMedia>> {
        if matches!(content_type, ContentType::Anime) {
            return Ok(vec![]);
        }
        let mut body = serde_json::Map::new();
        body.insert("page".into(), json!(page.max(1)));
        body.insert("perpage".into(), json!(limit.clamp(1, 100)));

        let query = query.map(str::trim).filter(|s| !s.is_empty());
        if let Some(q) = query {
            body.insert("search".into(), json!(q));
        }
        match sort.and_then(Self::map_sort) {
            Some(o) => {
                body.insert("orderby".into(), json!(o));
            }
            // Browsing without a query: most-read first instead of alphabetical.
            None if query.is_none() => {
                body.insert("orderby".into(), json!("list_reading"));
            }
            None => {}
        }

        let types: Vec<&str> = match format.and_then(Self::type_from_format) {
            Some(t) => vec![t],
            None if matches!(content_type, ContentType::Novel) => vec!["Novel"],
            None => COMIC_TYPES.to_vec(),
        };
        body.insert("type".into(), json!(types));

        let mut genres: Vec<String> = genre
            .map(|g| g.split(',').map(str::trim).filter(|g| !g.is_empty()).map(String::from).collect())
            .unwrap_or_default();
        // "NSFW only" when true, otherwise hide adult content (same as the other providers).
        if nsfw == Some(true) {
            genres.push("Adult".into());
        } else {
            body.insert("exclude_genre".into(), json!(ADULT_GENRES));
        }
        if !genres.is_empty() {
            body.insert("genre".into(), json!(genres));
        }
        if let Some(t) = tags.filter(|t| !t.is_empty()) {
            body.insert("category".into(), json!(t));
        }
        // The only release-state filter the API has.
        if status.map(Self::snake).as_deref().is_some_and(|s| matches!(s, "completed" | "finished" | "complete")) {
            body.insert("filters".into(), json!(["completed"]));
        }

        let req = self.client.post(format!("{API}/series/search")).json(&Value::Object(body));
        let res: MuSearch = self.get_json(req).await?;
        Ok(res
            .results
            .iter()
            .take(limit.max(1))
            .map(|h| Self::series_to_tracker_media(&h.record))
            .collect())
    }

    async fn get_by_id(&self, tracker_id: &str) -> CoreResult<Option<TrackerMedia>> {
        let id = Self::parse_id(tracker_id)?;
        let res = self.send(self.client.get(format!("{API}/series/{id}"))).await?;
        if res.status() == StatusCode::NOT_FOUND {
            return Ok(None);
        }
        let series: MuSeries = Self::parse(Self::ensure_ok(res).await?).await?;
        Ok(Some(Self::series_to_tracker_media(&series)))
    }

    async fn get_user_list(
        &self,
        access_token: &str,
        _tracker_user_id: &str,
        _score_format: Option<&str>, // ratings are always 1-10 with decimals
    ) -> CoreResult<Vec<UserListEntry>> {
        let mut out = Vec::new();
        let mut logged_shape = false;
        for list in self.lists(access_token).await? {
            let status = Self::list_type_to_generic(list.kind.as_deref().unwrap_or("wish"));
            for page in 1..=1_000usize {
                let req = self
                    .client
                    .post(format!("{API}/lists/{}/search", list.list_id))
                    .bearer_auth(access_token)
                    .json(&json!({ "page": page, "perpage": LIST_PAGE }));
                let res: MuListSearch = self.get_json(req).await?;
                let fetched = res.results.len();

                for hit in res.results {
                    let rec = hit.record;
                    let meta = hit.metadata.unwrap_or_default();
                    let record_title = rec.series.title.clone().unwrap_or_default();
                    let parsed = meta.series.as_ref().and_then(|v| match serde_json::from_value::<MuSeries>(v.clone()) {
                        Ok(s) => Some(s),
                        Err(e) => {
                            if !logged_shape {
                                logged_shape = true;
                                let raw: String = v.to_string().chars().take(600).collect();
                                warn!(error = %e, raw = %raw, "Unexpected list series shape, importing from the list record");
                            }
                            None
                        }
                    });
                    let mut media = match &parsed {
                        Some(s) => Self::series_to_tracker_media(s),
                        None => Self::stub(rec.series.id, record_title.clone()),
                    };
                    // The list record is the reliable part: id, title and url come from it.
                    media.tracker_id = Self::to_base36(rec.series.id);
                    if media.title.is_empty() {
                        media.title = record_title.clone();
                    }
                    if media.tracker_url.is_none() {
                        media.tracker_url = rec.series.url.clone();
                    }
                    out.push(UserListEntry {
                        tracker_media_id: Self::to_base36(rec.series.id),
                        title: rec.series.title.clone().unwrap_or_else(|| media.title.clone()),
                        poster: media.cover_image.clone(),
                        content_type: media.content_type.clone(),
                        format: media.format.clone(),
                        status: Some(status.to_string()),
                        progress: rec.status.and_then(|s| s.chapter).unwrap_or(0) as i32,
                        score: meta.user_rating.filter(|r| *r > 0.0),
                        start_date: None,
                        end_date: None,
                        repeat_count: 0,
                        notes: None,
                        is_private: false,
                        total_episodes: None,
                        total_chapters: media.chapter_count,
                        media: Some(media),
                    });
                }

                let total = res.total_hits.unwrap_or(0) as usize;
                if fetched == 0 || page * res.per_page.unwrap_or(LIST_PAGE as i64).max(1) as usize >= total {
                    break;
                }
            }
        }
        Ok(out)
    }

    async fn update_entry(&self, access_token: &str, params: UpdateEntryParams) -> CoreResult<()> {
        let id = Self::parse_id(&params.media_id)?;
        let existing = self.list_entry(access_token, id).await?;

        if params.status.is_some() || params.progress.is_some() || existing.is_none() {
            let list_id = match (&params.status, &existing) {
                (Some(st), _) => {
                    let kind = Self::list_type_from_generic(st)?;
                    Self::list_id_for(&self.lists(access_token).await?, kind)?
                }
                (None, Some(e)) => e.list_id.unwrap_or(0),
                (None, None) => Self::list_id_for(&self.lists(access_token).await?, "read")?,
            };

            let mut item = json!({ "series": { "id": id }, "list_id": list_id });
            if let Some(p) = params.progress {
                item["status"] = json!({ "chapter": p.clamp(0, 100_000) });
            }
            let path = if existing.is_some() { "lists/series/update" } else { "lists/series" };
            let req = self
                .client
                .post(format!("{API}/{path}"))
                .bearer_auth(access_token)
                .json(&json!([item]));
            Self::ensure_ok(self.send(req).await?).await?;
        }

        if let Some(score) = params.score {
            let url = format!("{API}/series/{id}/rating");
            let req = if score > 0.0 {
                // Ratings are 1-10 with one decimal; the app uses 0-10.
                let rating = (score.clamp(1.0, 10.0) * 10.0).round() / 10.0;
                self.client.put(url).bearer_auth(access_token).json(&json!({ "rating": rating }))
            } else {
                self.client.delete(url).bearer_auth(access_token)
            };
            let res = self.send(req).await?;
            // Clearing a rating that was never set is fine.
            if !(score <= 0.0 && res.status() == StatusCode::NOT_FOUND) {
                Self::ensure_ok(res).await?;
            }
        }
        Ok(())
    }

    async fn delete_entry(&self, access_token: &str, media_id: &str) -> CoreResult<bool> {
        let id = Self::parse_id(media_id)?;
        if self.list_entry(access_token, id).await?.is_none() {
            return Ok(false);
        }
        let req = self
            .client
            .post(format!("{API}/lists/series/delete"))
            .bearer_auth(access_token)
            .json(&json!([id]));
        Self::ensure_ok(self.send(req).await?).await?;
        Ok(true)
    }

    fn to_core_metadata(&self, cid: &str, media: &TrackerMedia) -> Metadata {
        let now = Utc::now().timestamp();
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
            eps_or_chapters: EpisodeData::Count(media.chapter_count.unwrap_or(0)),
            status: media.status.as_deref().map(Self::normalize_status),
            genres: media.genres.clone(),
            release_date: media.release_date.clone(),
            end_date: media.end_date.clone(),
            rating: media.rating,
            trailer_url: None,
            characters: media.characters.clone(),
            studio: None,
            staff: media.staff.clone(),
            external_ids: json!(media.cross_ids),
            episode_duration: None,
            created_at: now,
            updated_at: now,
        }
    }

    async fn refresh_score_format(&self, _access_token: &str) -> CoreResult<Option<String>> {
        Ok(Some("POINT_10_DECIMAL".to_string()))
    }
}

#[derive(Debug, Deserialize)]
struct MuLogin {
    #[serde(default)]
    reason: Option<String>,
    #[serde(default)]
    context: Option<MuLoginContext>,
}

#[derive(Debug, Deserialize)]
struct MuLoginContext {
    #[serde(default)]
    session_token: Option<String>,
}

#[derive(Debug, Deserialize)]
struct MuProfile {
    user_id: i64,
    username: String,
    #[serde(default)]
    url: Option<String>,
    #[serde(default)]
    avatar: Option<MuAvatar>,
}

#[derive(Debug, Deserialize)]
struct MuAvatar {
    #[serde(default)]
    url: Option<String>,
}

#[derive(Debug, Deserialize)]
struct MuSearch {
    #[serde(default)]
    results: Vec<MuSearchHit>,
}

#[derive(Debug, Deserialize)]
struct MuSearchHit {
    record: MuSeries,
}

#[derive(Debug, Deserialize)]
struct MuSeries {
    // The list endpoints' embedded series doesn't always match the spec; never fail on it.
    #[serde(default, alias = "id")]
    series_id: i64,
    #[serde(default)]
    title: String,
    #[serde(default)]
    url: Option<String>,
    #[serde(default)]
    associated: Vec<MuAssociated>,
    #[serde(default)]
    description: Option<String>,
    #[serde(default)]
    image: Option<MuImage>,
    #[serde(default, rename = "type")]
    kind: Option<String>,
    #[serde(default)]
    year: Option<String>,
    #[serde(default)]
    bayesian_rating: Option<f64>,
    #[serde(default)]
    genres: Vec<MuGenre>,
    #[serde(default)]
    categories: Vec<MuCategory>,
    #[serde(default)]
    latest_chapter: Option<i64>,
    #[serde(default)]
    status: Option<String>,
    #[serde(default)]
    completed: Option<bool>,
    #[serde(default)]
    related_series: Vec<MuRelated>,
    #[serde(default)]
    authors: Vec<MuAuthor>,
}

#[derive(Debug, Deserialize)]
struct MuAssociated {
    #[serde(default)]
    title: String,
}

#[derive(Debug, Deserialize)]
struct MuImage {
    #[serde(default)]
    url: Option<MuImageUrl>,
}

#[derive(Debug, Deserialize)]
struct MuImageUrl {
    #[serde(default)]
    original: Option<String>,
}

#[derive(Debug, Deserialize)]
struct MuGenre {
    genre: String,
}

#[derive(Debug, Deserialize)]
struct MuCategory {
    category: String,
    #[serde(default)]
    votes: Option<i64>,
}

#[derive(Debug, Deserialize)]
struct MuRelated {
    #[serde(default)]
    relation_type: Option<String>,
    #[serde(default)]
    related_series_id: Option<i64>,
    #[serde(default)]
    related_series_name: Option<String>,
}

#[derive(Debug, Deserialize)]
struct MuAuthor {
    name: String,
    #[serde(default, rename = "type")]
    kind: Option<String>,
}

#[derive(Debug, Deserialize)]
struct MuList {
    list_id: i64,
    #[serde(default, rename = "type")]
    kind: Option<String>,
    #[serde(default)]
    custom: Option<bool>,
}

#[derive(Debug, Deserialize)]
struct MuListSearch {
    #[serde(default)]
    total_hits: Option<i64>,
    #[serde(default)]
    per_page: Option<i64>,
    #[serde(default)]
    results: Vec<MuListHit>,
}

#[derive(Debug, Deserialize)]
struct MuListHit {
    record: MuListRecord,
    #[serde(default)]
    metadata: Option<MuListMeta>,
}

#[derive(Debug, Deserialize)]
struct MuListRecord {
    series: MuSeriesRef,
    #[serde(default)]
    list_id: Option<i64>,
    #[serde(default)]
    status: Option<MuProgress>,
}

#[derive(Debug, Deserialize)]
struct MuSeriesRef {
    id: i64,
    #[serde(default)]
    url: Option<String>,
    #[serde(default)]
    title: Option<String>,
}

#[derive(Debug, Deserialize)]
struct MuProgress {
    #[serde(default)]
    chapter: Option<i64>,
}

#[derive(Debug, Deserialize, Default)]
struct MuListMeta {
    /// Kept raw: its shape differs from the documented one, so it's parsed leniently.
    #[serde(default)]
    series: Option<Value>,
    #[serde(default)]
    user_rating: Option<f64>,
}

