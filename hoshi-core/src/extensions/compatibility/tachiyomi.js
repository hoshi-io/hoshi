// Kotlin stdlib

if (!Array.prototype.iterator) {
    Array.prototype.iterator = function() {
        let i = 0;
        const arr = this;
        return {
            hasNext: () => i < arr.length ? true : 0,
            next: () => i < arr.length ? arr[i++] : 0,
        };
    };
}

globalThis.StringsKt = {
    split$default(str, delimiters, ignoreCase, limit) {
        const sep = Array.isArray(delimiters) ? delimiters[0] : delimiters;
        const parts = str.split(sep);
        return (limit && limit > 0) ? parts.slice(0, limit) : parts;
    },
    removeSuffix(str, suffix) {
        return str.endsWith(suffix) ? str.slice(0, -suffix.length) : str;
    },
    isBlank(str) {
        return str == null || str.trim().length === 0;
    },
    substringBefore$default(str, delimiter, missingDelimiterValue, ...rest) {
        const idx = str.indexOf(delimiter);
        return idx === -1 ? (missingDelimiterValue ?? str) : str.slice(0, idx);
    },
    endsWith$default(str, suffix, ignoreCase, ...rest) {
        if (ignoreCase) return str.toLowerCase().endsWith(suffix.toLowerCase());
        return str.endsWith(suffix);
    },
};

globalThis.CharsKt = {
    isWhitespace(ch) { return /\s/.test(ch); },
};

globalThis.ordinal = function(v) { return typeof v === "number" ? v : v?.ordinal ?? 0; };


globalThis.CollectionsKt = {
    createListBuilder(capacity = 0) {
        return [];
    },
    build(builder) {
        return builder;
    },
    mutableListOf(...args) {
        return args.length === 1 && Array.isArray(args[0])
            ? [...args[0]]
            : Array.from(args);
    },
    joinToString$default(collection, separator, prefix, postfix, limit, truncated, transform) {
        separator = separator ?? ", ";
        prefix    = prefix    ?? "";
        postfix   = postfix   ?? "";
        limit     = limit     ?? -1;
        truncated = truncated ?? "...";
        let items = Array.from(collection);
        let over  = false;
        if (limit >= 0 && items.length > limit) { items = items.slice(0, limit); over = true; }
        const parts = items.map(x => transform ? transform(x) : String(x));
        if (over) parts.push(truncated);
        return prefix + parts.join(separator) + postfix;
    },
    throwIndexOverflow() { throw new RangeError("Index overflow"); },
    listOf(...args) { return args.length === 1 && Array.isArray(args[0]) ? args[0] : Array.from(args); },
    toList(collection) { return Array.from(collection); },
    listOfNotNull(...args) { return args.flat().filter(x => x != null); },
    collectionSizeOrDefault(collection, default_) {
        try {
            return collection?.length ?? default_;
        } catch(e) {
            return default_;
        }
    },
};

globalThis.Pair = class Pair {
    constructor(first, second) {
        this.first = first;
        this.second = second;
    }

    getFirst() {
        return this.first;
    }

    getSecond() {
        return this.second;
    }

    component1() {
        return this.first;
    }

    component2() {
        return this.second;
    }

    toString() {
        return `(${this.first}, ${this.second})`;
    }
};

// TODO: remove when settings collected
globalThis.__settings = {
    domain_pref: "0",
};

globalThis.Dispatchers = {
    getIO()      { return { type: "IO" }; },
    getMain()    { return { type: "Main" }; },
    getDefault() { return { type: "Default" }; },
};

globalThis.BuildersKt = {
    launch$default(scope, context, start, block) {
        if (typeof block === "function") {
            Promise.resolve().then(() => block());
        } else if (block?.invoke) {
            Promise.resolve().then(() => block.invoke(scope, null));
        }
    },

    runBlocking(ctx, block) {
        let result;

        // Don't call invoke() which re-calls create() and may wipe state
        // Call invokeSuspend directly since we're not doing real coroutine suspension
        if (typeof block?.invokeSuspend === "function") {
            result = block.invokeSuspend(Unit_INSTANCE);
        } else if (typeof block === "function") {
            result = block();
        } else {
            throw new Error("runBlocking: invalid block");
        }

        if (result === COROUTINE_SUSPENDED || result === _COROUTINE_SUSPENDED) {
            throw new Error("Real coroutine suspension not supported in this runtime");
        }

        return result;
    }
};

globalThis.StringBuilder = class StringBuilder {
    constructor(initial = "") { this._s = (initial == null) ? "" : "" + initial; }
    toString() { return this._s; }
    get length() { return this._s.length; }
};
StringBuilder.prototype.append = function(v, start, end) {
    const s = (v == null) ? "" : "" + v;
    this._s += (start !== undefined) ? s.slice(start, end) : s;
    return this;
};

globalThis.ArraysKt = {
    plus(arr, element) {
        const a = Array.isArray(arr) ? arr : Array.from(arr ?? []);
        return Array.isArray(element) ? [...a, ...element] : [...a, element];
    },
};

globalThis.ArrayList = class ArrayList {
    constructor() { this._a = []; }
    push(item) { if (item !== 0 && item != null) this._a.push(item); return this; }
    add(item)    { this._a.push(item); return true; }
    get(i)       { return this._a[i]; }
    size()       { return this._a.length; }
    isEmpty()    { return this._a.length === 0; }
    toArray()    { return [...this._a]; }
    [Symbol.iterator]() { return this._a[Symbol.iterator](); }
    map(fn) { return this._a.map(fn); }
    iterator() {
        let i = 0; const a = this._a;
        return { hasNext() { return i < a.length; }, next() { return a[i++]; } };
    }
};

globalThis.kotlin = { Unit: { INSTANCE: undefined } };
globalThis.Unit = { INSTANCE: { toString() { return "kotlin.Unit"; } } };

globalThis.java = {
    util: { Locale: { ROOT: "root" } },
};

java.util.Locale = globalThis.Locale;

globalThis.Locale = {
    ENGLISH: "en",
    ROOT:    "root",
    US:      "en-US",
    getDefault() { return "en"; },
};

globalThis.FormBody_Builder = class FormBody_Builder {
    constructor() { this._p = []; }
    add(k, v) { this._p.push([k, v]); return this; }
    build()   { return Object.fromEntries(this._p); }
};

globalThis.JsonTransformingSerializer = class JsonTransformingSerializer {
    constructor(tSerializer) {
        this.tSerializer = tSerializer;
    }

    transformDeserialize(element) {
        return element;
    }

    transformSerialize(element) {
        return element;
    }

    deserialize(decoder) {
        return decoder;
    }

    serialize(encoder, value) {
        return value;
    }
};

// kotlinx.serialization stubs
globalThis.PluginGeneratedSerialDescriptor = class PluginGeneratedSerialDescriptor {
    constructor(name, plugin, elementsCount) {
        this.serialName = name;
        this._plugin = plugin;
        this._elementsCount = elementsCount;

        this._elements = [];
        this._indices = new Map();
    }

    addElement(name, isOptional) {
        const index = this._elements.length;

        this._elements.push({
            name,
            optional: !!isOptional,
        });

        this._indices.set(name, index);
    }

    getElementName(i) {
        return this._elements[i]?.name ?? `element${i}`;
    }

    getElementIndex(name) {
        return this._indices.has(name)
            ? this._indices.get(name)
            : -1;
    }

    elementsCount() {
        return this._elements.length || this._elementsCount || 0;
    }

    isElementOptional(i) {
        return this._elements[i]?.optional ?? true;
    }

    getElementAnnotations(i) {
        return [];
    }

    getAnnotations() {
        return [];
    }

    getElementDescriptor(i) {
        return this;
    }

    toString() {
        return this.serialName;
    }
};

if (!PluginGeneratedSerialDescriptor.prototype.encodeSerializableElement) {
    PluginGeneratedSerialDescriptor.prototype.encodeSerializableElement = function() {};
    PluginGeneratedSerialDescriptor.prototype.encodeStringElement        = function() {};
    PluginGeneratedSerialDescriptor.prototype.decodeSerializableElement  = function() { return null; };
    PluginGeneratedSerialDescriptor.prototype.decodeStringElement        = function() { return ""; };
}

globalThis.PluginExceptionsKt = {
    throwMissingFieldException(seenBits, requiresBits, descriptor) {
        throw new Error(`Missing required field in ${descriptor?.serialName ?? "unknown"}`);
    },
};

globalThis.ArrayListSerializer = class ArrayListSerializer {
    constructor(elementSerializer) { this._el = elementSerializer; }
};

// Kotlin lazy / threading

globalThis.kotlin = globalThis.kotlin ?? {};
kotlin.LazyThreadSafetyMode = {
    PUBLICATION:    { name: "PUBLICATION" },
    SYNCHRONIZED:   { name: "SYNCHRONIZED" },
    NONE:           { name: "NONE" },
};

globalThis.LazyThreadSafetyMode = kotlin.LazyThreadSafetyMode;

globalThis.LazyKt = {
    lazy(mode, initializer) {
        let _value;
        let _init = true;
        return {
            get value() {
                if (_init) { _value = initializer(); _init = false; }
                return _value;
            },
            isInitialized() { return !_init; },
            invoke(...args) {
                if (_init) { _value = initializer(); _init = false; }
                return _value;
            },
        };
    },
};

// Kotlin Pair / TuplesKt

globalThis.TuplesKt = {
    to(first, second) { return { first, second, getFirst() { return first; }, getSecond() { return second; } }; },
};

// Kotlin number boxing

// intValue() appears when an Integer box is used
if (!Number.prototype.intValue) {
    Number.prototype.intValue  = function() { return this | 0; };
    Number.prototype.longValue = function() { return this; };
}

// java.util / Calendar / date shims

globalThis.KTypeProjection = {
    Companion: {
        invariant(type) { return { variance: "INVARIANT", type }; },
        covariant(type) { return { variance: "COVARIANT", type }; },
        contravariant(type) { return { variance: "CONTRAVARIANT", type }; },
        STAR: { variance: "STAR", type: null },
    },
};

globalThis.Reflection = {
    typeOf(cls, ...projections) { return { classifier: cls, arguments: projections }; },
};

// java.util.List as a type token
globalThis.java = globalThis.java ?? {};
globalThis.java.util = globalThis.java.util ?? {};
globalThis.java.util.List = { _type: "List" };

java.util.Calendar = {
    getInstance() {
        const now = new Date();
        return {
            _date: now,
            get(field) {
                switch(field) {
                    case java.util.Calendar.YEAR:         return now.getFullYear();
                    case java.util.Calendar.MONTH:        return now.getMonth();
                    case java.util.Calendar.DAY_OF_MONTH: return now.getDate();
                    case java.util.Calendar.HOUR_OF_DAY:  return now.getHours();
                    case java.util.Calendar.MINUTE:       return now.getMinutes();
                    case java.util.Calendar.SECOND:       return now.getSeconds();
                    default: return 0;
                }
            },
            getTimeInMillis() { return now.getTime(); },
            getTime()         { return now; },
        };
    },
    YEAR:         1,
    MONTH:        2,
    DAY_OF_MONTH: 5,
    HOUR_OF_DAY:  11,
    MINUTE:       12,
    SECOND:       13,
};

java.util.concurrent = {
    TimeUnit: {
        SECONDS:      { toMillis(v) { return v * 1000; } },
        MINUTES:      { toMillis(v) { return v * 60000; } },
        HOURS:        { toMillis(v) { return v * 3600000; } },
        MILLISECONDS: { toMillis(v) { return v; } },
    },
};

globalThis.Calendar = java.util.Calendar;
globalThis.TimeUnit = java.util.concurrent.TimeUnit;

globalThis.SimpleDateFormat = class SimpleDateFormat {
    constructor(pattern, locale) { this._pattern = pattern; }
    parse(str) {
        const ms = Date.parse(str);
        return isNaN(ms) ? new Date(0) : new Date(ms);
    }
    format(date) { return (date instanceof Date ? date : new Date(date)).toISOString(); }
    setTimeZone(tz) { this._tz = tz; }
};

// Kotlin ranges
globalThis.RangesKt = {
    downTo(from, to) {
        return {
            iterator() {
                let i = from;

                return {
                    hasNext() {
                        return i >= to ? true : 0;
                    },

                    nextInt() {
                        return i--;
                    },

                    next() {
                        return this.nextInt();
                    },
                };
            },

            [Symbol.iterator]() {
                let i = from;

                return {
                    next() {
                        return i >= to
                            ? { value: i--, done: false }
                            : { done: true };
                    },
                };
            },
        };
    },

    until(from, to) {
        return {
            iterator() {
                let i = from;

                return {
                    hasNext() {
                        return i < to ? true : 0;
                    },

                    nextInt() {
                        return i++;
                    },

                    next() {
                        return this.nextInt();
                    },
                };
            },

            [Symbol.iterator]() {
                let i = from;

                return {
                    next() {
                        return i < to
                            ? { value: i++, done: false }
                            : { done: true };
                    },
                };
            },
        };
    },

    step(range, step) {
        return range;
    },

    coerceAtMost(value, max) {
        return value > max ? max : value;
    },

    coerceAtLeast(value, min) {
        return value < min ? min : value;
    },

    coerceIn(value, min, max) {
        return value < min
            ? min
            : value > max
                ? max
                : value;
    },
};

// Kotlin Regex
globalThis.Regex = class Regex {
    constructor(pattern, options) {
        this._pattern = pattern;
        this._flags   = options ?? "";
        this._re      = new RegExp(pattern, this._flags);
    }
    containsMatchIn(str)  { return this._re.test(str); }
    matches(str)          { return new RegExp(`^(?:${this._pattern})$`).test(str); }
    find(str, start = 0)  {
        const re = new RegExp(this._pattern, "g" + this._flags.replace("g",""));
        re.lastIndex = start;
        const m = re.exec(str);
        if (!m) return null;
        return { value: m[0], groupValues: m, destructured: { component1: () => m[1] } };
    }
    findAll(str, start = 0) {
        const re = new RegExp(this._pattern, "g" + this._flags.replace("g",""));
        re.lastIndex = start;
        const results = [];
        let m;
        while ((m = re.exec(str)) !== null) {
            results.push({ value: m[0], groupValues: m });
        }
        return results;
    }
    replace(str, replacement)    { return str.replace(this._re, replacement); }
    replaceAll(str, replacement) { return str.replaceAll(new RegExp(this._pattern, "g"), replacement); }
    split(str)                   { return str.split(this._re); }
    toString()                   { return this._pattern; }
};

// Android Preferences stubs
globalThis.SwitchPreferenceCompat = class SwitchPreferenceCompat {
    constructor(context) {}
    setKey(v)          { this._key     = v; return this; }
    setTitle(v)        { this._title   = v; return this; }
    setSummary(v)      { this._summary = v; return this; }
    setDefaultValue(v) { this._default = v; return this; }
    setOnPreferenceChangeListener(l) { this._listener = l; return this; }
    // called on the preference screen
    key()     { return this._key; }
    title()   { return this._title; }
};

// Common preference types you'll likely hit too
globalThis.ListPreference = class ListPreference {
    constructor(context) {}
    setKey(v)          { this._key     = v; return this; }
    setTitle(v)        { this._title   = v; return this; }
    setSummary(v)      { this._summary = v; return this; }
    setEntries(v)      { this._entries = v; return this; }
    setEntryValues(v)  { this._values  = v; return this; }
    setDefaultValue(v) { this._default = v; return this; }
    setOnPreferenceChangeListener(l) {}
};

globalThis.EditTextPreference = class EditTextPreference {
    constructor(context) {}
    setKey(v)          { this._key     = v; return this; }
    setTitle(v)        { this._title   = v; return this; }
    setSummary(v)      { this._summary = v; return this; }
    setDefaultValue(v) { this._default = v; return this; }
    setOnPreferenceChangeListener(l) {}
};

// PreferenceScreen / PreferenceGroup
globalThis.PreferenceScreen = class PreferenceScreen {
    constructor() { this._prefs = []; }
    addPreference(p) { this._prefs.push(p); }
    getPreferences()  { return this._prefs; }
};

globalThis.Application = class Application {
    constructor() {
        this._prefs = new Map();
    }

    getSharedPreferences(name, mode) {
        let pref = this._prefs.get(name);
        if (!pref) {
            pref = new SharedPreferences(name);
            this._prefs.set(name, pref);
        }
        return pref;
    }
};

globalThis.SharedPreferences = class SharedPreferences {
    constructor(name) {
        this.name = name;
    }

    _get(key, def) {
        const v = __settings?.[key];
        console.log("SharedPreferences._get:", key, "->", v, "(default:", def, ")");
        return v === undefined ? def : v;
    }

    getString(key, def) {
        return this._get(key, def);
    }

    getBoolean(key, def) {
        return this._get(key, def);
    }

    getInt(key, def) {
        return this._get(key, def);
    }

    edit() {
        return new SharedPreferencesEditor(this);
    }
};

globalThis.SharedPreferencesEditor = class SharedPreferencesEditor {
    constructor(prefs) {
        this.prefs = prefs;
    }

    putString() { return this; }
    putBoolean() { return this; }
    putInt() { return this; }

    apply() {}
    commit() { return true; }
};

// java.lang boxing

globalThis.java = globalThis.java ?? {};
java.lang = java.lang ?? {};
java.lang.Boolean = {
    TRUE:  true,
    FALSE: false,
    valueOf(v) { return !!v; },
};
java.lang.Integer = {
    valueOf(v)    { return v | 0; },
    parseInt(v)   { return parseInt(v, 10); },
    MAX_VALUE:    2147483647,
    MIN_VALUE:    -2147483648,
};
globalThis.Integer = java.lang.Integer;
globalThis.String.valueOf = (v) => String(v);
globalThis.String.format  = (fmt, ...args) => {
    let i = 0;
    return fmt.replace(/%[sdf]/g, () => String(args[i++]));
};

// NetworkHelper shim
// cloudflareClient → real fetch via headless (handles CF challenges)
// client / nonCloudflareClient → plain fetch

const _makeOkHttpClient = (useHeadless, interceptors = [], networkInterceptors = []) => ({
    _useHeadless: useHeadless,
    _interceptors: interceptors,
    _networkInterceptors: networkInterceptors,

    newCall(request) {
        return new _Call(request);
    },

    newBuilder() {
        return _makeOkHttpClientBuilder(useHeadless, [...this._interceptors], [...this._networkInterceptors]);
    },
    interceptors()        { return _makeKotlinList(this._interceptors); },
    networkInterceptors() { return _makeKotlinList(this._networkInterceptors); },
});
const _makeOkHttpClientBuilder = (useHeadless = false, interceptors = [], networkInterceptors = []) => ({
    _useHeadless: useHeadless,
    _interceptors: interceptors,
    _networkInterceptors: networkInterceptors,

    addInterceptor(i)        { this._interceptors.push(i);        return this; },
    addNetworkInterceptor(i) { this._networkInterceptors.push(i); return this; },
    interceptors()        { return _makeKotlinList(this._interceptors); },
    networkInterceptors() { return _makeKotlinList(this._networkInterceptors); },

    // builder fluent no-ops
    cookieJar(v)        { return this; },
    connectTimeout(...a){ return this; },
    readTimeout(...a)   { return this; },
    callTimeout(...a)   { return this; },
    cache(v)            { return this; },
    rateLimitHost(...a) { return this; },

    // DoH no-ops
    dohCloudflare()  { return this; },
    dohGoogle()      { return this; },
    dohAdGuard()     { return this; },
    dohQuad9()       { return this; },
    dohAliDNS()      { return this; },
    dohDNSPod()      { return this; },
    doh360()         { return this; },
    dohQuad101()     { return this; },
    dohMullvad()     { return this; },
    dohControlD()    { return this; },
    dohNajalla()     { return this; },
    dohShecan()      { return this; },

    apply(fn) { fn.call(this); return this; },

    build() { return _makeOkHttpClient(this._useHeadless); },
});

const _makeKotlinList = (arr) => ({
    _arr: arr,
    iterator() {
        let i = 0; const a = arr;
        return { hasNext() { return i < a.length; }, next() { return a[i++]; } };
    },
    indexOfFirst(pred) { return arr.findIndex(pred); },
    removeAt(i)        { return arr.splice(i, 1)[0]; },
    add(item)          { arr.push(item); return true; },
    get size()         { return arr.length; },
    [Symbol.iterator]() { return arr[Symbol.iterator](); },
});

// BrotliInterceptor — no-op sentinel (the extension just looks for it by instanceof)
class BrotliInterceptor {}
globalThis.BrotliInterceptor = new BrotliInterceptor();

globalThis.okhttp3 = {
    brotli: { BrotliInterceptor: BrotliInterceptor },
};

globalThis.MediaType = {
    Companion: {
        get(str) { return { _type: str, toString() { return str; } }; },
        parse(str) { return { _type: str, toString() { return str; } }; },
    },
};

// j0 is the extension's NetworkInterceptor lambda (chain.proceed wrapper)
// We don't need to execute it; the real fetch handles headers.

const _cookieStore = new Map(); // persisted via state below

const _networkHelper = {
    cookieJar: {
        saveFromResponse(url, cookies) {
            for (const c of cookies) _cookieStore.set(c.name, c.value);
            state?.set?.("cookies", Object.fromEntries(_cookieStore));
        },
        loadForRequest(url) { return []; },
    },

    get cloudflareClient() { return _makeOkHttpClient(true);  },
    get client()            { return _makeOkHttpClient(true);  },
    get nonCloudflareClient(){ return _makeOkHttpClient(false); },
};

globalThis.NetworkHelper = function NetworkHelper() { return _networkHelper; };

globalThis.Request = class Request {
    constructor(url, method, headers, body) {
        this._url = url;
        this._method = method ?? "GET";
        this._headers = headers ?? new Headers();
        this._body = body ?? null;
    }
    url()     { return this._url; }
    method()  { return this._method; }
    headers() { return this._headers; }
    body()    { return this._body; }
    newBuilder() { return new Request.Builder(this); }

    static Builder = class RequestBuilder {
        constructor(req) {
            this._url     = req?._url ?? "";
            this._method  = req?._method ?? "GET";
            this._headers = req?._headers ?? new Headers();
            this._body    = req?._body ?? null;
        }
        url(u)           { this._url = u?.toString?.() ?? u; return this; }
        method(m, body)  { this._method = m; this._body = body; return this; }
        header(k, v)     { this._headers.set(k, v); return this; }
        addHeader(k, v)  { this._headers.set(k, v); return this; }
        removeHeader(k)  { this._headers.delete(k); return this; }
        post(body)       { this._method = "POST"; this._body = body; return this; }
        get()            { this._method = "GET"; return this; }
        cacheControl(c)  { return this; }
        build() {
            return new Request(this._url, this._method, this._headers, this._body);
        }
    };
};

function _wrapKotlinObject(obj) {
    if (obj === null || obj === undefined) return null;
    if (Array.isArray(obj)) return obj.map(_wrapKotlinObject);
    if (typeof obj !== "object") return obj;
    if (typeof obj.toSManga === "function") return obj;

    return new Proxy(obj, {
        get(target, prop) {
            if (prop === Symbol.iterator) return target[Symbol.iterator]?.bind(target);
            if (prop === "iterator") return () => {
                const arr = Array.isArray(target) ? target : Object.values(target);
                let i = 0;
                return {
                    hasNext: () => i < arr.length,
                    next: () => _wrapKotlinObject(arr[i++]),
                };
            };
            if (prop in target) {
                const val = target[prop];
                if (typeof val === "function") return val.bind(target);
                return () => _wrapKotlinObject(val);
            }
            return () => null;
        }
    });
}

globalThis.OkioStreamsKt = {
    decodeFromBufferedSource(deserializer, type, source) {
        try {
            const text = typeof source === "string" ? source : source?._text ?? "";
            const parsed = JSON.parse(text);

            // Add toSManga to each item in data array directly
            if (parsed.data && Array.isArray(parsed.data)) {
                parsed.data = parsed.data.map(item => {
                    const wrapped = { ...item };
                    Object.defineProperty(wrapped, 'toSManga', {
                        value: function() {
                            const m = SManga.create();
                            m.url = `/comic/${item.slug}`;
                            m.title = item.title ?? "";
                            m.thumbnail_url = item.default_thumbnail ?? null;
                            m.status = SManga.UNKNOWN;
                            return m;
                        },
                        writable: true,
                        configurable: true,
                        enumerable: true,
                    });
                    return wrapped;
                });
            }

            return _wrapKotlinObject(parsed);
        } catch(e) {
            return _wrapKotlinObject({ data: [], d: null });
        }
    },
};

globalThis.ResponseBody = class ResponseBody {
    constructor(text) { this._text = text; }
    source()  { return this; }
    string()  { return this._text; }
};

// What the extension actually calls: this.getNetwork()
// HttpSource base sets this up; we patch getNetwork() onto the prototype chain.
if (globalThis.HttpSource) {
    HttpSource.prototype.getNetwork = function() { return _networkHelper; };
}
// Also cover pre-construction access via a global fallback
globalThis.getNetwork = () => _networkHelper;

// Rate limit interceptor, no-op in sandbox

globalThis.SpecificHostRateLimitInterceptorKt = {
    rateLimitHost(client, host, permits, period, unit, ...rest) {
        return client ?? rest[0] ?? { build() { return _makeOkHttpClient(false); } };
    },
};

globalThis.SerializersKt = {
    serializer(klass) { return klass?.Companion ?? klass; },
};
globalThis.BuiltinSerializersKt = {
    ListSerializer(s) { return { _type: "List", _elem: s }; },
    ArrayListSerializer(s) { return { _type: "ArrayList", _elem: s }; },
    NullableSerializer(s) { return { _type: "Nullable", _elem: s }; },
};

globalThis.Json = {
    Default: {
        decodeFromString(deserializer, str) { return JSON.parse(str); },
        encodeToString(serializer, obj)     { return JSON.stringify(obj); },
    },
    decodeFromString(deserializer, str) { return JSON.parse(str); },
    encodeToString(serializer, obj)     { return JSON.stringify(obj); },
};

function _jsonToSManga(item) {
    const manga = SManga.create();
    manga.url         = `/comic/${item.slug}`;
    manga.title       = item.title ?? "";
    manga.thumbnail_url = item.cover ?? item.thumbnail ?? item.cover_url ?? null;
    manga.status      = SManga.UNKNOWN;
    return manga;
}

// Tachiyomi models

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

globalThis.Filter = class Filter {
    constructor(name, state) {
        this.name  = name;
        this.state = state;
    }

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
};

Filter.Sort.Selection = class Selection {
    constructor(index, ascending) {
        this.index     = index;
        this.ascending = ascending;
    }
};

globalThis["Filter$Header"]    = Filter.Header;
globalThis["Filter$Separator"] = Filter.Separator;
globalThis["Filter$Select"]    = Filter.Select;
globalThis["Filter$Text"]      = Filter.Text;
globalThis["Filter$CheckBox"]  = Filter.CheckBox;
globalThis["Filter$TriState"]  = Filter.TriState;
globalThis["Filter$Group"]     = Filter.Group;
globalThis["Filter$Sort"]      = Filter.Sort;
globalThis["Filter$Sort$Selection"] = Filter.Sort.Selection;

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

    equals() { return false; }
};

// Response wrapper

globalThis._SandboxResponse = class _SandboxResponse {
    constructor(text, status, url) {
        this._text   = text;
        this._status = status;
        this._url    = url;
    }
    body()    { return new ResponseBody(this._text); }
    code()         { return this._status; }
    isSuccessful() { return this._status >= 200 && this._status < 300; }
    header(name) { return null; }
    async text() { return this._text; }
    async json() { return JSON.parse(this._text); }
    request() {
        return {
            url() {
                return {
                    fragment() { return null; },
                    toString() { return this._url; },
                    toUrl()    { return { toString() { return this._url; } }; },
                };
            },
            header(name) { return null; },
        };
    }
    // asJsoup() comes later
}

// GET helper (used inline by extensions)

globalThis.GET = function GET(url, headers) {
    return { url: url?.toString?.() ?? url, headers, method: "GET" };
};

//  OkHttp shims

globalThis.Headers = class Headers {
    constructor(map = {}) { this._map = { ...map }; }

    get(name)          { return this._map[name.toLowerCase()] ?? null; }
    set(name, value)   { this._map[name.toLowerCase()] = value; }
    has(name)          { return name.toLowerCase() in this._map; }

    toFetchHeaders()   { return { ...this._map }; }

    static Builder = class HeadersBuilder {
        constructor() { this._map = {}; }
        add(name, value)  { this._map[name.toLowerCase()] = value; return this; }
        set(name, value)  { this._map[name.toLowerCase()] = value; return this; }
        build()           { return new Headers(this._map); }
    };
};

// CacheControl
globalThis.CacheControl = {
    FORCE_NETWORK: { noCache: true },
    FORCE_CACHE:   { onlyIfCached: true },

    Builder: class CacheControlBuilder {
        maxAge(v, unit) { return this; }
        noCache()       { return this; }
        build()         { return {}; }
    },
};

// Request
globalThis.Request = class Request {
    constructor(url, method, headers, body, cacheControl) {
        this.url          = url?.toString?.() ?? url;
        this.method       = method ?? "GET";
        this.headers      = headers ?? new Headers();
        this.body         = body   ?? null;
        this.cacheControl = cacheControl ?? null;
    }

    static Builder = class RequestBuilder {
        constructor() {
            this._url     = null;
            this._method  = "GET";
            this._headers = new Headers();
            this._body    = null;
            this._cache   = null;
        }
        url(v)           { this._url    = v?.toString?.() ?? v; return this; }
        headers(v)       { this._headers = v; return this; }
        addHeader(k, v)  { this._headers.set(k, v);            return this; }
        cacheControl(v)  { this._cache  = v;                   return this; }
        get(body)        { this._method = "GET";    this._body = body ?? null; return this; }
        post(body)       { this._method = "POST";   this._body = body;         return this; }
        put(body)        { this._method = "PUT";    this._body = body;         return this; }
        delete(body)     { this._method = "DELETE"; this._body = body ?? null; return this; }
        build() {
            return new Request(this._url, this._method, this._headers, this._body, this._cache);
        }
    };
};

// PreferencesKt — Kotlin generates an inline class named
// PreferencesKt$getPreferences$$inlined$get$1 at every call-site of
// Injekt.get<Application>(). We register a global proxy so that
// `new PreferencesKt$getPreferences$$inlined$get$1()` works and
// getType() returns a token that InjektKt.getInstance() resolves to
// the Application singleton (which itself delegates to __settings).
(function() {
    const _token = { _ctor: null }; // filled after Application is defined

    function _makePrefsTypeToken() {
        // Lazily resolve Application so definition order doesn't matter.
        if (!_token._ctor) _token._ctor = globalThis.Application ?? null;
        return _token;
    }

    const _PrefsInlined = function PreferencesKt$getPreferences$$inlined$get$1() {
        this._token = _makePrefsTypeToken();
    };
    _PrefsInlined.prototype.getType  = function() { return _makePrefsTypeToken(); };
    _PrefsInlined.prototype.invoke   = function() { return _makePrefsTypeToken(); };
    _PrefsInlined.prototype.toString = function() { return "PreferencesKt$get$1"; };

    globalThis["PreferencesKt$getPreferences$$inlined$get$1"] = _PrefsInlined;
    // Some builds use a shorter name; cover both.
    globalThis["PreferencesKt$get$1"] = _PrefsInlined;
    globalThis.PreferencesKt = globalThis.PreferencesKt ?? {
        getPreferences(context, name) {
            const app = globalThis.__appInstance ||= new Application();
            return app.getSharedPreferences(name ?? "prefs", 0);
        },
    };
})();

globalThis.InjektKt = class InjektKt {
    static getInjekt() {
        return {
            getInstance: (cls) => {
                // Kotlin's Injekt passes a type token (FullTypeReference / array) rather
                // than a plain constructor when the call-site uses an inline reified get<T>().
                // Unwrap: if it's an array, take the first element; if it has a _ctor field
                // (set by PreferencesKt shim below), use that.
                let ctor = cls;
                if (Array.isArray(cls))   ctor = cls[0];
                if (cls && cls._ctor)     ctor = cls._ctor;

                // Application is the only type extensions request via Injekt in practice —
                // it's the gateway to getSharedPreferences(), which reads __settings.
                if (ctor === Application || ctor == null || typeof ctor !== "function") {
                    return globalThis.__appInstance ||= new Application();
                }
                return new ctor();
            }
        };
    }
};
// RequestsKt
globalThis.RequestsKt = {
    GET(url, headers, cache) {
        return new Request.Builder().url(url).headers(headers ?? new Headers()).cacheControl(cache ?? null).build();
    },
    GET$default(url, headers, cache, flags, mask) {
        return new Request.Builder().url(url).headers(headers ?? new Headers()).cacheControl(cache ?? null).build();
    },
    POST$default(url, headers, body) {
        return new Request.Builder().url(url).headers(headers ?? new Headers()).post(body).build();
    },
};

globalThis.GET  = (url, headers, cache) => RequestsKt.GET(url, headers, cache);
globalThis.POST = (url, headers, body, cache) =>
    new Request.Builder().url(url).headers(headers ?? new Headers()).post(body).build();

// HttpUrl
globalThis.HttpUrl = class HttpUrl {
    constructor(url) { this._url = url; }
    toString()       { return this._url; }
    newBuilder()     { return new HttpUrl.Builder(this._url); }

    static Builder = class HttpUrlBuilder {
        constructor(base = "") { this._url = base; }
        addQueryParameter(k, v) {
            const sep = this._url.includes("?") ? "&" : "?";
            this._url += `${sep}${encodeURIComponent(k)}=${encodeURIComponent(v)}`;
            return this;
        }
        fragment(v) { this._url += `#${v}`; return this; }
        addPathSegment(v) { this._url += `/${v}`; return this; }
        build()           { return new HttpUrl(this._url); }
        toString()        { return this._url; }
    };
};

HttpUrl.Companion = {
    get(url) {
        return new HttpUrl(url?.toString?.() ?? url);
    },
};

// String.toHttpUrl() extension
globalThis.toHttpUrl = (str) => new HttpUrl(str.toString());

// OkHttpClient — newCall() returns a fake Call that _doRequest handles
globalThis.OkHttpClient = class OkHttpClient {
    newCall(request) {
        return new _Call(request);
    }
};

globalThis._Call = class _Call {
    constructor(req) { this._req = req; }

    execute() {
        // synchronous fetch isn't possible in JS, but runBlocking expects sync
        // store the request so the sandbox can dispatch it
        const url = this._req.url?.toString?.() ?? String(this._req.url);
        const method = this._req.method ?? "GET";
        const headers = this._req.headers?.toFetchHeaders?.() ?? {};

        // Return a response-like object that will be resolved by the sandbox
        return fetch(url, { method, headers })
            .then(res => res.text().then(text => new _SandboxResponse(text, res.status, url)));
    }
}

globalThis.firstInstance = function(iterator, predicate) {
    while (iterator.hasNext !== undefined ? iterator.hasNext() : false) {
        const item = iterator.next();
        if (predicate(item)) return item;
    }
    return null;
};

globalThis.CloseableKt = {
    closeFinally(closeable, cause) {
        try { closeable?.close?.(); } catch(e) {}
    },
};

// TODO: make sync fetch
globalThis.OkHttpExtensionsKt = {
    await(call, continuation) {
        // Return a fake failed response — extension will use cached/default filters
        return new _SandboxResponse("", 503, "");
    }
};

const COROUTINE_SUSPENDED = Symbol("COROUTINE_SUSPENDED");
globalThis.SuspendLambda = class SuspendLambda {
    constructor(arity, completion) {
        this.arity = arity;
        this.completion = completion || null;

        this.a_val = null;
        this.b_val = 0;

        this.label = 0;
    }

    create(value, completion) {
        this.completion = completion;
        return this;
    }

    invoke(p1, p2) {
        // p1 is the value/scope, p2 is the completion
        // Only set completion, don't reconstruct — captured fields are already set
        if (p2 !== undefined && p2 !== null) {
            this.completion = p2;
        }
        return this.invokeSuspend(Unit_INSTANCE);
    }

    resumeWith(result) {
        let current = this;
        let param = result;

        while (current) {
            try {
                const outcome = current.invokeSuspend(param);

                if (outcome === COROUTINE_SUSPENDED) {
                    return COROUTINE_SUSPENDED;
                }

                param = outcome;
            } catch (e) {
                param = e;
            }

            current = current.completion;
        }

        return param;
    }

    invokeSuspend(result) {
        return Unit_INSTANCE;
    }
};

globalThis.CoroutineImpl = globalThis.SuspendLambda;

const _COROUTINE_SUSPENDED = Symbol("COROUTINE_SUSPENDED");

globalThis.IntrinsicsKt = {
    getCOROUTINE_SUSPENDED() { return _COROUTINE_SUSPENDED; },
};


globalThis.ResultKt = {
    throwOnFailure(result) {
        if (result && result.__isFailure) {
            throw result.cause ?? new Error("Coroutine failed");
        }
    },
};

globalThis.FullTypeReference = class FullTypeReference {
    constructor(...args) {
        this._typeArgs = args;
        // If the first arg is a constructor, stash it so getInstance() can find it.
        this._ctor = (typeof args[0] === "function") ? args[0] : null;
    }

    getType() {
        // Return `this` so that getInstance() can inspect _ctor directly,
        // rather than returning the raw args array (which is not a constructor).
        return this;
    }

    toString() {
        return "FullTypeReference";
    }
};

const Unit_INSTANCE = { toString() { return "kotlin.Unit"; } };
globalThis.Unit_INSTANCE = Unit_INSTANCE;
if (!globalThis.kotlin) globalThis.kotlin = {};
if (!globalThis.kotlin.Unit) globalThis.kotlin.Unit = { INSTANCE: Unit_INSTANCE };

// IllegalStateException
globalThis.IllegalStateException = class IllegalStateException extends Error {
    constructor(msg) {
        super(typeof msg === "string" ? msg : msg?.toString?.() ?? "Illegal state");
        this.name = "IllegalStateException";
    }
};


// Override in HttpSource to unwrap Headers and HttpUrl properly
globalThis._origDoRequest = Manga.prototype._doRequest;

Manga.prototype._doRequest = async function(req) {
    // req may be a Request object or a _Call — unwrap if needed
    const r = req instanceof _Call ? req._req : req;

    const url    = r.url?.toString?.() ?? String(r.url);
    const method = r.method ?? "GET";
    const headers = r.headers?.toFetchHeaders?.() ?? r.headers ?? this.headers?.toFetchHeaders?.() ?? this.headers ?? {};
    const body   = r.body   ?? undefined;

    const res  = await fetch(url, { method, headers, body });
    if (!res.ok) throw new Error(`HTTP ${res.status}: ${url}`);
    const text = await res.text();
    return new _SandboxResponse(text, res.status, url);
};

if (!Number.prototype.toSManga) {
    Number.prototype.toSManga = function() { return 0; };
}

//  HttpSource

globalThis._SandboxManga = Manga;
class HttpSource extends _SandboxManga {

    get lang()      { return "en"; }
    get name()      { return "Unknown"; }
    get versionId() { return 1; }

    headersBuilder() {
        return { "User-Agent": "Mozilla/5.0 (compatible; TachiSandbox/1.0)" };
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

    //  sandbox API entry points 

    async search(query, filters, page) {
        page = page ?? 1;
        if (!query) {
            const res = await this._doRequest(this.popularMangaRequest(page));
            return this._parsePage(this.popularMangaParse(res));
        }
        const res = await this._doRequest(
            this.searchMangaRequest(page, query, this.getFilterList())
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
        return list.map((ch, i) => ({
            id:     ch.url,
            title:  ch.name,
            number: ch.chapter_number >= 0 ? ch.chapter_number : list.length - i,
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
    mangaDetailsRequest(manga)                       { return GET(this.baseUrl + manga.url, this.headers); }
    mangaDetailsParse(response)                      { throw new Error("not implemented"); }
    chapterListRequest(manga)                        { return GET(this.baseUrl + manga.url, this.headers); }
    chapterListParse(response)                       { throw new Error("not implemented"); }
    pageListRequest(chapter)                         { return GET(this.baseUrl + chapter.url, this.headers); }
    pageListParse(response)                          { throw new Error("not implemented"); }
    imageUrlParse(response)                          { throw new Error("not implemented"); }
    getFilterList()                                  { return new FilterList(); }

    //  internal 

    async _doRequest(req) {
        const url    = req.url?.toString?.() ?? String(req.url);
        console.log("base _doRequest url:", url, "type:", typeof req.url);

        const method = req.method ?? "GET";
        const headers = req.headers instanceof Object ? req.headers : this.headers;
        const body    = req.body ?? undefined;

        const res = await fetch(url, { method, headers, body });
        if (!res.ok) throw new Error(`HTTP ${res.status}: ${url}`);
        const text = await res.text();
        return new _SandboxResponse(text, res.status, url);
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
}

HttpSource.prototype.getHeaders = function() { if (!this._headers) this._headers = this.headersBuilder(); return this._headers;};

let __tachi_captured = null;

globalThis.HttpSource       = HttpSource;
globalThis.ParsedHttpSource = HttpSource;
globalThis.Manga            = HttpSource;
HttpSource.prototype.getNetwork = function() { return _networkHelper; };
_networkHelper.getCloudflareClient = function() { return _makeOkHttpClient(true); };
_networkHelper.getClient            = function() { return _makeOkHttpClient(true); };
_networkHelper.getNonCloudflareClient = function() { return _makeOkHttpClient(false); };

globalThis.__tachi_getCapturedClass = function() {
    if (__tachi_captured)
        return __tachi_captured;

    for (const key of Object.getOwnPropertyNames(globalThis)) {
        try {
            const v = globalThis[key];

            if (
                typeof v === "function" &&
                v !== HttpSource &&
                v.prototype instanceof HttpSource
            ) {
                __tachi_captured = v;
                return v;
            }
        } catch (_) {}
    }

    throw new Error(
        "[tachi-compat] No class extending HttpSource found"
    );
};