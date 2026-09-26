export type HomeSection = 'anime' | 'manga' | 'novel';
export type MetadataProvider = 'anilist' | 'myanimelist' | 'kitsu';

export type TitleLanguage = 'native' | 'romaji' | 'english' | 'chinese';
export type MangaLayout = 'scroll' | 'paged';
export type ReadingDirection = 'ltr' | 'rtl';
export type FitMode = 'width' | 'height' | 'fit';

export type NovelTheme = 'light' | 'dark' | 'sepia' | 'oled';
export type FontFamily = 'sans' | 'serif' | 'mono';
export type TextAlign = 'left' | 'justify';

export type MergeStrategy =
    | 'keepHighest'
    | 'keepLocal'
    | 'keepRemote'
    | 'keepLatest'
    | 'anilistFirst'
    | 'malFirst'
    | 'kitsuFirst'
    | 'simklFirst';

export type ScoreFormat =
    | 'point10'
    | 'point100'
    | 'point10Decimal'
    | 'point5Stars';

export interface GeneralConfig {
    showAdultContent: boolean;
    blurAdultContent: boolean;
    needSetup: boolean;
}

export interface UiConfig {
    sidebarCollapsed: boolean;
    disableCardTrailers: boolean;
    defaultHomeSection: HomeSection;
    titleLanguage: TitleLanguage;
}

export interface ContentConfig {
    preferredMetadataProvider: MetadataProvider;
    autoUpdateProgress: boolean;
    scoreFormat: ScoreFormat;
}

export interface ExtensionsConfig {
    repoUrls: string[];
}

export interface PlayerConfig {
    autoplayNextEpisode: boolean;
    preferredSubLang: string;
    preferredDubLang: string;
    autoSkipIntro: boolean;
    autoSkipOutro: boolean;
    seekStep: number;
    resumeFromLastPos: boolean;
    hwdec: string,
    scaleAlgorithm: string,
    interpolation: boolean,
    deband: boolean,
}

export interface SubtitleConfig {
    font: string,
    fontSize: number,
    color: string,          // "#RRGGBB" or "#RRGGBBAA"
    borderColor: string,
    borderSize: number,
    backgroundColor?: string,
    position: number,          // 0-100, sub-pos
    delay: number,
    forceStyle: boolean,      // maps to sub-ass-override no/force
    scale: number;
    shadowColor: string;
    shadowOffset: number;
    justify: "auto" | "left" | "center" | "right";
    sdhFilter: boolean;
    sdhFilterHarder: boolean;
}

export interface MangaConfig {
    layout: MangaLayout;
    direction: ReadingDirection;
    pagesPerView: number;
    fitMode: FitMode;
    gapX: number;
    gapY: number;
    preloadPages: number;
}

export interface NovelConfig {
    theme: NovelTheme;
    fontFamily: FontFamily;
    fontSize: number;
    lineHeight: number;
    maxWidth: number;
    textAlign: TextAlign;
    paragraphSpacing: number;
}

export interface DiscordConfig {
    enabled: boolean;
    showTitle: boolean;
    hideNsfw: boolean;
}

export interface ListConfig {
    mergeStrategy: MergeStrategy;
    syncOnStartup: boolean;
    syncIntervalSeconds: number;
    privateByDefault: boolean;
}

export interface AppConfig {
    general: GeneralConfig;
    ui: UiConfig;
    content: ContentConfig;
    extensions: ExtensionsConfig;
    player: PlayerConfig;
    subtitles: SubtitleConfig;
    manga: MangaConfig;
    novel: NovelConfig;
    discord: DiscordConfig;
    list: ListConfig;
}