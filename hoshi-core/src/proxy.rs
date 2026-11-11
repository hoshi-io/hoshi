use reqwest::StatusCode;
use reqwest::header::{HeaderMap, HeaderValue, CONTENT_TYPE};
use serde::Deserialize;
use std::time::Duration;
use url::Url;
use tracing::{instrument, warn};

use crate::core_err;
use crate::error::{CoreError, CoreResult};
use crate::state::AppState;

const SEND_TIMEOUT: Duration = Duration::from_secs(15);
const RETRY_DELAY: Duration = Duration::from_millis(500);
const MAX_ATTEMPTS: u8 = 2;

#[derive(Deserialize, Clone)]
pub struct ProxyQuery {
    pub url: String,
    pub referer: Option<String>,
    pub origin: Option<String>,
    #[serde(rename = "userAgent")]
    pub user_agent: Option<String>,
}

pub struct ProxyResponse {
    pub status: u16,
    pub content_type: String,
    pub body: Vec<u8>,
}

pub struct ProxyService;

impl ProxyService {
    #[instrument(skip(state, params), fields(url = %params.url))]
    pub async fn handle_request(state: &AppState, params: ProxyQuery) -> CoreResult<ProxyResponse> {
        if params.url.is_empty() {
            warn!("Proxy request rejected: No URL provided");
            return Err(CoreError::BadRequest("error.proxy.no_url_provided".into()));
        }

        let url = normalize_url(&params.url);
        let req_headers = Self::build_upstream_headers(&params)?;
        let mut last_error = CoreError::Network("error.proxy.upstream_error".into());

        for attempt in 1..=MAX_ATTEMPTS {
            if attempt > 1 {
                tokio::time::sleep(RETRY_DELAY).await;
            }

            let sent = tokio::time::timeout(
                SEND_TIMEOUT,
                state.http_client.get(&url).headers(req_headers.clone()).send(),
            ).await;

            let response = match sent {
                Err(_) => {
                    warn!(attempt, "Upstream request timed out");
                    last_error = CoreError::Network("error.proxy.upstream_timeout".into());
                    continue;
                }
                Ok(Err(e)) => {
                    warn!(attempt, error = ?e, "Upstream request failed");
                    last_error = CoreError::Network("error.proxy.upstream_error".into());
                    continue;
                }
                Ok(Ok(res)) => res,
            };

            let status = response.status();
            if !status.is_success() {
                warn!(attempt, %status, "Upstream returned an error status");
                last_error = CoreError::Network("error.proxy.upstream_error".into());
                if status == StatusCode::FORBIDDEN || status == StatusCode::NOT_FOUND {
                    break;
                }
                continue;
            }

            return Self::read_response(response).await;
        }

        Err(last_error)
    }

    fn build_upstream_headers(params: &ProxyQuery) -> CoreResult<HeaderMap> {
        let mut headers = HeaderMap::new();

        let ua = params.user_agent.as_deref().unwrap_or("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36");
        headers.insert("User-Agent", HeaderValue::from_str(ua)
            .map_err(|e| core_err!(BadRequest, "error.proxy.invalid_header", e))?);

        headers.insert("Accept", HeaderValue::from_static("*/*"));
        headers.insert("Accept-Language", HeaderValue::from_static("en-US,en;q=0.9"));

        if let Some(ref r) = params.referer {
            if let Ok(v) = HeaderValue::from_str(r) { headers.insert("Referer", v); }
        }
        if let Some(ref o) = params.origin {
            if let Ok(v) = HeaderValue::from_str(o) { headers.insert("Origin", v); }
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

    async fn read_response(response: reqwest::Response) -> CoreResult<ProxyResponse> {
        let status = response.status().as_u16();
        let content_type = response
            .headers()
            .get(CONTENT_TYPE)
            .and_then(|v| v.to_str().ok())
            .unwrap_or("application/octet-stream")
            .to_string();

        let body = response
            .bytes()
            .await
            .map_err(|e| core_err!(Network, "error.proxy.body_read_failed", e))?
            .to_vec();

        Ok(ProxyResponse { status, content_type, body })
    }
}

fn normalize_url(url: &str) -> String {
    if let Some(rest) = url.strip_prefix("//") {
        format!("https://{rest}")
    } else {
        url.to_string()
    }
}
