use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct UserConfig {
    #[serde(default)]
    pub general: GeneralConfig,
    #[serde(default)]
    pub ui: UiConfig,
    #[serde(default)]
    pub content: ContentConfig,
    #[serde(default)]
    pub extensions: ExtensionsConfig,
    #[serde(default)]
    pub player: PlayerConfig,
    #[serde(default)]
    pub subtitles: SubtitleConfig,
    #[serde(default)]
    pub manga: MangaConfig,
    #[serde(default)]
    pub novel: NovelConfig,
    #[serde(default)]
    pub discord: DiscordConfig,
    #[serde(default)]
    pub list: ListConfig,
    #[serde(default)]
    pub torrent: TorrentConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GeneralConfig {
    pub show_adult_content: bool,
    pub blur_adult_content: bool,
    pub need_setup: bool,
}

impl Default for GeneralConfig {
    fn default() -> Self {
        Self {
            show_adult_content: false,
            blur_adult_content: true,
            need_setup: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UiConfig {
    pub sidebar_collapsed: bool,
    pub disable_card_trailers: bool,
    pub default_home_section: HomeSection,
    pub title_language: TitleLanguage,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub enum HomeSection {
    #[default]
    Anime,
    Manga,
    Novel,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub enum TitleLanguage {
    Native,
    #[default]
    Romaji,
    English,
    Chinese,
}

impl Default for UiConfig {
    fn default() -> Self {
        Self {
            sidebar_collapsed: false,
            disable_card_trailers: false,
            default_home_section: HomeSection::default(),
            title_language: TitleLanguage::default(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ContentConfig {
    pub preferred_metadata_provider: String,
    pub auto_update_progress: bool,
    pub score_format: ScoreFormat,
}

impl Default for ContentConfig {
    fn default() -> Self {
        Self {
            preferred_metadata_provider: "anilist".into(),
            auto_update_progress: true,
            score_format: ScoreFormat::default(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum ScoreFormat {
    Point10,        // 0–10 (default, matches internal)
    Point100,       // 0–100
    Point10Decimal, // 0.0–10.0 (1 decimal)
    Point5Stars,    // ★★★★☆
}

impl Default for ScoreFormat {
    fn default() -> Self {
        Self::Point10
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExtensionsConfig {
    #[serde(default)]
    pub repo_urls: Vec<String>,
}

impl Default for ExtensionsConfig {
    fn default() -> Self {
        Self { repo_urls: vec![] }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlayerConfig {
    pub autoplay_next_episode: bool,
    pub preferred_sub_lang: String,
    pub preferred_dub_lang: String,
    pub auto_skip_intro: bool,
    pub auto_skip_outro: bool,
    pub seek_step: u8,
    pub resume_from_last_pos: bool,
    pub hwdec: String,
    pub scale_algorithm: String,
    pub interpolation: bool,
    pub deband: bool,
    pub anime4k_mode: String,
    pub anime4k_tier: String,
}

impl Default for PlayerConfig {
    fn default() -> Self {
        Self {
            autoplay_next_episode: true,
            preferred_sub_lang: "en".into(),
            preferred_dub_lang: "jp".into(),
            auto_skip_intro: false,
            auto_skip_outro: false,
            seek_step: 10,
            resume_from_last_pos: true,
            hwdec: "auto-safe".to_string(),
            scale_algorithm: "bilinear".to_string(),
            interpolation: false,
            deband: true,
            anime4k_mode: "A".to_string(),
            anime4k_tier: "off".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SubtitleConfig {
    pub font: String,
    pub font_size: f64,
    pub color: String,          // "#RRGGBB" or "#RRGGBBAA"
    pub border_color: String,
    pub border_size: f64,
    pub background_color: Option<String>,
    pub position: f64,          // 0-100, sub-pos
    pub delay: f64,
    pub force_style: bool,      // maps to sub-ass-override no/force
    pub scale: f64,
    pub shadow_color: String,
    pub shadow_offset: f64,
    pub justify: String,
    pub sdh_filter: bool,
    pub sdh_filter_harder: bool,
}

impl Default for SubtitleConfig {
    fn default() -> Self {
        Self {
            font: "sans-serif".to_string(),
            font_size: 55.0,
            color: "#FFFFFF".to_string(),
            border_color: "#000000".to_string(),
            border_size: 3.0,
            background_color: None,
            position: 95.0,
            delay: 0.0,
            force_style: false,
            scale: 1.0,
            shadow_color: "#00000000".into(),
            shadow_offset: 0.0,
            justify: "auto".into(),
            sdh_filter: false,
            sdh_filter_harder: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MangaConfig {
    pub layout: MangaLayout,
    pub direction: ReadingDirection,
    pub pages_per_view: u8,
    pub fit_mode: FitMode,
    pub gap_x: u8,
    pub gap_y: u8,
    pub preload_pages: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub enum MangaLayout {
    Scroll,
    #[default]
    Paged,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub enum ReadingDirection {
    Ltr,
    #[default]
    Rtl,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub enum FitMode {
    #[default]
    Width,
    Height,
}

impl Default for MangaConfig {
    fn default() -> Self {
        Self {
            layout: MangaLayout::default(),
            direction: ReadingDirection::default(),
            pages_per_view: 1,
            fit_mode: FitMode::default(),
            gap_x: 0,
            gap_y: 8,
            preload_pages: 3,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NovelConfig {
    pub theme: NovelTheme,
    pub font_family: FontFamily,
    pub font_size: u8,
    pub line_height: f32,
    pub max_width: u16,
    pub text_align: TextAlign,
    pub paragraph_spacing: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub enum NovelTheme {
    #[default]
    Light,
    Dark,
    Sepia,
    Oled,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub enum FontFamily {
    #[default]
    Sans,
    Serif,
    Mono,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub enum TextAlign {
    #[default]
    Left,
    Justify,
}

impl Default for NovelConfig {
    fn default() -> Self {
        Self {
            theme: NovelTheme::default(),
            font_family: FontFamily::default(),
            font_size: 16,
            line_height: 1.6,
            max_width: 700,
            paragraph_spacing: 2.0,
            text_align: TextAlign::default(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DiscordConfig {
    pub enabled: bool,
    pub show_title: bool,
    pub hide_nsfw: bool,
}

impl Default for DiscordConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            show_title: true,
            hide_nsfw: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListConfig {
    pub merge_strategy: MergeStrategy,
    pub sync_on_startup: bool,
    pub sync_interval_seconds: u32,
    pub private_by_default: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub enum MergeStrategy {
    #[default]
    KeepHighest,
    KeepLocal,
    KeepRemote,
    KeepLatest,
    AnilistFirst,
    MalFirst,
    KitsuFirst,
    SimklFirst
}

impl Default for ListConfig {
    fn default() -> Self {
        Self {
            merge_strategy: MergeStrategy::KeepHighest,
            sync_on_startup:         true,
            sync_interval_seconds:   3600,
            private_by_default:      false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TorrentConfig {
    pub auto_select: bool,
    pub preferred_groups: Vec<String>,
    pub preferred_resolution: String,
    pub preferred_codec: Option<String>,
    pub prefer_dual_audio: bool,
    pub min_seeders: u32,
    pub exclude_keywords: Vec<String>,
    pub require_batch: Option<bool>,
    pub fallback_to_manual: bool,
    pub max_disk_usage_bytes: u64,
    pub finished_file_ttl_seconds: u64,
    pub max_concurrent_torrents: usize,
    pub download_rate_limit_kbps: Option<u32>,
    pub upload_rate_limit_kbps: Option<u32>,
    pub stop_seeding_on_playback_end: bool,
    pub min_free_disk_space_bytes: u64,
}

impl Default for TorrentConfig {
    fn default() -> Self {
        Self {
            auto_select: false,
            preferred_groups: vec![],
            preferred_resolution: "1080p".into(),
            preferred_codec: None,
            prefer_dual_audio: false,
            min_seeders: 1,
            exclude_keywords: vec![],
            require_batch: None,
            fallback_to_manual: true,
            max_disk_usage_bytes: 10 * 1024 * 1024 * 1024,
            finished_file_ttl_seconds: 6 * 60 * 60,
            max_concurrent_torrents: 1,
            download_rate_limit_kbps: None,
            upload_rate_limit_kbps: None,
            stop_seeding_on_playback_end: true,
            min_free_disk_space_bytes: 2 * 1024 * 1024 * 1024, // 2 GiB
        }
    }
}