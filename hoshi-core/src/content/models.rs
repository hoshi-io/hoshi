use std::collections::HashMap;
use std::fmt;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use crate::tracker::types::TrackerMapping;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Content {
    pub cid: String,
    pub content_type: ContentType,
    pub nsfw: bool,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Metadata {
    pub id: Option<i64>,
    pub cid: String,
    pub source_name: String,
    pub source_id: Option<String>,
    pub subtype: Option<String>,
    pub title: String,
    pub alt_titles: Vec<String>,
    #[serde(default)]
    pub title_i18n: HashMap<String, String>,
    pub synopsis: Option<String>,
    pub cover_image: Option<String>,
    pub banner_image: Option<String>,
    pub eps_or_chapters: EpisodeData,
    pub status: Option<Status>,
    pub genres: Vec<String>,
    pub release_date: Option<String>,
    pub end_date: Option<String>,
    pub rating: Option<f32>,
    pub trailer_url: Option<String>,
    pub characters: Vec<Character>,
    pub studio: Option<String>,
    pub staff: Vec<StaffMember>,
    pub external_ids: Value,
    pub episode_duration: Option<i32>,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FullContent {
    pub content: Content,
    pub metadata: Vec<Metadata>,
    pub tracker_mappings: Vec<TrackerMapping>,
    pub extension_sources: Vec<ExtensionSource>,
    pub relations: Vec<Relation>,
    #[serde(default)]
    pub content_units: Vec<ContentUnit>,
}

impl FullContent {
    pub fn primary_metadata(&self) -> Option<&Metadata> {
        self.metadata.iter().find(|m| m.source_name == "anilist")
            .or_else(|| self.metadata.first())
    }

    pub fn slim_for_home(mut self) -> Self {
        for meta in &mut self.metadata {
            meta.characters  = vec![];
            meta.staff       = vec![];
            meta.external_ids = Value::Null;
        }
        self.relations         = vec![];
        self.content_units     = vec![];
        self.extension_sources = vec![];
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ContentType {
    Anime,
    Manga,
    Novel,
}

impl ContentType {
    pub fn as_str(&self) -> &'static str {
        match self {
            ContentType::Anime => "anime",
            ContentType::Manga => "manga",
            ContentType::Novel => "novel",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Status {
    Planned,
    Ongoing,
    Completed,
    Cancelled,
    Hiatus,
}

impl fmt::Display for Status {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum EpisodeData {
    Count(i32),
    List(Vec<EpisodeInfo>),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RelationType {
    Sequel,
    Prequel,
    SideStory,
    Spinoff,
    Adaptation,
    Alternative,
    Parent,
    Summary,
    Source,
    Compilation,
    Contains,
    Character,
    Other,
}

impl RelationType {
    pub fn as_str(&self) -> &'static str {
        match self {
            RelationType::Sequel      => "sequel",
            RelationType::Prequel     => "prequel",
            RelationType::SideStory   => "side_story",
            RelationType::Spinoff     => "spinoff",
            RelationType::Adaptation  => "adaptation",
            RelationType::Alternative => "alternative",
            RelationType::Parent      => "parent",
            RelationType::Summary     => "summary",
            RelationType::Source      => "source",
            RelationType::Compilation => "compilation",
            RelationType::Contains    => "contains",
            RelationType::Character   => "character",
            RelationType::Other       => "other",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EpisodeInfo {
    pub number: i32,
    pub title: Option<String>,
    pub aired: Option<String>,
    pub duration: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Character {
    pub name: String,
    pub role: String,
    pub actor: Option<String>,
    pub image: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StaffMember {
    pub name: String,
    pub role: String,
    pub image: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExtensionSource {
    pub id: Option<i64>,
    pub cid: String,
    pub extension_name: String,
    pub extension_id: String,
    pub nsfw: bool,
    pub language: Option<String>,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Relation {
    pub id: Option<i64>,
    pub source_cid: String,
    pub target_cid: String,
    pub relation_type: RelationType,
    pub source_name: String,
    pub created_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ContentUnit {
    pub id: Option<i64>,
    pub cid: String,
    pub unit_number: f64,
    pub content_type: String,
    pub title: Option<String>,
    pub description: Option<String>,
    pub thumbnail_url: Option<String>,
    pub released_at: Option<String>,
    pub duration: Option<i32>,
    pub absolute_number: Option<i32>,
    pub created_at: i64,
}