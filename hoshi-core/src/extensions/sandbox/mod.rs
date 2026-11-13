use rquickjs::context::EvalOptions;
use rquickjs::{AsyncContext, AsyncRuntime, CatchResultExt};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tracing::{error, instrument, warn};

mod script_builder;
pub mod native_apis;

use script_builder::build_sandbox_script;
use native_apis::register_native_apis;

use crate::error::{CoreError, CoreResult};
use crate::extensions::{ExtensionStateStore, TORRENT};
use crate::extensions::{ANIME, BASE, MANGA, NOVEL};
use crate::extensions::sandbox::native_apis::ConsoleBuffer;
use crate::extensions::types::{CompatLayer, ExtensionType};
use crate::headless::{HeadlessHandle, HeadlessOptions};

const SCRIPT_DEADLINE: std::time::Duration = std::time::Duration::from_secs(25);
const HEADLESS_FETCH_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(20);
const MAX_SLEEP_MS: u64 = 30_000;

const FETCH_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(20);
const REPLY_GRACE: std::time::Duration = std::time::Duration::from_secs(2);

/// Lets the (synchronous) native JS functions run network work on the app's
/// tokio runtime. Previously every extension call spun up two OS threads, each
/// with its own multi-thread runtime, and served fetches one at a time; a timed
/// out fetch kept that single worker busy and stalled every fetch behind it.
#[derive(Clone)]
pub(crate) struct NativeBridge {
    rt: tokio::runtime::Handle,
    http_client: reqwest::Client,
    headless: HeadlessHandle,
    deadline: std::time::Instant,
}

impl NativeBridge {
    /// Time left before the script deadline, capped at `cap`. None once it passed.
    fn budget(&self, cap: std::time::Duration) -> Option<std::time::Duration> {
        let left = self.deadline.checked_duration_since(std::time::Instant::now())?;
        (!left.is_zero()).then(|| left.min(cap))
    }

    /// Blocking: runs `fut` on the app runtime and waits for it, at most `cap`
    /// (and never past the script deadline). Called from the QuickJS thread,
    /// which is not a runtime worker, so blocking here is fine.
    fn run_blocking<F>(&self, cap: std::time::Duration, what: &str, fut: F) -> String
    where
        F: Future<Output = String> + Send + 'static,
    {
        let Some(wait) = self.budget(cap) else {
            return error_json(format!("{what}: script deadline exceeded"));
        };
        let (tx, rx) = std::sync::mpsc::sync_channel::<String>(1);
        let task = self.rt.spawn(async move {
            let _ = tx.send(fut.await);
        });
        match rx.recv_timeout(wait) {
            Ok(v) => v,
            Err(_) => {
                task.abort(); // nobody is waiting for it anymore
                warn!("{what} timed out");
                error_json(format!("{what} timeout"))
            }
        }
    }

    pub(crate) fn fetch(&self, url: String, method: String, headers: HashMap<String, String>, body: Option<String>) -> String {
        let client = self.http_client.clone();
        self.run_blocking(FETCH_TIMEOUT + REPLY_GRACE, "fetch", async move {
            let mut r = match method.to_uppercase().as_str() {
                "POST"   => client.post(&url),
                "PUT"    => client.put(&url),
                "DELETE" => client.delete(&url),
                "PATCH"  => client.patch(&url),
                _        => client.get(&url),
            }
            .timeout(FETCH_TIMEOUT);
            for (k, v) in &headers {
                r = r.header(k.as_str(), v.as_str());
            }
            if let Some(b) = body {
                r = r.body(b);
            }

            match r.send().await {
                Err(e) => error_json(e.to_string()),
                Ok(resp) => {
                    let status = resp.status().as_u16();
                    let ok = resp.status().is_success();
                    let cookies: HashMap<String, String> = resp.headers()
                        .get_all("set-cookie")
                        .iter()
                        .filter_map(|v| v.to_str().ok())
                        .filter_map(|s| {
                            let pair = s.split(';').next()?;
                            let mut kv = pair.splitn(2, '=');
                            let k = kv.next()?.trim().to_string();
                            let v = kv.next()?.trim().to_string();
                            Some((k, v))
                        })
                        .collect();

                    match resp.text().await {
                        Err(e) => error_json(e.to_string()),
                        Ok(text) => serde_json::json!({
                            "ok": ok,
                            "status": status,
                            "body": text,
                            "cookies": cookies,
                        }).to_string(),
                    }
                }
            }
        })
    }

    pub(crate) fn headless(&self, url: String, options: HeadlessOptions) -> String {
        let headless = self.headless.clone();
        self.run_blocking(HEADLESS_FETCH_TIMEOUT + REPLY_GRACE, "headless fetch", async move {
            match tokio::time::timeout(HEADLESS_FETCH_TIMEOUT, headless.fetch(&url, options)).await {
                Ok(Ok(resp)) => serde_json::to_string(&resp)
                    .unwrap_or_else(|e| error_json(e.to_string())),
                Ok(Err(e)) => error_json(e.to_string()),
                Err(_) => error_json("headless fetch timed out".into()),
            }
        })
    }

    /// Sleep, but never past the deadline.
    pub(crate) fn sleep(&self, ms: u64) {
        let cap = std::time::Duration::from_millis(ms.min(MAX_SLEEP_MS));
        if let Some(d) = self.budget(cap) {
            std::thread::sleep(d);
        }
    }
}

pub(crate) async fn execute_in_quickjs(
    extension_code: String,
    function_name: String,
    args: Vec<Value>,
    headless: HeadlessHandle,
    settings: HashMap<String, Value>,
    extension_id: String,
    state_store: ExtensionStateStore,
    compat_layer: Option<CompatLayer>,
    ext_type: ExtensionType,
    http_client: reqwest::Client,
    console_buffer: Option<ConsoleBuffer>,
) -> CoreResult<Value> {
    let base_classes = format!("{}\n{}\n{}\n{}\n{}", BASE, ANIME, MANGA, NOVEL, TORRENT);

    let args_json = serde_json::to_string(&args).map_err(|e| {
        error!(error = ?e, "Failed to serialize sandbox arguments");
        CoreError::Internal("error.sandbox.serialization_failed".into())
    })?;

    let settings_json = serde_json::to_string(&settings).map_err(|e| {
        error!(error = ?e, "Failed to serialize sandbox settings");
        CoreError::Internal("error.sandbox.serialization_failed".into())
    })?;

    let initial_state: HashMap<String, Value> = {
        let store = state_store.lock().unwrap_or_else(|p| p.into_inner());
        store.get(&extension_id).cloned().unwrap_or_default()
    };
    let state_json = serde_json::to_string(&initial_state).map_err(|e| {
        error!(error = ?e, "Failed to serialize sandbox state");
        CoreError::Internal("error.sandbox.serialization_failed".into())
    })?;

    let full_script = build_sandbox_script(
        &base_classes,
        compat_layer,
        &extension_code,
        &function_name,
        &args_json,
        &settings_json,
        &ext_type,
    );

    let headless_available = headless.is_available();
    let deadline = std::time::Instant::now() + SCRIPT_DEADLINE;
    let bridge = NativeBridge {
        rt: tokio::runtime::Handle::current(),
        http_client,
        headless,
        deadline,
    };

    let (json_str, updated_state_json) = tokio::task::spawn_blocking({
        let extension_id = extension_id.clone();
        let console_buffer = console_buffer.clone();
        move || {
            let rt = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .map_err(|e| {
                    error!(error = ?e, "Failed to build tokio runtime for QuickJS");
                    CoreError::Internal("error.sandbox.runtime_init_failed".into())
                })?;
            tokio::task::LocalSet::new().block_on(
                &rt,
                run_quickjs_local(full_script, extension_code, headless_available, bridge, state_json, extension_id, console_buffer),
            )
        }
    })
        .await
        .map_err(|e| {
            error!(error = ?e, "Sandbox spawn_blocking thread panicked");
            CoreError::Internal("error.sandbox.thread_panicked".into())
        })??;

    // Write back only what this call changed. Replacing the whole map made
    // concurrent calls of the same extension overwrite each other's state.
    if let Ok(new_state) =
        serde_json::from_str::<HashMap<String, Value>>(&updated_state_json)
    {
        let mut store = state_store.lock().unwrap_or_else(|p| p.into_inner());
        let entry = store.entry(extension_id).or_default();
        for (k, v) in &new_state {
            if initial_state.get(k) != Some(v) {
                entry.insert(k.clone(), v.clone());
            }
        }
        for k in initial_state.keys() {
            if !new_state.contains_key(k) {
                entry.remove(k);
            }
        }
    }

    serde_json::from_str::<Value>(&json_str).map_err(|e| {
        error!(error = ?e, "Failed to parse JSON result from sandbox");
        CoreError::Internal("error.sandbox.bad_json_response".into())
    })
}

#[instrument(skip(full_script, extension_code, bridge, state_json))]
async fn run_quickjs_local(
    full_script: String,
    extension_code: String,
    headless_available: bool,
    bridge: NativeBridge,
    state_json: String,
    extension_id: String,
    console_buffer: Option<ConsoleBuffer>,
) -> CoreResult<(String, String)> {
    unsafe {
        let locale = std::ffi::CString::new("C").unwrap();
        libc::setlocale(libc::LC_NUMERIC, locale.as_ptr());
    }

    let rt = AsyncRuntime::new().map_err(|e| {
        error!(error = ?e, "Failed to instantiate QuickJS runtime");
        CoreError::Internal("error.sandbox.runtime_init_failed".into())
    })?;

    rt.set_memory_limit(64 * 1024 * 1024).await;
    rt.set_max_stack_size(512 * 1024).await;

    let deadline = bridge.deadline;
    rt.set_interrupt_handler(Some(Box::new(move || {
        std::time::Instant::now() >= deadline
    })))
        .await;

    let ctx = AsyncContext::full(&rt).await.map_err(|e| {
        error!(error = ?e, "Failed to create QuickJS context");
        CoreError::Internal("error.sandbox.runtime_init_failed".into())
    })?;

    let state_map: Arc<Mutex<HashMap<String, Value>>> = Arc::new(Mutex::new(
        serde_json::from_str(&state_json).unwrap_or_default(),
    ));

    let full_script_for_error = full_script.clone();
    let state_map_for_output = Arc::clone(&state_map);

    let result: Result<String, String> = ctx.async_with(async |ctx| {
        register_native_apis(&ctx, headless_available, bridge, Arc::clone(&state_map), extension_id, console_buffer)
            .catch(&ctx)
            .map_err(|e| e.to_string())?;

        let val = ctx
            .eval_with_options::<rquickjs::Value, _>(full_script.as_bytes(), EvalOptions::default())
            .catch(&ctx)
            .map_err(|e| e.to_string())?;

        let resolved = if val.is_promise() {
            val.into_promise().unwrap()
                .into_future::<rquickjs::Value>()
                .await
                .catch(&ctx)
                .map_err(|e| e.to_string())?
        } else {
            val
        };

        match ctx.json_stringify(resolved).catch(&ctx).map_err(|e| e.to_string())? {
            Some(s) => s.to_string().map_err(|e| e.to_string()),
            None    => Ok("null".to_string()),
        }
    }).await;

    result
        .map_err(|e| {
            fn parse_loc(line: &str, marker: &str) -> Option<(usize, usize)> {
                let rest = line.split(marker).nth(1)?;
                let mut parts = rest.splitn(3, ':');
                let ln: usize = parts.next()?.trim().parse().ok()?;
                let col: usize = parts.next()?.trim_end_matches(|c: char| !c.is_numeric()).parse().ok()?;
                Some((ln, col))
            }

            fn make_snippet(source: &str, line: usize, col: usize) -> String {
                source
                    .lines()
                    .enumerate()
                    .filter(|(i, _)| (*i + 1).abs_diff(line) <= 3)
                    .map(|(i, l)| {
                        if i + 1 == line {
                            format!(">>> {:4} | {}    <-- col {}", i + 1, l, col)
                        } else {
                            format!("    {:4} | {}", i + 1, l)
                        }
                    })
                    .collect::<Vec<_>>()
                    .join("\n")
            }

            let mut input_loc: Option<(usize, usize)> = None;
            let mut eval_loc: Option<(usize, usize)> = None;
            for l in e.lines() {
                if input_loc.is_none() && l.contains("<input>:") {
                    input_loc = parse_loc(l, "<input>:");
                }
                if eval_loc.is_none() && l.contains("eval_script:") {
                    eval_loc = parse_loc(l, "eval_script:");
                }
            }

            const INPUT_LINE_OFFSET: usize = 2;

            let (snippet, line, col, source_label) = match (input_loc, eval_loc) {
                (Some((raw_line, col)), _) => {
                    let line = raw_line.saturating_sub(INPUT_LINE_OFFSET);
                    (make_snippet(&extension_code, line, col), line, col, "extension_code")
                }
                (None, Some((line, col))) => {
                    (make_snippet(&full_script_for_error, line, col), line, col, "full_script")
                }
                (None, None) => (String::new(), 0, 0, "unknown"),
            };

            const MAX_SNIPPET_LEN: usize = 2000;
            let snippet = if snippet.len() > MAX_SNIPPET_LEN {
                let mut truncated: String = snippet.chars().take(MAX_SNIPPET_LEN).collect();
                truncated.push_str(&format!("... [truncated, {} bytes total]", snippet.len()));
                truncated
            } else {
                snippet
            };

            warn!(
            error = %e,
            line = line,
            col = col,
            source = source_label,
            snippet = %snippet,
            "Sandbox JS exception"
        );

            let context = if !snippet.is_empty() {
                format!("\n--- {} @ {}:{} ---\n{}", source_label, line, col, snippet)
            } else {
                String::new()
            };
            CoreError::Internal(format!("error.sandbox.js_exception: {e}{context}"))
        })
        .map(|json_str| {
            let updated_state_json = {
                let guard = state_map_for_output.lock().unwrap_or_else(|p| p.into_inner());
                serde_json::to_string(&*guard).unwrap_or_else(|_| "{}".to_string())
            };
            (json_str, updated_state_json)
        })
}

#[inline]
pub(crate) fn error_json(msg: String) -> String {
    serde_json::json!({ "error": msg }).to_string()
}