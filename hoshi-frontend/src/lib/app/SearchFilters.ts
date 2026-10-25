export const MANGA_FORMATS = ["MANGA", "NOVEL", "LIGHT_NOVEL", "ONE_SHOT", "DOUJIN", "MANHWA", "MANHUA", "manga", "novel", "light_novel", "one_shot", "doujin", "manhwa", "manhua"];
export const ANIME_FORMATS = ["TV", "TV_SHORT", "MOVIE", "SPECIAL", "OVA", "ONA", "MUSIC", "tv", "movie", "ova", "ona", "special", "music"];

export const formatLabel = (key: string) => key.replace(/_/g, ' ').replace(/\b\w/g, l => l.toUpperCase());

export const TRACKER_FILTERS = {
    anilist: {
        status: [
            { value: "FINISHED", label: "search.completed" },
            { value: "RELEASING", label: "search.ongoing" },
            { value: "NOT_YET_RELEASED", label: "search.planned" },
            { value: "CANCELLED", label: "search.cancelled" },
            { value: "HIATUS", label: "search.hiatus" },
        ],
        formats: [
            { value: "TV", label: "search.tv" },
            { value: "TV_SHORT", label: "search.tv_short" },
            { value: "MOVIE", label: "search.movie" },
            { value: "SPECIAL", label: "search.special" },
            { value: "OVA", label: "search.ova" },
            { value: "ONA", label: "search.ona" },
            { value: "MANGA", label: "search.manga" },
            { value: "NOVEL", label: "search.novel" },
            { value: "ONE_SHOT", label: "search.one_shot" },
        ],
        genres: [
            { value: "Action", label: "search.action" },
            { value: "Adventure", label: "search.adventure" },
            { value: "Comedy", label: "search.comedy" },
            { value: "Drama", label: "search.drama" },
            { value: "Ecchi", label: "search.ecchi" },
            { value: "Fantasy", label: "search.fantasy" },
            { value: "Horror", label: "search.horror" },
            { value: "Mahou Shoujo", label: "search.mahou_shoujo" },
            { value: "Mecha", label: "search.mecha" },
            { value: "Music", label: "search.music" },
            { value: "Mystery", label: "search.mystery" },
            { value: "Psychological", label: "search.psychological" },
            { value: "Romance", label: "search.romance" },
            { value: "Sci-Fi", label: "search.sci_fi" },
            { value: "Slice of Life", label: "search.slice_of_life" },
            { value: "Sports", label: "search.sports" },
            { value: "Supernatural", label: "search.supernatural" },
            { value: "Thriller", label: "search.thriller" },
        ],
    },
    mal: {
        status: [
            { value: "airing", label: "search.ongoing" },
            { value: "complete", label: "search.completed" },
            { value: "upcoming", label: "search.planned" },
        ],
        formats: [
            { value: "tv", label: "search.tv" },
            { value: "movie", label: "search.movie" },
            { value: "ova", label: "search.ova" },
            { value: "ona", label: "search.ona" },
            { value: "special", label: "search.special" },
            { value: "music", label: "search.music" },
            { value: "manga", label: "search.manga" },
            { value: "novel", label: "search.novel" },
            { value: "light_novel", label: "search.light_novel" },
            { value: "one_shot", label: "search.one_shot" },
            { value: "doujin", label: "search.doujin" },
            { value: "manhwa", label: "search.manhwa" },
            { value: "manhua", label: "search.manhua" },
        ],
        genres: [
            // Numeric MAL genre/theme ids; the backend passes these straight to Tenrai.
            { value: "1", label: "search.action" },
            { value: "2", label: "search.adventure" },
            { value: "4", label: "search.comedy" },
            { value: "8", label: "search.drama" },
            { value: "9", label: "search.ecchi" },
            { value: "10", label: "search.fantasy" },
            { value: "14", label: "search.horror" },
            { value: "66", label: "search.mahou_shoujo" },
            { value: "18", label: "search.mecha" },
            { value: "19", label: "search.music" },
            { value: "7", label: "search.mystery" },
            { value: "40", label: "search.psychological" },
            { value: "22", label: "search.romance" },
            { value: "24", label: "search.sci_fi" },
            { value: "36", label: "search.slice_of_life" },
            { value: "30", label: "search.sports" },
            { value: "37", label: "search.supernatural" },
            { value: "45", label: "search.thriller" },
        ],
    },
    kitsu: {
        status: [
            { value: "current", label: "search.ongoing" },
            { value: "finished", label: "search.completed" },
            { value: "upcoming", label: "search.planned" },
        ],
        formats: [
            { value: "TV", label: "search.tv" },
            { value: "movie", label: "search.movie" },
            { value: "OVA", label: "search.ova" },
            { value: "manga", label: "search.manga" },
            { value: "novel", label: "search.novel" },
        ],
        genres: [
            { value: "action", label: "search.action" },
            { value: "adventure", label: "search.adventure" },
            { value: "comedy", label: "search.comedy" },
        ],
    }
};