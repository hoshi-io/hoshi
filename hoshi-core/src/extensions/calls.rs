use serde::de::DeserializeOwned;
use serde_json::{json, Value};
use tokio::fs;
use tracing::{error, instrument};

use super::{sandbox, ExtensionManager, LNREADER, LNREADER_ARC, SORA, SORA_ARC};
use crate::error::{CoreError, CoreResult};
use crate::extensions::types::{
    Chapter, CompatLayer, Episode, EpisodeSource, ExtensionFeatures, ExtensionFilters,
    ExtensionMetadata, ExtensionSearchResult, Page,
};

impl ExtensionManager {
    #[instrument(skip(self, args))]
    pub async fn call_extension_function(
        &self,
        extension_id: &str,
        function_name: &str,
        args: Vec<Value>,
        http_client: reqwest::Client,
    ) -> CoreResult<Value> {
        let extension = self.extensions.get(extension_id).ok_or_else(|| {
            error!(ext = %extension_id, func = %function_name, "Attempted to call function on unloaded extension");
            CoreError::NotFound("error.extension.not_found".into())
        })?;

        if !extension.script_path.exists() {
            error!(ext = %extension_id, path = %extension.script_path.display(), "Extension script file missing from disk");
            return Err(CoreError::NotFound("error.extension.script_missing".into()));
        }

        let extension_code = fs::read_to_string(&extension.script_path).await.map_err(CoreError::Io)?;

        let compat = match extension.source.as_deref() {
            Some("lnreader") => Some(CompatLayer::Lnreader(
                LNREADER_ARC.get_or_init(|| LNREADER.into()).clone()
            )),
            Some("sora") => Some(CompatLayer::Sora(
                SORA_ARC.get_or_init(|| SORA.into()).clone()
            )),
            _ => None,
        };

        let ext_type = extension.ext_type.clone();

        sandbox::execute_in_quickjs(
            extension_code,
            function_name.to_string(),
            args,
            self.headless.clone(),
            extension.settings.clone(),
            extension_id.to_string(),
            std::sync::Arc::clone(&self.extension_state),
            compat,
            ext_type,
            http_client
        ).await
    }

    #[instrument(skip(self, args))]
    async fn call_typed_function<T: DeserializeOwned>(
        &self,
        extension_id: &str,
        function_name: &str,
        args: Vec<Value>,
        http_client: reqwest::Client,
    ) -> CoreResult<T> {
        let raw_value = self.call_extension_function(extension_id, function_name, args, http_client).await?;

        serde_json::from_value(raw_value).map_err(|e| {
            error!(ext = %extension_id, func = %function_name, error = ?e, "Failed to deserialize response");
            CoreError::Internal("error.content.invalid_extension_response".into())
        })
    }

    pub async fn get_settings(&self, ext_id: &str) -> CoreResult<ExtensionFeatures> {
        self.call_typed_function(ext_id, "getStreamingSettings", vec![], self.http_client.clone()).await
    }

    pub async fn get_filters(&self, ext_id: &str) -> CoreResult<ExtensionFilters> {
        self.call_typed_function(ext_id, "getFilters", vec![], self.http_client.clone()).await
    }

    pub async fn search(&self, ext_id: &str, query: &str, filters: Value, page: u32) -> CoreResult<Vec<ExtensionSearchResult>> {
        self.call_typed_function(ext_id, "search", vec![json!(query), filters, json!(page)], self.http_client.clone()).await
    }

    pub async fn get_metadata(&self, ext_id: &str, content_id: &str) -> CoreResult<ExtensionMetadata> {
        self.call_typed_function(ext_id, "getMetadata", vec![json!(content_id)], self.http_client.clone()).await
    }

    pub async fn find_episodes(&self, ext_id: &str, content_id: &str) -> CoreResult<Vec<Episode>> {
        self.call_typed_function(ext_id, "findEpisodes", vec![json!(content_id)], self.http_client.clone()).await
    }

    pub async fn find_chapters(&self, ext_id: &str, content_id: &str) -> CoreResult<Vec<Chapter>> {
        self.call_typed_function(ext_id, "findChapters", vec![json!(content_id)], self.http_client.clone()).await
    }

    pub async fn find_episode_server(&self, ext_id: &str, content_id: &str, server: &str, category: &str) -> CoreResult<EpisodeSource> {
        self.call_typed_function(ext_id, "findEpisodeServer", vec![json!(content_id), json!(server), json!(category)], self.http_client.clone()).await
    }

    pub async fn list_episode_servers(&self, ext_id: &str, content_id: &str) -> CoreResult<Vec<String>> {
        self.call_typed_function(ext_id, "listEpisodeServers", vec![json!(content_id)], self.http_client.clone()).await
    }

    pub async fn find_manga_pages(&self, ext_id: &str, chapter_id: &str) -> CoreResult<Vec<Page>> {
        self.call_typed_function(ext_id, "findChapterPages", vec![json!(chapter_id)], self.http_client.clone()).await
    }
    
    pub async fn find_novel_html(&self, ext_id: &str, chapter_id: &str) -> CoreResult<String> {
        self.call_typed_function(ext_id, "findChapterPages", vec![json!(chapter_id)], self.http_client.clone()).await
    }
}