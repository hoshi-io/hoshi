use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};

use crate::error::{CoreError, CoreResult};
use crate::list::types::UpsertEntryBody;
use crate::progress::types::UpdateAnimeProgressBody;
use crate::state::AppState;
use crate::{list, progress};
use crate::extensions::types::EpisodeChapter;

#[derive(serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MpvLaunchOptions {
    pub source: MpvSource,
    pub subtitles: Vec<String>,
    pub chapters: Vec<EpisodeChapter>,

    pub start_time: f64,
    pub cid: String,
    pub ep_number: i32,
    pub total_episodes: i32,
    pub anime_title: String,
    pub episode_title: String,
    pub is_nsfw: bool,
    pub cover_image: Option<String>,
    pub auto_update_progress: bool,
    pub use_hoshi_mpv_config: bool,

    #[serde(skip)]
    pub state: Option<Arc<AppState>>,
    pub user_id: i32,
}

impl std::fmt::Debug for MpvLaunchOptions {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MpvLaunchOptions")
            .field("source", &self.source)
            .field("subtitles", &self.subtitles)
            .field("chapters", &self.chapters)
            .field("start_time", &self.start_time)
            .field("cid", &self.cid)
            .field("ep_number", &self.ep_number)
            .field("total_episodes", &self.total_episodes)
            .field("anime_title", &self.anime_title)
            .field("episode_title", &self.episode_title)
            .field("is_nsfw", &self.is_nsfw)
            .field("cover_image", &self.cover_image)
            .field("auto_update_progress", &self.auto_update_progress)
            .field("state", &self.state.as_ref().map(|_| "<AppState>"))
            .field("user_id", &self.user_id)
            .finish()
    }
}

impl Clone for MpvLaunchOptions {
    fn clone(&self) -> Self {
        Self {
            source: self.source.clone(),
            subtitles: self.subtitles.clone(),
            chapters: self.chapters.clone(),
            start_time: self.start_time,
            cid: self.cid.clone(),
            ep_number: self.ep_number,
            total_episodes: self.total_episodes,
            anime_title: self.anime_title.clone(),
            episode_title: self.episode_title.clone(),
            is_nsfw: self.is_nsfw,
            cover_image: self.cover_image.clone(),
            auto_update_progress: self.auto_update_progress,
            state: self.state.clone(),
            user_id: self.user_id,
            use_hoshi_mpv_config: self.use_hoshi_mpv_config,
        }
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum MpvSource {
    Url(String),
    File(PathBuf),
}

impl MpvSource {
    fn as_arg(&self) -> String {
        match self {
            MpvSource::Url(url) => url.clone(),
            MpvSource::File(path) => path.to_string_lossy().to_string(),
        }
    }
}

pub struct MpvService {
    process: Child,
    #[allow(dead_code)]
    socket_path: String,
}

impl MpvService {
    pub fn launch(opts: MpvLaunchOptions) -> CoreResult<Self> {
        let socket_path = Self::socket_path();

        let mut cmd = Command::new("mpv");

        cmd.arg(format!(
            "--force-media-title={} {}",
            opts.anime_title,
            opts.episode_title
        ));

        cmd.arg(opts.source.as_arg());
        cmd.arg(format!("--input-ipc-server={}", socket_path));

        if opts.start_time > 0.0 {
            cmd.arg(format!("--start={}", opts.start_time));
        }

        for sub in &opts.subtitles {
            cmd.arg(format!("--sub-file={}", sub));
        }

        if opts.use_hoshi_mpv_config {
            let state = opts.state.as_ref().expect("MpvLaunchOptions.state must be set");
            cmd.arg(format!("--config-dir={}", state.paths.mpv_path.display()));
        }

        if !opts.chapters.is_empty() {
            let path = write_chapters_file(&opts.chapters)
                .map_err(CoreError::Io)?;

            cmd.arg(format!("--chapters-file={}", path.display()));
        }

        cmd.stdout(Stdio::null()).stderr(Stdio::null());

        let process = cmd.spawn().map_err(|e| {
            if e.kind() == std::io::ErrorKind::NotFound {
                CoreError::Internal("mpv not found on PATH".to_string())
            } else {
                CoreError::Io(e)
            }
        })?;


        let socket_path_clone = socket_path.clone();
        tokio::spawn(async move {
            if let Err(e) = Self::run_ipc_tracker(socket_path_clone, opts).await {
                eprintln!("MPV IPC Tracker error: {:?}", e);
            }
        });

        Ok(Self { process, socket_path })
    }

    async fn run_ipc_tracker(socket_path: String, opts: MpvLaunchOptions) -> CoreResult<()> {
        let state = opts.state.as_ref().expect("MpvLaunchOptions.state must be set before launching");
        let config =
            crate::config::service::ConfigService::get_config(
                state,
                opts.user_id,
            )
                .await?;

        let auto_skip_intro = config.player.auto_skip_intro;
        let auto_skip_outro = config.player.auto_skip_outro;

        tokio::time::sleep(std::time::Duration::from_millis(500)).await;

        #[cfg(unix)]
        let stream =
            tokio::net::UnixStream::connect(&socket_path).await.map_err(CoreError::Io)?;

        #[cfg(windows)]
        let stream = {
            // Wait for named pipe
            let mut retries = 5;
            let mut opt_stream = None;
            while retries > 0 {
                match tokio::net::windows::named_pipe::ClientOptions::new().open(&socket_path) {
                    Ok(s) => {
                        opt_stream = Some(s);
                        break;
                    }
                    Err(_) => {
                        tokio::time::sleep(std::time::Duration::from_millis(200)).await;
                        retries -= 1;
                    }
                }
            }
            opt_stream.ok_or_else(|| {
                CoreError::Internal("Could not connect to MPV named pipe".to_string())
            })?
        };

        let (reader, mut writer) = tokio::io::split(stream);
        let mut reader = BufReader::new(reader);

        let init_commands = [
            r#"{"command": ["observe_property", 1, "time-pos"]}"#,
            r#"{"command": ["observe_property", 2, "duration"]}"#,
            r#"{"command": ["observe_property", 3, "pause"]}"#,
        ];

        for cmd in init_commands {
            writer
                .write_all(format!("{}\n", cmd).as_bytes())
                .await
                .map_err(CoreError::Io)?;
        }

        let mut last_sync_time: f64 = 0.0;
        let mut has_updated_list = false;
        let mut discord_status_updated = false;

        let mut current_time: f64 = 0.0;
        let mut duration: f64 = 0.0;
        let mut is_paused = false;

        let mut skipped_chapters = std::collections::HashSet::<usize>::new();

        let mut line = String::new();

        while reader.read_line(&mut line).await.unwrap_or(0) > 0 {
            if let Ok(msg) = serde_json::from_str::<serde_json::Value>(&line) {
                if msg["event"].as_str() == Some("property-change") {
                    let prop_name = msg["name"].as_str().unwrap_or("");
                    let data = &msg["data"];

                    match prop_name {
                        "time-pos" => {
                            if let Some(t) = data.as_f64() {
                                current_time = t;

                                for (idx, chapter) in opts.chapters.iter().enumerate() {
                                    if skipped_chapters.contains(&idx) {
                                        continue;
                                    }

                                    let should_skip =
                                        (auto_skip_intro && is_intro(&chapter.title))
                                            || (auto_skip_outro && is_outro(&chapter.title));

                                    if !should_skip {
                                        continue;
                                    }

                                    if current_time >= chapter.start
                                        && current_time < chapter.end
                                    {
                                        skipped_chapters.insert(idx);

                                        let cmd = serde_json::json!({
                                        "command": ["seek", chapter.end, "absolute"]
                                    });

                                        writer
                                            .write_all(format!("{}\n", cmd).as_bytes())
                                            .await
                                            .map_err(CoreError::Io)?;

                                        break;
                                    }
                                }
                            }
                        }
                        "duration" => {
                            if let Some(d) = data.as_f64() {
                                duration = d;
                            }
                        }
                        "pause" => {
                            if let Some(p) = data.as_bool() {
                                is_paused = p;
                                Self::update_discord(&opts, current_time, duration, is_paused)
                                    .await;
                            }
                        }
                        _ => {}
                    }

                    if duration > 0.0 {
                        if !discord_status_updated {
                            Self::update_discord(&opts, current_time, duration, is_paused).await;
                            discord_status_updated = true;
                        }

                        if (current_time - last_sync_time).abs() >= 10.0
                            || (last_sync_time == 0.0 && current_time > 2.0)
                        {
                            last_sync_time = current_time;

                            let body = UpdateAnimeProgressBody {
                                cid: opts.cid.clone(),
                                episode: opts.ep_number,
                                timestamp_seconds: current_time as i32,
                                episode_duration_seconds: if duration > 0.0 { Some(duration as i32) } else { None },
                                completed: Some(current_time / duration >= 0.9),
                            };

                            let _ = progress::service::ProgressService::update_anime_progress(
                                state,
                                opts.user_id,
                                body,
                            )
                                .await;
                        }

                        if !has_updated_list
                            && opts.auto_update_progress
                            && (current_time / duration >= 0.8)
                        {
                            has_updated_list = true;
                            let status = if opts.total_episodes > 0
                                && opts.ep_number >= opts.total_episodes
                            {
                                "COMPLETED"
                            } else {
                                "CURRENT"
                            };

                            let body = UpsertEntryBody {
                                cid: opts.cid.clone(),
                                status: status.to_string(),
                                progress: Some(opts.ep_number),
                                score: None,
                                start_date: None,
                                end_date: None,
                                repeat_count: None,
                                notes: None,
                                is_private: None,
                            };

                            let _ = list::service::ListService::upsert_entry(
                                state.clone(),
                                opts.user_id,
                                body,
                            )
                                .await;
                        }
                    }
                } else if msg["event"].as_str() == Some("end-file")
                    || msg["event"].as_str() == Some("shutdown")
                {
                    #[cfg(feature = "discord-rpc")]
                    state.discord_rpc.clear_activity();
                    break;
                }
            }
            line.clear();
        }

        Ok(())
    }

    async fn update_discord(opts: &MpvLaunchOptions, current_time: f64, duration: f64, paused: bool) {
        #[cfg(feature = "discord-rpc")]
        {
            let state = opts.state.as_ref().expect("MpvLaunchOptions.state must be set before launching");
            let now_in_seconds = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs() as i64;

            let start_time = if !paused {
                Some(now_in_seconds - current_time as i64)
            } else {
                None
            };

            let end_time = if !paused && duration > 0.0 {
                Some(start_time.unwrap() + duration as i64)
            } else {
                None
            };

            state.discord_rpc.set_activity(
                state,
                opts.user_id,
                &opts.anime_title,
                &opts.episode_title,
                opts.cover_image.as_deref(),
                start_time,
                end_time,
                true,
                opts.is_nsfw,
            ).await;
        }
    }

    /// Send a raw JSON IPC command to MPV over the socket.
    /// See https://mpv.io/manual/master/#json-ipc
    pub async fn send_command(&self, _command: serde_json::Value) -> CoreResult<()> {
        // TODO: open socket, write command, read response
        Ok(())
    }

    pub fn is_running(&mut self) -> bool {
        self.process.try_wait().map(|s| s.is_none()).unwrap_or(false)
    }

    pub fn kill(&mut self) -> CoreResult<()> {
        self.process.kill().map_err(CoreError::Io)
    }

    fn socket_path() -> String {
        #[cfg(unix)]
        {
            std::env::temp_dir()
                .join("hoshi-mpv.sock")
                .to_string_lossy()
                .to_string()
        }
        #[cfg(windows)]
        {
            r"\\.\pipe\hoshi-mpv".to_string()
        }
    }
}

impl Drop for MpvService {
    fn drop(&mut self) {
        let _ = self.process.kill();

        #[cfg(unix)]
        let _ = std::fs::remove_file(&self.socket_path);
    }
}

fn write_chapters_file(chapters: &[EpisodeChapter]) -> std::io::Result<PathBuf> {
    let path = std::env::temp_dir().join(format!(
        "hoshi-chapters-{}.ffmetadata",
        uuid::Uuid::new_v4()
    ));

    let mut sorted_chapters = chapters.to_vec();
    sorted_chapters.sort_by(|a, b| a.start.partial_cmp(&b.start).unwrap_or(std::cmp::Ordering::Equal));

    let mut content = String::from(";FFMETADATA1\n\n");
    let mut current_time = 0.0;

    for chapter in sorted_chapters {
        if chapter.start > current_time + 0.01 {
            content.push_str("[CHAPTER]\n");
            content.push_str("TIMEBASE=1/1000\n");
            content.push_str(&format!("START={}\n", (current_time * 1000.0) as u64));
            content.push_str(&format!("END={}\n", (chapter.start * 1000.0) as u64));
            content.push_str("title=Episode\n\n"); // Generic fallback layout marker
        }

        content.push_str("[CHAPTER]\n");
        content.push_str("TIMEBASE=1/1000\n");
        content.push_str(&format!("START={}\n", (chapter.start * 1000.0) as u64));
        content.push_str(&format!("END={}\n", (chapter.end * 1000.0) as u64));
        content.push_str(&format!("title={}\n\n", chapter.title));

        current_time = chapter.end;
    }

    std::fs::write(&path, content)?;

    Ok(path)
}

fn is_intro(title: &str) -> bool {
    let title = title.to_lowercase();

    title.contains("intro")
        || title.contains("opening")
        || title == "op"
}

fn is_outro(title: &str) -> bool {
    let title = title.to_lowercase();

    title.contains("outro")
        || title.contains("ending")
        || title == "ed"
}