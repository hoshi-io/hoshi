use serde::{Deserialize, Serialize};
use crate::impl_from_row;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BackupTrigger {
    PreImport,
    Manual,
    RemoteSync
}

impl BackupTrigger {
    pub fn as_str(&self) -> &'static str {
        match self {
            BackupTrigger::PreImport => "PRE_IMPORT",
            BackupTrigger::Manual => "MANUAL",
            BackupTrigger::RemoteSync => "REMOTE_SYNC",
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListBackupMeta {
    pub id: i64,
    pub user_id: i32,
    pub trigger: String,
    pub tracker_name: Option<String>,
    pub file_path: String,
    pub entry_count: i32,
    pub created_at: i64,
}
impl_from_row!(ListBackupMeta { id, user_id, trigger, tracker_name, file_path, entry_count, created_at });

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListEntrySnapshot {
    pub cid: String,
    pub status: String,
    pub progress: i32,
    pub score: Option<f64>,
    pub start_date: Option<String>,
    pub end_date: Option<String>,
    pub repeat_count: i32,
    pub notes: Option<String>,
    pub is_private: bool,
}