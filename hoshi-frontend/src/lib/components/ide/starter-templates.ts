// @/components/ide/starter-templates.ts

export const STARTER_TEMPLATES: Record<'anime' | 'manga' | 'novel', string> = {
    anime: `class MyAnime extends Anime {
    api = "https://example.com";

    async getFilters() {
        return {};
    }

    getStreamingSettings() {
        return {
            episodeServers: ["HLS"],
            supportsDub: false
        };
    }

    async search(query, filters, page) {
        return [];
    }

    async getMetadata(id) {
        return {
            title: "",
            synopsis: null,
            image: null,
            eps_or_chapters: null,
            rating: null,
            year: null,
            genres: [],
            nsfw: false,
            anilist_id: null,
            mal_id: null,
            external_ids: {}
        };
    }

    async findEpisodes(contentId) {
        return [];
    }

    async findEpisodeServer(episodeId, server, category = "sub") {
        return {
            headers: {},
            source: {
                url: "",
                subtitles: [],
                chapters: []
            }
        };
    }
}`,

    manga: `class MyManga extends Manga {
    async getFilters() {
        return {};
    }

    async search(query, filters, page) {
        return [];
    }

    async getMetadata(id) {
        return {
            title: "",
            synopsis: null,
            image: null,
            eps_or_chapters: null,
            rating: null,
            year: null,
            genres: [],
            nsfw: false,
            anilist_id: null,
            mal_id: null,
            external_ids: {}
        };
    }

    async findChapters(contentId) {
        return [];
    }

    async findChapterPages(chapterId) {
        return [];
    }
}`,

    novel: `class MyNovel extends Novel {
    async getFilters() {
        return {};
    }

    async search(query, filters, page) {
        return [];
    }

    async getMetadata(id) {
        return {
            title: "",
            synopsis: null,
            image: null,
            eps_or_chapters: null,
            rating: null,
            year: null,
            genres: [],
            nsfw: false,
            anilist_id: null,
            mal_id: null,
            external_ids: {}
        };
    }

    async findChapters(contentId) {
        return [];
    }

    async findChapterPages(chapterId) {
        return "";
    }
}`,
};