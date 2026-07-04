use rquickjs::context::EvalOptions;
use rquickjs::{
    async_with, AsyncContext, AsyncRuntime, CatchResultExt, Function,
};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use regex::Regex;
use scraper::{Html, Selector};
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
    http_client: reqwest::Client,
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
                run_quickjs_local(full_script, extension_code, headless_available, req_tx, state_json, extension_id, http_client),
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

#[instrument(skip(full_script, extension_code, req_tx, state_json, http_client))]
async fn run_quickjs_local(
    full_script: String,
    extension_code: String,
    headless_available: bool,
    req_tx: std::sync::mpsc::SyncSender<HeadlessRequest>,
    state_json: String,
    extension_id: String,
    http_client: reqwest::Client,
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
        register_native_apis(&ctx, headless_available, req_tx, Arc::clone(&state_map), extension_id, http_client)
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

            CoreError::Internal("error.sandbox.thread_panicked".into())
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

        Some(CompatLayer::Sora(js)) => {
            let runner = format!(
                r#"(async () => {{
            const src = {ext_repr};
            (0, eval)(src);

            if (typeof globalThis.fetchv2 !== "function") {{
                globalThis.fetchv2 = async (url, headers = {{}}, method = "GET", body = null) => {{
                    return await fetch(url, {{ method, headers, body }});
                }};
            }}

            const ExtClass = __sora_buildAnimeClass();
            const instance = new ExtClass();

            const fn_name = "{fn}";
            if (typeof instance[fn_name] !== "function")
                throw new Error(`Method "${{fn_name}}" not found on Sora compat class`);

            return await instance[fn_name](...{args});
        }})()"#,
                ext_repr = ext_code_repr,
                fn       = function_name,
                args     = args_json,
            );
            (js.clone(), runner)
        }

        Some(CompatLayer::Tachiyomi(js)) | Some(CompatLayer::Aniyomi(js)) => {
            let runner = format!(
                r#"(async () => {{
                    globalThis.__tachi_captured = null;

                    const src = {ext_repr};

                const patched = src
                    .replace(
                        /class\s+([a-zA-Z0-9_$]+)\s+extends\s+(HttpSource|ParsedHttpSource|Manga)/,
                        'globalThis.__tachi_captured = globalThis["$1"] = class $1 extends $2'
                    )
                    .replace(
                        /\bclass\s+([a-zA-Z0-9_$]+)\s+extends\s+(?!HttpSource|ParsedHttpSource|Manga\b)/g,
                        'globalThis["$1"] = class $1 extends '
                    )
                    .replace(
                        /\bclass\s+([a-zA-Z0-9_$]+)\s*\{{/g,
                        'globalThis["$1"] = class $1 {{'
                    );

                    eval(patched);

                    const lang = __settings.language ?? "en";

                    let instance;

                    const FactoryClass = globalThis.__tachi_getFactoryClass();

                    if (FactoryClass) {{
                        const factory = new FactoryClass();

                        const sources = factory.createSources();

                        instance =
                            sources.find?.(s => s.getLang?.() === lang)
                            ?? sources[0];
                    }} else {{
                        const ExtClass = globalThis.__tachi_getCapturedClass();

                        if (!ExtClass)
                            throw new Error("[tachi-compat] No class extending HttpSource found");

                        instance = new ExtClass(lang, lang);
                    }}

                    if (!instance)
                        throw new Error("[tachi-compat] No source found for lang: " + lang);

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

fn execute_selector(document: &Html, selector_str: &str) -> Vec<Value> {
    let parts: Vec<&str> = selector_str.split(',').map(|s| s.trim()).collect();
    let mut results = vec![];

    for part in parts {
        if let Some(r) = handle_has_contains_selector(document, part) {
            results.extend(r);
        } else if part.contains(":contains(") {
            if let Some(r) = handle_contains_selector(document, part) {
                results.extend(r);
            }
        } else {
            let sanitized = sanitize_selector(part);
            let sel = Selector::parse(&sanitized);
            match sel {
                Ok(sel) => {
                    let els: Vec<_> = document.select(&sel).map(|el| element_to_json(el)).collect();
                    results.extend(els);
                }
                Err(e) => warn!("[SELECTOR] parse error: {:?}", e),
            }
        }
    }

    results
}

fn handle_has_contains_selector(document: &Html, selector: &str) -> Option<Vec<Value>> {
    let re = Regex::new(
        r"^(.*?):has\(\s*>\s*([a-zA-Z0-9_-]+(?:\.[a-zA-Z0-9_-]+)*):contains\(([^)]+)\)\s*\)(.*)?$"
    ).unwrap();
    let caps = re.captures(selector)?;

    let outer_sel_str = caps.get(1).map_or("*", |m| m.as_str()).trim();
    let inner_tag     = caps.get(2).map_or("*", |m| m.as_str()).trim();
    let contains_text = caps.get(3).map_or("", |m| m.as_str()).trim();
    let suffix        = caps.get(4).map_or("", |m| m.as_str()).trim();

    let outer_sel_str = if outer_sel_str.is_empty() { "*" } else { outer_sel_str };
    let outer_sel = Selector::parse(&sanitize_selector(outer_sel_str)).ok()?;
    let inner_sel = Selector::parse(&sanitize_selector(inner_tag)).ok()?;

    let mut results = vec![];

    for outer_el in document.select(&outer_sel) {
        // Check if it has a direct child matching inner_tag that contains the text
        let has_match = outer_el.children()
            .filter_map(|c| scraper::ElementRef::wrap(c))
            .any(|child| {
                inner_sel.matches(&child) && child.text().collect::<String>().contains(contains_text)
            });

        if !has_match {
            continue;
        }

        if suffix.is_empty() {
            results.push(element_to_json(outer_el));
        } else {
            // Apply suffix combinator, e.g. "> div" or " div"
            let suffix = suffix.trim();
            let (combinator, child_sel_str) = if suffix.starts_with('>') {
                (">", suffix.trim_start_matches('>').trim())
            } else {
                ("", suffix)
            };

            let child_sel = Selector::parse(&sanitize_selector(child_sel_str)).ok()?;

            if combinator == ">" {
                // Direct children only
                for child in outer_el.children().filter_map(|c| scraper::ElementRef::wrap(c)) {
                    if child_sel.matches(&child) {
                        results.push(element_to_json(child));
                    }
                }
            } else {
                // Descendants
                for desc in outer_el.select(&child_sel) {
                    results.push(element_to_json(desc));
                }
            }
        }
    }

    Some(results)
}

fn handle_contains_selector(document: &Html, selector: &str) -> Option<Vec<Value>> {
    // Match pattern like: "div.summary-heading:contains(Status) + div"
    let re = regex::Regex::new(r"^(.*?):contains\(([^)]+)\)\s*(\+\s*\S+)?$").unwrap();
    let caps = re.captures(selector)?;

    let base_sel = caps.get(1).map_or("*", |m| m.as_str()).trim();
    let contains_text = caps.get(2).map_or("", |m| m.as_str()).trim();
    let sibling = caps.get(3).map_or("", |m| m.as_str());

    let base_sel = if base_sel.is_empty() { "*" } else { base_sel };
    let sanitized = sanitize_selector(base_sel);
    let sel = Selector::parse(&sanitized).ok()?;

    let mut results = vec![];

    for el in document.select(&sel) {
        let text: String = el.text().collect();
        if text.contains(contains_text) {
            if sibling.is_empty() {
                results.push(element_to_json(el));
            } else {
                // Get adjacent sibling (+ selector)
                let sibling_tag = sibling.trim_start_matches('+').trim();
                let sanitized_sib = sanitize_selector(sibling_tag);
                let sib_sel = Selector::parse(&sanitized_sib);
                if let Ok(sib_sel) = sib_sel {
                    // Find next sibling element matching the selector
                    let mut next = el.next_sibling();
                    while let Some(n) = next {
                        if let Some(el) = Html::parse_fragment(
                            &scraper::ElementRef::wrap(n)
                                .map(|e| e.html())
                                .unwrap_or_default()
                        ).select(&sib_sel).next() {
                            results.push(element_to_json(el));
                            break;
                        }
                        next = n.next_sibling();
                    }
                }
            }
        }
    }

    Some(results)
}

fn element_to_json(el: scraper::ElementRef) -> Value {
    let attrs: HashMap<String, String> = el.value().attrs()
        .map(|(k, v)| (k.to_string(), v.to_string()))
        .collect();

    let own_text: String = el.children()
        .filter_map(|child| child.value().as_text().map(|t| t.to_string()))
        .collect::<Vec<_>>()
        .join("")
        .trim()
        .to_string();

    serde_json::json!({
        "text": el.text().collect::<Vec<_>>().join(""),
        "own_text": own_text,
        "html": el.inner_html(),
        "outer": el.html(),
        "attrs": attrs,
    })
}

fn sanitize_selector(selector: &str) -> String {
    let re = Regex::new(r":not\(\s*:has\([^)]*\)\s*\)").unwrap();
    let s = re.replace_all(selector, "").to_string();

    let re2 = Regex::new(r":has\([^)]*\)").unwrap();
    let s = re2.replace_all(&s, "").to_string();

    let re3 = Regex::new(r":not\(\s*\)").unwrap();
    let s = re3.replace_all(&s, "").to_string();

    let re4 = Regex::new(r"\s*[+~>]\s*$").unwrap();
    let re5 = Regex::new(r"\[([^\]]*):([^\]]*)\]").unwrap();
    let s = re5.replace_all(&s, |caps: &regex::Captures| {
        format!("[{}\\:{}]", &caps[1], &caps[2])
    }).to_string();

    re4.replace_all(&s, "").to_string()
}

fn register_native_apis(
    ctx: &rquickjs::Ctx<'_>,
    headless_available: bool,
    req_tx: Arc<std::sync::mpsc::SyncSender<HeadlessRequest>>,
    state_map: Arc<Mutex<HashMap<String, Value>>>,
    extension_id: String,
    http_client: reqwest::Client,
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

    globals.set("__native_fetch", Function::new(ctx.clone(), {
        let client = http_client.clone();
        move |url: String, method: String, headers_json: String, body: String| {
            let headers: HashMap<String, String> =
                serde_json::from_str(&headers_json).unwrap_or_default();
            let body_opt = if body.is_empty() { None } else { Some(body) };

            let client = client.clone();
            let result = std::thread::spawn(move || {
                tokio::runtime::Builder::new_current_thread()
                    .enable_all()
                    .build()
                    .map(|rt| rt.block_on(async move {
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

                        match req.send().await {
                            Err(e) => error_json(e.to_string()),
                            Ok(resp) => {
                                let status = resp.status().as_u16();
                                let ok     = resp.status().is_success();

                                // extract Set-Cookie headers
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
                                    Err(e)   => error_json(e.to_string()),
                                    Ok(text) => serde_json::json!({
                                        "ok": ok,
                                        "status": status,
                                        "body": text,
                                        "cookies": cookies,
                                    }).to_string(),
                                }
                            }
                        }
                    }))
                    .unwrap_or_else(|e| error_json(e.to_string()))
            })
                .join()
                .unwrap_or_else(|_| error_json("fetch thread panicked".into()));

            Ok::<String, rquickjs::Error>(result)
        }
    })?)?;

    globals.set("__native_sleep", Function::new(ctx.clone(), |ms: u64| {
        std::thread::sleep(std::time::Duration::from_millis(ms));
        Ok::<(), rquickjs::Error>(())
    })?)?;

    globals.set("__native_html_query", Function::new(ctx.clone(),
     |html: String, selector: String| -> rquickjs::Result<String> {
         use scraper::Html;
         let document = Html::parse_document(&html);
         let results = execute_selector(&document, &selector);
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

    {
        let state_map = Arc::clone(&state_map);
        globals.set("__native_state_has", Function::new(ctx.clone(), {
            let state_map = Arc::clone(&state_map);
            move |key: String| -> rquickjs::Result<bool> {
                let guard = state_map.lock().unwrap_or_else(|p| p.into_inner());
                Ok(guard.contains_key(&key))
            }
        })?)?;
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
                  let mut m = Hmac::<Sha1>::new_from_slice(&key).map_err(|_e| rquickjs::Error::Unknown)?;
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