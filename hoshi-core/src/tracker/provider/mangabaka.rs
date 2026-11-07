use super::{
    TagInfo, TokenData, TrackerAuthConfig, TrackerMedia, TrackerProvider, UpdateEntryParams,
    UserListEntry,
};
use crate::content::models::{ContentType, EpisodeData, Metadata, StaffMember, Status};
use crate::core_err;
use crate::error::CoreResult;
use async_trait::async_trait;
use chrono::Utc;
use reqwest::{Client, RequestBuilder, Response, StatusCode};
use serde::de::DeserializeOwned;
use serde::Deserialize;
use serde_json::{json, Value};
use std::collections::HashMap;
use std::time::Duration;
use tokio::sync::OnceCell;

const API: &str = "https://api.mangabaka.org";
const OAUTH: &str = "https://mangabaka.org/auth/oauth2";
const MANGABAKA_CLIENT_ID: &str = "bFipuRxaCmYJFTPeEmbqcuBoDXdhHINI";
const SEARCH_WINDOW: usize = 12_000;
const MEDIA_TYPES: [&str; 6] = ["manga", "novel", "manhwa", "manhua", "oel", "other"];
const RATING_DIVISOR: f64 = 10.0;

pub struct MangaBakaProvider {
    client: Client,
    tags: OnceCell<Vec<MbTag>>,
}

impl MangaBakaProvider {
    pub fn new(client: Client) -> Self {
        Self {
            client,
            tags: OnceCell::new(),
        }
    }

    /// Sends a request; on a gateway 429 waits for `Retry-After` (max 15s) and retries once.
    async fn send(&self, req: RequestBuilder) -> CoreResult<Response> {
        let retry = req.try_clone();
        let mut res = req
            .send()
            .await
            .map_err(|e| core_err!(Network, "error.tracker.request_failed", e))?;

        if res.status() == StatusCode::TOO_MANY_REQUESTS {
            if let Some(retry) = retry {
                let wait = res
                    .headers()
                    .get(reqwest::header::RETRY_AFTER)
                    .and_then(|v| v.to_str().ok())
                    .and_then(|v| v.parse::<u64>().ok())
                    .unwrap_or(10)
                    .min(15);
                tokio::time::sleep(Duration::from_secs(wait)).await;
                res = retry
                    .send()
                    .await
                    .map_err(|e| core_err!(Network, "error.tracker.request_failed", e))?;
            }
        }
        Ok(res)
    }

    fn ensure_ok(res: Response) -> CoreResult<Response> {
        let status = res.status();
        if status.is_success() {
            return Ok(res);
        }
        Err(match status {
            StatusCode::UNAUTHORIZED => core_err!(AuthError, "error.tracker.invalid_credentials"),
            StatusCode::FORBIDDEN => core_err!(AuthError, "error.tracker.insufficient_scope"),
            StatusCode::NOT_FOUND => core_err!(NotFound, "error.tracker.not_found"),
            StatusCode::TOO_MANY_REQUESTS => core_err!(Network, "error.tracker.rate_limited"),
            StatusCode::BAD_REQUEST => {
                core_err!(BadRequest, "error.tracker.bad_request", status.as_u16())
            }
            _ => core_err!(Network, "error.tracker.http_error", status.as_u16()),
        })
    }

    async fn parse<T: DeserializeOwned>(res: Response) -> CoreResult<T> {
        res.json::<T>()
            .await
            .map_err(|e| core_err!(Parse, "error.tracker.parse_failed", e))
    }

    async fn tag_table(&self) -> CoreResult<&Vec<MbTag>> {
        self.tags
            .get_or_try_init(|| async {
                let res =
                    Self::ensure_ok(self.send(self.client.get(format!("{API}/v1/tags"))).await?)?;
                let list: MbList<MbTag> = Self::parse(res).await?;
                let tags: Vec<MbTag> = list
                    .data
                    .into_iter()
                    .filter(|t| t.merged_with.is_none())
                    .collect();
                CoreResult::Ok(tags)
            })
            .await
    }

    /// Resolves names (or numeric ids) to tag ids. A tag filter in MangaBaka
    /// also matches the tag's whole subtree.
    fn resolve_tag_ids<'a>(
        table: &[MbTag],
        names: impl Iterator<Item = &'a str>,
        prefer_genre: bool,
    ) -> Vec<i64> {
        let mut ids = Vec::new();
        for n in names.map(str::trim).filter(|n| !n.is_empty()) {
            if n.chars().all(|c| c.is_ascii_digit()) {
                if let Ok(id) = n.parse::<i64>() {
                    ids.push(id);
                }
                continue;
            }
            let matches =
                |t: &&MbTag| t.name.eq_ignore_ascii_case(n) || t.name_path.eq_ignore_ascii_case(n);
            let found = table
                .iter()
                .filter(|t| !prefer_genre || t.is_genre)
                .find(matches)
                .or_else(|| table.iter().find(matches));
            if let Some(t) = found {
                ids.push(t.id);
            }
        }
        ids
    }

    // ───────────────────────────── Mapping ──────────────────────────────────

    fn snake(s: &str) -> String {
        s.trim().to_lowercase().replace(['-', ' '], "_")
    }

    fn content_type_of(media_type: Option<&str>) -> ContentType {
        if media_type == Some("novel") {
            ContentType::Novel
        } else {
            ContentType::Manga
        }
    }

    fn map_format(f: &str) -> String {
        match Self::snake(f).as_str() {
            "light_novel" | "lightnovel" | "ln" => "novel".to_string(),
            other => other.to_string(),
        }
    }

    fn map_sort(sort: &str) -> Option<&'static str> {
        match Self::snake(sort).as_str() {
            "popularity" | "popular" | "rank" => Some("popularity_asc"),
            "trending" => Some("trending_7d"),
            "score" | "rating" | "average_score" | "top" => Some("score_desc"),
            "newest" | "start_date" | "release_date" => Some("published_start_date_desc"),
            "latest" => Some("latest"),
            "relevance" => Some("relevance_desc"),
            _ => None,
        }
    }

    fn map_search_status(s: &str) -> Option<&'static str> {
        match Self::snake(s).as_str() {
            "finished" | "completed" | "complete" | "finished_airing" => Some("completed"),
            "releasing"
            | "airing"
            | "currently_airing"
            | "ongoing"
            | "publishing"
            | "currently_publishing" => Some("releasing"),
            "not_yet_released" | "not_yet_aired" | "not_yet_published" | "upcoming" | "planned" => {
                Some("upcoming")
            }
            "hiatus" | "on_hiatus" => Some("hiatus"),
            "cancelled" | "discontinued" => Some("cancelled"),
            _ => None,
        }
    }

    /// MangaBaka states -> the MAL-style vocabulary the app already knows.
    fn state_to_generic(state: &str) -> &'static str {
        match state {
            "reading" | "rereading" => "reading",
            "completed" => "completed",
            "dropped" => "dropped",
            "paused" => "on_hold",
            _ => "plan_to_read", // plan_to_read, considering
        }
    }

    /// Accepts both the MAL-style and MangaBaka-native vocabularies.
    fn state_from_generic(s: &str) -> CoreResult<&'static str> {
        match Self::snake(s).as_str() {
            "reading" | "current" => Ok("reading"),
            "rereading" | "repeating" => Ok("rereading"),
            "completed" => Ok("completed"),
            "dropped" => Ok("dropped"),
            "on_hold" | "paused" => Ok("paused"),
            "plan_to_read" | "planning" | "planned" => Ok("plan_to_read"),
            "considering" => Ok("considering"),
            _ => Err(core_err!(Validation, "error.tracker.invalid_status")),
        }
    }

    fn normalize_status(s: &str) -> Status {
        match s {
            "completed" => Status::Completed,
            "upcoming" => Status::Planned,
            // Cancelled: no more releases, closest to "done".
            "cancelled" => Status::Completed,
            // releasing, hiatus, unknown (same default as the MAL provider).
            _ => Status::Ongoing,
        }
    }

    fn date10(s: Option<String>) -> Option<String> {
        s.filter(|s| !s.trim().is_empty())
            .map(|s| s.get(..10).unwrap_or(&s).to_string())
    }

    fn entry_id(e: &Option<MbSourceEntry>) -> Option<String> {
        match e.as_ref()?.id.as_ref()? {
            Value::Number(n) => n
                .as_i64()
                .or_else(|| n.as_f64().map(|f| f as i64))
                .map(|n| n.to_string()),
            Value::String(s) if !s.is_empty() => Some(s.clone()),
            _ => None,
        }
    }

    /// Prefers the primary title, then an official one, then the first.
    fn pick_title(titles: &[MbTitle], langs: &[String]) -> Option<String> {
        for lang in langs {
            let cands: Vec<&MbTitle> = titles
                .iter()
                .filter(|t| t.language.eq_ignore_ascii_case(lang) && !t.title.is_empty())
                .collect();
            let best = cands
                .iter()
                .find(|t| t.is_primary == Some(true))
                .or_else(|| {
                    cands
                        .iter()
                        .find(|t| t.traits.iter().any(|x| x == "official"))
                })
                .or_else(|| cands.first());
            if let Some(t) = best {
                return Some(t.title.clone());
            }
        }
        None
    }

    /// Works for both the lean and the `schema=full` series shapes.
    fn series_to_tracker_media(s: &MbSeries) -> TrackerMedia {
        let titles = s.titles.as_deref().unwrap_or(&[]);
        let orig = s
            .original_language
            .clone()
            .unwrap_or_else(|| "ja".to_string());
        let base = orig.split('-').next().unwrap_or("ja").to_string();

        let mut native_langs = vec![orig.clone()];
        native_langs.extend(["ja", "ko", "zh-Hans", "zh-Hant", "zh"].map(String::from));
        let mut romaji_langs = vec![format!("{base}-Latn")];
        romaji_langs.extend(["ja-Latn", "ko-Latn", "zh-Latn", "yue-Latn"].map(String::from));

        let english = Self::pick_title(titles, &["en".to_string()]);
        let native = Self::pick_title(titles, &native_langs);
        let romaji = Self::pick_title(titles, &romaji_langs);

        let title = romaji
            .clone()
            .or_else(|| english.clone())
            .or_else(|| native.clone())
            .or_else(|| titles.first().map(|t| t.title.clone()))
            .unwrap_or_else(|| format!("MangaBaka #{}", s.id));

        let mut title_i18n = HashMap::new();
        title_i18n.insert("romaji".to_string(), title.clone());
        if let Some(en) = english {
            title_i18n.insert("english".to_string(), en);
        }
        if let Some(n) = native {
            title_i18n.insert("native".to_string(), n);
        }

        let mut alt_titles: Vec<String> = Vec::new();
        for t in titles {
            if !t.title.is_empty() && t.title != title && !alt_titles.contains(&t.title) {
                alt_titles.push(t.title.clone());
            }
        }

        let (mut genres, mut tags) = (Vec::new(), Vec::new());
        for t in s.tags.as_deref().unwrap_or(&[]) {
            if t.is_spoiler == Some(true) {
                continue;
            }
            if t.is_genre {
                genres.push(t.name.clone())
            } else {
                tags.push(t.name.clone())
            }
        }

        let mut staff: Vec<StaffMember> = Vec::new();
        for a in s.authors.as_deref().unwrap_or(&[]) {
            staff.push(StaffMember {
                name: a.clone(),
                role: "Author".into(),
                image: None,
            });
        }
        for a in s.artists.as_deref().unwrap_or(&[]) {
            match staff.iter_mut().find(|m| &m.name == a) {
                Some(m) => m.role = "Author & Artist".into(),
                None => staff.push(StaffMember {
                    name: a.clone(),
                    role: "Artist".into(),
                    image: None,
                }),
            }
        }

        let mut cross_ids = HashMap::new();
        if let Some(src) = &s.source {
            if let Some(id) = Self::entry_id(&src.anilist) {
                cross_ids.insert("anilist".to_string(), id);
            }
            if let Some(id) = Self::entry_id(&src.my_anime_list) {
                cross_ids.insert("mal".to_string(), format!("manga:{id}"));
            }
            if let Some(id) = Self::entry_id(&src.kitsu) {
                cross_ids.insert("kitsu".to_string(), id);
            }
            if let Some(id) = Self::entry_id(&src.manga_updates) {
                cross_ids.insert("mangaupdates".to_string(), id);
            }
            if let Some(id) = Self::entry_id(&src.anime_planet) {
                cross_ids.insert("animeplanet".to_string(), id);
            }
        }

        let cover = s
            .cover
            .as_ref()
            .and_then(|c| c.x350.clone().or_else(|| c.x250.clone()));

        TrackerMedia {
            tracker_id: s.id.to_string(),
            tracker_url: s.canonical_url.clone(),
            cross_ids,
            content_type: Self::content_type_of(s.media_type.as_deref()),
            title,
            alt_titles,
            title_i18n,
            synopsis: s.description.clone(),
            cover_image: cover,
            banner_image: None,
            episode_count: None,
            chapter_count: s.total_chapters.map(|c| c.round() as i32),
            status: s.status.clone(),
            genres,
            tags,
            nsfw: matches!(
                s.content_rating.as_deref(),
                Some("erotica") | Some("pornographic")
            ),
            release_date: s.published.as_ref().and_then(|p| p.start_date.clone()),
            end_date: s.published.as_ref().and_then(|p| p.end_date.clone()),
            rating: s.rating.map(|r| (r / RATING_DIVISOR) as f32),
            trailer_url: None,
            format: s.media_type.clone(),
            studio: None,
            characters: vec![],
            staff,
            relations: vec![], // would need an extra batch call
            episode_duration: None,
        }
    }

    fn item_to_entry(item: MbLibraryItem) -> UserListEntry {
        let e = item.entry;
        let tm = Self::series_to_tracker_media(&item.series);

        UserListEntry {
            tracker_media_id: tm.tracker_id.clone(),
            title: tm.title.clone(),
            poster: tm.cover_image.clone(),
            content_type: tm.content_type.clone(),
            format: tm.format.clone(),
            status: e
                .state
                .as_deref()
                .map(|s| Self::state_to_generic(s).to_string()),
            progress: e.progress_chapter.map(|p| p as i32).unwrap_or(0),
            // Tracker stores 0-100; the app uses 0-10.
            score: e.rating.map(|r| r / RATING_DIVISOR),
            start_date: Self::date10(e.start_date),
            end_date: Self::date10(e.finish_date),
            repeat_count: e.number_of_rereads.map(|n| n as i32).unwrap_or(0),
            notes: e.note.filter(|n| !n.is_empty()),
            is_private: e.is_private.unwrap_or(false),
            total_episodes: None,
            total_chapters: tm.chapter_count,
            media: Some(tm),
        }
    }
}

#[async_trait]
impl TrackerProvider for MangaBakaProvider {
    fn name(&self) -> &'static str {
        "mangabaka"
    }
    fn display_name(&self) -> &'static str {
        "MangaBaka"
    }
    fn icon_url(&self) -> &'static str {
        "https://mangabaka.org/images/logo.png"
    }

    fn supported_types(&self) -> Vec<ContentType> {
        vec![ContentType::Manga, ContentType::Novel]
    }

    fn auth_config(&self) -> TrackerAuthConfig {
        TrackerAuthConfig {
            oauth_flow: "pkce".to_string(),
            auth_url: format!("{OAUTH}/authorize"),
            token_url: Some(format!("{OAUTH}/token")),
            client_id: Some(MANGABAKA_CLIENT_ID.to_string()),
            scopes: vec![
                "openid".to_string(),
                "profile".to_string(),
                "library.read".to_string(),
                "offline_access".to_string(),
            ],
        }
    }

    async fn validate_and_store_token(
        &self,
        access_token: &str,
        token_type: &str,
    ) -> CoreResult<TokenData> {
        let req = self
            .client
            .get(format!("{API}/v1/my/profile"))
            .bearer_auth(access_token);
        let res = Self::ensure_ok(self.send(req).await?)?;
        let profile: MbEnvelope<MbProfile> = Self::parse(res).await?;
        let p = profile.data;

        Ok(TokenData {
            access_token: access_token.to_string(),
            refresh_token: None,
            token_type: token_type.to_string(),
            expires_at: Utc::now()
                .checked_add_signed(chrono::Duration::days(7))
                .unwrap_or_else(Utc::now)
                .to_rfc3339(),
            tracker_user_id: p.id,
            display_name: p.nickname.or(p.preferred_username),
            avatar_url: p.picture,
            profile_url: None,
            score_format: Some("POINT_100".to_string()),
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
        let limit = limit.clamp(1, 100);
        let page = page.max(1);
        if page * limit > SEARCH_WINDOW {
            return Ok(vec![]);
        }

        let mut q: Vec<(&str, String)> =
            vec![("limit", limit.to_string()), ("page", page.to_string())];

        let has_query = match query.map(str::trim).filter(|s| !s.is_empty()) {
            Some(s) => {
                q.push(("q", s.to_string()));
                true
            }
            None => false,
        };

        match sort.and_then(Self::map_sort) {
            Some(s) => q.push(("sort_by", s.to_string())),
            // Browsing without a query: most popular first instead of arbitrary order.
            None if !has_query => q.push(("sort_by", "popularity_asc".to_string())),
            None => {}
        }

        let explicit_type = format
            .map(Self::map_format)
            .filter(|f| MEDIA_TYPES.contains(&f.as_str()));
        match explicit_type {
            Some(f) => q.push(("type", f)),
            None if matches!(content_type, ContentType::Novel) => {
                q.push(("type", "novel".to_string()))
            }
            None => q.push(("type_not", "novel".to_string())),
        }

        if let Some(s) = status.and_then(Self::map_search_status) {
            q.push(("status", s.to_string()));
        }

        // "NSFW only" when true, otherwise hide adult content (same as the MAL provider).
        if nsfw == Some(true) {
            q.push(("content_rating", "erotica".to_string()));
            q.push(("content_rating", "pornographic".to_string()));
        } else {
            q.push(("not_content_rating", "erotica".to_string()));
            q.push(("not_content_rating", "pornographic".to_string()));
        }

        let wants_genre = genre.map_or(false, |g| !g.trim().is_empty());
        let wants_tags = tags.map_or(false, |t| !t.is_empty());
        if wants_genre || wants_tags {
            // If the tag tree can't be fetched, search unfiltered rather than failing.
            if let Ok(table) = self.tag_table().await {
                let mut ids: Vec<i64> = Vec::new();
                if let Some(g) = genre {
                    ids.extend(Self::resolve_tag_ids(table, g.split(','), true));
                }
                if let Some(t) = tags {
                    ids.extend(Self::resolve_tag_ids(
                        table,
                        t.iter().map(String::as_str),
                        false,
                    ));
                }
                ids.sort_unstable();
                ids.dedup();
                q.extend(ids.into_iter().map(|id| ("tag", id.to_string())));
            }
        }

        let req = self.client.get(format!("{API}/v2/series/search")).query(&q);
        let res = Self::ensure_ok(self.send(req).await?)?;
        let list: MbList<MbSeries> = Self::parse(res).await?;

        Ok(list
            .data
            .iter()
            .filter(|s| s.state.as_deref().map_or(true, |st| st == "active"))
            .map(Self::series_to_tracker_media)
            .collect())
    }

    fn supports_tags(&self) -> bool {
        true
    }

    async fn list_tags(&self) -> CoreResult<Vec<TagInfo>> {
        let table = self.tag_table().await?;

        let mut counts: HashMap<String, usize> = HashMap::new();
        for t in table.iter() {
            *counts.entry(t.name.to_lowercase()).or_default() += 1;
        }

        let mut seen = std::collections::HashSet::new();
        let mut out = Vec::with_capacity(table.len());
        for t in table.iter() {
            let name = if counts.get(&t.name.to_lowercase()).copied().unwrap_or(0) > 1
                && !t.name_path.is_empty()
            {
                t.name_path.clone()
            } else {
                t.name.clone()
            };
            if !seen.insert(name.to_lowercase()) {
                continue;
            }
            out.push(TagInfo {
                name,
                category: t
                    .name_path
                    .split_once(" > ")
                    .map(|(root, _)| root.to_string()),
                description: t.description.clone().filter(|d| !d.is_empty()),
                is_adult: matches!(
                    t.content_rating.as_deref(),
                    Some("erotica") | Some("pornographic")
                ),
            });
        }
        Ok(out)
    }

    async fn get_by_id(&self, tracker_id: &str) -> CoreResult<Option<TrackerMedia>> {
        let raw = tracker_id.rsplit(':').next().unwrap_or(tracker_id);
        let mut id: i64 = raw
            .trim()
            .parse()
            .map_err(|e| core_err!(BadRequest, "error.tracker.invalid_id", e))?;

        // Follow merges (bounded) so stale ids keep resolving.
        for _ in 0..3 {
            let req = self
                .client
                .get(format!("{API}/v2/series/{id}"))
                .query(&[("schema", "full")]);
            let res = self.send(req).await?;
            if res.status() == StatusCode::NOT_FOUND {
                return Ok(None);
            }
            let res = Self::ensure_ok(res)?;
            let series = Self::parse::<MbEnvelope<MbSeries>>(res).await?.data;

            match series.state.as_deref() {
                Some("merged") => match series.merged_with {
                    Some(next) => id = next as i64,
                    None => return Ok(None),
                },
                Some("deleted") => return Ok(None),
                _ => return Ok(Some(Self::series_to_tracker_media(&series))),
            }
        }
        Ok(None)
    }


    async fn get_user_list(
        &self,
        access_token: &str,
        _tracker_user_id: &str,
        _score_format: Option<&str>, // scores are always POINT_100
    ) -> CoreResult<Vec<UserListEntry>> {
        let mut out = Vec::new();
        let mut page = 1u32;

        loop {
            let req = self
                .client
                .get(format!("{API}/v2/my/library"))
                .bearer_auth(access_token)
                .query(&[
                    ("page", page.to_string()),
                    ("limit", "100".to_string()),
                    ("schema", "full".to_string()),
                ]);
            let res = Self::ensure_ok(self.send(req).await?)?;
            let MbLibraryPage { data, pagination } = Self::parse(res).await?;

            out.extend(data.into_iter().map(Self::item_to_entry));

            let has_next = pagination.and_then(|p| p.next).is_some();
            if !has_next || page >= 10_000 {
                break;
            }
            page += 1;
        }
        Ok(out)
    }

    async fn update_entry(&self, access_token: &str, params: UpdateEntryParams) -> CoreResult<()> {
        let raw = params
            .media_id
            .rsplit(':')
            .next()
            .unwrap_or(&params.media_id);
        let series_id: i64 = raw
            .trim()
            .parse()
            .map_err(|e| core_err!(BadRequest, "error.tracker.invalid_id", e))?;

        let mut entry = serde_json::Map::new();
        entry.insert("series_id".into(), json!(series_id));

        if let Some(st) = &params.status {
            entry.insert("state".into(), json!(Self::state_from_generic(st)?));
        }
        if let Some(p) = params.progress {
            entry.insert("progress_chapter".into(), json!(p.clamp(0, 10_000)));
        }
        if let Some(sc) = params.score {
            // App uses 0-10; tracker expects 0-100.
            entry.insert(
                "rating".into(),
                json!((sc * RATING_DIVISOR).round().clamp(0.0, 100.0) as i64),
            );
        }
        if let Some(r) = params.repeat_count {
            entry.insert("number_of_rereads".into(), json!(r.clamp(0, 1_000)));
        }
        if let Some(n) = params.notes {
            entry.insert(
                "note".into(),
                if n.is_empty() { Value::Null } else { json!(n) },
            );
        }
        if let Some(d) = params.start_date {
            entry.insert("start_date".into(), json!(Self::date10(Some(d))));
        }
        if let Some(d) = params.end_date {
            entry.insert("finish_date".into(), json!(Self::date10(Some(d))));
        }
        if let Some(p) = params.is_private {
            entry.insert("is_private".into(), json!(p));
        }

        let req = self
            .client
            .post(format!("{API}/v1/my/library/batch"))
            .bearer_auth(access_token)
            .json(&Value::Array(vec![Value::Object(entry)]));
        Self::ensure_ok(self.send(req).await?)?;
        Ok(())
    }

    async fn delete_entry(&self, access_token: &str, media_id: &str) -> CoreResult<bool> {
        let raw = media_id.rsplit(':').next().unwrap_or(media_id);
        let series_id: i64 = raw
            .trim()
            .parse()
            .map_err(|e| core_err!(BadRequest, "error.tracker.invalid_id", e))?;

        let req = self
            .client
            .delete(format!("{API}/v1/my/library/{series_id}"))
            .bearer_auth(access_token);
        let res = self.send(req).await?;

        // Already gone counts as deleted (same as the MAL provider).
        Ok(res.status().is_success() || res.status() == StatusCode::NOT_FOUND)
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
        Ok(Some("POINT_100".to_string()))
    }
}

#[derive(Debug, Deserialize)]
struct MbEnvelope<T> {
    data: T,
}

#[derive(Debug, Deserialize)]
struct MbList<T> {
    #[serde(default)]
    data: Vec<T>,
}

#[derive(Debug, Deserialize)]
struct MbProfile {
    id: String,
    nickname: Option<String>,
    preferred_username: Option<String>,
    picture: Option<String>,
}

#[derive(Debug, Deserialize, Default)]
struct MbTag {
    id: i64,
    name: String,
    #[serde(default)]
    name_path: String,
    description: Option<String>,
    #[serde(default)]
    is_genre: bool,
    content_rating: Option<String>,
    merged_with: Option<f64>,
}

#[derive(Debug, Deserialize, Default)]
struct MbSeries {
    id: i64,
    state: Option<String>,
    merged_with: Option<f64>,
    canonical_url: Option<String>,
    authors: Option<Vec<String>>,
    artists: Option<Vec<String>>,
    description: Option<String>,
    published: Option<MbPublished>,
    status: Option<String>,
    content_rating: Option<String>,
    #[serde(rename = "type")]
    media_type: Option<String>,
    original_language: Option<String>,
    rating: Option<f64>,
    total_chapters: Option<f64>,
    titles: Option<Vec<MbTitle>>,
    cover: Option<MbCover>,
    /// Only present with `schema=full`.
    tags: Option<Vec<MbSeriesTag>>,
    source: Option<MbSource>,
}

#[derive(Debug, Deserialize)]
struct MbPublished {
    start_date: Option<String>,
    end_date: Option<String>,
}

#[derive(Debug, Deserialize)]
struct MbTitle {
    language: String,
    #[serde(default)]
    traits: Vec<String>,
    title: String,
    is_primary: Option<bool>,
}

#[derive(Debug, Deserialize)]
struct MbCover {
    x250: Option<String>,
    x350: Option<String>,
}

#[derive(Debug, Deserialize)]
struct MbSeriesTag {
    name: String,
    #[serde(default)]
    is_genre: bool,
    is_spoiler: Option<bool>,
}

#[derive(Debug, Deserialize)]
struct MbSource {
    anilist: Option<MbSourceEntry>,
    my_anime_list: Option<MbSourceEntry>,
    kitsu: Option<MbSourceEntry>,
    manga_updates: Option<MbSourceEntry>,
    anime_planet: Option<MbSourceEntry>,
}

#[derive(Debug, Deserialize)]
struct MbSourceEntry {
    id: Option<Value>,
}

#[derive(Debug, Deserialize)]
struct MbLibraryPage {
    #[serde(default)]
    data: Vec<MbLibraryItem>,
    pagination: Option<MbPagination>,
}

#[derive(Debug, Deserialize)]
struct MbPagination {
    next: Option<String>,
}

#[derive(Debug, Deserialize)]
struct MbLibraryItem {
    entry: MbEntry,
    series: MbSeries,
}

#[derive(Debug, Deserialize)]
struct MbEntry {
    note: Option<String>,
    rating: Option<f64>,
    state: Option<String>,
    is_private: Option<bool>,
    number_of_rereads: Option<f64>,
    progress_chapter: Option<f64>,
    start_date: Option<String>,
    finish_date: Option<String>,
}