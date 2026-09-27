use crate::content::models::FullContent;
use crate::tracker::provider::TrackerMedia;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone)]
pub struct AiringEpisode {
    pub episode:   i32,
    pub airing_at: i64,
    pub media:     Option<TrackerMedia>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AiringEntryEnriched {
    pub tracker_id:   String,
    pub episode:      i32,
    pub airing_at:    i64,
    pub full_content: FullContent,
}

fn default_days_back()  -> i64 { 1 }
fn default_days_ahead() -> i64 { 7 }

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScheduleWindow {
    #[serde(default = "default_days_back")]
    pub days_back:  i64,
    #[serde(default = "default_days_ahead")]
    pub days_ahead: i64,
}