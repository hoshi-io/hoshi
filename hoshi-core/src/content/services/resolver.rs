use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use sqlx::SqlitePool;
use chrono::Utc;
use tracing::{debug, error, info, instrument, warn};
use serde_json::{json, Value};

use crate::config::repository::ConfigRepository;
use crate::content::models::{ContentType, EpisodeData, ExtensionSource, FullContent, Metadata, Status};
use crate::content::repositories::content::ContentRepository;
use crate::content::repositories::extension::ExtensionRepository;
use crate::content::services::content_units::SimklUnitsService;
use crate::content::services::enrichment::EnrichmentService;
use crate::content::services::extensions::ExtensionService;
use crate::content::services::import::ImportService;
use crate::content::services::mapping::MappingService;
use crate::content::utils::{apply_year_penalty, best_title_match, generate_cid, normalize_title};
use crate::error::{CoreError, CoreResult};
use crate::extensions::types::ExtensionMetadata;
use crate::state::AppState;
use crate::tracker::provider::TrackerMedia;
use crate::tracker::repository::TrackerRepository;
use crate::tracker::types::TrackerMapping;

const TRACKER_SOURCES: &[&str] = &["anilist", "mal", "kitsu", "simkl", "mangabaka", "shikimori"];
const FUZZY_SCORE_THRESHOLD: f64 = 0.85;
const MAX_FUZZY_CANDIDATES: usize = 3;
const ACTIVE_REFRESH_SECS: i64 = 24 * 60 * 60;
const FINISHED_REFRESH_SECS: i64 = 30 * 24 * 60 * 60;

fn value_to_id(v: &Value) -> Option<String> {
    match v {
        Value::String(s) => {
            let s = s.trim();
            if s.is_empty() { None } else { Some(s.to_string()) }
        }
        Value::Number(n) => Some(n.as_i64().map(|i| i.to_string()).unwrap_or_else(|| n.to_string())),
        _ => None,
    }
}

fn mal_id_with_prefix(v: &Value, prefix: &str) -> Option<String> {
    let id = value_to_id(v)?;
    if id.starts_with(&format!("{prefix}:")) {
        Some(id)
    } else {
        Some(format!("{prefix}:{id}"))
    }
}

/// Splits a trailing "(YYYY)" off a title: "Jujutsu Kaisen (2023)" -> ("Jujutsu Kaisen", Some(2023)).
fn split_year_suffix(title: &str) -> (String, Option<i32>) {
    let t = title.trim();
    if let (true, Some(open)) = (t.ends_with(')'), t.rfind('(')) {
        let inner = &t[open + 1..t.len() - 1];
        if inner.len() == 4 && inner.chars().all(|c| c.is_ascii_digit()) {
            if let Ok(y) = inner.parse::<i32>() {
                if (1900..=2100).contains(&y) {
                    return (t[..open].trim_end().to_string(), Some(y));
                }
            }
        }
    }
    (t.to_string(), None)
}

fn is_ordinal(w: &str) -> bool {
    let w = w.to_lowercase();
    ["st", "nd", "rd", "th"].iter().any(|suf| {
        w.strip_suffix(suf).map_or(false, |n| !n.is_empty() && n.chars().all(|c| c.is_ascii_digit()))
    })
}

/// "Jujutsu Kaisen 2nd Season" / "Jujutsu Kaisen: Shimetsu Kaiyuu" -> "Jujutsu Kaisen".
/// Cuts at the first ':' and at the first season/part marker. None when nothing changed.
fn plain_title(title: &str) -> Option<String> {
    let (no_year, _) = split_year_suffix(title);
    let head = no_year.split(':').next().unwrap_or(&no_year);
    let mut kept: Vec<&str> = Vec::new();
    for w in head.split_whitespace() {
        let lw = w.to_lowercase();
        if matches!(lw.as_str(), "season" | "part" | "cour" | "final") || is_ordinal(w) {
            break;
        }
        kept.push(w);
    }
    let out = kept.join(" ");
    if out.is_empty() || out.eq_ignore_ascii_case(title.trim()) { None } else { Some(out) }
}

fn year_of(date: Option<&str>) -> Option<i32> {
    date.and_then(|d| d.get(..4)).and_then(|y| y.parse().ok())
}

pub struct ContentResolverService;

impl ContentResolverService {
    pub async fn resolve_source(
        state: &Arc<AppState>,
        source: &str,
        source_id: &str,
    ) -> CoreResult<FullContent> {
        if TRACKER_SOURCES.contains(&source) {
            Self::resolve_tracker_source(state, source, source_id).await
        } else {
            Self::resolve_extension_source(state, source, source_id).await
        }
    }

    #[instrument(skip(state))]
    pub async fn ensure_extension_link(
        state: &Arc<AppState>,
        cid: &str,
        ext_name: &str,
    ) -> CoreResult<(ContentType, String)> {
        let existing = ExtensionRepository::get_extension_id_and_type(&state.pool, cid, ext_name).await?;

        if let Some((type_str, id)) = existing {
            let ct = Self::parse_content_type(&type_str);

            let has_meta = ExtensionRepository::has_metadata(&state.pool, cid, ext_name).await?;
            if !has_meta {
                ExtensionService::save_extension_metadata(state, cid, ext_name, &id).await;
            }
            return Ok((ct, id));
        }

        let content = ContentRepository::get_content_by_cid(&state.pool, cid).await?
            .ok_or_else(|| CoreError::NotFound("error.content.not_found".into()))?;
        let meta = ContentRepository::get_by_cid(&state.pool, cid).await?
            .ok_or_else(|| CoreError::NotFound("error.content.metadata_not_found".into()))?;

        let title = meta.title;
        let ct = content.content_type;

        const MIN_SIMILARITY: f64 = 0.8;
        const MAX_FALLBACK_QUERIES: usize = 2;

        let mut all_titles: Vec<String> = Vec::with_capacity(meta.title_i18n.len() + 1);
        all_titles.push(title.clone());
        all_titles.extend(meta.title_i18n.values().cloned());
        let tracker_year = year_of(meta.release_date.as_deref());

        let mut results = Self::search_candidates(state, ext_name, &title).await?;
        let mut best = Self::pick_candidate(&results, &all_titles, tracker_year, MIN_SIMILARITY, false);

        // Pass 2 (rare, only on a miss): sites that disambiguate seasons with "(YYYY)" or
        // different subtitles. Search by plain title and let the year decide.
        if best.is_none() {
            let mut queries: Vec<String> = Vec::new();
            for t in &all_titles {
                if let Some(q) = plain_title(t) {
                    if !queries.contains(&q) && !q.eq_ignore_ascii_case(&title) {
                        queries.push(q);
                    }
                }
            }
            queries.truncate(MAX_FALLBACK_QUERIES);

            if !queries.is_empty() {
                info!(ext = %ext_name, ?queries, "No match on full title, retrying with plain title");
                let q0 = queries.first().cloned();
                let q1 = queries.get(1).cloned();
                let run = |q: Option<String>| async move {
                    match q {
                        Some(q) => Self::search_candidates(state, ext_name, &q).await.unwrap_or_else(|e| {
                            warn!(ext = %ext_name, error = ?e, "Fallback search failed");
                            Vec::new()
                        }),
                        None => Vec::new(),
                    }
                };
                let (r0, r1) = tokio::join!(run(q0), run(q1));
                for (id, t, y) in r0.into_iter().chain(r1) {
                    if !results.iter().any(|(rid, _, _)| *rid == id) {
                        results.push((id, t, y));
                    }
                }
                info!(ext = %ext_name, year = ?tracker_year, candidates = ?results, "Fallback search candidates");
                best = Self::pick_candidate(&results, &all_titles, tracker_year, MIN_SIMILARITY, true);
            }
        }

        let best_candidate_id = best.ok_or_else(|| {
            warn!(
                title = %title,
                titles = ?all_titles,
                year = ?tracker_year,
                ext = %ext_name,
                "No search result met similarity threshold across any title variant"
            );
            CoreError::NotFound("error.content.no_match_found".into())
        })?;

        let candidate_id = best_candidate_id;
        let ext_meta = Self::fetch_ext_metadata(state, ext_name, &candidate_id).await?;
        let ext_nsfw = state.extension_manager.is_nsfw(ext_name);

        Self::resolve_and_link(&state.pool, cid, ext_name, &candidate_id, &ext_meta, &ct, ext_nsfw).await?;
        ExtensionService::save_extension_metadata(state, cid, ext_name, &candidate_id).await;
        Ok((ct, candidate_id))
    }

    async fn search_candidates(
        state: &Arc<AppState>,
        ext_name: &str,
        query: &str,
    ) -> CoreResult<Vec<(String, String, Option<i32>)>> {
        let items = state
            .extension_manager
            .search(ext_name, query, json!({}), 0)
            .await
            .map_err(|e| {
                error!(ext = %ext_name, error = ?e, "Extension search failed");
                CoreError::Internal("error.content.extension_search_failed".into())
            })?;
        Ok(items
            .into_iter()
            .map(|i| {
                let (plain, year) = split_year_suffix(&i.title);
                (i.id, plain, year)
            })
            .collect())
    }

    fn pick_candidate(
        results: &[(String, String, Option<i32>)],
        all_titles: &[String],
        tracker_year: Option<i32>,
        min_similarity: f64,
        require_year: bool,
    ) -> Option<String> {
        results
            .iter()
            .filter_map(|(id, item_title, item_year)| {
                let (score, _) = best_title_match(
                    &normalize_title(item_title),
                    all_titles.iter().map(String::as_str),
                );
                if score < min_similarity {
                    return None;
                }
                let tier = match (*item_year, tracker_year) {
                    (Some(a), Some(b)) if a == b => 0,
                    (Some(a), Some(b)) if (a - b).abs() == 1 => 1,
                    (Some(_), Some(_)) => return None,
                    _ if require_year => return None,
                    _ => 2,
                };
                Some((tier, score, id))
            })
            .min_by(|a, b| {
                a.0.cmp(&b.0)
                    .then(b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal))
            })
            .map(|(_, _, id)| id.clone())
    }

    #[instrument(skip(pool, ext_meta))]
    pub async fn resolve_and_link(
        pool: &SqlitePool,
        cid: &str,
        ext_name: &str,
        ext_id: &str,
        ext_meta: &ExtensionMetadata,
        content_type: &ContentType,
        ext_nsfw: bool,
    ) -> CoreResult<()> {
        if Self::matches_by_tracker_ids(pool, cid, ext_meta, content_type).await? {
            debug!(cid = %cid, ext = %ext_name, "Resolved via tracker ID");
            return Self::link(pool, cid, ext_name, ext_id, ext_nsfw).await;
        }

        let (plain_ext_title, _) = split_year_suffix(&ext_meta.title);
        if let Some(matched) = ContentRepository::find_closest_match(
            pool, &plain_ext_title, Some(content_type.clone()), ext_meta.year,
        ).await? {
            if matched.cid == cid {
                debug!(cid = %cid, ext = %ext_name, "Resolved via fuzzy title match");
                return Self::link(pool, cid, ext_name, ext_id, ext_nsfw).await;
            }
            warn!(expected = %cid, matched = %matched.cid, "Fuzzy matched a different CID — rejecting");
        }

        Err(CoreError::NotFound("error.content.extension_no_match".into()))
    }

    pub async fn link_or_enrich_tracker(
        state: &Arc<AppState>,
        ext_name: &str,
        ext_id: &str,
        ext_nsfw: bool,
        tracker: &str,
        tracker_id: &str,
        content_type: &ContentType,
    ) -> CoreResult<Option<FullContent>> {
        let maybe_cid = TrackerRepository::find_cid_by_tracker(&state.pool, tracker, tracker_id).await?;

        let cid = if let Some(cid) = maybe_cid {
            cid
        } else {
            info!(tracker = %tracker, id = %tracker_id, "No CID found, enriching from tracker");
            let media = Self::fetch_tracker_media(state, tracker, tracker_id).await?;
            let full = EnrichmentService::create_enriched_content(
                state, content_type, &media, tracker_id, tracker, None
            ).await?;
            full.content.cid
        };

        Self::link(&state.pool, &cid, ext_name, ext_id, ext_nsfw).await?;

        Ok(Some(Self::load_full_content(state, &cid).await?))
    }

    async fn resolve_tracker_source(
        state: &Arc<AppState>,
        tracker: &str,
        tracker_id: &str,
    ) -> CoreResult<FullContent> {
        let maybe_cid = TrackerRepository::find_cid_by_tracker(&state.pool, tracker, tracker_id).await?;

        if let Some(cid) = maybe_cid {
            let full = Self::load_full_content(state, &cid).await?;
            tokio::spawn(Self::backfill_after_tracker_link(
                state.clone(), cid, tracker.to_string(), tracker_id.to_string(),
            ));
            return Self::refresh_derived_data(state, full, tracker, tracker_id).await;
        }

        let media = Self::fetch_tracker_media(state, tracker, tracker_id).await?;
        let full = EnrichmentService::create_enriched_content(
            state, &media.content_type, &media, tracker_id, tracker, None,
        ).await?;

        Self::refresh_derived_data(state, full, tracker, tracker_id).await
    }
    
    async fn refresh_derived_data(
        state: &Arc<AppState>,
        full: FullContent,
        tracker: &str,
        tracker_id: &str,
    ) -> CoreResult<FullContent> {
        let cid = full.content.cid.clone();

        let mut changed = Self::backfill_cross_ids(state, &cid, &full, tracker, tracker_id)
            .await
            .unwrap_or_else(|e| {
                warn!(cid = %cid, error = ?e, "Cross-ID backfill failed");
                false
            });

        changed |= SimklUnitsService::sync_units_if_needed(state, &full)
            .await
            .unwrap_or_else(|e| {
                warn!(cid = %cid, error = ?e, "Failed to sync units synchronously");
                false
            });

        if changed {
            Self::load_full_content(state, &cid).await
        } else {
            Ok(full)
        }
    }

    /// Same as `refresh_derived_data`, seeded from the content's first mapping.
    pub async fn refresh_derived_data_by_mappings(
        state: &Arc<AppState>,
        full: FullContent,
    ) -> CoreResult<FullContent> {
        let Some(seed) = full.tracker_mappings.first() else {
            return Ok(full);
        };
        let (tracker, tracker_id) = (seed.tracker_name.clone(), seed.tracker_id.clone());
        Self::refresh_derived_data(state, full, &tracker, &tracker_id).await
    }

    async fn resolve_extension_source(
        state: &Arc<AppState>,
        ext_name: &str,
        ext_id: &str,
    ) -> CoreResult<FullContent> {
        let maybe_cid = ExtensionRepository::find_cid_by_extension(&state.pool, ext_name, ext_id).await?;

        if let Some(cid) = maybe_cid {
            return Self::load_full_content(state, &cid).await;
        }

        let ext_meta = Self::fetch_ext_metadata(state, ext_name, ext_id).await?;
        let ext_nsfw = state.extension_manager.is_nsfw(ext_name);
        let skip = state.extension_manager.skip_default_processing(ext_name);
        let content_type = state.extension_manager.content_type(ext_name);

        if skip {
            let cid = Self::create_derived(state, ext_name, ext_id, &ext_meta, &content_type, ext_nsfw).await?;
            return Self::load_full_content(state, &cid).await;
        }

        if let Some(matched) = ContentRepository::find_closest_match(
            &state.pool, &ext_meta.title, Some(content_type.clone()), ext_meta.year,
        ).await? {
            info!(cid = %matched.cid, ext = %ext_name, title = %ext_meta.title, "Found existing entry in local DB, linking");
            Self::link(&state.pool, &matched.cid, ext_name, ext_id, ext_nsfw).await?;
            ExtensionService::save_extension_metadata(state, &matched.cid, ext_name, ext_id).await;
            return Self::load_full_content(state, &matched.cid).await;
        }

        if let Some(full) = Self::resolve_via_tracker_ids(
            state, ext_name, ext_id, &ext_meta, &content_type, ext_nsfw,
        ).await? {
            return Ok(full);
        }

        if let Some(full) = Self::resolve_via_fuzzy(
            state, ext_name, ext_id, &ext_meta, &content_type, ext_nsfw,
        ).await? {
            return Ok(full);
        }

        warn!(ext = %ext_name, title = %ext_meta.title, "No tracker match, creating derived entry");
        let cid = Self::create_derived(state, ext_name, ext_id, &ext_meta, &content_type, ext_nsfw).await?;
        Self::load_full_content(state, &cid).await
    }

    async fn resolve_via_tracker_ids(
        state: &Arc<AppState>,
        ext_name: &str,
        ext_id: &str,
        ext_meta: &ExtensionMetadata,
        content_type: &ContentType,
        ext_nsfw: bool,
    ) -> CoreResult<Option<FullContent>> {
        for (tracker, id) in Self::tracker_ids_from_ext_meta(ext_meta, content_type) {
            match Self::link_or_enrich_tracker(
                state, ext_name, ext_id, ext_nsfw, tracker, &id, content_type,
            ).await {
                Ok(Some(full)) => return Ok(Some(full)),
                Ok(None) => {}
                Err(e) => warn!(
                    ext = %ext_name, tracker = %tracker, id = %id, error = ?e,
                    "Tracker id from extension failed, trying next strategy"
                ),
            }
        }

        Ok(None)
    }

    async fn fuzzy_search_tracker(
        state: &Arc<AppState>,
        tracker: &str,
        query: &str,
        content_type: &ContentType,
    ) -> CoreResult<Vec<TrackerMedia>> {
        match state.tracker_registry.get(tracker) {
            Some(provider) => provider.search(
                Some(query), content_type.clone(), 10, 1, None, None, None, None, None, None
            ).await,
            None => Ok(vec![]),
        }
    }

    async fn resolve_via_fuzzy(
        state: &Arc<AppState>,
        ext_name: &str,
        ext_id: &str,
        ext_meta: &ExtensionMetadata,
        content_type: &ContentType,
        ext_nsfw: bool,
    ) -> CoreResult<Option<FullContent>> {
        let query = &ext_meta.title;
        let normalized_query = normalize_title(query);
        let mut candidates: Vec<(String, String)> = Vec::new();
        let mut seen_mal_ids: HashSet<String> = HashSet::new();

        let best_score = |item: &TrackerMedia| -> f64 {
            let titles = std::iter::once(item.title.as_str())
                .chain(item.alt_titles.iter().map(|s| s.as_str()))
                .chain(item.title_i18n.values().map(|s| s.as_str()));
            let (score, _) = best_title_match(&normalized_query, titles);
            apply_year_penalty(score, ext_meta.year.map(|y| y as i64), item.release_date.as_deref())
        };

        let (anilist_result, mal_result) = tokio::join!(
            Self::fuzzy_search_tracker(state, "anilist", query, content_type),
            Self::fuzzy_search_tracker(state, "mal", query, content_type),
        );

        match anilist_result {
            Ok(results) => {
                for item in results {
                    if best_score(&item) < FUZZY_SCORE_THRESHOLD {
                        continue;
                    }
                    if let Some(mal_id) = item.cross_ids.get("mal") {
                        if seen_mal_ids.insert(mal_id.clone()) {
                            candidates.push(("mal".into(), mal_id.clone()));
                        }
                    }
                    candidates.push(("anilist".into(), item.tracker_id.clone()));
                }
            }
            Err(e) => warn!(error = ?e, "AniList fuzzy search failed"),
        }

        match mal_result {
            Ok(results) => {
                for item in results {
                    if best_score(&item) < FUZZY_SCORE_THRESHOLD {
                        continue;
                    }
                    if !seen_mal_ids.contains(&item.tracker_id) {
                        candidates.push(("mal".into(), item.tracker_id.clone()));
                    }
                }
            }
            Err(e) => warn!(error = ?e, "MAL fuzzy search failed"),
        }

        for (tracker, tracker_id) in candidates.into_iter().take(MAX_FUZZY_CANDIDATES) {
            match Self::link_or_enrich_tracker(
                state, ext_name, ext_id, ext_nsfw, &tracker, &tracker_id, content_type,
            ).await {
                Ok(Some(full)) => return Ok(Some(full)),
                Ok(None) => {}
                Err(e) => warn!(
                    ext = %ext_name, tracker = %tracker, id = %tracker_id, error = ?e,
                    "Fuzzy tracker candidate failed, trying next"
                ),
            }
        }

        Ok(None)
    }

    pub async fn create_derived(
        state: &Arc<AppState>,
        ext_name: &str,
        ext_id: &str,
        ext_meta: &ExtensionMetadata,
        content_type: &ContentType,
        ext_nsfw: bool,
    ) -> CoreResult<String> {
        for (tracker, id) in Self::tracker_ids_from_ext_meta(ext_meta, content_type) {
            if let Ok(Some(cid)) = TrackerRepository::find_cid_by_tracker(&state.pool, tracker, &id).await {
                Self::link(&state.pool, &cid, ext_name, ext_id, ext_nsfw).await?;
                return Ok(cid);
            }
        }

        let cid = generate_cid();
        let now = Utc::now().timestamp();
        let meta = Self::ext_meta_to_metadata(&cid, ext_name, ext_id, ext_meta, now);
        ContentRepository::create_with_type(&state.pool, content_type, ext_nsfw, meta).await?;
        Self::link(&state.pool, &cid, ext_name, ext_id, ext_nsfw).await?;
        Ok(cid)
    }

    pub fn ext_meta_to_metadata(
        cid: &str,
        source_name: &str,
        source_id: &str,
        ext_meta: &ExtensionMetadata,
        now: i64,
    ) -> Metadata {
        Metadata {
            id: None,
            cid: cid.to_string(),
            source_name: source_name.to_string(),
            source_id: Some(source_id.to_string()),
            subtype: None,
            title: ext_meta.title.clone(),
            alt_titles: vec![],
            title_i18n: Default::default(),
            synopsis: ext_meta.synopsis.clone(),
            cover_image: ext_meta.image.clone(),
            banner_image: ext_meta.image.clone(),
            eps_or_chapters: ext_meta
                .eps_or_chapters
                .map(|n| EpisodeData::Count(n as i32))
                .unwrap_or(EpisodeData::Count(0)),
            status: None,
            genres: ext_meta.genres.clone().unwrap_or_default(),
            release_date: ext_meta.year.map(|y| format!("{}-01-01", y)),
            end_date: None,
            rating: ext_meta.rating.map(|v| v as f32),
            trailer_url: None,
            characters: vec![],
            studio: None,
            staff: vec![],
            external_ids: json!({}),
            episode_duration: None,
            created_at: now,
            updated_at: now,
        }
    }

    fn tracker_ids_from_ext_meta(
        ext_meta: &ExtensionMetadata,
        content_type: &ContentType,
    ) -> Vec<(&'static str, String)> {
        let mut ids = Vec::with_capacity(2);

        if let Some(id) = ext_meta.anilist_id.as_ref().and_then(value_to_id) {
            ids.push(("anilist", id));
        }

        let prefix = match content_type {
            ContentType::Anime => "anime",
            ContentType::Manga | ContentType::Novel => "manga",
        };
        if let Some(id) = ext_meta.mal_id.as_ref().and_then(|v| mal_id_with_prefix(v, prefix)) {
            ids.push(("mal", id));
        }

        ids
    }

    async fn matches_by_tracker_ids(
        pool: &SqlitePool,
        expected_cid: &str,
        ext_meta: &ExtensionMetadata,
        content_type: &ContentType,
    ) -> CoreResult<bool> {
        for (tracker, id) in Self::tracker_ids_from_ext_meta(ext_meta, content_type) {
            if let Some(cid) = TrackerRepository::find_cid_by_tracker(pool, tracker, &id).await? {
                if cid == expected_cid {
                    return Ok(true);
                }
                warn!(tracker = %tracker, found = %cid, expected = %expected_cid, "Tracker ID matched wrong CID");
            }
        }
        Ok(false)
    }

    pub async fn link(pool: &SqlitePool, cid: &str, ext_name: &str, ext_id: &str, nsfw: bool) -> CoreResult<()> {
        let now = Utc::now().timestamp();
        ExtensionRepository::add_source(pool, &ExtensionSource {
            id: None,
            cid: cid.to_string(),
            extension_name: ext_name.to_string(),
            extension_id: ext_id.to_string(),
            nsfw,
            language: None,
            created_at: now,
            updated_at: now,
        }).await?;
        Ok(())
    }

    pub async fn load_full_content(state: &Arc<AppState>, cid: &str) -> CoreResult<FullContent> {
        ContentRepository::get_full_content(&state.pool, cid).await?
            .ok_or_else(|| {
                warn!(cid = %cid, "Content not found after resolution");
                CoreError::NotFound("error.content.not_found".into())
            })
    }

    pub async fn fetch_tracker_media(
        state: &Arc<AppState>,
        tracker: &str,
        tracker_id: &str,
    ) -> CoreResult<TrackerMedia> {
        let provider = state.tracker_registry.get(tracker)
            .ok_or_else(|| CoreError::Internal(
                format!("error.tracker.{}_not_registered", tracker).into()
            ))?;
        provider.get_by_id(tracker_id)
            .await
            .map_err(|e| {
                error!(tracker = %tracker, id = %tracker_id, error = ?e, "Failed to fetch from tracker");
                e
            })?
            .ok_or_else(|| {
                warn!(tracker = %tracker, id = %tracker_id, "Tracker returned no media for ID");
                CoreError::NotFound("error.content.tracker_media_not_found".into())
            })
    }

    pub async fn fetch_ext_metadata(
        state: &Arc<AppState>,
        ext_name: &str,
        ext_id: &str,
    ) -> CoreResult<ExtensionMetadata> {
        state
            .extension_manager
            .get_metadata(ext_name, ext_id)
            .await
            .map_err(|e| {
                error!(ext = %ext_name, id = %ext_id, error = ?e, "getMetadata failed");
                CoreError::Internal("error.content.extension_metadata_failed".into())
            })
    }

    fn parse_content_type(s: &str) -> ContentType {
        serde_json::from_str::<ContentType>(&format!("\"{}\"", s))
            .unwrap_or(ContentType::Anime)
    }

    pub async fn backfill_via_preferred_mapping(state: Arc<AppState>, cid: String) {
        let Ok(config) = ConfigRepository::get_active_user_config(&state.pool).await else { return };
        let preferred = config.content.preferred_metadata_provider.clone();

        let Ok(mappings) = TrackerRepository::get_mappings_by_cid(&state.pool, &cid).await else { return };

        if let Some(mapping) = mappings.iter().find(|m| m.tracker_name == preferred) {
            let _ = Self::backfill_preferred_metadata_by_cid(
                &state, &cid, &preferred, &preferred, &mapping.tracker_id,
            ).await;
        }
    }

    async fn backfill_after_tracker_link(state: Arc<AppState>, cid: String, tracker: String, tracker_id: String) {
        let Ok(full) = Self::load_full_content(&state, &cid).await else { return };
        let _ = Self::backfill_preferred_metadata(&state, &cid, &full, &tracker, &tracker_id).await;
    }

    async fn resolve_preferred_tracker_id(
        state: &Arc<AppState>,
        cid: &str,
        preferred: &str,
        current_tracker: &str,
        current_tracker_id: &str,
    ) -> CoreResult<Option<String>> {
        if preferred == current_tracker {
            return Ok(Some(current_tracker_id.to_string()));
        }
        TrackerRepository::find_tracker_id_by_cid(&state.pool, cid, preferred).await
    }

    async fn fetch_and_store_preferred_metadata(
        state: &Arc<AppState>,
        cid: &str,
        preferred: &str,
        tracker_id: &str,
    ) -> CoreResult<()> {
        let media = match Self::fetch_tracker_media(state, preferred, tracker_id).await {
            Ok(m) => m,
            Err(e) => {
                warn!(error = ?e, "Failed to fetch preferred provider metadata, skipping backfill");
                return Ok(());
            }
        };

        let provider = state.tracker_registry.get(preferred)
            .ok_or_else(|| CoreError::NotFound(format!("Tracker provider '{}' not found", preferred)))?;

        let meta = provider.to_core_metadata(cid, &media);
        ContentRepository::upsert_metadata(&state.pool, &meta).await?;
        ImportService::sync_relations(&state.pool, cid, preferred, &media, true).await;
        Ok(())
    }

    /// Airing/upcoming entries change often (status, episode count, new sequels); finished ones rarely.
    fn is_metadata_stale(meta: &Metadata, now: i64) -> bool {
        let age = now - meta.updated_at;
        match meta.status {
            Some(Status::Ongoing | Status::Planned | Status::Hiatus) | None => age > ACTIVE_REFRESH_SECS,
            Some(Status::Completed | Status::Cancelled) => age > FINISHED_REFRESH_SECS,
        }
    }

    async fn ensure_preferred_metadata(
        state: &Arc<AppState>,
        cid: &str,
        metadata: &[Metadata],
        preferred: &str,
        current_tracker: &str,
        current_tracker_id: &str,
    ) -> CoreResult<()> {
        match metadata.iter().find(|m| m.source_name == preferred) {
            Some(m) if !m.characters.is_empty() && !Self::is_metadata_stale(m, Utc::now().timestamp()) => return Ok(()),
            Some(m) if !m.characters.is_empty() => debug!(cid = %cid, status = ?m.status, "Preferred metadata stale, refreshing"),
            Some(_) => {} // present but missing characters: refresh
            None => info!(cid = %cid, preferred = %preferred, "Missing preferred provider metadata, backfilling"),
        }

        let Some(tid) = Self::resolve_preferred_tracker_id(
            state, cid, preferred, current_tracker, current_tracker_id,
        ).await? else {
            warn!(cid = %cid, preferred = %preferred, "No tracker mapping for preferred provider, skipping backfill");
            return Ok(());
        };

        Self::fetch_and_store_preferred_metadata(state, cid, preferred, &tid).await
    }

    async fn backfill_preferred_metadata(
        state: &Arc<AppState>,
        cid: &str,
        full: &FullContent,
        current_tracker: &str,
        current_tracker_id: &str,
    ) -> CoreResult<()> {
        let config = ConfigRepository::get_active_user_config(&state.pool).await?;
        let preferred = &config.content.preferred_metadata_provider;
        Self::ensure_preferred_metadata(state, cid, &full.metadata, preferred, current_tracker, current_tracker_id).await
    }

    async fn backfill_preferred_metadata_by_cid(
        state: &Arc<AppState>,
        cid: &str,
        preferred: &str,
        current_tracker: &str,
        current_tracker_id: &str,
    ) -> CoreResult<()> {
        let metadata = ContentRepository::get_all_metadata(&state.pool, cid).await?;
        Self::ensure_preferred_metadata(state, cid, &metadata, preferred, current_tracker, current_tracker_id).await
    }

    async fn backfill_cross_ids(
        state: &Arc<AppState>,
        cid: &str,
        full: &FullContent,
        tracker: &str,
        tracker_id: &str,
    ) -> CoreResult<bool> {
        let known_trackers: HashSet<&str> = full.tracker_mappings
            .iter()
            .map(|m| m.tracker_name.as_str())
            .collect();

        let expected = match full.content.content_type {
            ContentType::Anime => &["anilist", "mal", "kitsu", "simkl"][..],
            ContentType::Manga | ContentType::Novel => &["anilist", "mal"][..],
        };

        let missing: Vec<&str> = expected.iter()
            .copied()
            .filter(|t| !known_trackers.contains(t))
            .collect();

        if missing.is_empty() {
            return Ok(false);
        }

        info!(cid = %cid, missing = ?missing, "Backfilling missing cross-ID mappings");

        let cross_ids: HashMap<String, String> = match full.content.content_type {
            ContentType::Anime => {
                let raw = EnrichmentService::resolve_anime_cross_ids(state, tracker, tracker_id, None).await?;
                EnrichmentService::normalize_prefixed_ids(raw, "anime")
            }
            ContentType::Manga | ContentType::Novel => {
                let raw = EnrichmentService::resolve_manga_cross_ids(state, tracker, tracker_id, None).await?;
                EnrichmentService::normalize_prefixed_ids(raw, "manga")
            }
        };

        if cross_ids.is_empty() {
            warn!(cid = %cid, "No cross IDs returned during lazy backfill");
            return Ok(false);
        }

        let now = Utc::now().timestamp();
        let mut saved_any = false;

        for (t_name, t_id) in &cross_ids {
            if known_trackers.contains(t_name.as_str()) {
                continue; // don't overwrite what we already have
            }
            let mapping = TrackerMapping {
                cid: cid.to_string(),
                tracker_name: t_name.clone(),
                tracker_id: t_id.clone(),
                tracker_url: None,
                created_at: now,
                updated_at: now,
            };
            match MappingService::add_tracker_mapping(&state.pool, mapping).await {
                Ok(_) => {
                    saved_any = true;
                    debug!(cid = %cid, tracker = %t_name, id = %t_id, "Lazy cross-ID mapping saved");
                }
                Err(e) => warn!(cid = %cid, tracker = %t_name, error = ?e, "Failed to save cross-ID mapping during backfill"),
            }
        }

        Ok(saved_any)
    }
}