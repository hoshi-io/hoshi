import type { ContentType } from "@/api/content/types";
import { normalizeFullContent, normalizeTrackerMedia, normalizeExtensionResult } from "@/utils/normalize";
import type { NormalizedCard } from "@/utils/normalize";
import { contentApi } from "@/api/content/content";
import { extensions } from "@/stores/extensions.svelte.js";
import type { CoreError } from "@/api/client";
import {extensionsApi} from "@/api/extensions/extensions";


class SearchState {
    query = $state("");
    contentType = $state<ContentType>("anime");
    searchMode = $state<"tracker" | "extension">("tracker");

    availableExtensions = $derived(
        extensions.installed.filter(ext => ext.ext_type === this.contentType)
    );

    hasSearched = $state(false);
    error = $state<CoreError | null>(null);

    selectedExtension = $state<string>("");
    page = $state<number>(1);

    status = $state<string>("");
    genre = $state<string>("");
    format = $state<string>("");
    nsfw = $state<boolean>(false);
    sort = $state<string>("");
    tracker = $state<"anilist" | "mal" | "kitsu" | "simkl">("anilist");

    extFilterValues = $state<Record<string, any>>({});

    results = $state<NormalizedCard[]>([]);

    isLoading = $state(false);

    private searchToken = 0;

    displayResults = $derived(this.results);

    nextPage() {
        const isTrending =
            this.searchMode === "tracker" &&
            !this.query.trim() &&
            !this.status &&
            !this.genre &&
            !this.format &&
            !this.nsfw;

        if (isTrending) return;

        this.page += 1;

        if (this.searchMode === "tracker") {
            this.search();
        } else {
            this.extensionSearch();
        }
    }

    updateContentType(type: ContentType) {
        this.contentType = type;
        this.results = [];
        this.page = 1;
        this.format = "";
        this.search();
    }

    clearFilters() {
        this.status = "";
        this.genre = "";
        this.format = "";
        this.extFilterValues = {};
    }

    async search() {
        const token = ++this.searchToken;
        this.isLoading = true;
        this.error = null;

        if (this.page === 1) this.results = [];

        if (!this.query.trim() && !this.status && !this.genre && !this.format && !this.nsfw) {
            try {
                const res = await contentApi.getTrending(this.contentType);
                if (token !== this.searchToken) return; // a newer search has since started, drop this

                this.results = (res || []).map(normalizeFullContent);
            } catch (e) {
                if (token !== this.searchToken) return;
                this.error = e as CoreError;
            } finally {
                if (token === this.searchToken) this.isLoading = false;
            }
            return;
        }

        try {
            const res = await contentApi.search({
                type: this.contentType,
                nsfw: this.nsfw,
                status: this.status || undefined,
                query: this.query || undefined,
                limit: 16,
                offset: (this.page - 1) * 16,
                genre: this.genre || undefined,
                format: this.format || undefined,
                sort: this.sort || undefined,
                tracker: this.tracker,
            });

            if (token !== this.searchToken) return; // a newer search has since started, drop this

            const normalized = res.data.map(item =>
                normalizeTrackerMedia(item, this.tracker)
            );

            const unique = Array.from(
                new Map(normalized.map(item => [item.cid, item])).values()
            );

            console.log(unique)

            if (this.page === 1) {
                this.results = unique;
            } else {
                const existingIds = new Set(this.results.map(i => i.cid));

                this.results = [
                    ...this.results,
                    ...unique.filter(i => !existingIds.has(i.cid))
                ];
            }

            this.hasSearched = true;
        } catch (e) {
            if (token !== this.searchToken) return;
            this.error = e as CoreError;
            console.error(e);
        } finally {
            if (token === this.searchToken) this.isLoading = false;
        }
    }

    async extensionSearch() {
        const token = ++this.searchToken;
        this.isLoading = true;
        this.error = null;

        if (this.page === 1) this.results = [];

        try {
            const cleanedFilters = Object.fromEntries(
                Object.entries(this.extFilterValues).filter(([_, v]) => {
                    if (Array.isArray(v)) return v.length > 0;
                    if (typeof v === "string") return v.trim() !== "";
                    return v !== null && v !== undefined && v !== false;
                })
            );

            const res = await contentApi.searchExtension(
                this.selectedExtension,
                this.query,
                cleanedFilters,
                this.page
            );

            if (token !== this.searchToken) return; // a newer search has since started, drop this

            const normalized = res.map(item =>
                normalizeExtensionResult(item, this.selectedExtension, this.contentType)
            );

            let imageHeaders: Record<string, string> | undefined;
            const selectedExt = extensions.installed.find(e => e.id === this.selectedExtension);
            if (selectedExt?.source === "tachiyomi" && normalized.length > 0) {
                try {
                    imageHeaders = await extensionsApi.getImageRequestHeaders(
                        this.selectedExtension,
                        normalized[0].cover
                    );
                    if (token !== this.searchToken) return;
                } catch (e) {
                    console.warn("Could not fetch image headers", e);
                }
            }

            const withHeaders = imageHeaders
                ? normalized.map(card => ({ ...card, imageHeaders }))
                : normalized;

            if (this.page === 1) {
                this.results = withHeaders;
            } else {
                const existingIds = new Set(this.results.map(i => i.cid));
                this.results = [...this.results, ...withHeaders.filter(i => !existingIds.has(i.cid))];
            }

            this.hasSearched = true;
        } catch (e) {
            if (token !== this.searchToken) return;
            this.error = e as CoreError;
            console.error(e);
        } finally {
            if (token === this.searchToken) this.isLoading = false;
        }
    }
}

export const searchState = new SearchState();