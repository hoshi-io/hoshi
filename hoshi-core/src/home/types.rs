use serde::{Deserialize, Serialize};
use crate::content::models::ContentType;
use crate::content::types::DiscoverQuery;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HomeLayout {
    pub tabs: Vec<HomeTab>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HomeTab {
    pub id: String,
    pub title: Label,
    pub icon: String,
    /// Section whose items feed the hero; none hides it.
    #[serde(default)]
    pub hero: Option<String>,
    pub sections: Vec<HomeSection>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HomeSection {
    pub id: String,
    pub title: Label,
    pub layout: SectionLayout,
    /// None = every content type.
    #[serde(default)]
    pub content_type: Option<ContentType>,
    pub source: SectionSource,
    #[serde(default)]
    pub hidden: bool,
}

/// Defaults use i18n keys; anything the user names is stored as plain text.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum Label {
    Key { key: String },
    Text { text: String },
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum SectionLayout {
    Carousel,
    Ranked,
    Strip,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum SectionSource {
    Builtin { builtin: BuiltinSection },
    Search { query: DiscoverQuery },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum BuiltinSection {
    ContinueWatching,
    RecentlyAired,
    AiringToday,
    NewEpisodes,
    UnstartedSequels,
    #[serde(rename_all = "camelCase")]
    ListStatus { status: String },
    RandomPlanning,
    RecentlyCompleted,
    Extensions,
}

/// A sequel of something the user finished that isn't in their list yet.
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct SequelCard {
    pub target_cid: Option<String>,
    pub target_tracker_name: String,
    pub target_tracker_id: String,
    pub target_title: String,
    pub target_cover_image: Option<String>,
}
