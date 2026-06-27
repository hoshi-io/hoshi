export type ContentType = "anime" | "manga" | "novel";

export type Status =
    | "planned"
    | "ongoing"
    | "completed"
    | "cancelled"
    | "hiatus";

export type RelationType =
    | "sequel"
    | "prequel"
    | "side_story"
    | "spinoff"
    | "adaptation"
    | "alternative"
    | "parent"
    | "source"
    | "compilation"
    | "contains"
    | "summary";

export interface Character {
    name: string;
    role: string;
    actor?: string | null;
    image?: string | null;
}

export interface StaffMember {
    name: string;
    role: string;
    image?: string | null;
}

export interface Relation {
    id?: number | null;
    sourceCid: string;
    targetCid: string;
    relationType: RelationType;
    sourceName: string;
    createdAt: number;
}

export interface ContentUnit {
    id?: number | null;
    cid: string;
    unitNumber: number;
    contentType: string;
    title?: string | null;
    description?: string | null;
    thumbnailUrl?: string | null;
    releasedAt?: string | null;
    duration?: number | null;
    absoluteNumber?: number | null;
    createdAt: number;
}

export interface TrackerMapping {
    cid: string;
    trackerName: string;
    trackerId: string;
    trackerUrl?: string | null;
    createdAt: number;
    updatedAt: number;
}

export interface ExtensionSource {
    id?: number | null;
    cid: string;
    extensionName: string;
    extensionId: string;
    nsfw: boolean;
    language?: string | null;
    createdAt: number;
    updatedAt: number;
}

export interface Content {
    cid: string;
    contentType: ContentType;
    nsfw: boolean;
    createdAt: number;
    updatedAt: number;
}

export interface Metadata {
    id?: number | null;
    cid: string;
    sourceName: string;
    sourceId?: string | null;
    subtype?: string | null;
    title: string;
    altTitles: string[];
    titleI18n: Record<string, string>;
    synopsis?: string | null;
    coverImage?: string | null;
    bannerImage?: string | null;
    epsOrChapters: number | null;
    status?: Status | null;
    genres: string[];
    releaseDate?: string | null;
    endDate?: string | null;
    rating?: number | null;
    trailerUrl?: string | null;
    characters: Character[];
    studio?: string | null;
    staff: StaffMember[];
    externalIds: any;
    episodeDuration?: number | null;
    createdAt: number;
    updatedAt: number;
}

export interface FullContent {
    content: Content;
    metadata: Metadata[];
    trackerMappings: TrackerMapping[];
    extensionSources: ExtensionSource[];
    relations: Relation[];
    contentUnits: ContentUnit[];
}

export interface SearchQuery {
    type?: ContentType;
    nsfw?: boolean;
    status?: string;
    query?: string;
    limit?: number;
    offset?: number;
    page?: number;
    sort?: string;
    genre?: string;
    format?: string;
    tracker?: "anilist" | "mal" | "kitsu";
    extensionFilters?: string;
}

export interface ContentListResponse {
    data: TrackerMedia[];
    total: number;
    limit: number;
    offset: number;
}
export interface PlayResponse {
    type: "video" | "reader" | any;
    data: any;
}

export interface AnimeSection {
    [key: string]: FullContent[];
    trending: FullContent[];
    popular: FullContent[];
    topRated: FullContent[];
    seasonal: FullContent[];
    upcoming: FullContent[];
    recentlyFinished: FullContent[];
    topAction: FullContent[];
    topRomance: FullContent[];
    topFantasy: FullContent[];
    topScifi: FullContent[];
    topSports: FullContent[];
}

export interface MangaSection {
    [key: string]: FullContent[];
    trending: FullContent[];
    popular: FullContent[];
    topRated: FullContent[];
    seasonal: FullContent[];
    recentlyFinished: FullContent[];
}

export interface NovelSection {
    [key: string]: FullContent[];
    trending: FullContent[];
    popular: FullContent[];
    topRated: FullContent[];
    recentlyFinished: FullContent[];
}

export interface HomeView {
    anime: AnimeSection;
    manga: MangaSection;
    novel: NovelSection;
    cachedAt: number;
}

export interface UpdateTrackerMappingRequest {
    trackerName: string;
    trackerId: string;
}

export interface UpdateExtensionMappingRequest {
    extensionName: string;
    extensionId: string;
}

export interface TrackerMedia {
    trackerId: string;
    trackerUrl?: string | null;
    crossIds: Record<string, string>;
    contentType: ContentType;
    title: string;
    altTitles: string[];
    titleI18n: Record<string, string>;
    synopsis?: string | null;
    coverImage?: string | null;
    bannerImage?: string | null;
    episodeCount?: number | null;
    chapterCount?: number | null;
    status?: string | null;
    genres: string[];
    tags: string[];
    nsfw: boolean;
    releaseDate?: string | null;
    endDate?: string | null;
    rating?: number | null;
    trailerUrl?: string | null;
    format?: string | null;
    studio?: string | null;
    characters: Character[];
    staff: StaffMember[];
    relations: TrackerRelation[];
    episodeDuration?: number | null;
}

export interface TrackerRelation {
    relationType: string;
    media: TrackerMedia;
}

export interface ExtensionSearchResult {
    id: string;
    title: string;
    image: string | null;
    url: string | null;
    nsfw: boolean | null;
}

export function primaryMetadata(
    fullContent: FullContent | null | undefined,
    preferredProvider: string = 'anilist'
): Metadata | undefined {
    if (!fullContent || !fullContent.metadata || fullContent.metadata.length === 0) {
        return undefined;
    }
    const preferred = fullContent.metadata.find(m =>
        m.sourceName.toLowerCase() === preferredProvider.toLowerCase()
    );
    if (preferred) return preferred;

    const anilist = fullContent.metadata.find(m =>
        m.sourceName.toLowerCase() === 'anilist'
    );
    return anilist || fullContent.metadata[0];
}

export interface RelationNode {
    cid: string;
    title: string;
    coverImage: string | null;
}

export interface RelationEdge {
    sourceCid: string;
    targetCid: string;
    relationType: RelationType;
}

export interface RelationGraph {
    nodes: RelationNode[];
    edges: RelationEdge[];
}