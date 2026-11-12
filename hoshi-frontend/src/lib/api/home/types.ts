import type { ContentType, ExtensionSearchResult, FullContent } from "@/api/content/types";

export type Label =
    | { kind: "key"; key: string }
    | { kind: "text"; text: string };

export type SectionLayout = "carousel" | "ranked" | "strip";

export type DiscoverQuery =
    | {
        kind: "tracker";
        tracker: string;
        contentType: ContentType;
        query?: string | null;
        sort?: string | null;
        genre?: string | null;
        format?: string | null;
        status?: string | null;
        tags?: string[];
    }
    | {
        kind: "extension";
        extension: string;
        query?: string | null;
        filters?: Record<string, unknown> | null;
    };

export type BuiltinSection =
    | { type: "continueWatching" }
    | { type: "recentlyAired" }
    | { type: "airingToday" }
    | { type: "newEpisodes" }
    | { type: "unstartedSequels" }
    | { type: "listStatus"; status: string }
    | { type: "randomPlanning" }
    | { type: "recentlyCompleted" }
    | { type: "extensions" };

export type SectionSource =
    | { kind: "builtin"; builtin: BuiltinSection }
    | { kind: "search"; query: DiscoverQuery };

export interface HomeSection {
    id: string;
    title: Label;
    layout: SectionLayout;
    contentType: ContentType | null;
    source: SectionSource;
    hidden?: boolean;
}

export interface HomeTab {
    id: string;
    title: Label;
    icon: string;
    hero: string | null;
    sections: HomeSection[];
}

export interface HomeLayout {
    tabs: HomeTab[];
}

export type DiscoverItem =
    | ({ kind: "content" } & FullContent)
    | { kind: "extension"; extension: string; item: ExtensionSearchResult };

export interface SequelCard {
    targetCid: string | null;
    targetTrackerName: string;
    targetTrackerId: string;
    targetTitle: string;
    targetCoverImage: string | null;
}
