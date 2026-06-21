use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use tracing::{error, info, instrument, warn};

use crate::content::models::ContentType;
use crate::content::services::enrichment::EnrichmentService;
use crate::error::{CoreError, CoreResult};
use crate::list::merge::MergeService;
use crate::state::AppState;
use crate::tracker::provider::{TrackerProvider};
use crate::tracker::repository::TrackerRepository;
use crate::tracker::types::{AddIntegrationRequest, ImportEvent, IntegrationsResponse, SuccessResponse, TrackerInfoResponse, TrackerIntegration};

const MANGA_RATE_LIMIT_MS: u64 = 500;
const ANIME_TSV_URL: &str = "https://animeapi.my.id/aa.tsv";

struct TsvIndex {
    anilist:     Option<usize>,
    myanimelist: Option<usize>,
    kitsu:       Option<usize>,
    simkl:       Option<usize>,
}

type AnimeIdIndex = HashMap<(String, String), HashMap<String, String>>;

pub fn normalize_list_status(s: &str) -> String {
    match s.to_uppercase().as_str() {
        "CURRENT" | "WATCHING" | "AIRING"                => "CURRENT",
        "COMPLETED" | "FINISHED" | "WATCHED"             => "COMPLETED",
        "PLANNING" | "PLAN_TO_WATCH" | "PTW"
        | "PLAN TO WATCH" | "WANT TO WATCH"              => "PLANNING",
        "PAUSED" | "ON_HOLD" | "HOLD"                    => "PAUSED",
        "DROPPED" | "ABANDONED"                          => "DROPPED",
        "REPEATING" | "REWATCHING" | "REREADING"         => "REPEATING",
        _                                                 => "PLANNING",
    }.to_string()
}

pub struct TrackerService;

impl TrackerService {
    pub async fn set_sync_enabled(
        state: &AppState,
        user_id: i32,
        tracker_name: &str,
        enabled: bool,
    ) -> CoreResult<SuccessResponse> {
        TrackerRepository::set_sync_enabled(state.pool(), user_id, tracker_name, enabled).await?;
        Ok(SuccessResponse { success: true })
    }

    pub async fn get_integrations(
        state: &AppState,
        user_id: i32,
    ) -> CoreResult<IntegrationsResponse> {
        let integrations = TrackerRepository::get_user_integrations(state.pool(), user_id).await?;
        Ok(IntegrationsResponse { integrations })
    }

    pub async fn list_trackers(
        state: &AppState,
        user_id: i32,
    ) -> CoreResult<Vec<TrackerInfoResponse>> {
        let integrations = TrackerRepository::get_user_integrations(state.pool(), user_id).await?;

        Ok(state.tracker_registry.all().into_iter().map(|provider| {
            let integration = integrations.iter().find(|i| i.tracker_name == provider.name());
            TrackerInfoResponse {
                name:             provider.name().to_string(),
                display_name:     provider.display_name().to_string(),
                icon_url:         provider.icon_url().to_string(),
                supported_types:  provider.supported_types().iter().map(|t| t.as_str().to_string()).collect(),
                auth:             provider.auth_config(),
                connected:        integration.is_some(),
                tracker_user_id:  integration.map(|i| i.tracker_user_id.clone()),
                sync_enabled:     integration.map(|i| i.sync_enabled),
                display_name_user: integration.and_then(|i| i.display_name.clone()),
                avatar_url:        integration.and_then(|i| i.avatar_url.clone()),
                profile_url:       integration.and_then(|i| i.profile_url.clone()),
                total_entries:     integration.and_then(|i| i.total_entries),
                last_synced_at:    integration.and_then(|i| i.last_synced_at),
            }
        }).collect())
    }

    #[instrument(skip(state, body))]
    pub async fn add_integration(
        state: Arc<AppState>,
        user_id: i32,
        body: AddIntegrationRequest,
    ) -> CoreResult<SuccessResponse> {
        info!(tracker = %body.tracker_name, "Adding new tracker integration");

        let provider = state.tracker_registry.get(&body.tracker_name)
            .ok_or_else(|| CoreError::NotFound("error.tracker.unknown_tracker".into()))?;
        let auth_config = provider.auth_config();

        let access_token = if auth_config.oauth_flow == "pkce" {
            let code     = body.access_token.ok_or_else(|| CoreError::AuthError("error.tracker.missing_auth_code".into()))?;
            let verifier = body.code_verifier.ok_or_else(|| CoreError::AuthError("error.tracker.missing_code_verifier".into()))?;
            let token_url = auth_config.token_url.as_ref()
                .ok_or_else(|| CoreError::Internal("error.tracker.missing_token_url".into()))?;
            let client_id = auth_config.client_id.as_deref().unwrap_or_default();

            let res = state
                .http_client
                .post(token_url)
                .form(&[
                    ("grant_type", "authorization_code"),
                    ("client_id", client_id),
                    ("code", &code),
                    ("code_verifier", &verifier),
                    ("redirect_uri", "hoshi://auth"),
                ])
                .send().await
                .map_err(|_| CoreError::Network("error.tracker.token_exchange_network_error".into()))?;

            if !res.status().is_success() {
                return Err(CoreError::AuthError("error.tracker.token_exchange_failed".into()));
            }
            #[derive(serde::Deserialize)] struct R { access_token: String }
            res.json::<R>().await
                .map_err(|_| CoreError::Parse("error.system.serialization".into()))?.access_token

        } else if let Some(token) = body.access_token {
            token
        } else if let (Some(username), Some(password)) = (body.username, body.password) {
            if auth_config.oauth_flow != "password" {
                return Err(CoreError::AuthError("error.tracker.password_login_unsupported".into()));
            }
            let token_url = auth_config.token_url.as_ref()
                .ok_or_else(|| CoreError::Internal("error.tracker.missing_token_url".into()))?;
            let client_id = auth_config.client_id.as_deref().unwrap_or_default();

            let res = state.http_client
                .post(token_url)
                .form(&[
                    ("grant_type", "password"),
                    ("username", username.as_str()),
                    ("password", password.as_str()),
                    ("client_id", client_id),
                ])
                .send().await
                .map_err(|_| CoreError::Network("error.tracker.auth_network_error".into()))?;

            if !res.status().is_success() {
                return Err(CoreError::AuthError("error.tracker.invalid_credentials".into()));
            }
            #[derive(serde::Deserialize)] struct R { access_token: String }
            res.json::<R>().await
                .map_err(|_| CoreError::Parse("error.system.serialization".into()))?.access_token
        } else {
            return Err(CoreError::AuthError("error.tracker.missing_credentials".into()));
        };

        let token_data = provider.validate_and_store_token(&access_token, "Bearer").await?;
        let expires_at = chrono::DateTime::parse_from_rfc3339(&token_data.expires_at)
            .map(|dt| dt.timestamp())
            .unwrap_or_else(|_| chrono::Utc::now().timestamp() + 31_536_000);

        let pool = state.pool();
        TrackerRepository::save_integration(
            pool, user_id, &body.tracker_name,
            &token_data.tracker_user_id, &token_data.access_token,
            token_data.refresh_token.as_deref(), &token_data.token_type, expires_at,
            token_data.display_name.as_deref(),
            token_data.avatar_url.as_deref(),
            token_data.profile_url.as_deref(),
            token_data.score_format.as_deref()
        ).await?;
        TrackerRepository::set_sync_enabled(pool, user_id, &body.tracker_name, false).await?;

        info!("Integration saved, returning. Caller is responsible for spawning the import.");
        Ok(SuccessResponse { success: true })
    }

    pub async fn remove_integration(
        state: &AppState,
        user_id: i32,
        tracker_name: &str,
    ) -> CoreResult<SuccessResponse> {
        TrackerRepository::delete_integration(state.pool(), user_id, tracker_name).await?;
        Ok(SuccessResponse { success: true })
    }
}

#[instrument(skip(state, on_event))]
pub async fn import_from_tracker_by_name(
    state: &Arc<AppState>,
    user_id: i32,
    tracker_name: &str,
    on_event: impl Fn(ImportEvent) + Send + 'static,
) -> CoreResult<i32> {
    on_event(ImportEvent::Started { tracker_name: tracker_name.into() });
    static IMPORT_RUNNING: AtomicBool = AtomicBool::new(false);

    if IMPORT_RUNNING.compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst).is_err() {
        warn!("Import already in progress, rejecting concurrent request");
        return Err(CoreError::BadRequest("error.import.already_running".into()));
    }
    struct Guard;
    impl Drop for Guard {
        fn drop(&mut self) { IMPORT_RUNNING.store(false, Ordering::SeqCst); }
    }
    let _guard = Guard;

    let integration = TrackerRepository::get_user_integrations(state.pool(), user_id)
        .await?
        .into_iter()
        .find(|i| i.tracker_name == tracker_name)
        .ok_or_else(|| CoreError::NotFound("error.tracker.integration_not_found".into()))?;

    let provider = state.tracker_registry.get(tracker_name)
        .ok_or_else(|| CoreError::Internal("error.tracker.not_in_registry".into()))?;

    match import_from_tracker(state, user_id, &integration, &provider, &on_event).await {
        Ok(count) => {
            on_event(ImportEvent::Done { tracker_name: tracker_name.into(), imported: count as usize });
            Ok(count)
        }
        Err(e) => {
            error!(error = ?e, tracker = %tracker_name, "Import failed");
            on_event(ImportEvent::Error {
                tracker_name: tracker_name.into(),
                message: e.to_string(),
            });
            Err(e)
        }
    }
}

async fn import_from_tracker(
    state: &Arc<AppState>,
    user_id: i32,
    integration: &TrackerIntegration,
    provider: &Arc<dyn TrackerProvider>,
    on_event: &(impl Fn(ImportEvent) + Send + 'static),
) -> CoreResult<i32> {
    let pool = state.pool();

    let remote_entries = provider
        .get_user_list(&integration.access_token, &integration.tracker_user_id, integration.score_format.as_deref())
        .await?;

    let needs_anime = remote_entries.iter().any(|e| e.content_type == ContentType::Anime);
    let anime_index: Option<AnimeIdIndex> = if needs_anime {
        match fetch_anime_tsv(state, &integration.tracker_name).await {
            Ok(idx) => {
                info!(entries = idx.len(), "Anime TSV loaded into memory");
                Some(idx)
            }
            Err(e) => {
                warn!(error = ?e, "Failed to fetch anime TSV, will fall back to per-entry API calls");
                None
            }
        }
    } else {
        None
    };

    let total = remote_entries.len();
    let mut count: i32 = 0;

    for remote in remote_entries {
        let tracker_id = &remote.tracker_media_id;

        let existing_cid = TrackerRepository::find_cid_by_tracker(
            pool, &integration.tracker_name, tracker_id,
        ).await?;

        if let Some(cid) = existing_cid {
            if let Err(e) = MergeService::merge_entry(state, user_id, &integration.tracker_name, &remote).await {
                warn!(error = ?e, cid = %cid, "Failed to upsert list entry");
            }
            count += 1;
            on_event(ImportEvent::Progress {
                tracker_name: integration.tracker_name.clone(),
                imported: count as usize,
                total: Some(total),
            });
            continue;
        }

        let tracker_media = {
            let inline = remote.media.clone();
            let needs_fetch = inline.as_ref()
                .map(|m| m.synopsis.is_none() && m.characters.is_empty())
                .unwrap_or(true);

            if needs_fetch {
                match provider.get_by_id(tracker_id).await {
                    Ok(Some(full)) => full,
                    Ok(None) => match inline {
                        Some(m) => m,
                        None => { warn!(id = %tracker_id, "No media found, skipping"); continue; }
                    },
                    Err(e) => {
                        warn!(error = ?e, id = %tracker_id, "get_by_id failed");
                        match inline { Some(m) => m, None => continue }
                    }
                }
            } else {
                inline.unwrap()
            }
        };

        let cross_ids: Option<HashMap<String, String>> = match tracker_media.content_type {
            ContentType::Anime => {
                anime_index.as_ref().and_then(|idx| {
                    idx.get(&(integration.tracker_name.clone(), tracker_id.clone())).cloned()
                })
            }
            ContentType::Manga | ContentType::Novel => None,
        };

        if matches!(tracker_media.content_type, ContentType::Manga | ContentType::Novel) {
            tokio::time::sleep(tokio::time::Duration::from_millis(MANGA_RATE_LIMIT_MS)).await;
        }

        let cid = match EnrichmentService::create_enriched_content(
            state,
            &tracker_media.content_type,
            &tracker_media,
            tracker_id,
            &integration.tracker_name,
            cross_ids.as_ref(),
        ).await {
            Ok(full) => full.content.cid,
            Err(e) => {
                error!(error = ?e, id = %tracker_id, "Enrichment failed, skipping entry");
                continue;
            }
        };

        if let Err(e) = MergeService::merge_entry(state, user_id, &integration.tracker_name, &remote).await {
            warn!(error = ?e, cid = %cid, "Failed to upsert list entry");
        }

        count += 1;
        on_event(ImportEvent::Progress {
            tracker_name: integration.tracker_name.clone(),
            imported: count as usize,
            total: Some(total),
        });
    }

    if let Err(e) = TrackerRepository::update_sync_stats(
        pool, user_id, &integration.tracker_name, total as i64,
    ).await {
        warn!(error = ?e, tracker = %integration.tracker_name, "Failed to update sync stats");
    }

    info!(count = count, tracker = %integration.tracker_name, "Import completed");
    Ok(count)
}

async fn fetch_anime_tsv(
    state: &AppState,
    source_tracker: &str
) -> CoreResult<AnimeIdIndex> {
    info!("Downloading anime ID mapping TSV");

    let text = state
        .http_client
        .get(ANIME_TSV_URL)
        .send()
        .await
        .map_err(|e| {
            error!(error = ?e, "TSV download failed");
            CoreError::Network("error.import.tsv_download_failed".into())
        })?
        .text()
        .await
        .map_err(|_| CoreError::Parse("error.import.tsv_parse_failed".into()))?;

    let mut lines = text.lines();
    let header_line = lines.next()
        .ok_or_else(|| CoreError::Parse("error.import.tsv_empty".into()))?;
    let headers: Vec<&str> = header_line.split('\t').collect();

    let idx = TsvIndex {
        anilist:     headers.iter().position(|h| *h == "anilist"),
        myanimelist: headers.iter().position(|h| *h == "myanimelist"),
        kitsu:       headers.iter().position(|h| *h == "kitsu"),
        simkl:       headers.iter().position(|h| *h == "simkl"),
    };

    let tracked = [
        ("anilist",     idx.anilist),
        ("mal",         idx.myanimelist),
        ("kitsu",       idx.kitsu),
        ("simkl",       idx.simkl),
    ];

    let mut index: AnimeIdIndex = HashMap::new();

    for line in lines {
        if line.is_empty() { continue; }
        let cols: Vec<&str> = line.split('\t').collect();

        let mut row_ids: HashMap<String, String> = HashMap::new();
        for (name, maybe_col) in &tracked {
            if let Some(col) = maybe_col {
                if let Some(val) = cols.get(*col) {
                    let v = val.trim();
                    if !v.is_empty() {
                        row_ids.insert(name.to_string(), v.to_string());
                    }
                }
            }
        }

        if row_ids.is_empty() { continue; }

        let row_clone = row_ids.clone();
        for (name, id) in &row_ids {
            index.insert((name.clone(), id.clone()), row_clone.clone());
        }
    }

    info!(rows = index.len(), source = %source_tracker, "TSV index built");
    Ok(index)
}