use reqwest::{StatusCode};
use reqwest::header::{HeaderMap, HeaderValue, CONTENT_TYPE, CONTENT_LENGTH, CONTENT_RANGE, ACCEPT_RANGES, CACHE_CONTROL};
use serde::Deserialize;
use std::time::Duration;
use url::Url;
use futures::{Stream, TryStreamExt};
use std::pin::Pin;
use bytes::Bytes;
use regex::Regex;
use tracing::{debug, warn, error, instrument};

use crate::error::{CoreError, CoreResult};
use crate::state::AppState;

#[derive(Deserialize, Clone)]
pub struct ProxyQuery {
    pub url: String,
    pub referer: Option<String>,
    pub origin: Option<String>,
    #[serde(rename = "userAgent")]
    pub user_agent: Option<String>,
}

pub enum ProxyBody {
    Text { content: String, content_type: String },
    Stream {
        stream: Pin<Box<dyn Stream<Item = Result<Bytes, std::io::Error>> + Send>>,
        content_length: Option<u64>
    },
}

pub struct ProxyResponse {
    pub status: StatusCode,
    pub headers: HeaderMap,
    pub body: ProxyBody,
}


pub struct ProxyService;

impl ProxyService {
    #[instrument(skip(state, params, range_header), fields(url = %params.url, range = ?range_header))]
    pub async fn handle_request(state: &AppState, params: ProxyQuery, range_header: Option<String>) -> CoreResult<ProxyResponse> {
        let normalized_url = normalize_url(&params.url);
        if params.url.is_empty() {
            warn!("Proxy request rejected: No URL provided");
            return Err(CoreError::BadRequest("error.proxy.no_url_provided".into()));
        }

        let req_headers = Self::build_upstream_headers(&params, range_header.as_deref())?;
        let is_range_request = range_header.is_some();
        let t0 = std::time::Instant::now();

        let mut last_error = None;
        let mut upstream_res = None;
        let max_retries = if is_range_request { 1 } else { 2 };

        for attempt in 0..max_retries {
            match tokio::time::timeout(
                Duration::from_secs(15),
                state.http_client.get(&normalized_url).headers(req_headers.clone()).send()
            ).await {
                Err(_) => {
                    warn!("Upstream request timed out (attempt {})", attempt + 1);
                    if attempt + 1 < max_retries {
                        tokio::time::sleep(Duration::from_millis(500)).await;
                    }
                    continue;
                }
                Ok(Err(e)) => {
                    warn!(error = ?e, "Upstream request failed, retrying...");
                    last_error = Some(e);
                    if attempt + 1 < max_retries {
                        tokio::time::sleep(Duration::from_millis(500)).await;
                    }
                }
                Ok(Ok(res)) => {
                    if !res.status().is_success() && res.status() != StatusCode::PARTIAL_CONTENT {
                        if res.status() == StatusCode::FORBIDDEN || res.status() == StatusCode::NOT_FOUND {
                            warn!(status = %res.status(), "Upstream returned definitive error");
                            return Err(CoreError::Network("error.proxy.upstream_error".into()));
                        }
                        warn!(status = %res.status(), "Upstream returned non-success, retrying...");
                        if attempt + 1 < max_retries {
                            tokio::time::sleep(Duration::from_millis(500)).await;
                        }
                        continue;
                    }
                    upstream_res = Some(res);
                    break;
                }
            }
        }

        let response = upstream_res.ok_or_else(|| {
            error!(error = ?last_error, "Proxy upstream failed after all retries");
            CoreError::Network("error.proxy.upstream_timeout".into())
        })?;

        Self::process_response(response, &params).await
    }

    fn build_upstream_headers(params: &ProxyQuery, range_header: Option<&str>) -> CoreResult<HeaderMap> {
        let mut headers = HeaderMap::new();

        let ua = params.user_agent.as_deref().unwrap_or("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36");
        headers.insert("User-Agent", HeaderValue::from_str(ua).map_err(|e| {
            warn!(error = ?e, "Invalid User-Agent string provided");
            CoreError::BadRequest("error.proxy.invalid_header".into())
        })?);

        headers.insert("Accept", HeaderValue::from_static("*/*"));
        headers.insert("Accept-Language", HeaderValue::from_static("en-US,en;q=0.9"));
        headers.insert("Accept-Encoding", HeaderValue::from_static("identity"));
        headers.insert("Connection", HeaderValue::from_static("keep-alive"));

        if let Some(ref r) = params.referer {
            if let Ok(v) = HeaderValue::from_str(r) { headers.insert("Referer", v); }
        }
        if let Some(ref o) = params.origin {
            if let Ok(v) = HeaderValue::from_str(o) { headers.insert("Origin", v); }
        }

        if let Some(range) = range_header {
            if let Ok(v) = HeaderValue::from_str(&range) {
                headers.insert("Range", v);
            }
        }

        if params.referer.is_none() && params.origin.is_none() {
            if let Ok(parsed) = Url::parse(&params.url) {
                let origin_str = parsed.origin().ascii_serialization();

                if let Ok(v) = HeaderValue::from_str(&origin_str) {
                    headers.insert("Origin", v);
                }

                let referer = format!("{}/", origin_str);
                if let Ok(v) = HeaderValue::from_str(&referer) {
                    headers.insert("Referer", v);
                }
            }
        }

        Ok(headers)
    }

    async fn process_response(response: reqwest::Response, params: &ProxyQuery) -> CoreResult<ProxyResponse> {
        let status = response.status();
        let content_length = response.content_length();
        let headers = response.headers().clone();

        let content_type_str = headers.get(CONTENT_TYPE)
            .and_then(|v| v.to_str().ok())
            .map(|s| s.to_string());

        let is_m3u8 = params.url.to_lowercase().ends_with(".m3u8") ||
            content_type_str.as_ref().map(|ct| {
                let ct_lower = ct.to_lowercase();
                ct_lower.contains("mpegurl") || ct_lower.contains("m3u8")
            }).unwrap_or(false);


        if is_m3u8 {
            debug!("Processing response as HLS m3u8 playlist");
            let body_text = response.text().await
                .map_err(|e| {
                    error!(error = ?e, "Failed to read m3u8 body text");
                    CoreError::Network("error.proxy.body_read_failed".into())
                })?;

            let base_url = Url::parse(&params.url)
                .map_err(|e| {
                    error!(error = ?e, url = %params.url, "Invalid upstream URL");
                    CoreError::Internal("error.proxy.invalid_upstream_url".into())
                })?;

            let processed = Self::process_m3u8_content(&body_text, &base_url, params)?;

            let mut out_headers = HeaderMap::new();
            out_headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/vnd.apple.mpegurl"));
            return Ok(ProxyResponse {
                status,
                headers: out_headers,
                body: ProxyBody::Text {
                    content: processed,
                    content_type: "application/vnd.apple.mpegurl".into()
                }
            });
        }

        let is_subtitle = params.url.contains(".vtt") || params.url.contains(".srt") || params.url.contains(".ass") ||
            content_type_str.as_deref().map(|ct| ct.contains("text/vtt") || ct.contains("text/srt")).unwrap_or(false);

        if is_subtitle {
            let body_text = response.text().await
                .map_err(|e| {
                    error!(error = ?e, "Failed to read subtitle body text");
                    CoreError::Network("error.proxy.body_read_failed".into())
                })?;

            let mime_type = if params.url.contains(".srt") || content_type_str.as_deref().map(|ct| ct.contains("srt")).unwrap_or(false) {
                "text/plain"
            } else if params.url.contains(".ass") {
                "text/plain"
            } else {
                "text/vtt"
            };

            let mut out_headers = HeaderMap::new();
            out_headers.insert(CONTENT_TYPE, HeaderValue::from_str(mime_type).unwrap_or(HeaderValue::from_static("text/plain")));
            out_headers.insert(CACHE_CONTROL, HeaderValue::from_static("public, max-age=3600"));

            return Ok(ProxyResponse {
                status,
                headers: out_headers,
                body: ProxyBody::Text {
                    content: body_text,
                    content_type: mime_type.into()
                }
            });
        }

        let mut out_headers = HeaderMap::new();

        let effective_ct = content_type_str.as_deref().unwrap_or("application/octet-stream");
        let is_likely_segment = effective_ct == "text/html"
            || effective_ct == "text/plain"
            || effective_ct == "application/octet-stream";

        let final_ct = if is_likely_segment {
            "video/mp2t"
        } else {
            effective_ct
        };

        if let Ok(v) = HeaderValue::from_str(final_ct) {
            out_headers.insert(CONTENT_TYPE, v);
        }
        if final_ct.starts_with("image/") || final_ct.starts_with("video/") {
            out_headers.insert(CACHE_CONTROL, HeaderValue::from_static("public, max-age=31536000, immutable"));
        }

        if status == StatusCode::PARTIAL_CONTENT {
            if let Some(cr) = headers.get(CONTENT_RANGE) {
                out_headers.insert(CONTENT_RANGE, cr.clone());
            }
        }

        if let Some(len) = content_length {
            out_headers.insert(CONTENT_LENGTH, HeaderValue::from(len));
        }

        out_headers.insert(ACCEPT_RANGES, HeaderValue::from_static("bytes"));

        let stream = response.bytes_stream().map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e));

        Ok(ProxyResponse {
            status,
            headers: out_headers,
            body: ProxyBody::Stream {
                stream: Box::pin(stream),
                content_length
            }
        })
    }

    fn process_m3u8_content(text: &str, base_url: &Url, params: &ProxyQuery) -> CoreResult<String> {
        let lines: Vec<&str> = text.lines().collect();
        let mut result = Vec::with_capacity(lines.len());

        let uri_regex = Regex::new(r#"URI="([^"]+)""#)
            .map_err(|e| {
                error!(error = ?e, "Failed to compile regex");
                CoreError::Internal("error.system.serialization".into())
            })?;

        for line in lines {
            let trimmed = line.trim();
            if trimmed.is_empty() {
                result.push(line.to_string());
                continue;
            }

            if trimmed.starts_with('#') {
                if trimmed.contains("URI=") {
                    let processed_line = uri_regex.replace_all(line, |caps: &regex::Captures| {
                        let uri = &caps[1];
                        let absolute_url = Self::resolve_url(base_url, uri);
                        let proxied = Self::build_proxy_url(&absolute_url, params);
                        format!("URI=\"{}\"", proxied)
                    });
                    result.push(processed_line.to_string());
                } else {
                    result.push(line.to_string());
                }
                continue;
            }

            let absolute_url = Self::resolve_url(base_url, trimmed);
            result.push(Self::build_proxy_url(&absolute_url, params));
        }

        Ok(result.join("\n"))
    }

    fn resolve_url(base: &Url, path: &str) -> String {
        match base.join(path) {
            Ok(u) => u.to_string(),
            Err(_) => path.to_string(),
        }
    }

    fn proxy_base_url() -> &'static str {
        #[cfg(target_os = "linux")]
        {
            "proxy://localhost"
        }

        #[cfg(not(target_os = "linux"))]
        {
            "http://proxy.localhost/proxy"
        }
    }

    fn build_proxy_url(target_url: &str, original_params: &ProxyQuery) -> String {
        let mut params = url::form_urlencoded::Serializer::new(String::new());

        params.append_pair("url", target_url);

        if let Some(ref r) = original_params.referer {
            params.append_pair("referer", r);
        }

        if let Some(ref o) = original_params.origin {
            params.append_pair("origin", o);
        }

        if let Some(ref ua) = original_params.user_agent {
            params.append_pair("userAgent", ua);
        }

        let qs = params.finish();

        format!("{}?{}", Self::proxy_base_url(), qs)
    }
}

fn normalize_url(url: &str) -> String {
    if let Some(rest) = url.strip_prefix("//") {
        format!("https://{rest}")
    } else {
        url.to_string()
    }
}