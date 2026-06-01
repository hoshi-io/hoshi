use std::sync::Arc;
use tracing::{error, info, warn};
use crate::config::repository::ConfigRepository;
use crate::state::AppState;
use crate::tracker::repository::TrackerRepository;
use crate::users::repository::UserRepo;
use crate::list::merge::MergeService;

pub struct StartupSyncService;

impl StartupSyncService {
    pub fn run(state: Arc<AppState>) {
        tokio::spawn(async move {
            tokio::time::sleep(std::time::Duration::from_secs(10)).await;

            let users = match UserRepo::get_all_users(&state.pool).await {
                Ok(u) => u,
                Err(e) => { error!(error = ?e, "Startup sync: failed to fetch users"); return; }
            };

            info!(users = users.len(), "Starting tracker startup sync");

            for user in users {
                let config = match ConfigRepository::get_config(&state.pool, user.id).await {
                    Ok(c) => c,
                    Err(e) => { warn!(error = ?e, user_id = user.id, "Failed to fetch config, skipping"); continue; }
                };

                if !config.list.sync_on_startup {
                    info!(user_id = user.id, "Startup sync disabled, skipping");
                    continue;
                }

                let integrations = match TrackerRepository::get_user_integrations(&state.pool, user.id).await {
                    Ok(i) => i,
                    Err(e) => { warn!(error = ?e, user_id = user.id, "Failed to fetch integrations"); continue; }
                };

                for integration in integrations {
                    if !integration.sync_enabled { continue; }

                    let state = state.clone();
                    tokio::spawn(async move {
                        let provider = match state.tracker_registry.get(&integration.tracker_name) {
                            Some(p) => p,
                            None => { warn!(tracker = %integration.tracker_name, "Not in registry, skipping"); return; }
                        };

                        let entries = match provider.get_user_list(
                            &integration.access_token,
                            &integration.tracker_user_id,
                        ).await {
                            Ok(e) => e,
                            Err(e) => { warn!(error = ?e, tracker = %integration.tracker_name, user_id = integration.user_id, "Failed to fetch remote list"); return; }
                        };

                        let total = entries.len() as i64;
                        let mut imported = 0usize;
                        let mut skipped  = 0usize;

                        for entry in entries {
                            match MergeService::merge_entry(&state, integration.user_id, &integration.tracker_name, &entry).await {
                                Ok(true)  => imported += 1,
                                Ok(false) => skipped  += 1,
                                Err(e)    => warn!(error = ?e, tracker_id = %entry.tracker_media_id, "Failed to merge entry"),
                            }
                        }

                        if let Err(e) = TrackerRepository::update_sync_stats(
                            &state.pool, integration.user_id, &integration.tracker_name, total,
                        ).await {
                            warn!(error = ?e, "Failed to update sync stats");
                        }

                        info!(
                            tracker = %integration.tracker_name,
                            user_id = integration.user_id,
                            imported, skipped,
                            "Tracker sync complete"
                        );
                    });
                }
            }
        });
    }
}