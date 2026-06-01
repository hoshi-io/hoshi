use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use crate::impl_from_row;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ListEntry {
    pub id: Option<i64>,
    pub user_id: i32,
    pub cid: String,
    pub status: String,
    pub progress: i32,
    pub score: Option<f64>,
    pub start_date: Option<String>,
    pub end_date: Option<String>,
    pub repeat_count: i32,
    pub notes: Option<String>,
    pub is_private: bool,
    pub created_at: String,
    pub updated_at: String,
}

impl_from_row!(ListEntry { id, user_id, cid, status, progress, score, start_date, end_date, repeat_count, notes, is_private, created_at, updated_at });

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EntrySource {
    pub tracker: String,
    pub remote_id: String,
    pub synced_at: Option<String>,
    pub status: Option<String>,
    pub progress: Option<i32>,
    pub score: Option<f64>,
    pub start_date: Option<String>,
    pub end_date: Option<String>,
    pub repeat_count: Option<i32>,
}

#[derive(Debug)]
pub struct EntrySourceRow {
    pub tracker: String,
    pub remote_id: String,
    pub synced_at: Option<String>,
    pub remote_snapshot: Option<String>,
}

impl_from_row!(EntrySourceRow { tracker, remote_id, synced_at, remote_snapshot });

impl From<EntrySourceRow> for EntrySource {
    fn from(row: EntrySourceRow) -> Self {
        let snap: Value = row.remote_snapshot
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or(Value::Null);

        EntrySource {
            tracker: row.tracker,
            remote_id: row.remote_id,
            synced_at: row.synced_at,
            status:       snap["status"].as_str().map(str::to_owned),
            progress:     snap["progress"].as_i64().map(|v| v as i32),
            score:        snap["score"].as_f64(),
            start_date:   snap["startDate"].as_str().map(str::to_owned),
            end_date:     snap["endDate"].as_str().map(str::to_owned),
            repeat_count: snap["repeatCount"].as_i64().map(|v| v as i32),
        }
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EnrichedListEntry {
    #[serde(flatten)]
    pub entry: ListEntry,
    pub title: String,
    #[serde(default)]
    pub title_i18n: HashMap<String, String>,
    pub cover_image: Option<String>,
    pub content_type: String,
    pub nsfw: bool,
    pub total_units: Option<i32>,
    pub tracker_ids: Value,
    pub external_ids: Value,
    pub has_extension_source: bool,
    pub sources: Vec<EntrySource>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ScoreDistribution {
    pub score: i32,
    pub count: i32,
}

impl_from_row!(ScoreDistribution { score, count });

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UserStats {
    pub total_entries: i32,
    pub watching: i32,
    pub completed: i32,
    pub planning: i32,
    pub paused: i32,
    pub dropped: i32,
    pub repeating: i32,
    pub total_episodes: i32,
    pub total_chapters: i32,
    pub mean_score: Option<f64>,
    pub score_distribution: Vec<ScoreDistribution>,
    pub days_since_last_activity: Option<i64>,
    pub completion_rate: Option<f64>,
    pub total_rewatches: i32,
    pub entries_with_notes: i32,
    pub private_entries: i32,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpsertEntryBody {
    pub cid: String,
    pub status: String,
    pub progress: Option<i32>,
    pub score: Option<f64>,
    pub start_date: Option<String>,
    pub end_date: Option<String>,
    pub repeat_count: Option<i32>,
    pub notes: Option<String>,
    pub is_private: Option<bool>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FilterQuery {
    pub status: Option<String>,
    pub content_type: Option<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ListResponse {
    pub results: Vec<EnrichedListEntry>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SingleEntryResponse {
    pub found: bool,
    pub entry: Option<EnrichedListEntry>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UpsertEntryResponse {
    pub success: bool,
    pub changes: usize,
    pub is_new: bool,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SuccessResponse {
    pub success: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ChangeSource {
    Local,
    RemoteSync,
    Import,
}

impl ChangeSource {
    pub fn as_str(&self) -> &'static str {
        match self {
            ChangeSource::Local      => "LOCAL",
            ChangeSource::RemoteSync => "REMOTE_SYNC",
            ChangeSource::Import     => "IMPORT",
        }
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ListEntryChange {
    pub id: Option<i64>,
    pub entry_id: i64,
    pub user_id: i32,
    pub changed_at: String,
    pub source: String,
    pub tracker: Option<String>,
    pub field: String,
    pub old_value: Option<String>,
    pub new_value: String,
}

impl_from_row!(ListEntryChange { id, entry_id, user_id, changed_at, source, tracker, field, old_value, new_value });

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EntryHistoryResponse {
    pub changes: Vec<ListEntryChange>,
}