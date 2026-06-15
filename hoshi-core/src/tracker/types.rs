use serde::Serialize;
use crate::tracker::provider::TrackerAuthConfig;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct IntegrationsResponse {
    pub integrations: Vec<TrackerIntegration>,
}

#[derive(Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TrackerInfoResponse {
    pub name: String,
    pub display_name: String,
    pub icon_url: String,
    pub supported_types: Vec<String>,
    pub auth: TrackerAuthConfig,
    pub connected: bool,
    pub tracker_user_id: Option<String>,
    pub sync_enabled: Option<bool>,
    pub display_name_user: Option<String>,
    pub avatar_url: Option<String>,
    pub profile_url: Option<String>,
    pub total_entries: Option<i32>,
    pub last_synced_at: Option<i64>,
}
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SyncResponse {
    pub success: bool,
    pub synced: i32,
    pub errors: Vec<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SuccessResponse {
    pub success: bool,
}

#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SetSyncEnabledRequest {
    pub enabled: bool,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct TrackerIntegration {
    pub user_id: i32,
    pub tracker_name: String,
    pub tracker_user_id: String,
    pub access_token: String,
    pub refresh_token: Option<String>,
    pub token_type: String,
    pub expires_at: i64,
    pub sync_enabled: bool,
    pub display_name: Option<String>,
    pub avatar_url: Option<String>,
    pub profile_url: Option<String>,
    pub total_entries: Option<i32>,
    pub last_synced_at: Option<i64>,
    pub created_at: i64,
    pub updated_at: i64,
    pub score_format: Option<String>
}

#[derive(Debug, serde::Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AddIntegrationRequest {
    pub tracker_name: String,
    pub access_token: Option<String>,
    pub username: Option<String>,
    pub password: Option<String>,
    pub code_verifier: Option<String>
}

pub struct IntegrationCredentials {
    pub access_token: String,
    pub refresh_token: Option<String>,
    pub tracker_user_id: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TrackerMapping {
    pub cid: String,
    pub tracker_name: String,
    pub tracker_id: String,
    pub tracker_url: Option<String>,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Clone, serde::Serialize)]
pub struct ImportProgress {
    pub tracker_name: String,
    pub imported: usize,
    pub total: Option<usize>,
}

#[derive(Clone, serde::Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ImportEvent {
    Started  { tracker_name: String },
    Progress { tracker_name: String, imported: usize, total: Option<usize> },
    Done     { tracker_name: String, imported: usize },
    Error    { tracker_name: String, message: String },
}