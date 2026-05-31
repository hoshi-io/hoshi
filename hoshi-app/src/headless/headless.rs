use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Runtime};
#[cfg(not(mobile))]
use tauri::{Listener, Manager, WebviewUrl, WebviewWindowBuilder};
use tracing::{debug, error, warn, instrument};

#[cfg(mobile)]
use crate::headless::headless_plugin::{CreatePayload, HeadlessPluginExt};

#[cfg(not(mobile))]
use tokio::sync::oneshot;

use hoshi_core::error::{CoreError, CoreResult};
use hoshi_core::headless::{BlockedResource, CapturedRequest, Cookie, HeadlessBrowser, HeadlessOptions, HeadlessResponse, WaitFor};

const DEFAULT_TIMEOUT_MS: u64 = 15_000;

static HEADLESS_COUNTER: AtomicU32 = AtomicU32::new(1);

fn next_label() -> String {
    format!("hl-{}", HEADLESS_COUNTER.fetch_add(1, Ordering::Relaxed))
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct HeadlessDonePayload {
    label:    String,
    url:      String,
    html:     String,
    result:   Option<serde_json::Value>,
    captured: Vec<CapturedRequest>,
    cookies: String
}

pub struct TauriHeadless<R: Runtime> {
    app: AppHandle<R>,
    #[cfg(mobile)]
    mobile_lock: Arc<std::sync::Mutex<()>>,
}

impl<R: Runtime> TauriHeadless<R> {
    pub fn new(app: AppHandle<R>) -> Self {
        Self {
            app,
            #[cfg(mobile)]
            mobile_lock: Arc::new(std::sync::Mutex::new(())),
        }
    }
}

#[async_trait]
impl<R: Runtime + 'static> HeadlessBrowser for TauriHeadless<R> {
    fn is_available(&self) -> bool { true }

    #[instrument(skip(self, options))]
    async fn fetch(&self, url: &str, options: HeadlessOptions) -> CoreResult<HeadlessResponse> {
        debug!(url = %url, "Initiating headless fetch request");

        #[cfg(mobile)]
        {
            let url  = url.to_string();
            let app  = self.app.clone();
            let lock = self.mobile_lock.clone();
            tokio::task::spawn_blocking(move || {
                TauriHeadless { app, mobile_lock: lock }.fetch_mobile_sync(&url, options)
            })
                .await
                .map_err(|e| {
                    error!(error = ?e, "Headless spawn_blocking thread panicked");
                    CoreError::Internal("error.system.internal".into())
                })?
        }
        #[cfg(not(mobile))]
        {
            self.fetch_desktop(url, options).await
        }
    }
}

impl<R: Runtime + 'static> TauriHeadless<R> {

    #[cfg(not(mobile))]
    async fn fetch_desktop(&self, url: &str, options: HeadlessOptions) -> CoreResult<HeadlessResponse> {
        let label      = next_label();
        let url        = url.to_string();
        let timeout_ms = options.timeout_ms.max(DEFAULT_TIMEOUT_MS);
        let app        = self.app.clone();
        let label_c    = label.clone();
        let options_c  = options.clone();

        let (tx, rx) = oneshot::channel::<CoreResult<HeadlessDonePayload>>();
        let tx = Arc::new(std::sync::Mutex::new(Some(tx)));

        {
            let tx_l = tx.clone();
            let label_event = label.clone();
            app.once(format!("headless-done-{}", label), move |event| {
                debug!(label = %label_event, "Received headless-done event from WebView");
                let raw = event.payload();
                let result = serde_json::from_str::<serde_json::Value>(raw)
                    .map_err(|e| {
                        error!(error = ?e, "Failed to parse raw headless payload");
                        CoreError::Parse("error.headless.invalid_payload".into())
                    })
                    .and_then(|v| {
                        let inner = v["data"].as_str().unwrap_or(raw);
                        serde_json::from_str::<HeadlessDonePayload>(inner)
                            .map_err(|e| {
                                error!(error = ?e, "Failed to parse HeadlessDonePayload structure");
                                CoreError::Parse("error.headless.invalid_payload".into())
                            })
                    });

                if let Ok(mut g) = tx_l.lock() {
                    if let Some(sender) = g.take() {
                        let _ = sender.send(result);
                    }
                }
            });
        }

        let url_for_webview = url.clone();
        let url_clone = url.clone();

        let app_c = app.clone();
        app.run_on_main_thread(move || {
            if let Err(e) = create_desktop_webview(&app_c, &label_c, &url_for_webview, &options_c) {
                error!(label = %label_c, error = ?e, "Failed to create desktop webview");
            }
        }).map_err(|e| {
            error!(error = ?e, "Failed to dispatch webview creation to main thread");
            CoreError::Internal("error.system.internal".into())
        })?;

        match tokio::time::timeout(Duration::from_millis(timeout_ms), rx).await {
            Ok(Ok(Ok(p))) => {
                debug!(url = %p.url, "Headless fetch completed successfully on desktop");

                let url_clone = url_clone;

                let cookies = app
                    .get_webview_window(&label)
                    .and_then(|w| {
                        let u1 = (&p.url).parse().unwrap();
                        let u2 = url_clone.parse().unwrap();
                        let r = w.cookies_for_url(u1);
                        r.or_else(|_| w.cookies_for_url(u2)).ok()
                    })
                    .map(|tauri_cookies| {
                        tauri_cookies.iter().map(|c| Cookie {
                            name:  c.name().to_string(),
                            value: c.value().to_string(),
                        }).collect::<Vec<_>>()
                    })
                    .unwrap_or_else(|| parse_cookies(&p.cookies));

                let app_destroy = app.clone();
                let label_d = label.clone();
                let _ = app.run_on_main_thread(move || {
                    if let Some(w) = app_destroy.get_webview_window(&label_d) {
                        let _ = w.destroy();
                    }
                });

                Ok(HeadlessResponse {
                    url: p.url, status: 200, html: p.html,
                    result: p.result, captured: p.captured,
                    cookies,
                })
            }
            Ok(Ok(Err(e))) => Err(e),
            Ok(Err(_))     => {
                error!("Headless response channel dropped prematurely");
                Err(CoreError::Internal("error.system.internal".into()))
            },
            Err(_)         => {
                warn!(timeout_ms = timeout_ms, label = %label, "Headless fetch timed out on desktop");
                let label_t = label.clone();
                let app_c = app.clone();
                let _ = app_c.run_on_main_thread(move || {
                    if let Some(w) = app.get_webview_window(&label_t) {
                        debug!(label = %label_t, "Destroying timed-out webview");
                        let _ = w.destroy();
                    }
                });
                Err(CoreError::Network("error.headless.timeout".into()))
            }
        }
    }

    #[cfg(mobile)]
    pub fn fetch_mobile_sync(&self, url: &str, options: HeadlessOptions) -> CoreResult<HeadlessResponse> {
        use crate::headless::headless_sync::{register_slot, unregister_slot, HeadlessSlot};

        debug!("Acquiring mobile headless lock");
        let _guard     = self.mobile_lock.lock().unwrap();
        let timeout_ms = options.timeout_ms.max(DEFAULT_TIMEOUT_MS);
        let label      = next_label();

        let url_with_flag = if url.contains('?') {
            format!("{}&__headless=1", url)
        } else {
            format!("{}?__headless=1", url)
        };

        let slot = HeadlessSlot::new();
        register_slot(label.clone(), slot.clone());

        debug!(label = %label, url = %url_with_flag, "Dispatching mobile headless plugin create command");
        let plugin = self.app.headless_plugin();
        plugin.create(CreatePayload {
            label:       label.clone(),
            url:         url_with_flag,
            init_script: build_init_script(&label, &options),
        }).map_err(|e| {
            error!(error = ?e, "Failed to create mobile headless plugin view");
            unregister_slot(&label);
            CoreError::Internal("error.headless.webview_build_failed".into())
        })?;

        let payload_str = slot
            .wait_timeout(Duration::from_millis(timeout_ms))
            .ok_or_else(|| {
                warn!(timeout_ms = timeout_ms, label = %label, "Headless fetch timed out on mobile");
                unregister_slot(&label);
                let _ = plugin.destroy(&label);
                CoreError::Network("error.headless.timeout".into())
            })?;

        debug!(label = %label, "Headless sync completed, cleaning up resources");
        unregister_slot(&label);
        let _ = plugin.destroy(&label);

        let payload: HeadlessDonePayload = serde_json::from_str(&payload_str)
            .map_err(|e| {
                error!(error = ?e, "Failed to parse mobile headless payload");
                CoreError::Parse("error.headless.invalid_payload".into())
            })?;

        Ok(HeadlessResponse {
            url:      payload.url,
            status:   200,
            html:     payload.html,
            result:   payload.result,
            captured: payload.captured,
            cookies: parse_cookies(&payload.cookies),
        })
    }
}

fn parse_cookies(cookie_str: &str) -> Vec<Cookie> {
    cookie_str
        .split(';')
        .filter_map(|pair| {
            let mut parts = pair.trim().splitn(2, '=');
            let name  = parts.next()?.trim().to_string();
            if name.is_empty() { return None; }
            let value = parts.next().unwrap_or("").trim().to_string();
            Some(Cookie {
                name,
                value,
            })
        })
        .collect()
}

#[cfg(not(mobile))]
fn create_desktop_webview<R: Runtime>(
    app: &AppHandle<R>,
    label: &str,
    url: &str,
    options: &HeadlessOptions,
) -> CoreResult<()> {
    let parsed_url = url.parse::<url::Url>().map_err(|e| {
        error!(error = ?e, url = %url, "Invalid URL provided for headless webview");
        CoreError::BadRequest("error.headless.invalid_url".into())
    })?;

    let init_script = build_init_script(label, options);
    let blocked_patterns = build_blocked_url_patterns(&options.block);

    debug!(label = %label, "Configuring desktop WebviewWindowBuilder");
    let mut builder = WebviewWindowBuilder::new(app, label, WebviewUrl::External(parsed_url))
        .visible(false)
        .decorations(false)
        .inner_size(1280.0, 800.0)
        .initialization_script(&init_script);

    if !blocked_patterns.is_empty() {
        builder = builder.on_navigation(move |nav_url: &url::Url| {
            let s = nav_url.as_str();
            !blocked_patterns.iter().any(|p| s.contains(p.as_str()))
        });
    }

    builder.build().map_err(|e| {
        error!(error = ?e, "Failed to build desktop webview");
        CoreError::Internal("error.headless.webview_build_failed".into())
    })?;

    Ok(())
}

fn build_init_script(label: &str, options: &HeadlessOptions) -> String {
    let label_json         = serde_json::to_string(label).unwrap_or_default();
    let capture_json       = serde_json::to_string(&options.capture).unwrap_or_default();
    let js_to_eval_json    = serde_json::to_string(&options.javascript.clone().unwrap_or_default()).unwrap_or_default();
    let wait_is_idle       = matches!(options.wait_for, WaitFor::NetworkIdle);
    let wait_selector      = match &options.wait_for { WaitFor::Selector(s) => s.clone(), _ => String::new() };
    let wait_selector_json = serde_json::to_string(&wait_selector).unwrap_or_default();
    let block_css          = build_css_block_rules(&options.block);

    format!(r#"
(function() {{
    "use strict";

    {block_css}

    const __capturePatterns = {capture_json};
    const __captured = [];

    if (__capturePatterns.length > 0) {{
        const __origFetch = window.fetch ? window.fetch.bind(window) : null;
        if (__origFetch) {{
            window.fetch = async function(input, init) {{
                const url    = typeof input === 'string' ? input : (input && input.url) || '';
                const method = (init && init.method) || 'GET';
                const resp   = await __origFetch(input, init);
                if (__capturePatterns.some(p => url.includes(p))) {{
                    try {{
                        const clone = resp.clone();
                        const body  = await clone.text().catch(() => null);
                        const headers = {{}};
                        clone.headers.forEach((v, k) => {{ headers[k] = v; }});
                        __captured.push({{ url, method, status: resp.status, body, headers }});
                    }} catch(_) {{}}
                }}
                return resp;
            }};
        }}
    }}

    async function __waitForBridge(retries, delayMs) {{
        for (let i = 0; i < retries; i++) {{
            if (window.HeadlessBridge && typeof window.HeadlessBridge.postMessage === 'function') return true;
            await new Promise(r => setTimeout(r, delayMs));
        }}
        return false;
    }}

    async function __hoshi_collect() {{
        if (window.__hoshi_sent) return;
        window.__hoshi_sent = true;

        let userResult = null;
        const jsCode = {js_to_eval_json};
        try {{
            if (jsCode) {{
                userResult = eval(jsCode);
                if (userResult && typeof userResult.then === 'function') userResult = await userResult;
            }}
        }} catch(e) {{
            userResult = {{ error: e.message }};
        }}

        const payload = {{
            label:    {label_json},
            url:      window.location.href,
            html:     document.documentElement.outerHTML,
            result:   userResult,
            captured: __captured,
            cookies:  document.cookie
        }};
        const jsonPayload = JSON.stringify(payload);

        if (await __waitForBridge(10, 100)) {{
            try {{ window.HeadlessBridge.postMessage(jsonPayload); return; }} catch(_) {{}}
        }}

        try {{
            await window.__TAURI_INTERNALS__.invoke('plugin:event|emit', {{
                event:   'headless-done-' + {label_json},
                payload: payload,
            }});
        }} catch(e) {{
            console.error("[hoshi] no channel worked:", e);
        }}
    }}

    function __hoshi_start() {{
        const selector = {wait_selector_json};
        if (selector) {{
            const check = () => document.querySelector(selector)
                ? __hoshi_collect()
                : setTimeout(check, 500);
            check();
            setTimeout(__hoshi_collect, 10000);
        }} else if ({wait_is_idle}) {{
            setTimeout(__hoshi_collect, 5000);
        }} else {{
            __hoshi_collect();
        }}
    }}

    setTimeout(__hoshi_start, 100);
}})();
"#,
            block_css          = block_css,
            capture_json       = capture_json,
            label_json         = label_json,
            js_to_eval_json    = js_to_eval_json,
            wait_is_idle       = wait_is_idle,
            wait_selector_json = wait_selector_json,
    )
}

fn build_css_block_rules(blocked: &[BlockedResource]) -> String {
    let rules: Vec<&str> = blocked.iter().filter_map(|r| match r {
        BlockedResource::Images     => Some("img,picture,image{display:none!important}"),
        BlockedResource::Fonts      => Some("@font-face{src:local('')!important}"),
        BlockedResource::Media      => Some("video,audio{display:none!important}"),
        _                           => None,
    }).collect();

    if rules.is_empty() { return String::new(); }
    format!(
        r#"(function(){{const s=document.createElement('style');s.textContent=`{}`;(document.head||document.documentElement).appendChild(s);}})();"#,
        rules.join("")
    )
}

fn build_blocked_url_patterns(blocked: &[BlockedResource]) -> Vec<String> {
    let mut patterns = vec![
        "googlesyndication.com", "doubleclick.net", "adservice.google",
        "googletagmanager.com", "google-analytics.com", "facebook.com/tr",
        "amazon-adsystem.com", "scorecardresearch.com",
    ].into_iter().map(String::from).collect::<Vec<_>>();

    for r in blocked {
        match r {
            BlockedResource::Pattern(p)  => patterns.push(p.clone()),
            BlockedResource::Fonts       => patterns.extend([".woff2", ".woff", ".ttf", ".otf"].map(String::from)),
            BlockedResource::Media       => patterns.extend([".mp4", ".webm", ".mp3", ".ogg"].map(String::from)),
            BlockedResource::Stylesheet  => patterns.push(".css".into()),
            BlockedResource::Images      => {}
        }
    }
    patterns
}