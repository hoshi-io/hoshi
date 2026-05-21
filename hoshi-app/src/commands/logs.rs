use hoshi_core::error::CoreError;
use hoshi_core::logs::LogEntry;
use serde::Serialize;
use std::path::PathBuf;

#[derive(Serialize)]
pub struct LogFileInfo {
    pub name: String,
    pub size_bytes: u64,
    pub created_at: i64,
}

fn logs_dir(state: &tauri::State<'_, std::sync::Arc<hoshi_core::state::AppState>>) -> PathBuf {
    state.paths.logs_path.clone()
}

#[tauri::command]
pub async fn get_system_logs(
    state: tauri::State<'_, std::sync::Arc<hoshi_core::state::AppState>>,
) -> Result<Vec<LogEntry>, CoreError> {
    let logs = {
        let lock = state.log_store.read().unwrap();
        lock.iter().cloned().collect()
    };
    Ok(logs)
}

#[tauri::command]
pub async fn list_log_files(
    state: tauri::State<'_, std::sync::Arc<hoshi_core::state::AppState>>,
) -> Result<Vec<LogFileInfo>, CoreError> {
    let dir = logs_dir(&state);
    let mut files = vec![];

    let entries = std::fs::read_dir(&dir).map_err(CoreError::Io)?;

    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("log") {
            continue;
        }
        let name = path.file_name().unwrap().to_string_lossy().to_string();
        let meta = std::fs::metadata(&path).map_err(CoreError::Io)?;
        let size_bytes = meta.len();

        let created_at = {
            parse_log_timestamp(name.trim_end_matches(".log")).unwrap_or(0)
        };

        fn parse_log_timestamp(s: &str) -> Option<i64> {
            // Parse "%Y-%m-%dT%H-%M-%S"
            let b = s.as_bytes();
            if b.len() != 19 { return None; }

            let year:  i64 = s[0..4].parse().ok()?;
            let month: u32 = s[5..7].parse().ok()?;
            let day:   u32 = s[8..10].parse().ok()?;
            let hour:  i64 = s[11..13].parse().ok()?;
            let min:   i64 = s[14..16].parse().ok()?;
            let sec:   i64 = s[17..19].parse().ok()?;

            if month < 1 || month > 12 || day < 1 || day > 31 { return None; }

            let days = days_from_civil(year, month, day)?;
            let millis = (days * 86_400 + hour * 3_600 + min * 60 + sec) * 1_000;
            Some(millis)
        }

        fn days_from_civil(y: i64, m: u32, d: u32) -> Option<i64> {
            // Algorithm by Howard Hinnant (public domain)
            let m = m as i64;
            let d = d as i64;
            let y = if m <= 2 { y - 1 } else { y };
            let era = y.div_euclid(400);
            let yoe = y.rem_euclid(400);                          // [0, 399]
            let doy = (153 * (m + (if m > 2 { -3 } else { 9 })) + 2) / 5 + d - 1; // [0, 365]
            let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;    // [0, 146096]
            Some(era * 146_097 + doe - 719_468)
        }

        files.push(LogFileInfo { name, size_bytes, created_at });
    }

    files.sort_by(|a, b| b.created_at.cmp(&a.created_at));
    Ok(files)
}

#[tauri::command]
pub async fn get_log_file(
    state: tauri::State<'_, std::sync::Arc<hoshi_core::state::AppState>>,
    name: String,
) -> Result<String, CoreError> {
    if name.contains('/') || name.contains('\\') || name.contains("..") {
        return Err(CoreError::NotFound("Invalid log file name".into()));
    }
    let path = logs_dir(&state).join(&name);
    std::fs::read_to_string(&path).map_err(CoreError::Io)
}

#[tauri::command]
pub async fn delete_log_file(
    state: tauri::State<'_, std::sync::Arc<hoshi_core::state::AppState>>,
    name: String,
) -> Result<(), CoreError> {
    if name.contains('/') || name.contains('\\') || name.contains("..") {
        return Err(CoreError::NotFound("Invalid log file name".into()));
    }
    let path = logs_dir(&state).join(&name);
    std::fs::remove_file(&path).map_err(CoreError::Io)
}