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
use crate::extensions::types::CompatLayer;
use crate::headless::{HeadlessHandle, HeadlessOptions};

pub(crate) async fn execute_in_quickjs(
    extension_code: String,
    function_name: String,
    args: Vec<Value>,
    headless: HeadlessHandle,
    settings: HashMap<String, Value>,
    extension_id: String,
    state_store: ExtensionStateStore,
    compat_layer: Option<CompatLayer>,
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
                run_quickjs_local(full_script, extension_code, headless_available, req_tx, state_json, extension_id),
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

#[instrument(skip(full_script, extension_code, req_tx, state_json))]
async fn run_quickjs_local(
    full_script: String,
    extension_code: String,
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

    let full_script_for_error = full_script.clone();
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

            let (snippet, line, col, source_label) = match (input_loc, eval_loc) {
                (Some((line, col)), _) => {
                    (make_snippet(&extension_code, line, col), line, col, "extension_code")
                }
                (None, Some((line, col))) => {
                    (make_snippet(&full_script_for_error, line, col), line, col, "full_script")
                }
                (None, None) => (String::new(), 0, 0, "unknown"),
            };

            warn!(
            error = %e,
            line = line,
            col = col,
            source = source_label,
            snippet = %snippet,
            "Sandbox JS exception"
        );
            CoreError::BadRequest(format!(
                "error.sandbox.execution_failed [{source_label} line {line}:{col}]\n{snippet}"
            ).into())
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
    compat_layer: Option<CompatLayer>,
    extension_code: &str,
    function_name: &str,
    args_json: &str,
    settings_json: &str,
) -> String {
    let ext_code_repr = serde_json::to_string(extension_code).unwrap_or_default();

    let (compat_js, runner) = match &compat_layer {
        Some(CompatLayer::Lnreader(js)) => {
            let runner = format!(
                r#"(async () => {{
    const src = {ext_repr};
    eval(src);
    const ExtClass = __lnr_buildNovelClass();
    const instance = new ExtClass();
    const fn_name = "{fn}";
    if (typeof instance[fn_name] !== "function")
        throw new Error(`Method "${{fn_name}}" not found on compat class`);
    return await instance[fn_name](...{args});
}})()"#,
                ext_repr = ext_code_repr,
                fn       = function_name,
                args     = args_json,
            );
            (js.clone(), runner)
        }

        Some(CompatLayer::Tachiyomi(js)) => {
            let runner = format!(
                r#"(async () => {{
    globalThis.__tachi_captured = null;

    const src = {ext_repr};

    const patched = src.replace(
        /class\s+([a-zA-Z0-9_$]+)\s+extends\s+(HttpSource|ParsedHttpSource|Manga)/,
        'globalThis.__tachi_captured = class $1 extends $2'
    );

    eval(patched);

    const ExtClass = globalThis.__tachi_captured;

    if (!ExtClass)
        throw new Error("[tachi-compat] No class extending HttpSource found");

    const instance = new ExtClass();

    const fn_name = "{fn}";

    if (typeof instance[fn_name] !== "function")
        throw new Error(`Method "${{fn_name}}" not found on compat class`);

    return await instance[fn_name](...{args});
}})()"#,
                ext_repr = ext_code_repr,
                fn       = function_name,
                args     = args_json,
            );

            (js.clone(), runner)
        }

        None => {
            let runner = format!(
                r#"(async () => {{
    const VALID_BASES = ["Base", "Anime", "Manga", "Novel"];

    const src            = {ext_repr};
    const classNameMatch = src.match(/class\s+([a-zA-Z0-9_]+)\s+extends\s+([a-zA-Z0-9_]+)/);
    if (!classNameMatch) throw new Error("No class extending a base was found in the extension");

    const [, className, parentName] = classNameMatch;
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
}})()"#,
                ext_repr = ext_code_repr,
                fn       = function_name,
                args     = args_json,
            );
            (String::new(), runner)
        }
    };

    format!(
        r#"
{bootstrap}

globalThis.__settings = Object.freeze({settings});

{base}

{compat}

{runner}
"#,
        bootstrap = SANDBOX_BOOTSTRAP,
        settings  = settings_json,
        base      = base_classes,
        compat    = compat_js,
        runner    = runner,
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

    globals.set("__native_crypto_hash", Function::new(ctx.clone(),
      |algo: String, data: String| -> rquickjs::Result<String> {
          use sha2::{Sha256, Sha512, Digest};
          use sha1::Sha1;
          use md5::Md5;
          let b = data.as_bytes();
          let out = match algo.as_str() {
              "md5"    => { let mut h = Md5::new();    h.update(b); hex::encode(h.finalize()) }
              "sha1"   => { let mut h = Sha1::new();   h.update(b); hex::encode(h.finalize()) }
              "sha256" => { let mut h = Sha256::new(); h.update(b); hex::encode(h.finalize()) }
              "sha512" => { let mut h = Sha512::new(); h.update(b); hex::encode(h.finalize()) }
              _        => return Ok(error_json(format!("unknown hash algo: {}", algo))),
          };
          Ok(out)
      },
    )?)?;

    globals.set("__native_crypto_hmac", Function::new(ctx.clone(),
      |algo: String, key_hex: String, data: String| -> rquickjs::Result<String> {
          use hmac::{Hmac, Mac};
          use sha2::{Sha256, Sha512};
          use sha1::Sha1;
          let key = match hex::decode(&key_hex) {
              Ok(k) => k,
              Err(e) => return Ok(error_json(format!("hmac: bad key hex: {}", e))),
          };
          let out = match algo.as_str() {
              "sha1" => {
                  let mut m = Hmac::<Sha1>::new_from_slice(&key).map_err(|e| rquickjs::Error::Unknown)?;
                  m.update(data.as_bytes()); hex::encode(m.finalize().into_bytes())
              }
              "sha256" => {
                  let mut m = Hmac::<Sha256>::new_from_slice(&key).map_err(|_| rquickjs::Error::Unknown)?;
                  m.update(data.as_bytes()); hex::encode(m.finalize().into_bytes())
              }
              "sha512" => {
                  let mut m = Hmac::<Sha512>::new_from_slice(&key).map_err(|_| rquickjs::Error::Unknown)?;
                  m.update(data.as_bytes()); hex::encode(m.finalize().into_bytes())
              }
              _ => return Ok(error_json(format!("unknown hmac algo: {}", algo))),
          };
          Ok(out)
      },
    )?)?;

    globals.set("__native_crypto_aes", Function::new(ctx.clone(),
     |op: String, key_hex: String, data_hex: String,
      iv_hex: Option<String>, mode: String| -> rquickjs::Result<String> {
         use aes::cipher::{block_padding::Pkcs7, BlockEncryptMut, BlockDecryptMut, KeyIvInit};
         use cbc::{Encryptor, Decryptor};
         use aes::{Aes128, Aes256};

         let key = match hex::decode(&key_hex) {
             Ok(k) => k, Err(e) => return Ok(error_json(format!("aes: bad key hex: {}", e))),
         };
         let data = match hex::decode(&data_hex) {
             Ok(d) => d, Err(e) => return Ok(error_json(format!("aes: bad data hex: {}", e))),
         };
         let iv = match iv_hex.as_deref().map(hex::decode) {
             Some(Err(e)) => return Ok(error_json(format!("aes: bad iv hex: {}", e))),
             Some(Ok(v)) => v,
             None => vec![0u8; 16],
         };

         let result = match (key.len(), mode.as_str(), op.as_str()) {
             (16, "cbc", "encrypt") => {
                 let enc = match Encryptor::<Aes128>::new_from_slices(&key, &iv) {
                     Ok(e) => e, Err(e) => return Ok(error_json(e.to_string())),
                 };
                 hex::encode(enc.encrypt_padded_vec_mut::<Pkcs7>(&data))
             }
             (16, "cbc", "decrypt") => {
                 let dec = match Decryptor::<Aes128>::new_from_slices(&key, &iv) {
                     Ok(d) => d, Err(e) => return Ok(error_json(e.to_string())),
                 };
                 match dec.decrypt_padded_vec_mut::<Pkcs7>(&data) {
                     Ok(d) => hex::encode(d), Err(e) => return Ok(error_json(e.to_string())),
                 }
             }
             (32, "cbc", "encrypt") => {
                 let enc = match Encryptor::<Aes256>::new_from_slices(&key, &iv) {
                     Ok(e) => e, Err(e) => return Ok(error_json(e.to_string())),
                 };
                 hex::encode(enc.encrypt_padded_vec_mut::<Pkcs7>(&data))
             }
             (32, "cbc", "decrypt") => {
                 let dec = match Decryptor::<Aes256>::new_from_slices(&key, &iv) {
                     Ok(d) => d, Err(e) => return Ok(error_json(e.to_string())),
                 };
                 match dec.decrypt_padded_vec_mut::<Pkcs7>(&data) {
                     Ok(d) => hex::encode(d), Err(e) => return Ok(error_json(e.to_string())),
                 }
             }
             (kl, _, _) => return Ok(error_json(format!("unsupported key length {}b or mode {}", kl * 8, mode))),
         };
         Ok(result)
     },
    )?)?;

    Ok(())
}

#[inline]
fn error_json(msg: String) -> String {
    serde_json::json!({ "error": msg }).to_string()
}