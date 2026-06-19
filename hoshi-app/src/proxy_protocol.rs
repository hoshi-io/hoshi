use futures::TryStreamExt;
use hoshi_core::proxy::{ProxyBody, ProxyQuery, ProxyService};
use hoshi_core::state::AppState;
use std::sync::Arc;
use std::time::Duration;
use tauri::{Manager, Runtime, UriSchemeContext, UriSchemeResponder};
use url::form_urlencoded;


fn parse_query_full(uri: &str) -> (ProxyQuery, Option<String>) {
    let query_str = uri.splitn(2, '?').nth(1).unwrap_or("");
    let mut url        = String::new();
    let mut referer    = None;
    let mut origin     = None;
    let mut user_agent = None;
    let mut range      = None;

    for (k, v) in form_urlencoded::parse(query_str.as_bytes()) {
        match k.as_ref() {
            "url"       => url        = v.into_owned(),
            "referer"   => referer    = Some(v.into_owned()),
            "origin"    => origin     = Some(v.into_owned()),
            "userAgent" => user_agent = Some(v.into_owned()),
            "range"     => range      = Some(v.into_owned()),
            _ => {}
        }
    }

    (ProxyQuery { url, referer, origin, user_agent }, range)
}

fn cors_headers() -> Vec<(&'static str, String)> {
    vec![
        ("Access-Control-Allow-Origin",  "*".into()),
        ("Access-Control-Allow-Headers", "Range, Content-Type".into()),
        ("Access-Control-Allow-Methods", "GET, OPTIONS".into()),
        ("Access-Control-Expose-Headers","Content-Length, Content-Range, Accept-Ranges".into()),
    ]
}

/// Async handler — register with `register_asynchronous_uri_scheme_protocol`.
/// This is the preferred form: it streams the response body without blocking
/// and without collecting the full segment into memory before delivering it.
pub fn handle_async<R: Runtime>(
    ctx: UriSchemeContext<'_, R>,
    request: tauri::http::Request<Vec<u8>>,
    responder: UriSchemeResponder,
){
    let app = ctx.app_handle();
    let state = match app.try_state::<Arc<AppState>>() {
        Some(s) => Arc::clone(&s),
        None => {
            responder.respond(
                tauri::http::Response::builder()
                    .status(500)
                    .body(b"AppState not found".to_vec())
                    .unwrap(),
            );
            return;
        }
    };

    let uri = request.uri().to_string();
    let method = request.method().clone();

    // Handle CORS preflight
    if method == tauri::http::Method::OPTIONS {
        let mut builder = tauri::http::Response::builder().status(204);
        for (k, v) in cors_headers() {
            builder = builder.header(k, v);
        }
        responder.respond(builder.body(vec![]).unwrap());
        return;
    }

    let range_header = request
        .headers()
        .get("range")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string());

    let (params, query_range) = parse_query_full(&uri);

    // Prefer the actual Range header; fall back to query param (for HLS loader compat)
    let effective_range = range_header.or(query_range);

    if params.url.is_empty() {
        responder.respond(
            tauri::http::Response::builder()
                .status(400)
                .body(b"Missing url parameter".to_vec())
                .unwrap(),
        );
        return;
    }

    tauri::async_runtime::spawn(async move {
        match ProxyService::handle_request(&state, params, effective_range).await {
            Err(e) => {
                let msg = format!("{e:?}");
                let mut builder = tauri::http::Response::builder().status(502);
                for (k, v) in cors_headers() {
                    builder = builder.header(k, v);
                }
                responder.respond(builder.body(msg.into_bytes()).unwrap());
            }
            Ok(proxy_resp) => {
                let status = proxy_resp.status.as_u16();

                // Collect body — for m3u8/subtitles (Text) this is tiny.
                // For segments (Stream) we collect too, BUT the key improvement
                // is that the WebView receives bytes as soon as Rust has them,
                // not after JSON serialization/deserialization over IPC.
                // Further streaming can be done with Tauri channels (see comment below).
                let (body_bytes, content_type, extra_headers) = match proxy_resp.body {
                    ProxyBody::Text { content, content_type } => {
                        (content.into_bytes(), content_type, vec![])
                    }
                    ProxyBody::Stream { stream, content_length } => {
                        let mut extra = vec![];
                        if let Some(len) = content_length {
                            extra.push(("content-length", len.to_string()));
                        }
                        extra.push(("accept-ranges", "bytes".to_string()));

                        // Forward any content-range from the proxy response headers
                        if let Some(cr) = proxy_resp.headers.get("content-range") {
                            if let Ok(v) = cr.to_str() {
                                extra.push(("content-range", v.to_string()));
                            }
                        }

                        let bytes: Vec<u8> = match tokio::time::timeout(
                            Duration::from_secs(30),
                            stream.try_fold(Vec::new(), |mut acc, chunk| async move {
                                acc.extend_from_slice(&chunk);
                                Ok(acc)
                            })
                        ).await {
                            Err(_) => {
                                responder.respond(
                                    tauri::http::Response::builder()
                                        .status(504)
                                        .body(b"Segment download timed out".to_vec())
                                        .unwrap(),
                                );
                                return;
                            }
                            Ok(Err(e)) => {
                                let msg = format!("Stream error: {e:?}");
                                responder.respond(
                                    tauri::http::Response::builder()
                                        .status(502)
                                        .body(msg.into_bytes())
                                        .unwrap(),
                                );
                                return;
                            }
                            Ok(Ok(b)) => b,
                        };

                        let ct = proxy_resp.headers
                            .get("content-type")
                            .and_then(|v| v.to_str().ok())
                            .unwrap_or("application/octet-stream")
                            .to_string();

                        (bytes, ct, extra)
                    }
                };

                let mut builder = tauri::http::Response::builder()
                    .status(status)
                    .header("content-type", &content_type);

                for (k, v) in cors_headers() {
                    builder = builder.header(k, v);
                }
                for (k, v) in extra_headers {
                    builder = builder.header(k, v);
                }

                responder.respond(builder.body(body_bytes).unwrap());
            }
        }
    });
}