use hoshi_core::proxy::{ProxyQuery, ProxyService};
use hoshi_core::state::AppState;
use std::sync::Arc;
use tauri::{Manager, Runtime, UriSchemeContext, UriSchemeResponder};
use url::form_urlencoded;

fn parse_query(uri: &str) -> ProxyQuery {
    let query_str = uri.splitn(2, '?').nth(1).unwrap_or("");
    let mut url        = String::new();
    let mut referer    = None;
    let mut origin     = None;
    let mut user_agent = None;

    for (k, v) in form_urlencoded::parse(query_str.as_bytes()) {
        match k.as_ref() {
            "url"       => url        = v.into_owned(),
            "referer"   => referer    = Some(v.into_owned()),
            "origin"    => origin     = Some(v.into_owned()),
            "userAgent" => user_agent = Some(v.into_owned()),
            _ => {}
        }
    }

    ProxyQuery { url, referer, origin, user_agent }
}

fn cors_headers() -> [(&'static str, &'static str); 3] {
    [
        ("Access-Control-Allow-Origin",  "*"),
        ("Access-Control-Allow-Headers", "Content-Type"),
        ("Access-Control-Allow-Methods", "GET, OPTIONS"),
    ]
}

fn respond(responder: UriSchemeResponder, status: u16, content_type: &str, body: Vec<u8>) {
    let mut builder = tauri::http::Response::builder()
        .status(status)
        .header("content-type", content_type);
    for (k, v) in cors_headers() {
        builder = builder.header(k, v);
    }
    if content_type.starts_with("image/") {
        builder = builder.header("cache-control", "public, max-age=31536000, immutable");
    }
    match builder.body(body) {
        Ok(res) => responder.respond(res),
        Err(e) => tracing::error!(error = ?e, "failed to build proxy response"),
    }
}

pub fn handle_async<R: Runtime>(
    ctx: UriSchemeContext<'_, R>,
    request: tauri::http::Request<Vec<u8>>,
    responder: UriSchemeResponder,
) {
    let app = ctx.app_handle();
    let state = match app.try_state::<Arc<AppState>>() {
        Some(s) => Arc::clone(&s),
        None => {
            respond(responder, 500, "text/plain", b"AppState not found".to_vec());
            return;
        }
    };

    if request.method() == tauri::http::Method::OPTIONS {
        respond(responder, 204, "text/plain", vec![]);
        return;
    }

    let params = parse_query(&request.uri().to_string());
    if params.url.is_empty() {
        respond(responder, 400, "text/plain", b"Missing url parameter".to_vec());
        return;
    }

    tauri::async_runtime::spawn(async move {
        match ProxyService::handle_request(&state, params).await {
            Ok(r) => respond(responder, r.status, &r.content_type, r.body),
            Err(e) => respond(responder, 502, "text/plain", format!("{e:?}").into_bytes()),
        }
    });
}
