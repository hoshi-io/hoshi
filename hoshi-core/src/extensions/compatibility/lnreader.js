// LNReader → Sandbox Compat Layer
// After the plugin code runs, __lnr_buildNovelClass() wraps exports.default
// into a Novel subclass that the sandbox runner can instantiate normally.

(function () {

    // Capture the sandbox's native parseHTML before we shadow it
    const _nativeParse = parseHTML;

    // 1. CommonJS module system
    const __modules = {};

    __modules["cheerio"] = {
        load(html) {
            const $ = _makeCheerio(html);
            // Some plugins do load(str).text() directly to strip HTML tags
            $.text = function() {
                return _nativeParse(html)("*").text() ||
                    html.replace(/<[^>]*>/g, "");
            };
            return $;
        },
    };

    __modules["@/types/constants"] = {
        defaultCover: 'https://github.com/LNReader/lnreader-plugins/blob/main/icons/src/coverNotAvailable.jpg?raw=true',
        NovelStatus: {
            Unknown:            'Unknown',
            Ongoing:            'Ongoing',
            Completed:          'Completed',
            Licensed:           'Licensed',
            PublishingFinished: 'Publishing Finished',
            Cancelled:          'Cancelled',
            OnHiatus:           'On Hiatus',
        },
    };

    __modules["@libs/utils"] = {
        isUrlAbsolute(url) {
            if (url) {
                if (url.indexOf('//') === 0) return true;
                if (url.indexOf('://') === -1) return false;
                if (url.indexOf('.') === -1) return false;
                if (url.indexOf('/') === -1) return false;
                if (url.indexOf(':') > url.indexOf('/')) return false;
                if (url.indexOf('://') < url.indexOf('.')) return true;
            }
            return false;
        },
        cn(...inputs) {
            return inputs.flat().filter(Boolean).join(' ');
        },
    };

    function _wrap(nativeResults, contextHtml) {
        const arr = Array.from(nativeResults).map(el => {
            if (!el.attribs) {
                el.attribs = new Proxy({}, {
                    get(_, name) { return el.attr(String(name)) ?? undefined; },
                    has(_, name) { return el.attr(String(name)) !== null; },
                });
            }
            if (el.name === undefined) {
                const match = (el.outer() || "").match(/^<([a-zA-Z][a-zA-Z0-9]*)/);
                el.name = match ? match[1].toLowerCase() : "";
            }
            return el;
        });

        // Must be a function so plugins can do $(el)(selector)
        // Can't assign .length directly on a function — use defineProperty
        function w(selectorOrEl) {
            if (!selectorOrEl) return _wrap([], contextHtml);
            if (typeof selectorOrEl === "object" && typeof selectorOrEl.text === "function" && !selectorOrEl._isWrapped) {
                return _wrap([selectorOrEl], contextHtml);
            }
            if (selectorOrEl && selectorOrEl._isWrapped) return selectorOrEl;
            return _wrap(_nativeParse(contextHtml)(String(selectorOrEl)), contextHtml);
        }

        Object.defineProperty(w, "length", { value: arr.length, writable: true, configurable: true });
        w._isWrapped = true;

        arr.forEach((el, i) => { w[i] = el; });

        w.each   = function(cb) { arr.forEach((el, i) => cb.call(el, i, el)); return w; };
        w.map = function(cb) {
            const out = arr.map((el, i) => cb(i, el));
            out.get = () => out;
            out.toArray = () => out;
            return out;
        };
        w.filter = function(cb) { return _wrap(arr.filter((el, i) => cb(i, el)), contextHtml); };
        w.find   = function(sel) {
            if (arr.length === 0) return _wrap([], contextHtml);
            const inner = arr[0].html() || "";
            return _wrap(_nativeParse(inner)(sel), inner);
        };
        w.eq     = function(n) { const el = arr[n]; return el ? _wrap([el], contextHtml) : _wrap([], contextHtml); };
        w.first  = function() { return w.eq(0); };
        w.last   = function() { return w.eq(arr.length - 1); };
        w.not    = function(sel) {
            const excluded = new Set(Array.from(_nativeParse(contextHtml)(sel)).map(e => e.outer()));
            return _wrap(arr.filter(el => !excluded.has(el.outer())), contextHtml);
        };
        w.text   = function() { return arr.map(el => el.text()).join(""); };
        w.html   = function(newHtml) {
            if (newHtml !== undefined) return w; // setter no-op, sandbox HTML is immutable
            return arr.length > 0 ? arr[0].html() : null;
        };
        w.attr   = function(name) { return arr.length > 0 ? arr[0].attr(name) : null; };

        // No-op stubs for methods that mutate the DOM — sandbox HTML is immutable,
        // these just need to not throw so the plugin continues running
        w.remove     = function() { return w; };
        w.replaceWith= function() { return w; };
        w.append     = function() { return w; };
        w.prepend    = function() { return w; };
        w.after      = function() { return w; };
        w.before     = function() { return w; };
        w.addClass   = function() { return w; };
        w.removeClass= function() { return w; };
        w.toggleClass= function() { return w; };
        w.toArray = function() { return arr; };

        // Traversal stubs
        w.parent   = function() { return _wrap([], contextHtml); };
        w.parents  = function() { return _wrap([], contextHtml); };
        w.children = function(sel) {
            if (arr.length === 0) return _wrap([], contextHtml);
            const inner = arr[0].html() || "";
            if (sel) return _wrap(_nativeParse(inner)(sel), inner);
            return _wrap(_nativeParse(inner)("*"), inner);
        };
        w.siblings = function() { return _wrap([], contextHtml); };
        w.next     = function() { return _wrap([], contextHtml); };
        w.prev     = function() { return _wrap([], contextHtml); };
        w.closest  = function() { return _wrap([], contextHtml); };
        w.addBack  = function() { return w; };
        w.contents = function() {
            if (arr.length === 0) return _wrap([], contextHtml);
            const inner = arr[0].html() || "";
            return _wrap(_nativeParse(inner)("*"), inner);
        };

        return w;
    }

    function _makeCheerio(html) {
        function $(selectorOrEl) {
            if (!selectorOrEl) return _wrap([], html);
            // Already a wrapped result
            if (selectorOrEl && selectorOrEl._isWrapped) return selectorOrEl;
            // Native sandbox element — wrap it and attach .attribs
            if (typeof selectorOrEl === "object" && typeof selectorOrEl.text === "function") {
                return _wrap([selectorOrEl], html);
            }
            // Selector string — query using native parser, attach .attribs to results
            const arr = Array.from(_nativeParse(html)(String(selectorOrEl)));
            arr.forEach(el => {
                if (!el.attribs) {
                    el.attribs = new Proxy({}, {
                        get(_, name) { return el.attr(String(name)) ?? undefined; },
                        has(_, name) { return el.attr(String(name)) !== null; },
                    });
                }
                if (el.name === undefined) {
                    const match = (el.outer() || "").match(/^<([a-zA-Z][a-zA-Z0-9]*)/);
                    el.name = match ? match[1].toLowerCase() : "";
                }
            });
            return _wrap(arr, html);
        }
        $.load = (newHtml) => _makeCheerio(newHtml);
        return $;
    }

    // Shadow globalThis.parseHTML so plugin code that does:
    //   loadedCheerio = parseHTML(html)  (cheerio's load pattern)
    // gets a proper _makeCheerio $ back instead of the raw sandbox query fn.
    globalThis.parseHTML = _makeCheerio;

    // @libs/fetch shim
    __modules["@libs/fetch"] = {
        async fetchApi(url, options) {
            return fetch(url, options);
        },
    };

    // @libs/filterInputs shim
    __modules["@libs/filterInputs"] = {
        FilterTypes: {
            CheckboxGroup: "multiselect",
            Picker:        "select",
            Switch:        "boolean",
            Text:          "text",
            ExcludableCheckboxGroup: "multiselect",
        },
    };

    // @libs/defaultCover shim
    __modules["@libs/defaultCover"] = {
        defaultCover: null,
    };

    // @libs/storage shim — backed by sandbox `state`
    __modules["@libs/storage"] = {
        storage: {
            set(key, value, expires) {
                state.set(key, { value, expires: expires instanceof Date ? expires.getTime() : expires, created: Date.now() });
            },
            get(key, raw) {
                const item = state.get(key);
                if (!item) return undefined;
                if (item.expires && Date.now() > item.expires) {
                    state.delete(key);
                    return undefined;
                }
                return raw ? item : item.value;
            },
            delete(key) { state.delete(key); },
            getAllKeys() { return state.keys(); },
            clearAll() { state.keys().forEach(k => state.delete(k)); },
        },
        localStorage:   { get: () => ({}) },
        sessionStorage: { get: () => ({}) },
    };

    // dayjs shim, plugins use it mainly to parse/format chapter release dates
    __modules["dayjs"] = function dayjs(input) {
        const d = input ? new Date(input) : new Date();
        return {
            isValid()          { return !isNaN(d.getTime()); },
            toISOString()      { return d.toISOString(); },
            format(fmt)        { return d.toISOString(); },
            valueOf()          { return d.getTime(); },
            toDate()           { return d; },
            unix()             { return Math.floor(d.getTime() / 1000); },
            diff(other, unit)  {
                const ms = d - new Date(other);
                if (unit === "days")    return Math.floor(ms / 86400000);
                if (unit === "hours")   return Math.floor(ms / 3600000);
                if (unit === "minutes") return Math.floor(ms / 60000);
                return ms;
            },
        };
    };
    __modules["dayjs"].extend = () => {};

    // @libs/novelStatus shim
    __modules["@libs/novelStatus"] = {
        NovelStatus: {
            Unknown:            "Unknown",
            Ongoing:            "Ongoing",
            Completed:          "Completed",
            Licensed:           "Licensed",
            PublishingFinished: "Publishing Finished",
            Cancelled:          "Cancelled",
            OnHiatus:           "On Hiatus",
        },
    };

    __modules["@noble/ciphers/aes.js"] = {
        gcm(key, iv) {
            const keyHex = crypto.hex.fromString(String.fromCharCode(...key));
            const ivHex  = crypto.hex.fromString(String.fromCharCode(...iv));
            return {
                encrypt(data) {
                    const dataHex = crypto.hex.fromString(String.fromCharCode(...data));
                    const hex = crypto.aes.gcm.encrypt(keyHex, dataHex, { iv: ivHex });
                    return Uint8Array.from(hex.match(/.{2}/g).map(b => parseInt(b, 16)));
                },
                decrypt(data) {
                    const dataHex = crypto.hex.fromString(String.fromCharCode(...data));
                    const hex = crypto.aes.gcm.decrypt(keyHex, dataHex, { iv: ivHex });
                    return Uint8Array.from(hex.match(/.{2}/g).map(b => parseInt(b, 16)));
                },
            };
        },
    };

    globalThis.require = function (id) {
        if (id === "cheerio")             return __modules["cheerio"];
        if (id === "@libs/fetch")         return __modules["@libs/fetch"];
        if (id === "@libs/filterInputs")  return __modules["@libs/filterInputs"];
        if (id === "@libs/defaultCover")  return __modules["@libs/defaultCover"];
        if (id === "@libs/novelStatus")   return __modules["@libs/novelStatus"];
        if (id === "@libs/storage")       return __modules["@libs/storage"];
        if (id === "dayjs")               return __modules["dayjs"];
        if (id === "@libs/utils")         return __modules["@libs/utils"];
        if (id === "@noble/ciphers/aes.js") return __modules["@noble/ciphers/aes.js"];
        if (id === "@/types/constants") return __modules["@/types/constants"];
        // Unknown lib — log and return empty object so the plugin doesn't hard-crash
        console.warn(`[compat] require("${id}") is not shimmed, returning {}`);
        return {};
    };

    // CommonJS exports / module
    globalThis.exports = {};
    globalThis.module  = { exports: globalThis.exports };


    // 2. Novel subclass factory
    // Called by the sandbox wrapper AFTER index.js has been eval'd.
    // Reads exports.default (the plugin instance) and returns source code for
    // a Novel subclass that delegates every call to it.
    //
    // The sandbox runner expects to find a class that extends Novel in the
    // extension source string — so we generate that class dynamically and
    // expose it as __lnr_CompatNovel for the wrapper to use.

    globalThis.__lnr_buildNovelClass = function () {
        const plugin = (globalThis.module.exports && globalThis.module.exports.default)
            ? globalThis.module.exports.default
            : globalThis.exports.default;

        if (!plugin) throw new Error("[compat] No exports.default found after loading plugin");

        function buildFilters(rawFilters) {
            if (!rawFilters) return {};
            const out = {};
            for (const [key, def] of Object.entries(rawFilters)) {
                out[key] = {
                    label:   def.label,
                    type:    def.type,
                    options: def.options || [],
                };
            }
            return out;
        }

        function mapStatus(raw) {
            if (!raw) return null;
            const map = { Completed: "completed", Ongoing: "ongoing", Unknown: null };
            return map[raw] ?? null;
        }

        async function fetchAllChapters(plugin, novelPath, totalPages) {
            const all = [];
            for (let p = 1; p <= totalPages; p++) {
                const page = await plugin.parsePage(novelPath, String(p));
                if (page && page.chapters) all.push(...page.chapters);
            }
            return all;
        }

        class LNReaderCompat extends Novel {

            async getFilters() {
                return buildFilters(plugin.filters);
            }

            async search(query, filters, page) {
                // popularNovels when no query, searchNovels when query present
                let rawItems;
                if (query && query.trim()) {
                    rawItems = await plugin.searchNovels(query.trim(), page);
                } else {
                    // Map our filter shape back to what the plugin expects
                    const pluginFilters = {};
                    if (plugin.filters) {
                        for (const [key, def] of Object.entries(plugin.filters)) {
                            const incoming = filters && filters[key];
                            pluginFilters[key] = {
                                ...def,
                                value: incoming !== undefined ? incoming : def.value,
                            };
                        }
                    }
                    rawItems = await plugin.popularNovels(page, {
                        showLatestNovels: false,
                        filters: pluginFilters,
                    });
                }

                return (rawItems || []).map(item => ({
                    id:    item.path || item.url || "",
                    title: item.name || item.title || "",
                    image: item.cover || item.image || null,
                    url:   item.path ? plugin.site + item.path : (item.url || ""),
                }));
            }

            async getMetadata(id) {
                const raw = await plugin.parseNovel(id);

                return {
                    title:            raw.name   || "Untitled",
                    synopsis:         raw.summary || null,
                    image:            raw.cover   || null,
                    eps_or_chapters:  null,
                    rating:           null,
                    year:             null,
                    genres:           raw.genres ? raw.genres.split(",").map(g => g.trim()).filter(Boolean) : [],
                    nsfw:             false,
                    anilist_id:       null,
                    mal_id:           null,
                    external_ids:     {},
                    _totalPages:      raw.totalPages || 1,
                    _path:            id,
                };
            }

            async findChapters(contentId) {
                const raw = await plugin.parseNovel(contentId);
                const chapters = raw.chapters || [];

                return chapters.map((ch, idx) => ({
                    id:     ch.path || ch.chapterUrl || String(idx),
                    title:  ch.name || ch.chapterName || `Chapter ${ch.chapterNumber || idx + 1}`,
                    number: typeof ch.chapterNumber === "number" ? ch.chapterNumber : (idx + 1),
                    index:  idx,
                }));
            }

            async findChapterPages(chapterId) {
                const text = await plugin.parseChapter(chapterId);
                if (!text) return "";

                // The original plugin returns plain text with spaces between paragraphs.
                // Wrap each line in <p> so readers can render it properly.
                const paragraphs = text
                    .split(/\n+/)
                    .map(line => line.trim())
                    .filter(Boolean)
                    .map(line => `<p>${line}</p>`)
                    .join("\n");

                return paragraphs || `<p>${text}</p>`;
            }
        }

        // Expose so the sandbox script-builder regex can find it
        globalThis.__lnr_CompatNovel = LNReaderCompat;
        return LNReaderCompat;
    };

})();