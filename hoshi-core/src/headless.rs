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
    #[serde(default = "default_method")]
    pub method: String,
    #[serde(default)]
    pub headers: std::collections::HashMap<String, String>,
    #[serde(default)]
    pub body: Option<String>,
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

fn default_method() -> String { "GET".to_string() }
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