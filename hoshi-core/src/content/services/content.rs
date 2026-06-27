use std::sync::Arc;
use std::collections::{HashMap, HashSet};
use tracing::{info, instrument, warn};
use crate::config::model::TitleLanguage;
use crate::config::repository::ConfigRepository;
use crate::content::models::{ContentType, FullContent, Metadata, Relation, RelationType};
use crate::content::repositories::content::ContentRepository;
use crate::content::repositories::extension::ExtensionRepository;
use crate::content::repositories::relations::RelationRepository;
use crate::content::services::chinese_title::ChineseTitleService;
use crate::content::services::content_units::SimklUnitsService;
use crate::content::services::enrichment::EnrichmentService;
use crate::content::services::extensions::ExtensionService;
use crate::content::services::resolver::ContentResolverService;
use crate::error::{CoreError, CoreResult};
use crate::extensions::types::ExtensionMetadata;
use crate::state::AppState;
use crate::tracker::provider::TrackerMedia;
use crate::tracker::repository::TrackerRepository;

const TRACKER_SOURCES: &[&str] = &["anilist", "mal", "kitsu", "anidb"];
const FUZZY_SCORE_THRESHOLD: f64 = 0.85;

pub struct ContentService;

impl ContentService {

    #[instrument(skip(state))]
    pub async fn get_content(
        state: &Arc<AppState>,
        source: &str,
        source_id: &str,
    ) -> CoreResult<FullContent> {
        let mut full = if TRACKER_SOURCES.contains(&source) {
            Self::resolve_tracker_source(state, source, source_id).await?
        } else {
            Self::resolve_extension_source(state, source, source_id).await?
        };

        Self::maybe_inject_chinese_title(state, &mut full).await;
        Ok(full)
    }

    pub async fn get_content_by_cid(
        state: &Arc<AppState>,
        cid: &str,
    ) -> CoreResult<FullContent> {
        let mut full = ContentRepository::get_full_content(&state.pool, cid).await?
            .ok_or_else(|| CoreError::NotFound("error.content.not_found".into()))?;

        let state_bg = state.clone();
        let cid_bg = cid.to_string();

        tokio::spawn(async move {
            if let Err(e) = SimklUnitsService::sync_units_if_needed(&state_bg, &cid_bg).await {
                warn!(cid = %cid_bg, error = ?e, "Background unit sync failed");
            }
            if let Err(e) = Self::resolve_pending_relations(&state_bg, &cid_bg).await {
                warn!(cid = %cid_bg, error = ?e, "Background relation resolution failed");
            }
            if let Ok(config) = ConfigRepository::get_config(&state_bg.pool, 1).await {
                let preferred = &config.content.preferred_metadata_provider;
                if let Ok(mappings) = TrackerRepository::get_mappings_by_cid(&state_bg.pool, &cid_bg).await {
                    if let Some(mapping) = mappings.iter().find(|m| &m.tracker_name == preferred) {
                        let _ = Self::backfill_preferred_metadata_by_cid(
                            &state_bg, &cid_bg, preferred, preferred, &mapping.tracker_id,
                        ).await;
                    }
                    if let Ok(full) = ContentResolverService::load_full_content(&state_bg, &cid_bg).await {
                        if let Some(mapping) = mappings.first() {
                            let _ = Self::backfill_cross_ids(
                                &state_bg, &cid_bg, &full, &mapping.tracker_name, &mapping.tracker_id,
                            ).await;
                        }
                    }
                }
            }
        });

        Self::maybe_inject_chinese_title(state, &mut full).await;
        Ok(full)
    }

    async fn resolve_tracker_source(
        state: &Arc<AppState>,
        tracker: &str,
        tracker_id: &str,
    ) -> CoreResult<FullContent> {
        let maybe_cid = TrackerRepository::find_cid_by_tracker(&state.pool, tracker, tracker_id).await?;

        if let Some(cid) = maybe_cid {
            let state_bg = state.clone();
            let cid_bg = cid.clone();
            let tracker_bg = tracker.to_string();
            let tracker_id_bg = tracker_id.to_string();
            tokio::spawn(async move {
                let full = match ContentResolverService::load_full_content(&state_bg, &cid_bg).await {
                    Ok(f) => f,
                    Err(_) => return,
                };
                let _ = Self::backfill_preferred_metadata(&state_bg, &cid_bg, &full, &tracker_bg, &tracker_id_bg).await;
                let _ = SimklUnitsService::sync_units_if_needed(&state_bg, &cid_bg).await;
                let _ = Self::resolve_pending_relations(&state_bg, &cid_bg).await;
                let _ = Self::backfill_cross_ids(&state_bg, &cid_bg, &full, &tracker_bg, &tracker_id_bg).await;
            });

            return ContentResolverService::load_full_content(state, &cid).await;
        }

        let media = ContentResolverService::fetch_tracker_media(state, tracker, tracker_id).await?;
        let full = EnrichmentService::create_enriched_content(
            state, &media.content_type, &media, tracker_id, tracker, None,
        ).await?;

        let state_bg = state.clone();
        let cid_bg = full.content.cid.clone();
        let _ = SimklUnitsService::sync_units_if_needed(&state_bg, &cid_bg).await;

        tokio::spawn(async move {
            let _ = Self::resolve_pending_relations(&state_bg, &cid_bg).await;
        });

        Ok(full)
    }

    async fn backfill_preferred_metadata(
        state: &Arc<AppState>,
        cid: &str,
        full: &FullContent,
        current_tracker: &str,
        current_tracker_id: &str,
    ) -> CoreResult<()> {
        let config = ConfigRepository::get_config(&state.pool, 1).await?;
        let preferred = &config.content.preferred_metadata_provider;

        let already_has = full.metadata.iter().any(|m| &m.source_name == preferred);

        if already_has {
            let needs_character_refresh = full.metadata.iter()
                .find(|m| &m.source_name == preferred)
                .map(|m| m.characters.is_empty())
                .unwrap_or(false);

            if !needs_character_refresh {
                return Ok(());
            }

            let tid = if preferred == current_tracker {
                Some(current_tracker_id.to_string())
            } else {
                TrackerRepository::find_tracker_id_by_cid(&state.pool, cid, preferred).await?
            };

            let Some(tid) = tid else {
                warn!(cid = %cid, "No tracker mapping for preferred provider, skipping character refresh");
                return Ok(());
            };

            let media = match ContentResolverService::fetch_tracker_media(state, preferred, &tid).await {
                Ok(m) => m,
                Err(e) => {
                    warn!(error = ?e, "Failed to refresh characters, skipping");
                    return Ok(());
                }
            };

            let provider = state.tracker_registry.get(preferred)
                .ok_or_else(|| CoreError::NotFound(format!("Tracker provider '{}' not found", preferred)))?;
            let meta = provider.to_core_metadata(cid, &media);
            ContentRepository::upsert_metadata(&state.pool, &meta).await?;

            return Ok(());
        }

        info!(cid = %cid, preferred = %preferred, "Missing preferred provider metadata, backfilling");

        let preferred_tracker_id = if preferred == current_tracker {
            Some(current_tracker_id.to_string())
        } else {
            TrackerRepository::find_tracker_id_by_cid(&state.pool, cid, preferred).await?
        };

        let Some(tid) = preferred_tracker_id else {
            warn!(cid = %cid, preferred = %preferred, "No tracker mapping found for preferred provider, skipping backfill");
            return Ok(());
        };

        let media = match ContentResolverService::fetch_tracker_media(state, preferred, &tid).await {
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

        Ok(())
    }

    async fn resolve_extension_source(
        state: &Arc<AppState>,
        ext_name: &str,
        ext_id: &str,
    ) -> CoreResult<FullContent> {
        let maybe_cid = ExtensionRepository::find_cid_by_extension(
            &state.pool, ext_name, ext_id,
        ).await?;

        if let Some(cid) = maybe_cid {
            return ContentResolverService::load_full_content(state, &cid).await;
        }

        let ext_meta = ContentResolverService::fetch_ext_metadata(state, ext_name, ext_id).await?;
        let ext_nsfw = state.extension_manager.read().await.is_nsfw(ext_name);
        let skip = state.extension_manager.read().await.skip_default_processing(ext_name);
        let content_type = state.extension_manager.read().await.content_type(ext_name);

        if skip {
            let cid = ContentResolverService::create_derived(
                state, ext_name, ext_id, &ext_meta, &content_type, ext_nsfw,
            ).await?;
            return ContentResolverService::load_full_content(state, &cid).await;
        }

        if let Some(matched) = ContentRepository::find_closest_match(
            &state.pool, &ext_meta.title, Some(content_type.clone()), ext_meta.year,
        ).await? {
            info!(cid = %matched.cid, ext = %ext_name, title = %ext_meta.title, "Found existing entry in local DB, linking");
            ContentResolverService::link(&state.pool, &matched.cid, ext_name, ext_id, ext_nsfw).await?;
            ExtensionService::save_extension_metadata(state, &matched.cid, ext_name, ext_id).await;
            return ContentResolverService::load_full_content(state, &matched.cid).await;
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
        let cid = ContentResolverService::create_derived(
            state, ext_name, ext_id, &ext_meta, &content_type, ext_nsfw,
        ).await?;
        ContentResolverService::load_full_content(state, &cid).await
    }

    async fn resolve_via_tracker_ids(
        state: &Arc<AppState>,
        ext_name: &str,
        ext_id: &str,
        ext_meta: &ExtensionMetadata,
        content_type: &ContentType,
        ext_nsfw: bool,
    ) -> CoreResult<Option<FullContent>> {
        if let Some(ref raw) = ext_meta.anilist_id {
            let id_str = raw.to_string();
            if let Some(full) = ContentResolverService::link_or_enrich_tracker(
                state, ext_name, ext_id, ext_nsfw, "anilist", &id_str, content_type,
            ).await? {
                return Ok(Some(full));
            }
        }

        if let Some(ref raw) = ext_meta.mal_id {
            let prefix = match content_type { ContentType::Anime => "anime", _ => "manga" };
            let id_str = format!("{}:{}", prefix, raw);
            if let Some(full) = ContentResolverService::link_or_enrich_tracker(
                state, ext_name, ext_id, ext_nsfw, "mal", &id_str, content_type,
            ).await? {
                return Ok(Some(full));
            }
        }

        Ok(None)
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
        let normalized_query = crate::content::utils::normalize_title(query);
        let mut candidates: Vec<(String, String)> = Vec::new();
        let mut seen_mal_ids: HashSet<String> = HashSet::new();

        let best_score = |item: &TrackerMedia| -> f64 {
            std::iter::once(item.title.as_str())
                .chain(item.alt_titles.iter().map(|s| s.as_str()))
                .chain(item.title_i18n.values().map(|s| s.as_str()))
                .map(|t| crate::content::utils::similarity(
                    &normalized_query,
                    &crate::content::utils::normalize_title(t),
                ))
                .fold(0.0_f64, f64::max)
        };

        if let Some(provider) = state.tracker_registry.get("anilist") {
            match provider.search(Some(query.as_str()), content_type.clone(), 10, 1, None, None, None, None, None).await {
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
        }

        if let Some(provider) = state.tracker_registry.get("mal") {
            match provider.search(Some(query.as_str()), content_type.clone(), 10, 1, None, None, None, None, None).await {
                Ok(results) => {
                    for item in results {
                        if best_score(&item) < FUZZY_SCORE_THRESHOLD {
                            continue;
                        }
                        if !seen_mal_ids.contains(&item.tracker_id) {
                            candidates.push((
                                "mal".into(),
                                item.tracker_id.clone()
                            ));
                        }
                    }
                }
                Err(e) => warn!(error = ?e, "MAL fuzzy search failed"),
            }
        }

        for (tracker, tracker_id) in candidates {
            if let Some(full) = ContentResolverService::link_or_enrich_tracker(
                state, ext_name, ext_id, ext_nsfw, &tracker, &tracker_id, content_type,
            ).await? {
                return Ok(Some(full));
            }
        }

        Ok(None)
    }

    #[instrument(skip(state, meta))]
    pub async fn update_content(
        state: &Arc<AppState>,
        cid: &str,
        meta: Metadata,
    ) -> CoreResult<FullContent> {
        ContentRepository::upsert_metadata(&state.pool, &meta).await?;

        ContentRepository::get_full_content(&state.pool, cid).await?
            .ok_or_else(|| {
                warn!(cid = %cid, "Content not found after metadata update");
                CoreError::NotFound("error.content.not_found".into())
            })
    }

    async fn maybe_inject_chinese_title(state: &Arc<AppState>, full: &mut FullContent) {
        let config = match ConfigRepository::get_config(&state.pool, 1).await {
            Ok(c) => c,
            Err(e) => {
                warn!(error = ?e, "Could not read user config for Chinese title check");
                return;
            }
        };

        if !matches!(config.ui.title_language, TitleLanguage::Chinese) {
            ChineseTitleService::evict().await;
            return;
        }

        ChineseTitleService::ensure_loaded().await;

        let anilist_id: u32 = match full
            .tracker_mappings
            .iter()
            .find(|m| m.tracker_name == "anilist")
        {
            Some(m) => match m.tracker_id.parse() {
                Ok(id) => id,
                Err(_) => {
                    warn!(tracker_id = %m.tracker_id, "AniList tracker_id is not a valid u32, skipping Chinese title");
                    return;
                }
            },
            None => return,
        };

        let Some(chinese_title) = ChineseTitleService::lookup(anilist_id).await else {
            return;
        };

        for meta in &mut full.metadata {
            meta.title_i18n.insert("chinese".into(), chinese_title.clone());
        }
    }

    async fn resolve_pending_relations(state: &Arc<AppState>, cid: &str) -> CoreResult<()> {
        let pending = RelationRepository::get_pending(&state.pool, cid).await?;
        if pending.is_empty() { return Ok(()); }

        for (id, tracker_name, tracker_id, rel_type) in pending {
            let target_cid = match TrackerRepository::find_cid_by_tracker(
                &state.pool, &tracker_name, &tracker_id
            ).await? {
                Some(tcid) => tcid,
                None => {
                    match ContentResolverService::fetch_tracker_media(state, &tracker_name, &tracker_id).await {
                        Ok(media) => {
                            match EnrichmentService::create_enriched_content(
                                state,
                                &media.content_type,
                                &media,
                                &tracker_id,
                                &tracker_name,
                                None,
                            ).await {
                                Ok(full) => full.content.cid,
                                Err(e) => {
                                    warn!(error = ?e, tracker_id = %tracker_id, "Failed to enrich pending relation target");
                                    continue;
                                }
                            }
                        }
                        Err(e) => {
                            warn!(error = ?e, tracker_id = %tracker_id, "Failed to fetch pending relation media");
                            continue;
                        }
                    }
                }
            };

            let rel_enum = serde_json::from_str::<RelationType>(&format!("\"{}\"", rel_type))
                .unwrap_or(RelationType::Alternative);

            if let Err(e) = RelationRepository::upsert(&state.pool, &Relation {
                id: None,
                source_cid: cid.to_string(),
                target_cid,
                relation_type: rel_enum,
                source_name: tracker_name,
                created_at: chrono::Utc::now().timestamp(),
            }).await {
                warn!(error = ?e, "Failed to upsert resolved relation");
                continue;
            }

            RelationRepository::delete_pending(&state.pool, id).await.ok();
        }

        Ok(())
    }

    async fn backfill_preferred_metadata_by_cid(
        state: &Arc<AppState>,
        cid: &str,
        preferred: &str,
        current_tracker: &str,
        current_tracker_id: &str,
    ) -> CoreResult<()> {
        let metadata = ContentRepository::get_all_metadata(&state.pool, cid).await?;
        let already_has = metadata.iter().any(|m| m.source_name == preferred);
        let needs_char_refresh = already_has && metadata.iter()
            .find(|m| m.source_name == preferred)
            .map(|m| m.characters.is_empty())
            .unwrap_or(false);

        if already_has && !needs_char_refresh {
            return Ok(());
        }

        let tid = if preferred == current_tracker {
            Some(current_tracker_id.to_string())
        } else {
            TrackerRepository::find_tracker_id_by_cid(&state.pool, cid, preferred).await?
        };

        let Some(tid) = tid else {
            warn!(cid = %cid, preferred = %preferred, "No tracker mapping for backfill, skipping");
            return Ok(());
        };

        let media = match ContentResolverService::fetch_tracker_media(state, preferred, &tid).await {
            Ok(m) => m,
            Err(e) => {
                warn!(error = ?e, "Failed to fetch metadata for backfill, skipping");
                return Ok(());
            }
        };

        let provider = state.tracker_registry.get(preferred)
            .ok_or_else(|| CoreError::NotFound(format!("Tracker provider '{}' not found", preferred)))?;
        let meta = provider.to_core_metadata(cid, &media);
        ContentRepository::upsert_metadata(&state.pool, &meta).await?;

        Ok(())
    }

    async fn backfill_cross_ids(
        state: &Arc<AppState>,
        cid: &str,
        full: &FullContent,
        tracker: &str,
        tracker_id: &str,
    ) -> CoreResult<()> {
        let known_trackers: HashSet<&str> = full.tracker_mappings
            .iter()
            .map(|m| m.tracker_name.as_str())
            .collect();

        let expected = match full.content.content_type {
            ContentType::Anime => &["anilist", "mal", "kitsu"][..],
            ContentType::Manga | ContentType::Novel => &["anilist", "mal"][..],
        };

        let missing: Vec<&&str> = expected.iter()
            .filter(|t| !known_trackers.contains(**t))
            .collect();

        if missing.is_empty() {
            return Ok(());
        }

        info!(cid = %cid, missing = ?missing, "Backfilling missing cross-ID mappings");

        let now = chrono::Utc::now().timestamp();

        let cross_ids = match full.content.content_type {
            ContentType::Anime => {
                let endpoint = match tracker {
                    "anilist"             => format!("anilist/{}", tracker_id),
                    "mal" | "myanimelist" => {
                        let raw = tracker_id.splitn(2, ':').last().unwrap_or(tracker_id);
                        format!("myanimelist/{}", raw)
                    }
                    "kitsu"  => format!("kitsu/{}", tracker_id),
                    "simkl"  => format!("simkl/{}", tracker_id),
                    _ => {
                        warn!(cid = %cid, tracker = %tracker, "Unsupported tracker for cross-ID backfill");
                        return Ok(());
                    }
                };
                let url = format!("https://animeapi.my.id/{}", endpoint);
                let resp = state.http_client.get(&url).send().await
                    .map_err(|e| CoreError::Network(format!("cross-id fetch failed: {:?}", e).into()))?;
                let data: serde_json::Value = resp.json().await
                    .map_err(|e| CoreError::Parse(format!("cross-id parse failed: {:?}", e).into()))?;
                let raw = EnrichmentService::extract_anime_cross_ids(&data);
                raw.into_iter().map(|(k, v)| {
                    if k == "mal" { (k, format!("anime:{}", v)) } else { (k, v) }
                }).collect::<HashMap<String, String>>()
            }
            ContentType::Manga | ContentType::Novel => {
                let endpoint = match tracker {
                    "anilist"                        => format!("/v1/source/anilist/{}", tracker_id),
                    "kitsu"                          => format!("/v1/source/kitsu/{}", tracker_id),
                    "mal" | "myanimelist"            => format!("/v1/source/my-anime-list/{}", tracker_id),
                    "mangaupdates" | "manga-updates" => format!("/v1/source/manga-updates/{}", tracker_id),
                    _ => {
                        warn!(cid = %cid, tracker = %tracker, "Unsupported tracker for cross-ID backfill");
                        return Ok(());
                    }
                };
                let url = format!("https://api.mangabaka.dev{}", endpoint);
                let resp = state.http_client.get(&url).send().await
                    .map_err(|e| CoreError::Network(format!("cross-id fetch failed: {:?}", e).into()))?;
                let data: serde_json::Value = resp.json().await
                    .map_err(|e| CoreError::Parse(format!("cross-id parse failed: {:?}", e).into()))?;
                let raw = EnrichmentService::extract_manga_cross_ids(&data);
                raw.into_iter().map(|(k, v)| {
                    if k == "mal" { (k, format!("manga:{}", v)) } else { (k, v) }
                }).collect::<HashMap<String, String>>()
            }
        };

        if cross_ids.is_empty() {
            warn!(cid = %cid, "No cross IDs returned during lazy backfill");
            return Ok(());
        }

        for (t_name, t_id) in &cross_ids {
            if known_trackers.contains(t_name.as_str()) {
                continue; // don't overwrite what we already have
            }
            if let Err(e) = crate::content::services::mapping::MappingService::add_tracker_mapping(
                &state.pool,
                crate::tracker::types::TrackerMapping {
                    cid: cid.to_string(),
                    tracker_name: t_name.clone(),
                    tracker_id: t_id.clone(),
                    tracker_url: None,
                    created_at: now,
                    updated_at: now,
                },
            ).await {
                warn!(cid = %cid, tracker = %t_name, error = ?e, "Failed to save cross-ID mapping during backfill");
            } else {
                info!(cid = %cid, tracker = %t_name, id = %t_id, "Lazy cross-ID mapping saved");
            }
        }

        Ok(())
    }
}