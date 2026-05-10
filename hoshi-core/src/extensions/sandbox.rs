use rquickjs::context::EvalOptions;
use rquickjs::{
    async_with, AsyncContext, AsyncRuntime, CatchResultExt, Function,
};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tracing::{debug, error, instrument, warn};

use crate::error::{CoreError, CoreResult};
use crate::extensions::ExtensionStateStore;
use crate::extensions::{ANIME, BASE, MANGA, NOVEL, SANDBOX_BOOTSTRAP};
use crate::headless::{HeadlessHandle, HeadlessOptions};

pub(crate) async fn execute_in_quickjs(
    extension_code: String,
    function_name: String,
    args: Vec<Value>,
    headless: HeadlessHandle,
    settings: HashMap<String, Value>,
    extension_id: String,
    state_store: ExtensionStateStore,
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
        &extension_code,
        &function_name,
        &args_json,
        &settings_json,
    );

    let headless_available = headless.is_available();
    let (req_tx, req_rx) = std::sync::mpsc::sync_channel::<HeadlessRequest>(4);

    let headless_thread = std::thread::spawn(move || {
        let rt = tokio::runtime::Builder::new_multi_thread()
            .worker_threads(2)
            .enable_all()
            .build()
            .expect("headless thread runtime");

        while let Ok(req) = req_rx.recv() {
            let headless = headless.clone();
            let result = rt.block_on(async move {
                match headless.fetch(&req.url, req.options).await {
                    Ok(resp) => serde_json::to_string(&resp)
                        .unwrap_or_else(|e| error_json(e.to_string())),
                    Err(e) => error_json(e.to_string()),
                }
            });
            let _ = req.reply.send(result);
        }
    });

    let (json_str, updated_state_json) = tokio::task::spawn_blocking({
        let req_tx = req_tx.clone();
        let extension_id = extension_id.clone();
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
                run_quickjs_local(full_script, headless_available, req_tx, state_json, extension_id),
            )
        }
    })
    .await
    .map_err(|e| {
        error!(error = ?e, "Sandbox spawn_blocking thread panicked");
        CoreError::Internal("error.sandbox.thread_panicked".into())
    })??;

    drop(req_tx);
    let _ = headless_thread.join();

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

struct HeadlessRequest {
    url:     String,
    options: HeadlessOptions,
    reply:   std::sync::mpsc::SyncSender<String>,
}

#[instrument(skip(full_script, req_tx, state_json))]
async fn run_quickjs_local(
    full_script: String,
    headless_available: bool,
    req_tx: std::sync::mpsc::SyncSender<HeadlessRequest>,
    state_json: String,
    extension_id: String,
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

    let ctx = AsyncContext::full(&rt).await.map_err(|e| {
        error!(error = ?e, "Failed to create QuickJS context");
        CoreError::Internal("error.sandbox.runtime_init_failed".into())
    })?;

    let req_tx = Arc::new(req_tx);
    let state_map: Arc<Mutex<HashMap<String, Value>>> = Arc::new(Mutex::new(
        serde_json::from_str(&state_json).unwrap_or_default(),
    ));

    let state_map_for_output = Arc::clone(&state_map);
    let result: Result<String, String> = async_with!(ctx => |ctx| {
        register_native_apis(&ctx, headless_available, req_tx, Arc::clone(&state_map), extension_id)
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
            warn!(error = %e, "Sandbox execution threw a JavaScript exception");
            CoreError::BadRequest("error.sandbox.execution_failed".into())
        })
        .map(|json_str| {
            let updated_state_json = {
                let guard = state_map_for_output.lock().unwrap_or_else(|p| p.into_inner());
                serde_json::to_string(&*guard).unwrap_or_else(|_| "{}".to_string())
            };
            (json_str, updated_state_json)
        })
}

fn build_sandbox_script(
    base_classes: &str,
    extension_code: &str,
    function_name: &str,
    args_json: &str,
    settings_json: &str,
) -> String {
    let ext_code_repr = serde_json::to_string(extension_code).unwrap_or_default();
    format!(r#"
{bootstrap}

globalThis.__settings = Object.freeze({settings});

{base}

(async () => {{
    const VALID_BASES = ["Base", "Anime", "Manga", "Novel"];

    const src   = {ext_repr};
    const match = src.match(/class\s+([a-zA-Z0-9_]+)\s+extends\s+([a-zA-Z0-9_]+)/);
    if (!match) throw new Error("No class extending a base was found in the extension");

    const [, className, parentName] = match;
    if (!VALID_BASES.includes(parentName))
        throw new Error(`Class must extend one of: ${{VALID_BASES.join(", ")}}. Got: ${{parentName}}`);

    const ExtClass = new Function("Base", "Anime", "Manga", "Novel", `
${{src}}
return ${{className}};
`)(Base, Anime, Manga, Novel);

    if (typeof ExtClass !== "function")
        throw new Error(`Class '${{className}}' could not be loaded`);

    const instance = new ExtClass();
    const callable = typeof instance[`_{fn}`] === "function" ? `_{fn}` : "{fn}";

    if (typeof instance[callable] !== "function")
        throw new Error(`Method "{fn}" does not exist on ${{className}}`);

    return await instance[callable](...{args});
}})()
"#,
        bootstrap = SANDBOX_BOOTSTRAP,
        base      = base_classes,
        ext_repr  = ext_code_repr,
        fn        = function_name,
        args      = args_json,
        settings  = settings_json,
    )
}

fn register_native_apis(
    ctx: &rquickjs::Ctx<'_>,
    headless_available: bool,
    req_tx: Arc<std::sync::mpsc::SyncSender<HeadlessRequest>>,
    state_map: Arc<Mutex<HashMap<String, Value>>>,
    extension_id: String,
) -> rquickjs::Result<()> {
    let globals = ctx.globals();

    globals.set(
        "__native_log",
        Function::new(ctx.clone(), {
            let extension_id = extension_id.clone();
            move |msg: String| {
                debug!(target: "sandbox_js", extension = %extension_id, "{}", msg);
                Ok::<(), rquickjs::Error>(())
            }
        })?,
    )?;

    globals.set("__native_fetch", Function::new(ctx.clone(),
        |url: String, method: String, headers_json: String, body: String| {
            let headers: HashMap<String, String> = serde_json::from_str(&headers_json).unwrap_or_default();
            let body_opt = if body.is_empty() { None } else { Some(body) };

            debug!(method = %method, url = %url, "Native fetch called from sandbox");

            let result = std::thread::spawn(move || -> String {
                let client = match reqwest::blocking::Client::builder()
                    .timeout(std::time::Duration::from_secs(15))
                    .connect_timeout(std::time::Duration::from_secs(5))
                    .user_agent("Mozilla/5.0 (X11; Linux x86_64; rv:147.0) Gecko/20100101 Firefox/147.0")
                    .build()
                {
                    Ok(c)  => c,
                    Err(e) => return error_json(e.to_string()),
                };

                let mut req = match method.to_uppercase().as_str() {
                    "POST"   => client.post(&url),
                    "PUT"    => client.put(&url),
                    "DELETE" => client.delete(&url),
                    "PATCH"  => client.patch(&url),
                    _        => client.get(&url),
                };

                for (k, v) in &headers {
                    req = req.header(k.as_str(), v.as_str());
                }

                if let Some(b) = body_opt {
                    req = req.body(b);
                }

                match req.send() {
                    Err(e)   => {
                        warn!(error = %e, url = %url, "Native fetch request failed");
                        error_json(e.to_string())
                    },
                    Ok(resp) => {
                        let status = resp.status().as_u16();
                        let ok     = resp.status().is_success();
                        match resp.text() {
                            Err(e)   => error_json(e.to_string()),
                            Ok(text) => serde_json::json!({ "ok": ok, "status": status, "body": text }).to_string(),
                        }
                    }
                }
            }).join().unwrap_or_else(|_| error_json("fetch thread panicked".into()));

            Ok::<String, rquickjs::Error>(result)
        },
    )?)?;

    globals.set("__native_html_query", Function::new(ctx.clone(),
        |html: String, selector: String| -> rquickjs::Result<String> {
            use scraper::{Html, Selector};
            let document = Html::parse_document(&html);
            let sel = match Selector::parse(&selector) {
                Ok(s)  => s,
                Err(e) => return Ok(serde_json::json!({ "error": format!("Invalid selector: {:?}", e) }).to_string()),
            };

            let results: Vec<Value> = document.select(&sel).map(|el| {
                let attrs: HashMap<String, String> = el.value().attrs()
                    .map(|(k, v)| (k.to_string(), v.to_string()))
                    .collect();
                serde_json::json!({
                    "text":  el.text().collect::<Vec<_>>().join(""),
                    "html":  el.inner_html(),
                    "outer": el.html(),
                    "attrs": attrs,
                })
            }).collect();

            Ok(serde_json::to_string(&results).unwrap_or_default())
        },
    )?)?;

    globals.set("__headless_available", headless_available)?;

    globals.set(
        "__native_headless_sync",
        Function::new(
            ctx.clone(),
            move |url: String, options_json: String| -> rquickjs::Result<String> {
                let options: HeadlessOptions =
                    serde_json::from_str(&options_json).unwrap_or_default();
                let (reply_tx, reply_rx) = std::sync::mpsc::sync_channel::<String>(1);

                debug!(url = %url, "Native headless sync called from sandbox");

                if req_tx
                    .send(HeadlessRequest {
                        url,
                        options,
                        reply: reply_tx,
                    })
                    .is_err()
                {
                    warn!("Headless channel closed unexpectedly");
                    return Ok(error_json("headless channel closed".into()));
                }

                Ok(
                    reply_rx
                        .recv_timeout(std::time::Duration::from_secs(15))
                        .unwrap_or_else(|_| {
                            warn!("Headless fetch timed out");
                            error_json("headless timeout".into())
                        }),
                )
            },
        )?,
    )?;

    {
        let state_map = Arc::clone(&state_map);
        globals.set(
            "__native_state_get",
            Function::new(
                ctx.clone(),
                move |key: String| -> rquickjs::Result<String> {
                    let guard = state_map.lock().unwrap_or_else(|p| p.into_inner());
                    let value = guard.get(&key).cloned().unwrap_or(Value::Null);
                    Ok(
                        serde_json::to_string(&value)
                            .unwrap_or_else(|_| "null".to_string()),
                    )
                },
            )?,
        )?;
    }

    {
        let state_map = Arc::clone(&state_map);
        globals.set(
            "__native_state_set",
            Function::new(
                ctx.clone(),
                move |key: String, json_val: String| -> rquickjs::Result<()> {
                    let value: Value =
                        serde_json::from_str(&json_val).unwrap_or(Value::Null);
                    let mut guard = state_map.lock().unwrap_or_else(|p| p.into_inner());
                    guard.insert(key, value);
                    Ok(())
                },
            )?,
        )?;
    }

    {
        let state_map = Arc::clone(&state_map);
        globals.set(
            "__native_state_delete",
            Function::new(ctx.clone(), move |key: String| -> rquickjs::Result<()> {
                let mut guard = state_map.lock().unwrap_or_else(|p| p.into_inner());
                guard.remove(&key);
                Ok(())
            })?,
        )?;
    }

    {
        let state_map = Arc::clone(&state_map);
        globals.set(
            "__native_state_keys",
            Function::new(ctx.clone(), move || -> rquickjs::Result<String> {
                let guard = state_map.lock().unwrap_or_else(|p| p.into_inner());
                let keys: Vec<&String> = guard.keys().collect();
                Ok(serde_json::to_string(&keys).unwrap_or_else(|_| "[]".to_string()))
            })?,
        )?;
    }

    Ok(())
}

#[inline]
fn error_json(msg: String) -> String {
    serde_json::json!({ "error": msg }).to_string()
}