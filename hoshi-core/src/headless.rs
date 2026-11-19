use std::sync::Arc;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use crate::error::CoreResult;


#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum BlockedResource {
    Images,
    Fonts,
    Media,
    Stylesheet,
    Pattern(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WaitFor {
    DomReady,
    NetworkIdle,
    Selector(String),
}

impl Default for WaitFor {
    fn default() -> Self { WaitFor::DomReady }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct HeadlessOptions {
    #[serde(default)]
    pub wait_for: WaitFor,
    #[serde(default)]
    pub javascript: Option<String>,
    #[serde(default)]
    pub block: Vec<BlockedResource>,
    #[serde(default)]
    pub capture: Vec<String>,
    #[serde(default = "default_timeout_ms")]
    pub timeout_ms: u64,
}

fn default_timeout_ms() -> u64 { 15_000 }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapturedRequest {
    pub url:     String,
    pub method:  String,
    pub status:  Option<u16>,
    pub body:    Option<String>,
    pub headers: std::collections::HashMap<String, String>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeadlessResponse {
    pub url:      String,
    pub status:   u16,
    pub html:     String,
    pub result:   Option<serde_json::Value>,
    pub captured: Vec<CapturedRequest>,
    pub cookies:  Vec<Cookie>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Cookie {
    pub name:      String,
    pub value:     String,
}

#[async_trait]
pub trait HeadlessBrowser: Send + Sync {
    async fn fetch(&self, url: &str, options: HeadlessOptions) -> CoreResult<HeadlessResponse>;
    fn is_available(&self) -> bool;
}

pub struct NoopHeadless;

#[async_trait]
impl HeadlessBrowser for NoopHeadless {
    async fn fetch(&self, _url: &str, _options: HeadlessOptions) -> CoreResult<HeadlessResponse> {
        tracing::error!("Attempted to use Headless browser on a platform where it's not supported");
        Err(crate::error::CoreError::Internal(
            "error.headless.not_available".into()
        ))
    }

    fn is_available(&self) -> bool { false }
}

pub type HeadlessHandle = Arc<dyn HeadlessBrowser>;

pub fn noop_headless() -> HeadlessHandle {
    Arc::new(NoopHeadless)
}

/// The page script every headless backend injects: blocks resources via CSS, captures matching
/// fetches, waits per `wait_for`, runs `javascript`, then posts the payload (label, url, html,
/// result, captured, cookies) to `window.HeadlessBridge.postMessage` (or the Tauri event).
pub fn init_script(label: &str, options: &HeadlessOptions) -> String {
    let label_json         = serde_json::to_string(label).unwrap_or_default();
    let capture_json       = serde_json::to_string(&options.capture).unwrap_or_default();
    let js_to_eval_json    = serde_json::to_string(&options.javascript.clone().unwrap_or_default()).unwrap_or_default();
    let wait_is_idle       = matches!(options.wait_for, WaitFor::NetworkIdle);
    let wait_selector      = match &options.wait_for { WaitFor::Selector(s) => s.clone(), _ => String::new() };
    let wait_selector_json = serde_json::to_string(&wait_selector).unwrap_or_default();
    let block_css          = css_block_rules(&options.block);

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

fn css_block_rules(blocked: &[BlockedResource]) -> String {
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

/// URL substrings to block (ad/analytics hosts + the requested resource kinds).
pub fn blocked_url_patterns(blocked: &[BlockedResource]) -> Vec<String> {
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