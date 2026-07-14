globalThis.SManga = class SManga {
    static UNKNOWN            = 0;
    static ONGOING            = 1;
    static COMPLETED          = 2;
    static LICENSED           = 3;
    static PUBLISHING_FINISHED = 4;
    static CANCELLED          = 5;
    static ON_HIATUS          = 6;

    static create() { return new SManga(); }

    constructor() {
        this.url             = "";
        this.title           = "";
        this.artist          = null;
        this.author          = null;
        this.description     = null;
        this.genre           = null;
        this.status          = 0;
        this.thumbnail_url   = null;
        this.update_strategy = UpdateStrategy.ALWAYS_UPDATE;
        this.initialized     = false;
    }

    // setters (used by translated bytecode)
    setUrl(v)           { this.url           = v; }
    setTitle(v)         { this.title         = v; }
    setArtist(v)        { this.artist        = v; }
    setAuthor(v)        { this.author        = v; }
    setDescription(v)   { this.description   = v; }
    setGenre(v)         { this.genre         = v; }
    setStatus(v)        { this.status        = v; }
    setThumbnail_url(v) { this.thumbnail_url = v; }
    setInitialized(v)   { this.initialized   = v; }

    // getters (Kotlin property accessors compile to these)
    getUrl()           { return this.url; }
    getTitle()         { return this.title; }
    getArtist()        { return this.artist; }
    getAuthor()        { return this.author; }
    getDescription()   { return this.description; }
    getGenre()         { return this.genre; }
    getStatus()        { return this.status; }
    getThumbnail_url() { return this.thumbnail_url; }
    getInitialized()   { return this.initialized; }

    setUpdate_strategy(strat) {this.update_strategy = strat; }

    getGenres() {
        if (!this.genre) return null;
        return [...new Set(
            this.genre.split(", ").map(s => s.trim()).filter(s => s.length > 0)
        )];
    }

    copy() {
        const c = SManga.create();
        c.url             = this.url;
        c.title           = this.title;
        c.artist          = this.artist;
        c.author          = this.author;
        c.description     = this.description;
        c.genre           = this.genre;
        c.status          = this.status;
        c.thumbnail_url   = this.thumbnail_url;
        c.update_strategy = this.update_strategy;
        c.initialized     = this.initialized;
        return c;
    }
};

SManga.Companion = {
    UNKNOWN:             0,
    ONGOING:             1,
    COMPLETED:           2,
    LICENSED:            3,
    PUBLISHING_FINISHED: 4,
    CANCELLED:           5,
    ON_HIATUS:           6,
    create() { return new SManga(); },
};

globalThis.SChapter = class SChapter {
    static create() { return new SChapter(); }

    constructor() {
        this.url            = "";
        this.name           = "";
        this.date_upload    = 0;
        this.chapter_number = -1;
        this.scanlator      = null;
    }

    // setters
    setUrl(v)            { this.url            = v; }
    setName(v)           { this.name           = v; }
    setDate_upload(v)    { this.date_upload    = v; }
    setChapter_number(v) { this.chapter_number = v; }
    setScanlator(v)      { this.scanlator      = v; }

    // getters
    getUrl()            { return this.url; }
    getName()           { return this.name; }
    getDate_upload()    { return this.date_upload; }
    getChapter_number() { return this.chapter_number; }
    getScanlator()      { return this.scanlator; }

    copyFrom(other) {
        this.name           = other.name;
        this.url            = other.url;
        this.date_upload    = other.date_upload;
        this.chapter_number = other.chapter_number;
        this.scanlator      = other.scanlator;
    }
};

SChapter.Companion = {
    create() { return new SChapter(); }
};

globalThis.Page = class Page {
    static State = {
        Queue:         "Queue",
        LoadPage:      "LoadPage",
        DownloadImage: "DownloadImage",
        Ready:         "Ready",
        Error:         (err) => ({ type: "Error", error: err }),
    };

    constructor(index, url = "", imageUrl = null) {
        this.index    = index;
        this.url      = url;
        this.imageUrl = imageUrl;
        this.status   = Page.State.Queue;
        this.progress = 0;
    }

    get number() { return this.index + 1; }

    // setters
    setImageUrl(v) { this.imageUrl = v; }
    getImageUrl()  { return this.imageUrl; }
    getUrl()       { return this.url; }
    getIndex()     { return this.index; }
};

globalThis.MangasPage = class MangasPage {
    constructor(mangas, hasNextPage) {
        this.mangas      = mangas;
        this.hasNextPage = hasNextPage;
    }
};

// UpdateStrategy enum
globalThis.UpdateStrategy = {
    ALWAYS_UPDATE: { ordinal: 0, name: "ALWAYS_UPDATE" },
    ONLY_FETCH_ONCE: { ordinal: 1, name: "ONLY_FETCH_ONCE" },
};
// Filter base & subclasses




// FilterList

globalThis.FilterList = class FilterList extends Array {
    constructor(...args) {
        super();
        const items = args.length === 1 && Array.isArray(args[0]) ? args[0] : args;
        this.push(...items.filter(x => x != null));
        this.list = this;
    }

    iterator() {
        let i = 0;
        const arr = this;
        return {
            hasNext: () => i < arr.length ? true : 0,
            next: () => {
                if (i >= arr.length) {
                    throw new Error("NoSuchElementException");
                }
                return arr[i++];
            }
        };
    }

    isEmpty() {
        return this.length === 0 ? 1 : 0;
    }

    getList() { return [...this]; }

    equals() { return false; }
};

// Response wrapper

//  OkHttp shims



// CacheControl


// Request


globalThis.InjektKt = class InjektKt {
    static getInjekt() {
        return {
            getInstance: (cls) => {
                return globalThis.__universalInstance ||= {
                    // Json
                    decodeFromString(deserializer, str) {
                        const parsed = JSON.parse(str);
                        const data = parsed?.data ?? parsed;
                        const decoder = new JsonDecoder(data, null);
                        return deserializer.deserialize(decoder);
                    },
                    encodeToString(serializer, obj) {
                        return JSON.stringify(deepSerialize(obj));
                    },

                    // Application / SharedPreferences
                    getSharedPreferences(name, mode) { return new SharedPreferences(name); },

                    // OkHttpClient
                    newCall(request) { return new OkHttpCall(request); },

                    getClient() {
                        return _networkHelper.client;
                    },
                };
            }
        };
    }
};

// RequestsKt
globalThis.__lastExtractedUrl = "";



// HttpUrl


globalThis.AppInfo = {
    INSTANCE: {
        // Tachiyomi extensions call these to read app metadata
        getVersionName()  { return "1.0.0"; },
        getVersionCode()  { return 1; },
        getPackageName()  { return "eu.kanade.tachiyomi"; },
        getApplicationId(){ return "eu.kanade.tachiyomi"; },

        // Kotlin object — make it look like a companion/singleton
        versionName:  "1.0.0",
        versionCode:  1,
        packageName:  "eu.kanade.tachiyomi",

        toString() { return "AppInfo"; },
    }
};




globalThis.Source = class Source {
    get id() { return 0n; }
    get name() { return ""; }
    get lang() { return ""; }

    async getMangaDetails(manga) { return manga; }
    async getChapterList(manga) { return []; }
    async getPageList(chapter) { return []; }

    // Deprecated RxJava stubs — translated code may call these
    fetchMangaDetails(manga) { throw new Error("Not used"); }
    fetchChapterList(manga) { throw new Error("Not used"); }
    fetchPageList(chapter) { throw new Error("Not used"); }
};

// eu.kanade.tachiyomi.source.SourceFactory
globalThis.SourceFactory = class SourceFactory {
    createSources() { return []; }
};


//  HttpSource

globalThis._SandboxManga = Manga;
class HttpSource extends _SandboxManga {

    getClass() {
        return {
            getClassLoader: () => ({
                getResourceAsStream: (path) => null,
            }),
            getName: () => this.constructor?.name ?? "Object",
            getSimpleName: () => this.constructor?.name ?? "Object",
        };
    }

    get lang()      { return "en"; }
    get name()      { return "Unknown"; }
    get versionId() { return 1; }

    headersBuilder() {
        return new Headers.Builder()
            .add("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/114.0.0.0 Safari/537.36")
    }

    getId() {
        if (this.__id == null) {
            this.__id = this.generateId(
                this.name,
                this.lang,
                this.versionId
            );
        }

        return this.__id;
    }

    generateId(name, lang, versionId) {
        const key = `${name.toLowerCase()}/${lang}/${versionId}`;

        const md5hex = crypto.md5(key);
        const first64 = md5hex.slice(0, 16);

        let id = BigInt("0x" + first64);
        id &= 0x7fffffffffffffffn;
        return id.toString();
    }

    async getImageHeaders() {
        const h = this.headers?.toFetchHeaders?.() ?? this.headers ?? {};
        return h;
    }

    __getTachiyomiSettings() {
        if (typeof this.setupPreferenceScreen !== "function") {
            return [];
        }

        const screen = new PreferenceScreen();
        try {
            this.setupPreferenceScreen(screen);
        } catch (e) {
            const errorDetails = e instanceof Error ? `${e.message}\n${e.stack}` : String(e);
            console.error("[tachiyomi] setupPreferenceScreen threw: " + errorDetails);
        }

        return screen._prefs
            .filter(p => typeof p._toManifest === "function" && p._key)
            .map(p => p._toManifest());
    }

    getUrlWithoutDomain(orig) {
        if (!orig) return "";
        try {
            const safeUrl = orig.replace(/ /g, "%20");

            const parsed = safeUrl.includes("://") || safeUrl.startsWith("//")
                ? new URL(safeUrl)
                : new URL(safeUrl, "http://localhost");

            let out = parsed.pathname;
            if (parsed.search) {
                out += parsed.search;
            }
            if (parsed.hash) {
                out += parsed.hash;
            }
            return out;
        } catch (e) {
            return orig;
        }
    }

    setUrlWithoutDomain(obj, url) {
        if (obj && typeof obj.setUrl === "function") {
            obj.setUrl(this.getUrlWithoutDomain(url));
        }
    }

    //  sandbox API entry points

    async getFilters() {
        const filterList = this.getFilterList();
        const result = {};

        for (const filter of filterList) {
            if (filter instanceof Filter.Header || filter instanceof Filter.Separator) continue;

            if (filter instanceof Filter.Select) {
                result[filter.name] = {
                    label: filter.name,
                    type: "select",
                    options: filter.values.map((v, i) => {
                        const label = typeof v === "string" ? v
                            : v?.first ?? v?.toString?.() ?? String(i);
                        return { label, value: String(i) };
                    }),
                };
            } else if (filter instanceof Filter.Text) {
                result[filter.name] = {
                    label: filter.name,
                    type: "text",
                };
            } else if (filter instanceof Filter.CheckBox) {
                result[filter.name] = {
                    label: filter.name,
                    type: "boolean",
                };
            } else if (filter instanceof Filter.TriState) {
                result[filter.name] = {
                    label: filter.name,
                    type: "select",
                    options: [
                        { label: "Any", value: "0" },
                        { label: "Include", value: "1" },
                        { label: "Exclude", value: "2" },
                    ],
                };
            } else if (filter instanceof Filter.Group) {
                const state = Array.isArray(filter.state) ? filter.state : [...(filter.state ?? [])];
                if (state.length > 0 && state[0] instanceof Filter.CheckBox) {
                    result[filter.name] = {
                        label: filter.name,
                        type: "multiselect",
                        options: state.map(f => ({
                            label: typeof f.name === "string" ? f.name : f.name?.first ?? String(f.name),
                            value: typeof f.name === "string" ? f.name : f.name?.first ?? String(f.name),
                        })),
                    };
                } else if (state.length > 0 && state[0] instanceof Filter.TriState) {
                    result[filter.name] = {
                        label: filter.name,
                        type: "multiselect",
                        options: state.map(f => ({ label: f.name, value: f.name })),
                    };
                }
            } else if (filter instanceof Filter.Sort) {
                result[filter.name] = {
                    label: filter.name,
                    type: "select",
                    options: filter.values.map((v, i) => ({
                        label: typeof v === "string" ? v : v?.first ?? String(v),
                        value: String(i),
                    })),
                };
            }
        }

        return result;
    }

    async search(query, filters, page) {
        page = page ?? 1;

        const filterList = this.getFilterList();
        const hasFilters = filters && typeof filters === "object" && Object.keys(filters).length > 0;

        if (hasFilters) {
            for (const filter of filterList) {
                const incoming = filters[filter.name];
                if (incoming === undefined || incoming === null) continue;

                if (filter instanceof Filter.Select || filter instanceof Filter.Sort) {
                    filter.state = parseInt(incoming, 10) || 0;
                } else if (filter instanceof Filter.Text) {
                    filter.state = incoming;
                } else if (filter instanceof Filter.CheckBox) {
                    filter.state = incoming === true || incoming === "true" || incoming === 1;
                } else if (filter instanceof Filter.TriState) {
                    filter.state = parseInt(incoming, 10) || 0;
                } else if (filter instanceof Filter.Group) {
                    const state = Array.isArray(filter.state) ? filter.state : [...(filter.state ?? [])];
                    const selectedValues = Array.isArray(incoming) ? incoming : [incoming];
                    for (const child of state) {
                        if (child instanceof Filter.CheckBox || child instanceof Filter.TriState) {
                            child.state = selectedValues.includes(child.name)
                                ? (child instanceof Filter.TriState ? 1 : true)
                                : (child instanceof Filter.TriState ? 0 : false);
                        }
                    }
                }
            }
        }

        if (!query && !hasFilters) {
            const res = await this._doRequest(this.popularMangaRequest(page));
            return this._parsePage(this.popularMangaParse(res));
        }

        const res = await this._doRequest(
            this.searchMangaRequest(page, query, filterList)
        );
        return this._parsePage(this.searchMangaParse(res));
    }

    async getMetadata(id) {
        const manga = SManga.create();
        manga.url   = id;
        const res   = await this._doRequest(this.mangaDetailsRequest(manga));
        const m     = this.mangaDetailsParse(res);
        return {
            title:           m.title,
            synopsis:        m.description   ?? null,
            image:           m.thumbnail_url ?? null,
            eps_or_chapters: null,
            rating:          null,
            year:            null,
            genres:          m.genre ? m.genre.split(", ") : [],
            nsfw:            m.status === 5,
            anilist_id:      null,
            mal_id:          null,
            external_ids:    {},
        };
    }

    async findChapters(contentId) {
        const manga = SManga.create();
        manga.url   = contentId;
        const res   = await this._doRequest(this.chapterListRequest(manga));
        const list  = this.chapterListParse(res);
        const arr   = Array.isArray(list) ? list : [...list];
        return arr.map((ch, i) => ({
            id:     ch.url      ?? ch.getUrl?.(),
            title:  ch.name     ?? ch.getName?.(),
            number: (ch.chapter_number ?? ch.getChapter_number?.() ?? -1) >= 0
                ? (ch.chapter_number ?? ch.getChapter_number?.())
                : arr.length - i,
            index:  i,
        }));
    }

    async findChapterPages(chapterId) {
        const ch  = SChapter.create();
        ch.url    = chapterId;
        const res = await this._doRequest(this.pageListRequest(ch));
        return this.pageListParse(res).map(p => ({ url: p.imageUrl ?? p.url }));
    }

    //  default Tachiyomi method stubs
    // subclasses override these

    popularMangaRequest(page)                        { throw new Error("not implemented"); }
    popularMangaParse(response)                      { throw new Error("not implemented"); }
    searchMangaRequest(page, query, filters)         { throw new Error("not implemented"); }
    searchMangaParse(response)                       { throw new Error("not implemented"); }
    latestUpdatesRequest(page)                       { throw new Error("not implemented"); }
    latestUpdatesParse(response)                     { throw new Error("not implemented"); }
    mangaDetailsRequest(manga)                       { return GET(this.getBaseUrl() + manga.url, this.headers); }
    mangaDetailsParse(response)                      { throw new Error("not implemented"); }
    chapterListRequest(manga)                        { return GET(this.getBaseUrl() + manga.url, this.headers); }
    chapterListParse(response)                       { throw new Error("not implemented"); }
    pageListRequest(chapter)                         { return GET(this.getBaseUrl() + chapter.url, this.headers); }
    pageListParse(response)                          { throw new Error("not implemented"); }
    imageUrlParse(response)                          { throw new Error("not implemented"); }
    getFilterList()                                  { return new FilterList(); }

    imageRequest(page) {
        return GET(page.imageUrl, this.headers);
    }

    async getImageRequestHeaders(imageUrl, chapterUrl = null) {
        const pageUrl =
            chapterUrl != null && chapterUrl !== ""
                ? chapterUrl
                : imageUrl;

        const page = new Page(0, pageUrl, imageUrl);

        const req = this.imageRequest(page);
        const r = req instanceof _Call ? req._req : req;

        const h = new Headers(
            r.headers?.toFetchHeaders?.() ??
            r.headers ??
            {}
        );

        if (!h.has("referer")) {
            h.set("referer", this.getBaseUrl() + "/");
        }

        return h.toFetchHeaders();
    }

    _parsePage(page) {
        return page.mangas.map(m => ({
            id:    m.url,
            title: m.title,
            image: m.thumbnail_url ?? null,
            url:   m.url,
            nsfw:  false,
        }));
    }

    get client() {
        return _networkHelper.client;
    }

    getClient() {
        return _networkHelper.client;
    }
}

let __tachi_captured = null;

globalThis.HttpSource       = HttpSource;
globalThis.ParsedHttpSource = HttpSource;
globalThis.Manga            = HttpSource;

_networkHelper.getCloudflareClient = function() { return _makeOkHttpClient(true); };
_networkHelper.getClient            = function() { return _makeOkHttpClient(true); };
_networkHelper.getNonCloudflareClient = function() { return _makeOkHttpClient(false); };


// ─── Aniyomi models ──────────────────────────────────────────────────────────

globalThis.SAnime = class SAnime {
    static UNKNOWN             = 0;
    static ONGOING             = 1;
    static COMPLETED           = 2;
    static LICENSED            = 3;
    static PUBLISHING_FINISHED = 4;
    static CANCELLED           = 5;
    static ON_HIATUS           = 6;

    static create() { return new SAnime(); }

    constructor() {
        this.url             = "";
        this.title           = "";
        this.artist          = null;
        this.author          = null;
        this.description     = null;
        this.genre           = null;
        this.status          = 0;
        this.thumbnail_url   = null;
        this.background_url  = null;
        this.season_number   = 0;
        this.fetch_type      = 0;
        this.update_strategy = 0;
        this.initialized     = false;
    }

    // setters
    setUrl(v)            { this.url            = v; }
    setTitle(v)          { this.title          = v; }
    setArtist(v)         { this.artist         = v; }
    setAuthor(v)         { this.author         = v; }
    setDescription(v)    { this.description    = v; }
    setGenre(v)          { this.genre          = v; }
    setStatus(v)         { this.status         = v; }
    setThumbnail_url(v)  { this.thumbnail_url  = v; }
    setBackground_url(v) { this.background_url = v; }
    setSeason_number(v)  { this.season_number  = v; }
    setInitialized(v)    { this.initialized    = v; }

    // getters
    getUrl()            { return this.url; }
    getTitle()          { return this.title; }
    getArtist()         { return this.artist; }
    getAuthor()         { return this.author; }
    getDescription()    { return this.description; }
    getGenre()          { return this.genre; }
    getStatus()         { return this.status; }
    getThumbnail_url()  { return this.thumbnail_url; }
    getBackground_url() { return this.background_url; }
    getSeason_number()  { return this.season_number; }
    getInitialized()    { return this.initialized; }

    getGenres() {
        if (!this.genre) return null;
        return [...new Set(
            this.genre.split(", ").map(s => s.trim()).filter(s => s.length > 0)
        )];
    }

    copy() {
        const c = SAnime.create();
        c.url             = this.url;
        c.title           = this.title;
        c.artist          = this.artist;
        c.author          = this.author;
        c.description     = this.description;
        c.genre           = this.genre;
        c.status          = this.status;
        c.thumbnail_url   = this.thumbnail_url;
        c.background_url  = this.background_url;
        c.season_number   = this.season_number;
        c.initialized     = this.initialized;
        return c;
    }
};

SAnime.Companion = {
    UNKNOWN:             0,
    ONGOING:             1,
    COMPLETED:           2,
    LICENSED:            3,
    PUBLISHING_FINISHED: 4,
    CANCELLED:           5,
    ON_HIATUS:           6,
    create() { return new SAnime(); },
};

globalThis.SEpisode = class SEpisode {
    static create() { return new SEpisode(); }

    constructor() {
        this.url            = "";
        this.name           = "";
        this.date_upload    = 0;
        this.episode_number = -1;
        this.scanlator      = null;
        this.summary        = null;
        this.preview_url    = null;
        this.fillermark     = false;
    }

    // setters
    setUrl(v)            { this.url            = v; }
    setName(v)           { this.name           = v; }
    setDate_upload(v)    { this.date_upload    = v; }
    setEpisode_number(v) { this.episode_number = v; }
    setScanlator(v)      { this.scanlator      = v; }
    setSummary(v)        { this.summary        = v; }
    setPreview_url(v)    { this.preview_url    = v; }
    setFillermark(v)     { this.fillermark     = v; }

    // getters
    getUrl()            { return this.url; }
    getName()           { return this.name; }
    getDate_upload()    { return this.date_upload; }
    getEpisode_number() { return this.episode_number; }
    getScanlator()      { return this.scanlator; }
    getSummary()        { return this.summary; }
    getPreview_url()    { return this.preview_url; }
    getFillermark()     { return this.fillermark; }
};

SEpisode.Companion = {
    create() { return new SEpisode(); },
};

// Track — used inside Video for subtitles and audio tracks
globalThis.Track = class Track {
    constructor(url, lang) {
        this.url  = url  ?? "";
        this.lang = lang ?? "";
    }
};

// TimeStamp — used inside Video
globalThis.TimeStamp = class TimeStamp {
    constructor(startMs, endMs, title) {
        this.startMs = startMs ?? 0;
        this.endMs   = endMs   ?? 0;
        this.title   = title   ?? "";
    }
};

// Video — the actual stream object extensions produce
globalThis.Video = class Video {
    // Legacy 3-arg constructor: Video(url, quality, videoUrl, headers?, subtitleTracks?, audioTracks?)
    // Modern constructor:       Video(videoUrl, videoTitle, resolution?, bitrate?, headers?, ...)
    constructor(
        videoUrlOrUrl,
        videoTitleOrQuality,
        videoUrlOrResolution,
        headersOrBitrate,
        subtitleTracksOrHeaders,
        audioTracksOrSubtitles,
        timestamps,
        mpvArgs,
        ffmpegStreamArgs,
        ffmpegVideoArgs,
        internalData,
        initialized,
    ) {
        // Detect legacy constructor: 3rd arg is a string (videoUrl) or null
        if (typeof videoUrlOrResolution === "string" || videoUrlOrResolution === null) {
            // legacy: (url, quality, videoUrl, headers?, subtitleTracks?, audioTracks?)
            this.videoUrl       = videoUrlOrResolution ?? videoUrlOrUrl ?? "";
            this.videoTitle     = videoTitleOrQuality  ?? "";
            this.resolution     = null;
            this.bitrate        = null;
            this.headers        = headersOrBitrate         ?? null;
            this.subtitleTracks = subtitleTracksOrHeaders  ?? [];
            this.audioTracks    = audioTracksOrSubtitles   ?? [];
            this.timestamps     = [];
            this.mpvArgs        = [];
            this.ffmpegStreamArgs = [];
            this.ffmpegVideoArgs  = [];
            this.internalData   = "";
            this.initialized    = false;
            // legacy `url` field (some extensions read it back)
            this.url            = videoUrlOrUrl ?? "";
            this.quality        = videoTitleOrQuality ?? "";
        } else {
            // modern: (videoUrl, videoTitle, resolution?, bitrate?, headers?, ...)
            this.videoUrl         = videoUrlOrUrl          ?? "";
            this.videoTitle       = videoTitleOrQuality    ?? "";
            this.resolution       = videoUrlOrResolution   ?? null;
            this.bitrate          = headersOrBitrate       ?? null;
            this.headers          = subtitleTracksOrHeaders ?? null;
            this.preferred        = false;
            this.subtitleTracks   = audioTracksOrSubtitles ?? [];
            this.audioTracks      = timestamps             ?? [];
            this.timestamps       = mpvArgs                ?? [];
            this.mpvArgs          = ffmpegStreamArgs       ?? [];
            this.ffmpegStreamArgs = ffmpegVideoArgs        ?? [];
            this.ffmpegVideoArgs  = internalData           ?? [];
            this.internalData     = initialized            ?? "";
            this.initialized      = false;
            this.url              = this.videoUrl;
            this.quality          = this.videoTitle;
        }
    }

    getVideoUrl()   { return this.videoUrl; }
    getVideoTitle() { return this.videoTitle; }
    getHeaders()    { return this.headers; }
    getSubtitleTracks() { return this.subtitleTracks; }
    getAudioTracks()    { return this.audioTracks; }
    getTimestamps()     { return this.timestamps; }
};

// Hoster — intermediate object between episode and video list
globalThis.Hoster = class Hoster {
    constructor(hosterUrl = "", hosterName = "", videoList = null, internalData = "", lazy = false) {
        this.hosterUrl    = hosterUrl;
        this.hosterName   = hosterName;
        this.videoList    = videoList;
        this.internalData = internalData;
        this.lazy         = lazy;
    }

    getHosterUrl()    { return this.hosterUrl; }
    getUrl()    { return this.hosterUrl; }
    getHosterName()   { return this.hosterName; }
    getVideoList()    { return this.videoList; }
    getInternalData() { return this.internalData; }
    getLazy()         { return this.lazy; }
};

Hoster.Companion = {
    NO_HOSTER_LIST: "__NO_HOSTER_LIST__",
    // List<Video>.toHosterList() — wraps each video in its own Hoster
    toHosterList(videos) {
        return videos.map(v => new Hoster(
            v.videoUrl ?? "",
            v.videoTitle ?? v.quality ?? "",
            [v],
        ));
    },
};

// Also expose as extension method on Array (compiled Kotlin extension calls)


// ─── AnimeHttpSource ──────────────────────────────────────────────────────────

globalThis._SandboxAnime = _SandboxManga;

class AnimeHttpSource extends _SandboxManga {

    getClass() {
        return {
            getClassLoader: () => ({ getResourceAsStream: () => null }),
            getName:        () => this.constructor?.name ?? "Object",
            getSimpleName:  () => this.constructor?.name ?? "Object",
        };
    }

    get lang()      { return "en"; }
    get name()      { return "Unknown"; }
    get versionId() { return 1; }

    headersBuilder() {
        return new Headers.Builder()
            .add("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/114.0.0.0 Safari/537.36");
    }

    getId() {
        if (this.__id == null) {
            this.__id = this.generateId(this.name, this.lang, this.versionId);
        }
        return this.__id;
    }

    generateId(name, lang, versionId) {
        const key    = `${name.toLowerCase()}/${lang}/${versionId}`;
        const md5hex = crypto.md5(key);
        let id = BigInt("0x" + md5hex.slice(0, 16));
        id &= 0x7fffffffffffffffn;
        return id.toString();
    }

    // ── Tachiyomi preference discovery (same as manga) ────────────────────────

    __getTachiyomiSettings() {
        if (typeof this.setupPreferenceScreen !== "function") return [];
        const screen = new PreferenceScreen();
        try {
            this.setupPreferenceScreen(screen);
        } catch (e) {
            console.error("[aniyomi] setupPreferenceScreen threw: " + e);
        }
        return screen._prefs
            .filter(p => typeof p._toManifest === "function" && p._key)
            .map(p => p._toManifest());
    }

    // ── URL helpers ───────────────────────────────────────────────────────────

    getUrlWithoutDomain(orig) {
        if (!orig) return "";
        try {
            const safeUrl = orig.replace(/ /g, "%20");
            const parsed  = safeUrl.includes("://") || safeUrl.startsWith("//")
                ? new URL(safeUrl)
                : new URL(safeUrl, "http://localhost");
            let out = parsed.pathname;
            if (parsed.search) out += parsed.search;
            if (parsed.hash)   out += parsed.hash;
            return out;
        } catch (e) {
            return orig;
        }
    }

    setUrlWithoutDomain(obj, url) {
        if (obj && typeof obj.setUrl === "function") {
            obj.setUrl(this.getUrlWithoutDomain(url));
        }
    }

    // ── Network (same _doRequest as HttpSource) ───────────────────────────────

    get client() { return _networkHelper.client; }
    getClient()  { return _networkHelper.client; }
    getNetwork() { return _networkHelper; }

    // ── Sandbox API entry points ──────────────────────────────────────────────

    // getFilters() — identical filter mapping to manga
    async getFilters() {
        const filterList = this.getFilterList();
        const result = {};

        for (const filter of filterList) {
            if (filter instanceof Filter.Header || filter instanceof Filter.Separator) continue;

            if (filter instanceof Filter.Select) {
                result[filter.name] = {
                    label: filter.name,
                    type: "select",
                    options: filter.values.map((v, i) => {
                        const label = typeof v === "string" ? v : v?.first ?? v?.toString?.() ?? String(i);
                        return { label, value: String(i) };
                    }),
                };
            } else if (filter instanceof Filter.Text) {
                result[filter.name] = { label: filter.name, type: "text" };
            } else if (filter instanceof Filter.CheckBox) {
                result[filter.name] = { label: filter.name, type: "boolean" };
            } else if (filter instanceof Filter.TriState) {
                result[filter.name] = {
                    label: filter.name,
                    type: "select",
                    options: [
                        { label: "Any",     value: "0" },
                        { label: "Include", value: "1" },
                        { label: "Exclude", value: "2" },
                    ],
                };
            } else if (filter instanceof Filter.Group) {
                const state = Array.isArray(filter.state) ? filter.state : [...(filter.state ?? [])];
                if (state.length > 0 && (state[0] instanceof Filter.CheckBox || state[0] instanceof Filter.TriState)) {
                    result[filter.name] = {
                        label: filter.name,
                        type: "multiselect",
                        options: state.map(f => ({
                            label: typeof f.name === "string" ? f.name : f.name?.first ?? String(f.name),
                            value: typeof f.name === "string" ? f.name : f.name?.first ?? String(f.name),
                        })),
                    };
                }
            } else if (filter instanceof Filter.Sort) {
                result[filter.name] = {
                    label: filter.name,
                    type: "select",
                    options: filter.values.map((v, i) => ({
                        label: typeof v === "string" ? v : v?.first ?? String(v),
                        value: String(i),
                    })),
                };
            }
        }

        return result;
    }

    // search() — popular anime when no query/filters, search otherwise
    async search(query, filters, page) {
        page = page ?? 1;

        const filterList = this.getFilterList();
        const hasFilters = filters && typeof filters === "object" && Object.keys(filters).length > 0;

        if (hasFilters) {
            for (const filter of filterList) {
                const incoming = filters[filter.name];
                if (incoming === undefined || incoming === null) continue;

                if (filter instanceof Filter.Select || filter instanceof Filter.Sort) {
                    filter.state = parseInt(incoming, 10) || 0;
                } else if (filter instanceof Filter.Text) {
                    filter.state = incoming;
                } else if (filter instanceof Filter.CheckBox) {
                    filter.state = incoming === true || incoming === "true" || incoming === 1;
                } else if (filter instanceof Filter.TriState) {
                    filter.state = parseInt(incoming, 10) || 0;
                } else if (filter instanceof Filter.Group) {
                    const state = Array.isArray(filter.state) ? filter.state : [...(filter.state ?? [])];
                    const selectedValues = Array.isArray(incoming) ? incoming : [incoming];
                    for (const child of state) {
                        if (child instanceof Filter.CheckBox || child instanceof Filter.TriState) {
                            child.state = selectedValues.includes(child.name)
                                ? (child instanceof Filter.TriState ? 1 : true)
                                : (child instanceof Filter.TriState ? 0 : false);
                        }
                    }
                }
            }
        }

        if (!query && !hasFilters) {
            const res = await this._doRequest(this.popularAnimeRequest(page));
            return this._parseAnimePage(this.popularAnimeParse(res));
        }

        const res = await this._doRequest(this.searchAnimeRequest(page, query, filterList));
        return this._parseAnimePage(this.searchAnimeParse(res));
    }

    // getMetadata() — fetch anime details
    async getMetadata(id) {
        const anime = SAnime.create();
        anime.url   = id;
        const res   = await this._doRequest(this.animeDetailsRequest(anime));
        const a     = this.animeDetailsParse(res);
        console.log(a.description)
        return {
            title:           a.title,
            synopsis:        a.description   ?? null,
            image:           a.thumbnail_url ?? null,
            eps_or_chapters: null,
            rating:          null,
            year:            null,
            genres:          a.genre ? a.genre.split(", ").map(s => s.trim()).filter(Boolean) : [],
            nsfw:            false,
            anilist_id:      null,
            mal_id:          null,
            external_ids:    {},
        };
    }

    // findEpisodes() — episode list
    async findEpisodes(contentId) {
        const anime = SAnime.create();
        anime.url   = contentId;
        const res   = await this._doRequest(this.episodeListRequest(anime));
        const list  = this.episodeListParse(res);
        const arr   = Array.isArray(list) ? list : [...list];
        return arr.map((ep, i) => ({
            id:     ep.url              ?? ep.getUrl?.(),
            number: (ep.episode_number  ?? ep.getEpisode_number?.() ?? -1) >= 0
                ? (ep.episode_number    ?? ep.getEpisode_number?.())
                : arr.length - i,
            title:  ep.name             ?? ep.getName?.() ?? null,
            url:    ep.preview_url      ?? ep.getPreview_url?.() ?? null,
            image:  null,
        }));
    }

    // getStreamingSettings() — pass through to extension, with safe default
    async getStreamingSettings(episodeId) {
        const ep = SEpisode.create();
        ep.url = episodeId;

        let hosters = [];

        try {
            hosters = await this.getHosterList(ep);
        } catch (e) {
            if (e.message?.includes("not implemented")) {
                hosters = [
                    new Hoster("", "default", null, episodeId, false)
                ];
            } else {
                throw e;
            }
        }

        const hosterArr = Array.isArray(hosters) ? hosters : [...hosters];

        return {
            episodeServers: hosterArr.map(h => h.hosterName || "default"),
            supportsDub: !!this.supportsDub,
        };
    }

    // findEpisodeServer() — hoster→video two-step collapsed into one call
    async findEpisodeServer(episodeId, server, category = "sub") {
        const ep  = SEpisode.create();
        ep.url    = episodeId;

        // Step 1: hosters
        const hosters     = await this.getHosterList(ep);
        const hosterArr   = Array.isArray(hosters) ? hosters : [...hosters];
        if (hosterArr.length === 0) throw new Error("[aniyomi] no hosters returned");

        // Pick by server name if specified, otherwise first
        const hoster = (server && server !== "")
            ? (hosterArr.find(h => (h.hosterName ?? h.name ?? "").toLowerCase() === server.toLowerCase()) ?? hosterArr[0])
            : hosterArr[0];

        // Step 2: videos — may already be on hoster.videoList (non-lazy)
        let videos;
        if (hoster.videoList && hoster.videoList.length > 0) {
            videos = hoster.videoList;
        } else {
            const vList = await this.getVideoList(hoster);
            videos = Array.isArray(vList) ? vList : [...vList];
        }

        if (videos.length === 0) throw new Error("[aniyomi] no videos returned from hoster");

        // For dub/sub: extensions that supportsDub tend to put "dub"/"sub" in videoTitle
        const isDub = category === "dub";
        const video = (isDub
                ? videos.find(v => (v.videoTitle ?? v.quality ?? "").toLowerCase().includes("dub"))
                : videos.find(v => !(v.videoTitle ?? v.quality ?? "").toLowerCase().includes("dub"))
        ) ?? videos[0];

        // Normalise headers — OkHttp Headers object or plain object
        let headersObj = {};
        if (video.headers) {
            if (typeof video.headers.toFetchHeaders === "function") {
                headersObj = video.headers.toFetchHeaders();
            } else if (typeof video.headers === "object") {
                headersObj = { ...video.headers };
            }
        }

        const subtitles = (video.subtitleTracks ?? []).map((t, i) => ({
            id:         t.lang ?? String(i),
            url:        t.url  ?? "",
            language:   t.lang ?? "",
            is_default: i === 0,
        }));

        const chapters = (video.timestamps ?? []).map(ts => ({
            start: (ts.startMs ?? 0) / 1000,
            end:   (ts.endMs   ?? 0) / 1000,
            title: ts.title ?? "",
        }));

        return {
            headers: headersObj,
            source: {
                url:       video.videoUrl ?? video.url ?? "",
                subtitles,
                chapters,
            },
        };
    }

    _parseAnimePage(page) {
        // AnimesPage has .animes (list of SAnime) and .hasNextPage
        const list = page?.animes ?? page ?? [];
        const arr  = Array.isArray(list) ? list : [...list];
        return arr.map(a => ({
            id:    a.url           ?? a.getUrl?.(),
            title: a.title         ?? a.getTitle?.(),
            image: a.thumbnail_url ?? a.getThumbnail_url?.() ?? null,
            url:   a.url           ?? null,
            nsfw:  false,
        }));
    }

    // ── Default stubs (subclasses override) ───────────────────────────────────

    popularAnimeRequest(page)                        { throw new Error("not implemented"); }
    popularAnimeParse(response)                      { throw new Error("not implemented"); }
    searchAnimeRequest(page, query, filters)         { throw new Error("not implemented"); }
    searchAnimeParse(response)                       { throw new Error("not implemented"); }
    latestUpdatesRequest(page)                       { throw new Error("not implemented"); }
    latestUpdatesParse(response)                     { throw new Error("not implemented"); }
    animeDetailsRequest(anime)                       { return GET(this.getBaseUrl() + anime.url, this.headers); }
    animeDetailsParse(response)                      { throw new Error("not implemented"); }
    episodeListRequest(anime)                        { return GET(this.getBaseUrl() + anime.url, this.headers); }
    episodeListParse(response)                       { throw new Error("not implemented"); }
    hosterListParse(response)                        { throw new Error("not implemented"); }
    videoListParse(response, hoster)                 { throw new Error("not implemented"); }
    getFilterList()                                  { return new FilterList(); }

    // getHosterList / getVideoList — extensions override these or the *Parse variants
    async getHosterList(episode) {
        try {
            const req = this.hosterListRequest(episode);
            const res = await this._doRequest(req);
            return this.hosterListParse(res);
        } catch (e) {
            if (e.message?.includes("not implemented")) {
                return [
                    new Hoster(
                        "",                 // hosterUrl
                        "default",          // hosterName
                        null,               // videoList
                        episode.url,        // internalData
                        false,              // lazy
                    )
                ];
            }
            throw e;
        }
    }

    hosterListRequest(episode) {
        return GET(this.getBaseUrl() + episode.url, this.headers);
    }

    async getVideoList(hoster) {
        const req = this.videoListRequest(hoster);
        const res = await this._doRequest(req);
        return this.videoListParse(res, hoster);
    }

    videoListRequest(hoster) {
        return GET(hoster.hosterUrl, this.headers);
    }
}


Array.prototype.toHosterList = function() {
    return Hoster.Companion.toHosterList(this);
};
// AnimesPage — returned by popularAnimeParse / searchAnimeParse
globalThis.AnimesPage = class AnimesPage {
    constructor(animes, hasNextPage) {
        this.animes      = animes      ?? [];
        this.hasNextPage = hasNextPage ?? false;
    }
};

globalThis.AnimeHttpSource       = AnimeHttpSource;
globalThis.ParsedAnimeHttpSource = AnimeHttpSource;
globalThis.Anime                 = AnimeHttpSource;

async function _doRequest(req) {
    const r = req instanceof _Call ? req._req : req;

    const useCloudflare = req instanceof _Call
        ? req._useCloudflare
        : (this.client?._useCloudflare ?? true);

    const url    = r.url?.toString?.() ?? String(r.url);
    const method = r.method ?? "GET";
    const { body, contentType } = _serializeBody(r.body);

    const origin = new URL(url).origin;
    const cfState = useCloudflare ? (_cfStateByOrigin.get(origin) ?? null) : false;

    const buildHeaders = (base, contentType, extraCookies = null) => {
        const h = base instanceof Headers
            ? new Headers(base)
            : new Headers(base instanceof Object ? base : {});

        // Merge cookie jar
        if (_cookieStore.size > 0) {
            const stored = [..._cookieStore.entries()]
                .map(([k, v]) => `${k}=${v}`).join("; ");
            const existing = h.get("cookie") ?? "";
            h.set("cookie", existing ? `${existing}; ${stored}` : stored);
        }

        const xsrf = _cookieStore.get("XSRF-TOKEN");
        if (xsrf && !h.has("X-XSRF-TOKEN")) {
            h.set("X-XSRF-TOKEN", decodeURIComponent(xsrf));
        }

        if (contentType) {
            h.set("content-type", contentType);
        }

        if (extraCookies) {
            _mergeCloudfareCookies(h, extraCookies);
        }

        return h;
    };

    const baseHeaders = r.headers?.toFetchHeaders?.() ?? r.headers
        ?? this.headers?.toFetchHeaders?.() ?? this.headers ?? {};

    // ── Path A: origin is known CF-protected, go straight to headless ────────
    if (cfState && cfState !== false) {
        const h = buildHeaders(baseHeaders, contentType, cfState.cookies);
        h.set("user-agent",    cfState.userAgent);
        h.set("referer",       origin + "/");
        h.set("sec-fetch-site", "same-origin");
        h.set("sec-fetch-mode", "cors");
        h.set("sec-fetch-dest", "empty");

        const headersObj = h.toFetchHeaders?.() ?? {};
        const res = await headless.fetch(url, {
            method,
            headers: headersObj,
            body,
        });

        if (res.status < 200 || res.status >= 300) throw new Error(`HTTP ${res.status}: ${url}`);
        return new _SandboxResponse(res.html ?? res.body ?? res.text ?? "", res.status, url);
    }

    // ── Path B: origin unknown or known-clean, try plain fetch first ─────────
    const headers = buildHeaders(baseHeaders, contentType);

    const res  = await fetch(url, {
        method,
        headers: headers.toFetchHeaders?.() ?? deepSerialize(headers),
        body,
    });
    const text = await res.text();

    if (res.cookies) {
        for (const [k, v] of Object.entries(res.cookies)) {
            _cookieStore.set(k, v);
        }
    }

    const isCfChallenge = useCloudflare
        && (res.status === 403 || res.status === 503)
        && (
            text.includes("challenge-error-title") ||
            text.includes("challenge-error-text") ||
            text.includes("Attention Required") ||
            text.includes("cdn-cgi/content") ||
            text.includes("cf_styles-css") ||
            text.includes("cloudflare")
        );

    if (!isCfChallenge) {
        if (cfState === null) _cfStateByOrigin.set(origin, false);
        _saveCfState();

        if (!res.ok) throw new Error(`HTTP ${res.status}: ${url}`);
        return new _SandboxResponse(text, res.status, url);
    }

    const { cookies, userAgent } = await _resolveCloudflare(url);
    _cfStateByOrigin.set(origin, { cookies, userAgent });
    _saveCfState();

    const retryHeaders = buildHeaders(baseHeaders, contentType, cookies);
    retryHeaders.set("user-agent",    userAgent);
    retryHeaders.set("referer",       origin + "/");
    retryHeaders.set("sec-fetch-site", "same-origin");
    retryHeaders.set("sec-fetch-mode", "cors");
    retryHeaders.set("sec-fetch-dest", "empty");

    const cfSession = await headless.fetch(origin, {
        waitFor: { selector: "body" },
        capture: [url],
        javascript: `
        fetch(${JSON.stringify(url)}, {
            method: ${JSON.stringify(method)},
            headers: ${JSON.stringify(retryHeaders.toFetchHeaders())},
            ${body ? `body: ${JSON.stringify(body)},` : ""}
        }).then(r => r.text())
    `,
        timeout_ms: 20000,
    });

    const captured = cfSession.captured?.find(c => c.url.includes(url));
    if (!captured) throw new Error(`CF capture failed for ${url}`);
    return new _SandboxResponse(captured.body, captured.status, url);
}

globalThis.Filter = class Filter {
    constructor(name, state) {
        this.name = name;
        this.state = state;
    }

    getState() { return this.state; }
    setState(s) { this.state = s; }

    getName() { return this.name; }

    equals(other) {
        if (this === other) return true;
        if (!(other instanceof Filter)) return false;
        return this.name === other.name && this.state === other.state;
    }
};

Filter.Header = class Header extends Filter {
    constructor(name) { super(name, 0); }
};

Filter.Separator = class Separator extends Filter {
    constructor(name = "") { super(name, 0); }
};

Filter.Select = class Select extends Filter {
    constructor(name, values, state = 0) {
        super(name, state);
        this.values = values;
    }

    getValues() {
        return this.values;
    }

    setValues(values) {
        this.values = values;
    }
};

Filter.Text = class Text extends Filter {
    constructor(name, state = "") { super(name, state); }
};

Filter.CheckBox = class CheckBox extends Filter {
    constructor(name, state = false) { super(name, state); }
};

Filter.TriState = class TriState extends Filter {
    static STATE_IGNORE  = 0;
    static STATE_INCLUDE = 1;
    static STATE_EXCLUDE = 2;

    constructor(name, state = Filter.TriState.STATE_IGNORE) { super(name, state); }

    isIgnored()  { return this.state === Filter.TriState.STATE_IGNORE; }
    isIncluded() { return this.state === Filter.TriState.STATE_INCLUDE; }
    isExcluded() { return this.state === Filter.TriState.STATE_EXCLUDE; }
};

Filter.Group = class Group extends Filter {
    constructor(name, state) { super(name, state ?? []); }
};

Filter.Sort = class Sort extends Filter {
    constructor(name, values, state = null) {
        super(name, state);
        this.values = values;
    }
    getValues() { return this.values; }
};

Filter.Sort.Selection = class Selection {
    constructor(index, ascending) {
        this.index     = index;
        this.ascending = ascending;
    }
    getIndex()     { return this.index; }
    getFirst()     { return this.index; }
    getSecond()    { return this.ascending; }
    getAscending() { return this.ascending; }
    component1()   { return this.index; }
    component2()   { return this.ascending; }
};

AnimeHttpSource.prototype._doRequest = _doRequest;
HttpSource.prototype._doRequest = _doRequest;

function getHeaders() {
    const builder = this.headersBuilder();
    return typeof builder?.build === "function" ? builder.build() : new Headers(builder ?? {});
}

AnimeHttpSource.prototype.getHeaders = getHeaders;
HttpSource.prototype.getHeaders = getHeaders;

HttpSource.prototype.getNetwork = function() { return _networkHelper; };
AnimeHttpSource.prototype.getNetwork = function() { return _networkHelper; };

globalThis._origDoRequest = Manga.prototype._doRequest;

globalThis.AnimeFilter = Filter;
globalThis.AnimeFilterList = FilterList;

globalThis.__tachi_getCapturedClass = function() {
    if (__tachi_captured) return __tachi_captured;

    const candidates = [];
    for (const key of Object.getOwnPropertyNames(globalThis)) {
        try {
            const v = globalThis[key];
            if (typeof v === "function" && v !== HttpSource && (v.prototype instanceof HttpSource || v.prototype instanceof AnimeHttpSource)) {
                candidates.push(v);
            }
        } catch (_) {}
    }

    const leaf = candidates.find(cls =>
        !candidates.some(other => other !== cls && other.prototype instanceof cls)
    );

    __tachi_captured = leaf ?? candidates[0];
    if (!__tachi_captured) throw new Error("[tachi-compat] No class extending HttpSource found");
    return __tachi_captured;
};

globalThis.__tachi_getFactoryClass = function() {
    const candidates = [];

    for (const key of Object.getOwnPropertyNames(globalThis)) {
        try {
            const v = globalThis[key];

            if (
                typeof v === "function" &&
                v !== SourceFactory &&
                v.prototype instanceof SourceFactory
            ) {
                candidates.push(v);
            }
        } catch (_) {}
    }

    const leaf = candidates.find(cls =>
        !candidates.some(other =>
            other !== cls &&
            other.prototype instanceof cls
        )
    );

    return leaf ?? candidates[0] ?? null;
};