"use strict";

// Load order:
//   1. sandbox_bootstrap.js   (fetch, parseHTML, crypto, state, headless)
//   2. Base.js + Manga.js     (your extension base classes)
//   3. tachiyomi.js           (this file)
//   4. translated_extension.js

function GET(url, headers) {
    return {
        url:     url instanceof HttpUrl ? url.toString() : String(url),
        method:  "GET",
        headers: _headersToObj(headers),
    };
}

function POST(url, body, headers) {
    return {
        url:     url instanceof HttpUrl ? url.toString() : String(url),
        method:  "POST",
        headers: _headersToObj(headers),
        body:    body != null ? String(body) : null,
    };
}

function _headersToObj(headers) {
    if (!headers)                      return {};
    if (headers instanceof Headers)    return { ...headers._map };
    if (typeof headers === "object")   return { ...headers };
    return {};
}

// Mirrors CloudflareInterceptor logic:
//   1. Try plain fetch()
//   2. If 403/503 + cloudflare server → solve via headless, store cf_clearance
//   3. Retry original request with stored cookies injected

async function _executeRequest(req) {
    const url     = req.url instanceof HttpUrl ? req.url.toString() : String(req.url);
    const method  = req.method  || "GET";
    const headers = _headersToObj(req.headers);
    const body    = req.body ?? undefined;

    const cfKey = _cfStateKey(url);
    const cfData = state.get(cfKey, null);
    if (cfData) {
        headers["Cookie"] = _mergeCookies(headers["Cookie"], cfData.cookies);
        if (cfData.userAgent) headers["User-Agent"] = cfData.userAgent;
    }

    const raw = await fetch(url, { method, headers, body });
    const text = await raw.text();

    if ((raw.status === 403 || raw.status === 503) && _isCloudflarePage(text)) {
        return await _solveCloudflare(url, method, headers, body);
    }

    return _wrapResponse(raw, text, url, method, headers, body);
}

function _isCloudflarePage(body) {
    return body.includes("challenge-error-title") ||
        body.includes("challenge-error-text")  ||
        body.includes("cf-browser-verification");
}

function _cfStateKey(url) {
    try {
        const host = new URL(url).hostname;
        return `cf:${host}`;
    } catch { return `cf:${url}`; }
}

function _mergeCookies(existing, newCookies) {
    if (!existing) return newCookies;
    const base = existing.split(";").map(s => s.trim()).filter(s => !s.startsWith("cf_clearance="));
    return [...base, newCookies].join("; ");
}

async function _solveCloudflare(url, method, headers, body) {
    if (!headless.available) {
        throw new Error("Cloudflare challenge detected but headless browser is not available");
    }

    const result = await headless.fetch(url, {
        waitFor:   "network_idle",
        block:     ["images", "fonts", "media"],
        timeoutMs: 25000,
    });

    const cfCookie = result.cookies?.find(c => c.name === "cf_clearance");
    if (cfCookie) {
        const cookieStr = result.cookies.map(c => `${c.name}=${c.value}`).join("; ");
        state.set(_cfStateKey(url), {
            cookies:   cookieStr,
            userAgent: headers["User-Agent"] || null,
        });
        headers["Cookie"] = cookieStr;
    }

    const fakeRaw = { ok: true, status: 200 };
    return _wrapResponse(fakeRaw, result.html || "", url, method, headers, body);
}

function _wrapResponse(raw, text, url, method, reqHeaders, reqBody) {
    return {
        ok:           raw.ok,
        status:       raw.status,
        isSuccessful: raw.ok,
        _body:        text,
        
        asJsoup() {
            return _wrapDocument(parseHTML(text));
        },

        parseAs() {
            try { return JSON.parse(text); }
            catch (e) { throw new Error("parseAs: not valid JSON: " + e.message); }
        },

        body: {
            string()      { return text; },
            contentType() { return null; },
            byteStream()  { return null; },
        },

        request: {
            url: _wrapHttpUrl(url),
            headers: { get: (k) => reqHeaders[k] ?? null },
        },

        newBuilder() {
            return {
                body:  () => this,
                build: () => this,
            };
        },

        headers: {
            get: (k) => null,
            has: (k) => false,
        },
    };
}


class HttpUrl {
    constructor(urlStr) {
        this._u = new URL(String(urlStr));
    }

    get host()         { return this._u.hostname; }
    get encodedPath()  { return this._u.pathname; }
    get fragment()     { return this._u.hash ? this._u.hash.slice(1) : null; }
    get pathSegments() { return this._u.pathname.split("/").filter(Boolean); }

    queryParameter(name) { return this._u.searchParams.get(name); }
    newBuilder()         { return new HttpUrlBuilder(this._u.toString()); }
    toString()           { return this._u.toString(); }

    static from(str)     { return new HttpUrl(str); }
    static Builder()     { return new HttpUrlBuilder(""); }
}

class HttpUrlBuilder {
    constructor(base = "") {
        this._base   = base;
        this._segs   = [];
        this._params = [];   // [{k, v, mode}]  mode: add | set | remove
        this._frag   = null;
    }

    addPathSegment(s)           { this._segs.push(encodeURIComponent(String(s))); return this; }
    addQueryParameter(k, v)     { this._params.push({ k, v: v == null ? "" : String(v), mode: "add" }); return this; }
    setQueryParameter(k, v)     { this._params.push({ k, v: String(v), mode: "set" }); return this; }
    removeQueryParameter(k)     { this._params.push({ k, mode: "remove" }); return this; }
    fragment(f)                 { this._frag = f; return this; }

    build() {
        let str = this._base;
        if (this._segs.length) str = str.replace(/\/?$/, "/") + this._segs.join("/");

        const map = new URLSearchParams(str.includes("?") ? str.split("?")[1] : "");
        str = str.split("?")[0];
        for (const { k, v, mode } of this._params) {
            if (mode === "add")    { map.append(k, v); }
            else if (mode === "set")    { map.set(k, v); }
            else if (mode === "remove") { map.delete(k); }
        }
        const qs = map.toString();
        if (qs) str += "?" + qs;
        if (this._frag != null) str += "#" + this._frag;
        return new HttpUrl(str);
    }
}

function _wrapHttpUrl(urlStr) {
    try { return new HttpUrl(urlStr); }
    catch { return { toString: () => urlStr, fragment: null, queryParameter: () => null, pathSegments: [], host: "" }; }
}

String.prototype.toHttpUrl = function() { return new HttpUrl(this.toString()); };


class Headers {
    constructor(map = {}) { this._map = { ...map }; }
    get(name)       { return this._map[name] ?? null; }
    has(name)       { return name in this._map; }
    newBuilder()    { return new HeadersBuilder({ ...this._map }); }
    static Builder(){ return new HeadersBuilder(); }
}

class HeadersBuilder {
    constructor(init = {}) { this._map = { ...init }; }
    add(k, v)       { this._map[k] = v;       return this; }
    set(k, v)       { this._map[k] = v;       return this; }
    remove(k)       { delete this._map[k];    return this; }
    removeAll(k)    { delete this._map[k];    return this; }
    build()         { return new Headers(this._map); }
}

const network = {
    get cloudflareClient() { return _fakeClient(true); },
    get client()           { return _fakeClient(false); },
};

function _fakeClient(cfEnabled) {
    return {
        _cf: cfEnabled,
        newBuilder() {
            return {
                _cf: cfEnabled,
                addInterceptor()        { return this; },
                addNetworkInterceptor() { return this; },
                rateLimitHost()         { return this; },
                apply(fn)               { fn.call(this); return this; },
                build()                 { return _fakeClient(this._cf); },
            };
        },
        newCall(req) {
            return {
                async execute()   { return _executeRequest(req); },
                async await()     { return _executeRequest(req); },
                enqueue()         { /* fire and forget */ _executeRequest(req).catch(() => {}); },
            };
        },
    };
}

function _wrapDocument(dollarFn) {
    return {
        selectFirst(sel) {
            const els = dollarFn(sel);
            return els.length > 0 ? _wrapElement(els[0], dollarFn) : null;
        },
        select(sel) {
            return _wrapElements(dollarFn(sel), dollarFn);
        },
        wholeText() {
            const els = dollarFn("body");
            return els.length > 0 ? els[0].text() : "";
        },
        getElementById(id) {
            const els = dollarFn(`#${id}`);
            return els.length > 0 ? _wrapElement(els[0], dollarFn) : null;
        },
    };
}

function _wrapElement(raw, dollarFn) {
    if (!raw) return null;
    const findFn = (sel) => raw.find(sel);

    return {
        text()           { return raw.text(); },
        html()           { return raw.html(); },
        outerHtml()      { return raw.outer(); },
        ownText()        { return raw.text(); },
        data()           { return raw.html(); },
        attr(name)       { return raw.attr(name) ?? ""; },

        selectFirst(sel) {
            const found = findFn(sel);
            return found.length > 0 ? _wrapElement(found[0], dollarFn) : null;
        },
        select(sel)      { return _wrapElements(findFn(sel), dollarFn); },
        find(sel)        { return _wrapElements(findFn(sel), dollarFn); },

        get value()      { return this; },

        textNodes()      { return [{ text: () => raw.text(), wholeText: () => raw.text() }]; },
        hasClass(cls)    { return (raw.attr("class") || "").split(/\s+/).includes(cls); },
        id()             { return raw.attr("id") ?? ""; },

        absUrl(attr)     {
            const val = raw.attr(attr) ?? "";
            if (!val || /^https?:/.test(val)) return val;
            return val; // best effort without a base URL
        },

        _raw: raw,
    };
}

function _wrapElements(raws, dollarFn) {
    const wrapped = Array.from(raws || []).map(r => _wrapElement(r, dollarFn));

    wrapped.first       = () => wrapped[0] ?? null;
    wrapped.last        = () => wrapped[wrapped.length - 1] ?? null;
    wrapped.getOrNull   = (i) => wrapped[i] ?? null;
    wrapped.isNotEmpty  = () => wrapped.length > 0;
    wrapped.isEmpty     = () => wrapped.length === 0;
    wrapped.text        = () => wrapped.map(e => e.text()).join("");
    wrapped.attr        = (name) => wrapped[0]?.attr(name) ?? "";
    wrapped.select      = (sel) => _wrapElements(raws.flatMap(r => [...r.find(sel)]), dollarFn);

    wrapped.joinToString = function(optsOrTransform = {}) {
        let sep = ", ", transform = null;
        if (typeof optsOrTransform === "function") {
            transform = optsOrTransform;
        } else {
            sep = optsOrTransform.separator ?? ", ";
            transform = optsOrTransform.transform ?? null;
        }
        return wrapped.map(el => transform ? transform(el) : el.text()).join(sep);
    };

    wrapped.mapNotNull = function(fn) {
        return wrapped.map(fn).filter(v => v != null);
    };

    return wrapped;
}

const Jsoup = {
    parse(html)             { return _wrapDocument(parseHTML(html)); },
    parseBodyFragment(html) { return _wrapDocument(parseHTML(html)); },
};

class SManga {
    constructor() {
        this.url           = "";
        this.title         = "";
        this.thumbnail_url = null;
        this.description   = null;
        this.author        = null;
        this.artist        = null;
        this.genre         = null;
        this.status        = 0;
        this.initialized   = false;
    }
    setUrlWithoutDomain(url) { this.url = url; }
    static create()          { return new SManga(); }
}
SManga.UNKNOWN             = 0;
SManga.ONGOING             = 1;
SManga.COMPLETED           = 2;
SManga.LICENSED            = 3;
SManga.PUBLISHING_FINISHED = 4;
SManga.CANCELLED           = 5;
SManga.ON_HIATUS           = 6;

class SChapter {
    constructor() {
        this.url            = "";
        this.name           = "";
        this.date_upload    = 0;
        this.chapter_number = -1;
        this.scanlator      = null;
    }
    setUrlWithoutDomain(url) { this.url = url; }
    static create()          { return new SChapter(); }
}

class Page {
    constructor(index, url = "", imageUrl = null) {
        this.index    = index;
        this.url      = url;
        this.imageUrl = imageUrl;
    }
}

class MangasPage {
    constructor(mangas, hasNextPage) {
        this.mangas      = mangas;
        this.hasNextPage = hasNextPage;
    }
}

function buildString(fn) {
    const parts = [];
    const b = {
        append(...args) { args.forEach(a => parts.push(a == null ? "null" : String(a))); return b; },
        toString()      { return parts.join(""); },
    };
    fn(b);
    return parts.join("").trim();
}

function buildList(fn) {
    const list = [];
    const b = {
        add(item)       { list.push(item); },
        addAll(items)   { if (items) list.push(...items); },
    };
    fn(b);
    return list;
}

String.prototype.isBlank           = function() { return this.trim() === ""; };
String.prototype.isNotBlank        = function() { return this.trim() !== ""; };
String.prototype.removeSuffix      = function(s) { return this.endsWith(s) ? String(this.slice(0, -s.length)) : String(this); };
String.prototype.removePrefix      = function(s) { return this.startsWith(s) ? String(this.slice(s.length)) : String(this); };
String.prototype.substringAfterLast  = function(d) { const i = this.lastIndexOf(d); return i < 0 ? String(this) : String(this.slice(i + d.length)); };
String.prototype.substringBeforeLast = function(d) { const i = this.lastIndexOf(d); return i < 0 ? String(this) : String(this.slice(0, i)); };
String.prototype.substringAfter      = function(d) { const i = this.indexOf(d); return i < 0 ? "" : String(this.slice(i + d.length)); };
String.prototype.substringBefore     = function(d) { const i = this.indexOf(d); return i < 0 ? String(this) : String(this.slice(0, i)); };
String.prototype.replaceFirst        = function(s, r) { return String(this).replace(s, r); };
String.prototype.trimIndent          = function() { return String(this).trim(); };
String.prototype.lowercase           = function() { return this.toLowerCase(); };
String.prototype.uppercase           = function() { return this.toUpperCase(); };

function isNullOrBlank(s)  { return s == null || String(s).trim() === ""; }
function isNullOrEmpty(s)  { return s == null || String(s) === ""; }

// Array extras
Array.prototype.isNotEmpty   = function() { return this.length > 0; };
Array.prototype.isEmpty      = function() { return this.length === 0; };
Array.prototype.firstOrNull  = function() { return this[0] ?? null; };
Array.prototype.lastOrNull   = function() { return this[this.length - 1] ?? null; };
Array.prototype.mapNotNull   = function(fn) { return this.map(fn).filter(v => v != null); };
Array.prototype.filterIsInstance = function(cls) { return this.filter(v => v instanceof cls); };
Array.prototype.toMutableList = function() { return [...this]; };
Array.prototype.joinToString = function(optsOrSep = {}) {
    let sep = ", ", transform = null;
    if (typeof optsOrSep === "string") { sep = optsOrSep; }
    else { sep = optsOrSep.separator ?? ", "; transform = optsOrSep.transform ?? null; }
    return this.map(el => transform ? transform(el) : (el == null ? "" : String(el))).join(sep);
};

// Kotlin number extensions
Number.prototype.coerceAtMost  = function(max) { return Math.min(Number(this), max); };
Number.prototype.coerceAtLeast = function(min) { return Math.max(Number(this), min); };

// Date parsing (SimpleDateFormat.tryParse equivalent)
function _tryParse(str) {
    if (!str) return 0;
    try { return new Date(str).getTime() || 0; } catch { return 0; }
}

class Filter {
    constructor(name) { this.name = name; this.state = null; }
    static Header(name)    { const f = new Filter(name); f._kind = "header";    return f; }
    static Separator()     { const f = new Filter("");   f._kind = "separator"; return f; }
}
Filter.Header    = (name) => { const f = new Filter(name); f._kind = "header";    return f; };
Filter.Separator = ()     => { const f = new Filter("");   f._kind = "separator"; return f; };

class SelectFilter extends Filter {
    constructor(name, options, state = 0) { super(name); this.options = options; this.state = state; }
    get selected() { return this.options?.[this.state] ?? null; }
}

class TextFilter extends Filter {
    constructor(name, state = "") { super(name); this.state = state; }
}

class CheckBoxFilter extends Filter {
    constructor(name, state = false) { super(name); this.state = state; }
}

class TriStateFilter extends Filter {
    // 0=ignore 1=include 2=exclude
    constructor(name, state = 0) { super(name); this.state = state; }
}

class GroupFilter extends Filter {
    constructor(name, filters) { super(name); this.filters = filters; this.state = filters; }
}

class SortFilter extends Filter {
    constructor(name, options, state) {
        super(name);
        this.options = options;
        this.state   = state ?? { index: 0, ascending: true };
    }
}

function FilterList(...filters) {
    const list = filters.flat();
    list.firstInstance = (cls) => {
        const found = list.find(f => f instanceof cls);
        if (!found) throw new Error(`Filter ${cls.name} not found`);
        return found;
    };
    list.firstInstanceOrNull = (cls) => list.find(f => f instanceof cls) ?? null;
    list.filterIsInstance    = (cls) => list.filter(f => f instanceof cls);
    list.ifEmpty             = (fn)  => list.length === 0 ? fn() : list;
    return list;
}

function getPreferences() {
    return {
        getString(key, def = null)    { return state.get(`pref:${key}`, def); },
        getBoolean(key, def = false)  { return state.get(`pref:${key}`, def); },
        getInt(key, def = 0)          { return state.get(`pref:${key}`, def); },
        edit() {
            const edits = {};
            return {
                putString(key, val)  { edits[key] = val; return this; },
                putBoolean(key, val) { edits[key] = val; return this; },
                putInt(key, val)     { edits[key] = val; return this; },
                apply()  { Object.entries(edits).forEach(([k, v]) => state.set(`pref:${k}`, v)); },
                commit() { this.apply(); return true; },
            };
        },
    };
}

function getPreferencesLazy() { return getPreferences(); }

class HttpSource {
    constructor() {
        this._headers = new HeadersBuilder()
            .add("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36")
            .build();
    }

    get headers()        { return this._headers._map; }
    get supportsLatest() { return false; }
    get lang()           { return "all"; }

    headersBuilder()           { return new HeadersBuilder({ ...this._headers._map }); }
    mangaDetailsRequest(manga) { return GET(`${this.baseUrl}${manga.url}`, this.headers); }
    chapterListRequest(manga)  { return GET(`${this.baseUrl}${manga.url}`, this.headers); }
    pageListRequest(chapter)   { return GET(`${this.baseUrl}${chapter.url}`, this.headers); }
    imageUrlRequest(page)      { return GET(page.url, this.headers); }
    imageUrlParse(_res)        { throw new Error("imageUrlParse not implemented"); }
    getFilterList()            { return FilterList(); }
    getMangaUrl(manga)         { return this.baseUrl + manga.url; }
    getChapterUrl(chapter)     { return this.baseUrl + chapter.url; }
    setupPreferenceScreen()    {}
}

class ParsedHttpSource extends HttpSource {
    popularMangaSelector()          { return ""; }
    popularMangaNextPageSelector()  { return null; }
    latestUpdatesSelector()         { return ""; }
    latestUpdatesNextPageSelector() { return null; }
    searchMangaSelector()           { return ""; }
    searchMangaNextPageSelector()   { return null; }
    chapterListSelector()           { return ""; }

    popularMangaFromElement(_el)    { return SManga.create(); }
    latestUpdatesFromElement(_el)   { return SManga.create(); }
    searchMangaFromElement(_el)     { return SManga.create(); }
    chapterFromElement(_el)         { return SChapter.create(); }
    mangaDetailsParse(_doc)         { return SManga.create(); }
    pageListParse(_doc)             { return []; }

    popularMangaParse(response) {
        const doc     = response.asJsoup();
        const mangas  = doc.select(this.popularMangaSelector()).map(el => this.popularMangaFromElement(el));
        const nextSel = this.popularMangaNextPageSelector();
        return new MangasPage(mangas, nextSel ? doc.selectFirst(nextSel) != null : false);
    }

    latestUpdatesParse(response) {
        const doc     = response.asJsoup();
        const mangas  = doc.select(this.latestUpdatesSelector()).map(el => this.latestUpdatesFromElement(el));
        const nextSel = this.latestUpdatesNextPageSelector?.();
        return new MangasPage(mangas, nextSel ? doc.selectFirst(nextSel) != null : false);
    }

    searchMangaParse(response) {
        const doc     = response.asJsoup();
        const mangas  = doc.select(this.searchMangaSelector()).map(el => this.searchMangaFromElement(el));
        const nextSel = this.searchMangaNextPageSelector();
        return new MangasPage(mangas, nextSel ? doc.selectFirst(nextSel) != null : false);
    }

    chapterListParse(response) {
        const doc = response.asJsoup();
        return doc.select(this.chapterListSelector()).map(el => this.chapterFromElement(el));
    }
}

class TachiyomiAdapter extends Manga {
    constructor(source) {
        super();
        this._source = source;
    }

    async search(query, filters, page = 1) {
        const hasQuery = query && query.trim().length > 0;
        const fl       = FilterList(...(filters || []));
        const req      = hasQuery
            ? this._source.searchMangaRequest(page, query, fl)
            : this._source.popularMangaRequest(page);

        const response = await _executeRequest(req);
        const result   = hasQuery
            ? this._source.searchMangaParse(response)
            : this._source.popularMangaParse(response);

        return result.mangas.map(m => ({
            id:    m.url,
            title: m.title,
            image: m.thumbnail_url ?? null,
            url:   m.url,
        }));
    }

    async getMetadata(id) {
        const manga  = SManga.create();
        manga.url    = id;
        const req    = this._source.mangaDetailsRequest(manga);
        const res    = await _executeRequest(req);
        const detail = this._source.mangaDetailsParse(res);

        return {
            title:           detail.title,
            synopsis:        detail.description   ?? null,
            image:           detail.thumbnail_url  ?? null,
            author:          detail.author         ?? null,
            artist:          detail.artist         ?? null,
            genres:          detail.genre ? detail.genre.split(",").map(g => g.trim()) : null,
            status:          _mapStatus(detail.status),
            eps_or_chapters: null,
            rating:          null,
            year:            null,
            nsfw:            false,
        };
    }

    async findChapters(id) {
        const manga  = SManga.create();
        manga.url    = id;
        const req    = this._source.chapterListRequest(manga);
        const res    = await _executeRequest(req);
        const list   = this._source.chapterListParse(res);

        return list.map((ch, i) => ({
            id:     ch.url,
            title:  ch.name,
            number: ch.chapter_number >= 0 ? ch.chapter_number : null,
            index:  i,
        }));
    }

    async findChapterPages(chapterId) {
        const chapter  = SChapter.create();
        chapter.url    = chapterId;
        const req      = this._source.pageListRequest(chapter);
        const res      = await _executeRequest(req);
        const pages    = this._source.pageListParse(res);

        return pages.map(p => ({
            url:   p.imageUrl ?? p.url,
            index: p.index,
        }));
    }

    getFilters() {
        try {
            return _serializeFilters(this._source.getFilterList());
        } catch {
            return {};
        }
    }
}

function _mapStatus(s) {
    switch (s) {
        case SManga.ONGOING:             return "ongoing";
        case SManga.COMPLETED:           return "completed";
        case SManga.PUBLISHING_FINISHED: return "finished";
        case SManga.ON_HIATUS:           return "hiatus";
        case SManga.CANCELLED:           return "cancelled";
        default:                         return "unknown";
    }
}

function _serializeFilters(filters) {
    const out = {};
    for (const f of (filters || [])) {
        if (!f.name || f._kind === "separator" || f._kind === "header") continue;
        if      (f instanceof SortFilter)     out[f.name] = { type: "sort",      options: f.options, value: f.state };
        else if (f instanceof SelectFilter)   out[f.name] = { type: "select",    options: f.options, value: f.state };
        else if (f instanceof TextFilter)     out[f.name] = { type: "text",      value: f.state };
        else if (f instanceof CheckBoxFilter) out[f.name] = { type: "checkbox",  value: f.state };
        else if (f instanceof TriStateFilter) out[f.name] = { type: "tristate",  value: f.state };
        else if (f instanceof GroupFilter)    out[f.name] = { type: "group",     filters: _serializeFilters(f.filters) };
    }
    return out;
}


function registerSource(httpSource) {
    globalThis.__extension__ = new TachiyomiAdapter(httpSource);
}

function registerSources(httpSources) {
    globalThis.__extensions__ = httpSources.map(s => new TachiyomiAdapter(s));
}