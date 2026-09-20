use std::sync::Arc;
use tracing::debug;
use crate::content::types::AniSkipResponse;
use crate::extensions::types::EpisodeChapter;
use crate::state::AppState;
use crate::tracker::repository::TrackerRepository;

pub struct AniSkipService;

impl AniSkipService {

    pub async fn fetch_chapters(state: &Arc<AppState>, cid: &str, episode_number: f64) -> Vec<EpisodeChapter> {
        let Some(mal_id) = Self::find_mal_id(state, cid).await else {
            return Vec::new();
        };

        debug!(mal_id = %mal_id, ep = %episode_number, "Fetching skip times from AniSkip");

        let Ok(response) = state.http_client
            .get(format!("https://api.aniskip.com/v2/skip-times/{mal_id}/{episode_number}"))
            .query(&[
                ("types", "op"),
                ("types", "ed"),
                ("types", "recap"),
                ("types", "mixed-op"),
                ("types", "mixed-ed"),
                ("episodeLength", "0"),
            ])
            .send()
            .await
        else {
            return Vec::new();
        };

        let Ok(skip_data) = response.json::<AniSkipResponse>().await else {
            return Vec::new();
        };

        let mut chapters: Vec<EpisodeChapter> = skip_data.results.into_iter().map(|r| {
            let title = match r.skip_type.as_str() {
                "op" => "Opening",
                "ed" => "Ending",
                "recap" => "Recap",
                "mixed-op" => "Mixed Opening",
                "mixed-ed" => "Mixed Ending",
                _ => "Skip",
            };
            EpisodeChapter {
                start: r.interval.start_time,
                end: r.interval.end_time,
                title: title.to_string(),
            }
        }).collect();

        chapters.sort_by(|a, b| a.start.partial_cmp(&b.start).unwrap());
        chapters
    }

    async fn find_mal_id(state: &Arc<AppState>, cid: &str) -> Option<i64> {
        let mappings = TrackerRepository::get_mappings_by_cid(&state.pool, cid).await.ok()?;
        mappings.iter()
            .find(|m| m.tracker_name == "mal")
            .and_then(|m| m.tracker_id.strip_prefix("anime:")?.parse::<i64>().ok())
    }
}