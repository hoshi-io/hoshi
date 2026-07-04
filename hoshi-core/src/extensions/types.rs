use std::collections::HashMap;
use std::path::PathBuf;
use serde::{Deserialize, Deserializer, Serialize};
use serde_json::Value;

pub enum CompatLayer {
    Lnreader(String),
    Sora(String),
    Tachiyomi(String),
    Aniyomi(String),
}

#[derive(Debug, Clone)]
pub struct SoraAuthor {
    pub name: String,
    pub icon: Option<String>,
    pub url: Option<String>,
}

impl<'de> Deserialize<'de> for SoraAuthor {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(untagged)]
        enum AuthorShape {
            Name(String),
            Object {
                name: String,
                #[serde(default)]
                icon: Option<String>,
                #[serde(default)]
                url: Option<String>,
            },
        }

        Ok(match AuthorShape::deserialize(deserializer)? {
            AuthorShape::Name(name) => SoraAuthor { name, icon: None, url: None },
            AuthorShape::Object { name, icon, url } => SoraAuthor { name, icon, url },
        })
    }
}

#[derive(Debug, Deserialize)]
pub struct SoraMarketplaceEntry {
    pub id: String,
    #[serde(rename = "sourceName")]
    pub source_name: String,
    #[serde(rename = "iconUrl")]
    pub icon_url: String,
    pub language: String,
    #[serde(rename = "baseUrl")]
    pub base_url: Option<String>,
    #[serde(rename = "manifestUrl")]
    pub manifest_url: String,
    #[serde(rename = "type")]
    pub ext_type: String,
    pub author: SoraAuthor,
}

#[derive(Debug, Deserialize)]
pub struct SoraModuleManifest {
    pub version: String,
    #[serde(rename = "scriptUrl")]
    pub script_url: String,
    #[serde(rename = "streamType")]
    pub stream_type: Option<String>,
    pub quality: Option<String>,
    pub note: Option<String>,
    #[serde(default)]
    pub softsub: bool,
}


pub fn normalize_sora_type(raw: &str) -> &'static str {
    match raw {
        "anime" => "anime",
        "novel" => "novel",
        "movies/shows" | "movies" | "shows" => "anime",
        _ => "anime",
    }
}

#[derive(Debug, Deserialize)]
pub struct TachiyomiMarketplaceEntry {
    pub name: String,
    pub pkg: String,
    pub apk: String,
    pub lang: String,
    pub version: String,
    pub nsfw: u8,
    pub sources: Vec<TachiyomiSource>,

    pub repo_url: String,
    pub icon_url: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct TachiyomiSource {
    pub name: String,
    pub lang: String,
    pub id: String,
    #[serde(rename = "baseUrl")]
    pub base_url: String,
}

#[derive(Debug, Deserialize)]
pub struct ExtensionManifest {
    pub id: String,
    pub name: String,
    pub version: String,
    pub author: Option<String>,
    #[serde(rename = "type")]
    pub ext_type: ExtensionType,
    pub main: String,
    pub icon: Option<String>,
    pub language: String,
    #[serde(default)]
    pub nsfw: bool,
    #[serde(default)]
    pub skip_default_processing: bool,
    #[serde(default)]
    pub settings: Vec<SettingDefinition>,
    #[serde(default)]
    pub source: Option<String>,
}

#[derive(Debug, serde::Deserialize)]
pub struct LNReaderMarketplaceEntry {
    pub id:      String,
    pub name:    String,
    pub site:    String,
    pub lang:    String,
    pub version: String,
    pub url:     String,
    #[serde(rename = "iconUrl")]
    pub icon_url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SettingDefinition {
    pub key: String,
    pub label: String,
    #[serde(rename = "type")]
    pub setting_type: SettingType,
    pub default: Value,
    #[serde(default)]
    pub options: Vec<SettingOption>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SettingOption {
    pub value: String,
    pub label: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum SettingType {
    String,
    Number,
    Boolean,
    Select,
    MultiSelect,
    #[serde(other)]
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Extension {
    pub id: String,
    pub name: String,
    pub version: String,
    pub author: String,
    pub icon: Option<String>,
    pub ext_type: ExtensionType,
    #[serde(skip)]
    pub script_path: PathBuf,
    pub language: String,
    pub nsfw: bool,
    pub skip_default_processing: bool,
    pub setting_definitions: Vec<SettingDefinition>,
    pub settings: HashMap<String, Value>,
    pub source: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub enum ExtensionType {
    Anime,
    Manga,
    Novel,
    #[serde(other)]
    Unknown,
}

#[derive(Deserialize)]
pub struct ExtensionMetadata {
   pub title: String,
   pub synopsis: Option<String>,
   pub image: Option<String>,
   pub eps_or_chapters: Option<i64>,
   pub rating: Option<f64>,
   pub year: Option<i64>,
   pub genres: Option<Vec<String>>,
   pub nsfw: Option<bool>,
   pub anilist_id: Option<Value>,
   pub mal_id: Option<Value>,
   pub external_ids: Option<Value>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ExtensionSearchResult {
    pub id: String,
    pub title: String,
    pub image: Option<String>,
    pub url: Option<String>,
    pub nsfw: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Episode {
    pub id: String,
    pub number: Option<f64>,
    pub title: Option<String>,
    pub url: Option<String>,
    pub image: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Chapter {
    pub id: String,
    pub title: String,
    pub number: Option<f64>,
    pub index: Option<f64>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ContentItems {
    Episodes(Vec<Episode>),
    Chapters(Vec<Chapter>),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EpisodeSource {
    pub headers: HashMap<String, String>,
    pub source: Source,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Source {
    pub url: String,
    pub subtitles: Vec<Subtitle>,
    pub chapters: Vec<EpisodeChapter>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Subtitle {
    pub id: String,
    pub url: String,
    pub language: String,
    pub is_default: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct EpisodeChapter {
    pub start: f64,
    pub end: f64,
    pub title: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Page {
    pub url: String,
    pub index: Option<f64>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum PlayContentResult {
    Video(EpisodeSource),
    Reader(Vec<Page>),
    Novel(String),
}

#[derive(Debug, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct ExtensionFeatures {
    pub episode_servers: Option<Vec<String>>,
    pub supports_dub: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FilterOption {
    pub value: String,
    pub label: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FilterDefinition {
    pub label: String,
    #[serde(rename = "type")]
    pub filter_type: String,
    pub options: Option<Vec<FilterOption>>,
}

pub type ExtensionFilters = HashMap<String, FilterDefinition>;