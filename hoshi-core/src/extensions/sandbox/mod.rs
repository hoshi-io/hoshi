use rquickjs::context::EvalOptions;
use rquickjs::{async_with, AsyncContext, AsyncRuntime, CatchResultExt};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tracing::{error, instrument, warn};

mod script_builder;
pub mod native_apis;

use script_builder::build_sandbox_script;
use native_apis::register_native_apis;

use crate::error::{CoreError, CoreResult};
use crate::extensions::ExtensionStateStore;
use crate::extensions::{ANIME, BASE, MANGA, NOVEL};
use crate::extensions::sandbox::native_apis::ConsoleBuffer;
use crate::extensions::types::{CompatLayer, ExtensionType};
use crate::headless::{HeadlessHandle, HeadlessOptions};

const SCRIPT_DEADLINE: std::time::Duration = std::time::Duration::from_secs(25);
const HEADLESS_FETCH_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(20);
const MAX_SLEEP_MS: u64 = 30_000;

pub(crate) struct HeadlessRequest {
    url:     String,
    options: HeadlessOptions,
    reply:   std::sync::mpsc::SyncSender<String>,
}

pub(crate) struct FetchRequest {
    pub(crate) url:     String,
    pub(crate) method:  String,
    pub(crate) headers: HashMap<String, String>,
    pub(crate) body:    Option<String>,
    pub(crate) reply:   std::sync::mpsc::SyncSender<String>,
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
    let base_classes = format!("{}\n{}\n{}\n{}", BASE, ANIME, MANGA, NOVEL);

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
    let (req_tx, req_rx) = std::sync::mpsc::sync_channel::<HeadlessRequest>(4);
    let (fetch_tx, fetch_rx) = std::sync::mpsc::sync_channel::<FetchRequest>(8);

    let fetch_thread = std::thread::spawn({
        let http_client = http_client.clone();
        move || {
            let rt = tokio::runtime::Builder::new_multi_thread()
                .worker_threads(2)
                .enable_all()
                .build()
                .expect("fetch thread runtime");

            while let Ok(req) = fetch_rx.recv() {
                let client = http_client.clone();
                let result = rt.block_on(async move {
                    let mut r = match req.method.to_uppercase().as_str() {
                        "POST"   => client.post(&req.url),
                        "PUT"    => client.put(&req.url),
                        "DELETE" => client.delete(&req.url),
                        "PATCH"  => client.patch(&req.url),
                        _        => client.get(&req.url),
                    };
                    for (k, v) in &req.headers {
                        r = r.header(k.as_str(), v.as_str());
                    }
                    if let Some(b) = req.body {
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
                });
                let _ = req.reply.send(result);
            }
        }
    });

    let headless_thread = std::thread::spawn(move || {
        let rt = tokio::runtime::Builder::new_multi_thread()
            .worker_threads(2)
            .enable_all()
            .build()
            .expect("headless thread runtime");

        while let Ok(req) = req_rx.recv() {
            let headless = headless.clone();
            let result = rt.block_on(async move {
                match tokio::time::timeout(
                    HEADLESS_FETCH_TIMEOUT,
                    headless.fetch(&req.url, req.options),
                )
                    .await
                {
                    Ok(Ok(resp)) => serde_json::to_string(&resp)
                        .unwrap_or_else(|e| error_json(e.to_string())),
                    Ok(Err(e)) => error_json(e.to_string()),
                    Err(_) => {
                        warn!("Headless fetch exceeded internal deadline, aborting future");
                        error_json("headless fetch timed out".into())
                    }
                }
            });
            let _ = req.reply.send(result);
        }
    });

    let (json_str, updated_state_json) = tokio::task::spawn_blocking({
        let req_tx = req_tx.clone();
        let fetch_tx = fetch_tx.clone();
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
                run_quickjs_local(full_script, extension_code, headless_available, req_tx, fetch_tx, state_json, extension_id, console_buffer),
            )
        }
    })
        .await
        .map_err(|e| {
            error!(error = ?e, "Sandbox spawn_blocking thread panicked");
            CoreError::Internal("error.sandbox.thread_panicked".into())
        })??;

    drop(req_tx);
    drop(fetch_tx);
    let _ = headless_thread.join();
    let _ = fetch_thread.join();

    if let Ok(new_state) =
        serde_json::from_str::<HashMap<String, Value>>(&updated_state_json)
    {
        let mut store = state_store.lock().unwrap_or_else(|p| p.into_inner());
        store.insert(extension_id, new_state);
    }

    serde_json::from_str::<Value>(&json_str).map_err(|e| {
        error!(error = ?e, "Failed to parse JSON result from sandbox");
        CoreError::Internal("error.sandbox.bad_json_response".into())
    })
}

#[instrument(skip(full_script, extension_code, req_tx, state_json))]
async fn run_quickjs_local(
    full_script: String,
    extension_code: String,
    headless_available: bool,
    req_tx: std::sync::mpsc::SyncSender<HeadlessRequest>,
    fetch_tx: std::sync::mpsc::SyncSender<FetchRequest>,
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

    let deadline = std::time::Instant::now() + SCRIPT_DEADLINE;
    rt.set_interrupt_handler(Some(Box::new(move || {
        std::time::Instant::now() >= deadline
    })))
        .await;

    let ctx = AsyncContext::full(&rt).await.map_err(|e| {
        error!(error = ?e, "Failed to create QuickJS context");
        CoreError::Internal("error.sandbox.runtime_init_failed".into())
    })?;

    let req_tx = Arc::new(req_tx);
    let fetch_tx = Arc::new(fetch_tx);
    let state_map: Arc<Mutex<HashMap<String, Value>>> = Arc::new(Mutex::new(
        serde_json::from_str(&state_json).unwrap_or_default(),
    ));

    let full_script_for_error = full_script.clone();
    let state_map_for_output = Arc::clone(&state_map);

    let result: Result<String, String> = async_with!(ctx => |ctx| {
        register_native_apis(&ctx, headless_available, req_tx, fetch_tx, Arc::clone(&state_map), extension_id, console_buffer)
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