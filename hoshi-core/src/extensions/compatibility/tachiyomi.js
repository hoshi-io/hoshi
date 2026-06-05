const _cookieStore = new Map(Object.entries(state?.get?.("cookies") ?? {}));

const _cfStateByOrigin = new Map(Object.entries(state?.get?.("cf_state") ?? {}));

function _saveCfState() {
    state.set("cf_state", Object.fromEntries(_cfStateByOrigin));
}

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
Object.defineProperty(Array.prototype, 'firstInstance', {
    value: function(predicate) {
        const result = this.find(predicate);
        return result === undefined ? null : result;
    },
    enumerable: false,
    writable: true,
    configurable: true
});
Array.prototype.get = function(i) { return this[i]; };

// Polyfill to catch obfuscated Dalvik array lengths
Object.defineProperty(Array.prototype, 'length_val', {
    get: function() {
        return this.length;
    },
    enumerable: false,
    configurable: true
});

if (!String.prototype.hashCode) {
    String.prototype.hashCode = function() {
        let h = 0;
        for (let i = 0; i < this.length; i++) {
            h = (Math.imul(31, h) + this.charCodeAt(i)) | 0;
        }
        return h;
    };
}

if (!Number.prototype.hashCode) {
    Number.prototype.hashCode = function() { return this | 0; };
}

Object.defineProperty(Boolean.prototype, 'booleanValue', {
    value: function() {
        return this.valueOf() ? 1 : 0;
    },
    enumerable: false,
    writable: true,
    configurable: true
});

// Polyfill for Number just in case the transpiler has already unboxed it into an int
Object.defineProperty(Number.prototype, 'booleanValue', {
    value: function() {
        return this.valueOf() !== 0 ? 1 : 0;
    },
    enumerable: false,
    writable: true,
    configurable: true
});

globalThis.StringsKt = {

    split$default(str, delimiters, ignoreCase, limit, mask, marker) {
        if (str == null) return _makeKotlinList([]);
        const sep = Array.isArray(delimiters) ? delimiters[0] : delimiters;
        const parts = str.split(sep);
        const result = (limit && limit > 0) ? parts.slice(0, limit) : parts;
        return _makeKotlinList(result);
    },

    replaceFirst$default(str, oldValue, newValue, ignoreCase, mask, marker) {
        if (str == null) return str;
        if (ignoreCase) {
            const escapedOld = String(oldValue).replace(/[.*+?^${}()|[\]\\]/g, '\\$&');
            return str.replace(new RegExp(escapedOld, 'i'), newValue);
        }
        return str.replace(oldValue, newValue);
    },

    isBlank(str) {
        return (str == null || typeof str !== "string" || str.trim().length === 0) ? 1 : 0;
    },

    toIntOrNull(str) {
        if (str == null || typeof str !== "string") return null;
        const s = str.trim();
        // Kotlin toIntOrNull strictly expects digits (optional +/-), no trailing garbage
        if (/^[+-]?\d+$/.test(s)) {
            return parseInt(s, 10);
        }
        return null;
    },

    removeSuffix(str, suffix) {
        if (str == null) return str;
        return str.endsWith(suffix) ? str.slice(0, -suffix.length) : str;
    },

    substringBefore$default(str, delimiter, missingDelimiterValue, mask, marker) {
        if (str == null) return str;
        const idx = str.indexOf(delimiter);
        return idx === -1 ? (missingDelimiterValue ?? str) : str.slice(0, idx);
    },

    endsWith$default(str, suffix, ignoreCase, mask, marker) {
        if (str == null) return 0;
        let result = false;
        if (ignoreCase) {
            result = str.toLowerCase().endsWith(suffix.toLowerCase());
        } else {
            result = str.endsWith(suffix);
        }
        return result ? 1 : 0; // Return Dalvik boolean format
    },

    trim(str) {
        return (typeof str === "string") ? str.trim() : str;
    },

    append(sb, parts) {
        if (Array.isArray(parts)) {
            for (const p of parts) sb.append(p);
        } else {
            sb.append(parts);
        }
        return sb;
    }
};

globalThis.kotlin = globalThis.kotlin || {};
globalThis.kotlin.text = globalThis.kotlin.text || {};

globalThis.kotlin.text.StringsKt = globalThis.StringsKt;

globalThis.ParsePosition = class ParsePosition {
    constructor(index) {
        this._index = index;
        this._errorIndex = -1;
    }
    getIndex()        { return this._index; }
    setIndex(v)       { this._index = v; }
    getErrorIndex()   { return this._errorIndex; }
    setErrorIndex(v)  { this._errorIndex = v; }
};

globalThis.CharsKt = {
    isWhitespace(ch) { return /\s/.test(ch); },
};

globalThis.ordinal = function(v) { return typeof v === "number" ? v : v?.ordinal ?? 0; };


globalThis.CollectionsKt = {
    build(builder) {
        return builder;
    },
    joinToString$default(collection, separator, prefix, postfix, limit, truncated, transform, flags, marker) {
        if (!collection) return "";
        separator = separator ?? ", ";
        prefix    = prefix    ?? "";
        postfix   = postfix   ?? "";
        limit     = limit     ?? -1;
        truncated = truncated ?? "...";
        let items;
        if (Array.isArray(collection)) {
            items = collection.map(_wrapKotlinObject);
        } else if (collection[Symbol.iterator]) {
            items = Array.from(collection);
        } else {
            items = Object.values(collection).map(_wrapKotlinObject);
        }
        let over = false;
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
        if (!collection) return 0;
        return collection.length ?? default_;
    },
    addAll(collection, elements) {
        const items = Array.isArray(elements) ? elements : [...elements];
        for (const item of items) {
            if (Array.isArray(collection)) {
                collection.push(item);
            } else {
                collection.add(item);
            }
        }
        return true;
    },

    emptyMap:     () => new LinkedHashMap(),
    emptySet:     () => new Set(),
    setOf:        (...args) => new Set(args),
    mapOf:        (...args) => new LinkedHashMap(args),
    plus:         (a, b)   => [...a, ...b],
    single:       (list)   => { if (list.length !== 1) throw new Error("Expected single element"); return list[0]; },
    firstOrNull:  (list, pred) => pred ? (list.find(pred) ?? null) : (list[0] ?? null),
    filter:       (list, pred) => list.filter(pred),
    map:          (list, fn)   => list.map(fn),
    forEach:      (list, fn)   => list.forEach(fn),

    emptyList:     () => _mutableList(),
    toMutableList: (col) => _mutableList(col?.[Symbol.iterator] ? [...col] : []),
    mutableListOf: (...args) => _mutableList(args.length === 1 && Array.isArray(args[0]) ? args[0] : args),
    createListBuilder: () => _mutableList(),
};

function _mutableList(items = []) {
    const arr = Array.isArray(items) ? [...items] : [...items];
    arr.add    = (item) => { arr.push(item); return true; };
    arr.addAll = (col)  => {
        const els = Array.isArray(col) ? col : [...col];
        for (const e of els) arr.push(e);
        return true;
    };
    arr.remove = (item) => {
        const i = arr.indexOf(item);
        if (i >= 0) { arr.splice(i, 1); return true; }
        return false;
    };
    arr.isEmpty   = () => arr.length === 0;
    arr.size      = () => arr.length;  // kotlin .size property won't work but .size() will
    arr.contains  = (item) => arr.includes(item);
    return arr;
}
globalThis._mutableList = _mutableList;

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

globalThis.Pair_2 = Pair;

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

        if (typeof block?.invoke === "function") {
            result = block.invoke(null, null);
        } else if (typeof block?.invokeSuspend === "function") {
            result = block.invokeSuspend(Unit_INSTANCE);
        } else if (typeof block === "function") {
            result = block();
        } else {
            throw new Error("runBlocking: invalid block");
        }

        if (
            result === COROUTINE_SUSPENDED ||
            result === _COROUTINE_SUSPENDED
        ) {
            throw new Error(
                "Coroutine suspension unsupported in sync runtime"
            );
        }

        return result;
    }
};

globalThis.CoroutineScopeKt = {
    CoroutineScope(context) {
        return { _context: context };
    }
};

globalThis.LinkedHashMap = class LinkedHashMap extends Map {
    constructor(init) {
        super();
        if (init) {
            for (const [k, v] of init) this.set(k, v);
        }
    }
    get(key)            { return super.get(key) ?? null; }
    put(key, value)     { this.set(key, value); return null; }
    containsKey(key)    { return this.has(key); }
    containsValue(val)  { for (const v of this.values()) if (v === val) return true; return false; }
    remove(key)         { const v = this.get(key); this.delete(key); return v; }
    isEmpty()           { return this.size === 0; }
    entrySet()          { return [...this.entries()].map(([k,v]) => ({ getKey: () => k, getValue: () => v })); }
    keySet()            { return new Set(this.keys()); }
    values()            { return [...super.values()]; }
    putAll(map)         { for (const [k,v] of map) this.set(k,v); return this; }
};

globalThis.HashMap = LinkedHashMap;

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
    toArray()    { return [...this._a]; }
    [Symbol.iterator]() { return this._a[Symbol.iterator](); }
    map(fn) { return this._a.map(fn); }

    iterator() {
        let i = 0; const a = this._a;
        return {
            hasNext() { return i < a.length ? 1 : 0; },
            next() { return a[i++]; }
        };
    }

    isEmpty() { return this._a.length === 0 ? 1 : 0; }  // 1=true, 0=false
    size()    { return this._a.length; }

    get length() { return this._a.length; }
    get length_val() { return this._a.length; }
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
        const transformed = this.transformDeserialize(decoder._json);
        const newDecoder = new JsonDecoder(transformed, decoder._descriptor);
        return this.tSerializer.deserialize(newDecoder);
    }

    serialize(encoder, value) {
        const transformed = this.transformSerialize(value);
        return this.tSerializer.serialize(encoder, transformed);
    }
};

// kotlinx.serialization stubs
globalThis.PluginGeneratedSerialDescriptor = class PluginGeneratedSerialDescriptor {
    constructor(name, serializer, size) {
        this._name = name;
        this._serializer = serializer;
        this._fields = [];
    }
    addElement(name, isOptional) {
        this._fields.push(name);
    }
    getElementIndex(name) {
        return this._fields.indexOf(name);
    }
    getElementName(index) {
        return this._fields[index];
    }
};

globalThis.JsonDecoder = class JsonDecoder {
    constructor(json, descriptor) {
        this._json = json;
        this._descriptor = descriptor;
        this._index = 0;
    }

    decodeNullableSerializableElement(descriptor, index, serializer, old) {
        const key = descriptor._fields[index];
        const val = this._json[key];
        if (val === null || val === undefined) return null;
        if (serializer && typeof serializer.deserialize === 'function') {
            const childDescriptor = serializer.getDescriptor?.() ?? descriptor;
            return serializer.deserialize(new JsonDecoder(val, childDescriptor));
        }
        return val;
    }

    beginStructure(descriptor) {
        const json = this._json instanceof JsonArray ? this._json._arr : this._json;
        return new JsonDecoder(json, descriptor);
    }
    endStructure(descriptor) {}
    decodeSequentially() {
        return 0;
    }
    decodeElementIndex(descriptor) {
        const idx =
            this._index < descriptor._fields.length
                ? this._index++
                : -1;

        return idx;
    }
    decodeStringElement(descriptor, index) {
        const key = descriptor._fields[index];
        return this._json[key] ?? null;
    }
    decodeIntElement(descriptor, index) {
        const key = descriptor._fields[index];
        return this._json[key] ?? 0;
    }
    decodeBooleanElement(descriptor, index) {
        const key = descriptor._fields[index];
        return this._json[key] ?? false;
    }
    decodeSerializableElement(descriptor, index, serializer, old) {
        const key = descriptor._fields[index];
        const val = this._json[key];
        if (val === undefined || val === null) return old ?? null;

        if (serializer && typeof serializer.deserialize === 'function') {
            const childDescriptor = serializer.getDescriptor?.() ?? descriptor;
            return serializer.deserialize(new JsonDecoder(val, childDescriptor));
        }
        return val;
    }
};

globalThis.PluginExceptionsKt = {
    throwMissingFieldException(seenBits, requiresBits, descriptor) {
        throw new Error(`Missing required field in ${descriptor?.serialName ?? "unknown"}`);
    },
};

globalThis.ArrayListSerializer = class ArrayListSerializer {
    constructor(elementSerializer) {
        this._elementSerializer = elementSerializer;
    }
    deserialize(decoder) {
        let raw = decoder._json;
        if (raw?.__isJsonArray) raw = raw._arr;
        const arr = Array.isArray(raw) ? raw : [];
        const list = new ArrayList();
        for (const item of arr) {
            if (this._elementSerializer && typeof this._elementSerializer.deserialize === "function") {
                const childDescriptor = this._elementSerializer.getDescriptor?.() ?? decoder._descriptor;
                list.add(this._elementSerializer.deserialize(new JsonDecoder(item, childDescriptor)));
            } else {
                list.add(_wrapKotlinObject(item));
            }
        }
        return list;
    }
    getDescriptor() { return null; }
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
        let value;
        let initialized = false;
        return {
            getValue() {
                if (!initialized) {
                    try {
                        value = initializer();
                    } catch(e) {
                        console.log("lazy getValue error:", e.message);
                    }
                    initialized = true;
                }
                return value;
            }
        };
    }
};

globalThis.Intrinsics = {
    areEqual(a, b) {
        if (a === b) return 1;
        if (a === null || b === null) return 0;
        if (typeof a === 'object' && typeof a.equals === 'function') {
            return a.equals(b) ? 1 : 0;
        }
        return 0;
    },

    checkNotNull(value, message) {
        if (value === null || value === undefined) {
            throw new Error(message ?? "Required value was null");
        }
        return value;
    },

    checkNotNullParameter(value, name) {
        if (value === null || value === undefined) {
            throw new Error(`Parameter specified as non-null is null: ${name}`);
        }
        return value;
    },

    checkExpressionValueIsNotNull(value, expression) {
        if (value === null || value === undefined) {
            throw new Error(`Expression '${expression}' must not be null`);
        }
        return value;
    },

    checkFieldIsNotNull(value, className, fieldName) {
        if (value === null || value === undefined) {
            throw new Error(`Field '${fieldName}' in '${className}' must not be null`);
        }
        return value;
    },

    throwUninitializedPropertyAccessException(name) {
        throw new Error(`lateinit property ${name} has not been initialized`);
    },

    throwNpe() {
        throw new Error("NullPointerException");
    },

    stringPlus(a, b) {
        return String(a ?? "null") + String(b ?? "null");
    },

    areEqualOrBothNull(a, b) {
        if (a === null && b === null) return 1;
        if (a === null || b === null) return 0;
        return a === b ? 1 : 0;
    },
};

// StringSerializer
globalThis.StringSerializer = {
    INSTANCE: {
        deserialize(decoder) {
            const val = decoder._json;
            if (val === null || val === undefined) return null;
            return String(val);
        },
        serialize(encoder, value) { return String(value ?? ""); },
        getDescriptor() { return { _fields: [], serialName: "kotlin.String" }; },
    }
};

// IntSerializer, BooleanSerializer etc. while we're at it
globalThis.IntSerializer = {
    INSTANCE: {
        deserialize(decoder) { return decoder._json ?? 0 | 0; },
        getDescriptor() { return { _fields: [], serialName: "kotlin.Int" }; },
    }
};
globalThis.BooleanSerializer = {
    INSTANCE: {
        deserialize(decoder) { return decoder._json ? 1 : 0; },
        getDescriptor() { return { _fields: [], serialName: "kotlin.Boolean" }; },
    }
};
globalThis.LongSerializer = {
    INSTANCE: {
        deserialize(decoder) { return decoder._json ?? 0; },
        getDescriptor() { return { _fields: [], serialName: "kotlin.Long" }; },
    }
};

globalThis.SetsKt = {
    emptySet() {
        return new Set();
    },

    hashSetOf(...items) {
        // Kotlin hashSetOf(vararg)
        if (items.length === 1 && Array.isArray(items[0])) {
            return new Set(items[0]);
        }
        return new Set(items);
    },

    setOf(...items) {
        return new Set(items);
    },

    mutableSetOf(...items) {
        return new Set(items);
    },

    plus(set, value) {
        const s = new Set(set);

        if (value instanceof Set) {
            for (const v of value) s.add(v);
        } else {
            s.add(value);
        }

        return s;
    },

    minus(set, value) {
        const s = new Set(set);

        if (value instanceof Set) {
            for (const v of value) s.delete(v);
        } else {
            s.delete(value);
        }

        return s;
    },
};

// Kotlin Pair / TuplesKt

globalThis.TuplesKt = {
    to(first, second) {
        return {
            first,
            second,
            getFirst()  { return first; },
            getSecond() { return second; },
            component1() { return first; },
            component2() { return second; },
        };
    }
};

globalThis.Long = class Long {
    static valueOf(v, _unused) {
        return Number(v);
    }
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
    parse(str, pos) {
        const ms = Date.parse(str);
        if (pos) pos.setIndex(str.length);
        return isNaN(ms) ? null : new Date(ms);
    }
    format(date) { return (date instanceof Date ? date : new Date(date)).toISOString(); }
    setTimeZone(tz) { this._tz = tz; }
};

globalThis.TimeZone = class TimeZone {
    constructor(id) {
        this.id = id;
    }

    getID() {
        return this.id;
    }

    static getTimeZone(id) {
        return new TimeZone(id);
    }
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
        let flags = "";

        if (options instanceof Set) {
            for (const opt of options) {
                flags += opt.flag ?? "";
            }
        } else if (Array.isArray(options)) {
            for (const opt of options) {
                flags += opt.flag ?? "";
            }
        } else if (typeof options === "string") {
            flags = options;
        }

        this._pattern = pattern;
        this._flags = flags;
        this._re = new RegExp(pattern, flags);
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

globalThis.RegexOption = {
    IGNORE_CASE: { flag: "i" },
    MULTILINE: { flag: "m" },
    DOT_MATCHES_ALL: { flag: "s" },
    LITERAL: { flag: "" },
    COMMENTS: { flag: "" },
    UNIX_LINES: { flag: "" },
    CANON_EQ: { flag: "" },
};

RegexOption.Companion = {};

// Android Preferences stubs
globalThis.SwitchPreferenceCompat = class SwitchPreferenceCompat {
    constructor(context) {}
    setKey(v)          { this._key     = v; return this; }
    setTitle(v)        { this._title   = v; return this; }
    setSummary(v)      { this._summary = v; return this; }
    setSummaryOn(v)    { this._summaryOn = v; return this; }
    setSummaryOff(v)   { this._summaryOff = v; return this; }
    setDefaultValue(v) { this._default = v; return this; }
    setOnPreferenceChangeListener(l) { return this; }
    _toManifest() {
        return {
            key:     this._key     ?? "",
            label:   this._title   ?? this._key ?? "",
            type:    "boolean",
            default: this._default ?? false,
        };
    }
};

globalThis.ListPreference = class ListPreference {
    constructor(context) {}
    setKey(v)          { this._key     = v; return this; }
    setTitle(v)        { this._title   = v; return this; }
    setSummary(v)      { this._summary = v; return this; }
    setEntries(v)      { this._entries = v; return this; }
    setEntryValues(v)  { this._values  = v; return this; }
    setDefaultValue(v) { this._default = v; return this; }
    setOnPreferenceChangeListener(l) { return this; }
    _toManifest() {
        const entries = this._entries ?? [];
        const values  = this._values  ?? entries;
        return {
            key:     this._key     ?? "",
            label:   this._title   ?? this._key ?? "",
            type:    "select",
            default: this._default ?? (values[0] ?? ""),
            options: entries.map((label, i) => ({ label, value: values[i] ?? label })),
        };
    }
};

globalThis.MultiSelectListPreference = class MultiSelectListPreference {
    constructor(context) {}
    setKey(v)          { this._key     = v; return this; }
    setTitle(v)        { this._title   = v; return this; }
    setSummary(v)      { this._summary = v; return this; }
    setEntries(v)      { this._entries = v; return this; }
    setEntryValues(v)  { this._values  = v; return this; }
    setDefaultValue(v) { this._default = v; return this; }
    setOnPreferenceChangeListener(l) { return this; }
    _toManifest() {
        const entries = this._entries ?? [];
        const values  = this._values  ?? entries;
        const defaultVal = Array.isArray(this._default)
            ? (this._default[0] ?? values[0] ?? "")
            : (this._default    ?? values[0] ?? "");
        return {
            key:     this._key     ?? "",
            label:   this._title   ?? this._key ?? "",
            type:    "select",
            default: defaultVal,
            options: entries.map((label, i) => ({ label, value: values[i] ?? label })),
        };
    }
};

globalThis.EditTextPreference = class EditTextPreference {
    constructor(context) {}
    setKey(v)          { this._key     = v; return this; }
    setTitle(v)        { this._title   = v; return this; }
    setSummary(v)      { this._summary = v; return this; }
    setDefaultValue(v) { this._default = v; return this; }
    setOnPreferenceChangeListener(l) { return this; }
    _toManifest() {
        return {
            key:     this._key     ?? "",
            label:   this._title   ?? this._key ?? "",
            type:    "string",
            default: this._default ?? "",
        };
    }
};

// PreferenceScreen / PreferenceGroup
globalThis.PreferenceScreen = class PreferenceScreen {
    constructor() {
        this._prefs = [];
    }

    getContext() {
        return {};
    }

    addPreference(p) {
        this._prefs.push(p);
    }

    getPreferences() {
        return this._prefs;
    }
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

const _makeOkHttpClient = (useCloudflare, interceptors = [], networkInterceptors = []) => {
    const client = new OkHttpClient(useCloudflare);
    client._interceptors = interceptors;
    client._networkInterceptors = networkInterceptors;
    client.newBuilder = () => _makeOkHttpClientBuilder(useCloudflare, [...interceptors], [...networkInterceptors]);
    client.interceptors = () => _makeKotlinList(interceptors);
    client.networkInterceptors = () => _makeKotlinList(networkInterceptors);
    return client;
};
const _makeOkHttpClientBuilder = (useCloudflare = false, interceptors = [], networkInterceptors = []) => ({
    _interceptors: interceptors,
    _networkInterceptors: networkInterceptors,
    _useCloudflare: useCloudflare,

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

    build() { return _makeOkHttpClient(this._useCloudflare); },
});

const _makeKotlinList = (arr) => {
    arr.iterator    = function() {
        let i = 0;
        return { hasNext() { return i < arr.length; }, next() { return arr[i++]; } };
    };
    arr.indexOfFirst = (pred) => arr.findIndex(pred);
    arr.removeAt     = (i)    => arr.splice(i, 1)[0];
    arr.add          = (item) => { arr.push(item); return true; };
    Object.defineProperty(arr, 'size', { get: () => arr.length, configurable: true });

    // Proxy to handle any obfuscated method name (e.g. .r(), .q(), .z())
    return new Proxy(arr, {
        get(target, prop) {
            if (prop in target) return typeof target[prop] === 'function'
                ? target[prop].bind(target)
                : target[prop];

            // Unknown prop — return a function that handles iterator/forEach patterns
            if (typeof prop === 'string') {
                return function(...args) {
                    if (typeof args[0] === 'function') { target.forEach(args[0]); return; }
                    if (args.length === 0)              return target; // iterator call
                    if (typeof args[0] === 'number')    return target[args[0]]; // get(i)
                    return target;
                };
            }
        }
    });
};

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

const _networkHelper = {
    cookieJar: {
        saveFromResponse(url, cookies) {
            for (const c of cookies) _cookieStore.set(c.name, c.value);
            state?.set?.("cookies", Object.fromEntries(_cookieStore));
        },
        loadForRequest(url) { return []; },
    },

    get client()             { return _makeOkHttpClient(true);  },
    get nonCloudflareClient(){ return _makeOkHttpClient(false); },
    get cloudflareClient()   { return _makeOkHttpClient(true);  },
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

globalThis.JsoupDocument = class JsoupDocument {
    constructor(html) {
        this._html = html;
        this._$ = parseHTML(html);
    }

    select(selector)              { return new JsoupElements(this._$(selector)); }
    text()                        { return this._$("body").text(); }
    html()                        { return this._html; }
    outerHtml()                   { return this._html; }
    title()                       { return this._$("title").text(); }
    body()                        { return this.select("body").first(); }
    head()                        { return this.select("head").first(); }
    getElementById(id)            { return this.select(`#${id}`).first(); }
    getElementsByTag(tag)         { return this.select(tag); }
    getElementsByClass(cls)       { return this.select(`.${cls}`); }

    selectFirst(selector) {
        return this.select(selector).first();
    }

    wholeText() { return this._$("body").text(); }
};

globalThis.JsoupElements = class JsoupElements {
    constructor(raw) {
        this._els = raw.map(item => new JsoupElement(item._raw));
    }

    get(i)      { return this._els[i] ?? null; }
    first()     { return this._els[0] ?? null; }
    last()      { return this._els[this._els.length - 1] ?? null; }
    size()      { return this._els.length; }
    isEmpty()   { return this._els.length === 0; }
    text()      { return this._els.map(el => el.text()).join(""); }
    html()      { return this._els[0]?.html() ?? ""; }
    outerHtml() { return this._els[0]?.outerHtml() ?? ""; }
    attr(name)  { return this._els[0]?.attr(name) ?? null; }
    select(sel) { return this._els[0] ? this._els[0].select(sel) : new JsoupElements([]); }

    forEach(fn) { this._els.forEach(fn); }
    map(fn)     { return this._els.map(fn); }
    filter(fn)  { return this._els.filter(fn); }

    selectFirst(selector) { return this.select(selector).first(); }
};

globalThis.JsoupElement = class JsoupElement {
    constructor(raw) {
        this._raw = raw;
    }

    text()        { return this._raw.text; }
    html()        { return this._raw.html; }
    outerHtml()   { return this._raw.outer; }
    attr(name)    { return this._raw.attrs?.[name] ?? null; }
    hasAttr(name) { return name in (this._raw.attrs ?? {}); }
    id()          { return this.attr("id") ?? ""; }
    className()   { return this.attr("class") ?? ""; }
    tagName()     { return this._raw.tag ?? ""; }

    select(selector) {
        const $ = parseHTML(this._raw.html);
        return new JsoupElements($(selector));
    }

    wholeText() { return this._raw.text; }

    selectFirst(selector) { return this.select(selector).first(); }

    data() { return this._raw.text; }
};

globalThis["JsoupExtensionsKt"] = {
    ["asJsoup$default"]: function(body, baseUri, charset, flags) {
        const html = typeof body?.string === "function" ? body.string() : body._text;
        return new JsoupDocument(html);
    },
    ["asJsoup"]: function(body, baseUri, charset) {
        return new JsoupDocument(body.string());
    },
};

globalThis.Jsoup = {
    parseBodyFragment(html) {
        return new JsoupDocument(html);
    },
    parse(html) {
        return new JsoupDocument(html);
    },
};

function _wrapKotlinObject(obj) {
    if (obj === null || obj === undefined) return null;
    if (Array.isArray(obj)) return obj.map(_wrapKotlinObject);
    if (typeof obj !== "object") return obj;

    const keys = Object.keys(obj);
    const normalizedMap = {};
    for (const key of keys) {
        normalizedMap[String(key).toLowerCase().replace(/[_-]/g, "")] = key;
    }

    // ordered list of values for positional obfuscated access (a, b, c... l, n, o...)
    const orderedValues = keys.map(k => obj[k]);

    // build a map of single/short obfuscated names to positional values
    // obfuscated names are typically 1-2 chars: a-z, aa, ab etc.
    const obfuscatedNames = [];
    for (let i = 0; i < 26; i++) obfuscatedNames.push(String.fromCharCode(97 + i));
    for (let i = 0; i < 26; i++) for (let j = 0; j < 26; j++) obfuscatedNames.push(String.fromCharCode(97+i) + String.fromCharCode(97+j));

    return new Proxy(obj, {
        get(target, prop) {
            if (prop === Symbol.iterator) {
                const arr = Array.isArray(target) ? target : Object.values(target);
                return arr[Symbol.iterator].bind(arr.map(_wrapKotlinObject));
            }
            if (prop === "iterator") return () => {
                const arr = Array.isArray(target) ? target : Object.values(target);
                let i = 0;
                return {
                    hasNext: () => i < arr.length,
                    next: () => _wrapKotlinObject(arr[i++]),
                };
            };

            // exact match
            if (prop in target) {
                const val = target[prop];
                if (typeof val === "function") return val.bind(target);
                return () => _wrapKotlinObject(val);
            }

            // fuzzy snake_case/camelCase match
            const normalizedProp = String(prop).toLowerCase().replace(/[_-]/g, "");
            const matchedKey = normalizedMap[normalizedProp];
            if (matchedKey !== undefined) {
                const val = target[matchedKey];
                if (typeof val === "function") return val.bind(target);
                return () => _wrapKotlinObject(val);
            }

            // positional obfuscated match: l = 11th field, n = 13th etc.
            const idx = obfuscatedNames.indexOf(String(prop));
            if (idx !== -1 && idx < orderedValues.length) {
                return () => _wrapKotlinObject(orderedValues[idx]);
            }

            return () => null;
        }
    });
}

globalThis.JsonArray = class JsonArray {
    constructor(list) {
        this._arr = Array.isArray(list) ? list : (list ? [...list] : []);
        this.__isJsonArray = true;
    }
    values()  { return this._arr; }
    size()    { return this._arr.length; }
    isEmpty() { return this._arr.length === 0 ? 1 : 0; }
    get(i)    { return this._arr[i]; }
    iterator() {
        let i = 0; const a = this._arr;
        return { hasNext() { return i < a.length ? 1 : 0; }, next() { return a[i++]; } };
    }
    [Symbol.iterator]() { return this._arr[Symbol.iterator](); }

    static [Symbol.hasInstance](instance) {
        if (instance === null || instance === undefined) return false;
        return instance.__isJsonArray === true || Array.isArray(instance);
    }
};

const _JsonObject = class JsonObject {
    static [Symbol.hasInstance](instance) {
        return instance !== null && typeof instance === 'object'
            && !Array.isArray(instance) && !instance.__isJsonArray;
    }
};

// Kotlin's JsonObject.values is a property returning the map's values collection.
// Since plain JS objects are used as JsonObject at runtime, we patch Object.prototype
// carefully so any plain object gets a values() method.
// We do it on _JsonObject.prototype but since instanceof is faked, actual objects
// won't have it — so we need to inject it differently.

// The cleanest approach: wrap values() as a global helper that works on plain objects.
// But since the translated code calls it as v2_1.values(), we need it on the object itself.
// Patch Object.prototype as a last resort, guarded to avoid breaking arrays/primitives:
Object.defineProperty(Object.prototype, 'values', {
    value: function() {
        return Object.values(this);
    },
    writable: true,
    configurable: true,
    enumerable: false,  // non-enumerable so it doesn't show up in for..in loops
});

const _JsonPrimitive = class JsonPrimitive {
    static [Symbol.hasInstance](instance) {
        return typeof instance === 'string' || typeof instance === 'number' || typeof instance === 'boolean';
    }
};

globalThis.kotlinx = {
    serialization: {
        json: {
            JsonObject:    _JsonObject,
            JsonArray:     JsonArray,
            JsonPrimitive: _JsonPrimitive,
            JsonElement:   _JsonObject,
            JsonNull:      { INSTANCE: null },
        }
    }
};

globalThis.OkioStreamsKt = {
    decodeFromBufferedSource(deserializer, type, source) {
        try {
            const text = typeof source === "string" ? source : source?._text ?? "";
            const parsed = JSON.parse(text);
            const actualSerializer = type ?? deserializer;
            const decoder = new JsonDecoder(parsed, null);
            return actualSerializer.deserialize(decoder);
        } catch(e) {
            console.log("error msg:", e.message);
            throw e;
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
    ListSerializer(elementSerializer) {
        return new ArrayListSerializer(elementSerializer);
    },
    ArrayListSerializer(elementSerializer) {
        return new ArrayListSerializer(elementSerializer);
    },
    NullableSerializer(elementSerializer) {
        return {
            _elem: elementSerializer,
            deserialize(decoder) {
                const val = decoder._json;
                if (val === null || val === undefined) return null;
                return elementSerializer.deserialize(new JsonDecoder(val, decoder._descriptor));
            },
            getDescriptor() { return elementSerializer.getDescriptor?.() ?? null; },
        };
    },
};

globalThis.Json = {
    Default: {
        decodeFromString(deserializer, str) {
            const parsed = JSON.parse(str);
            const data = parsed?.data ?? parsed;
            const decoder = new JsonDecoder(data, null);
            return deserializer.deserialize(decoder);
        },
        encodeToString(serializer, obj)     { return JSON.stringify(obj); },
    },
    decodeFromString(deserializer, str) {
        const parsed = JSON.parse(str);
        const data = parsed?.data ?? parsed;
        const decoder = new JsonDecoder(data, null);
        return deserializer.deserialize(decoder);
    },
    encodeToString(serializer, obj)     { return JSON.stringify(obj); },
};

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

globalThis.Filter = class Filter {
    constructor(name, state) {
        this.name  = name;
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

globalThis["Filter$Header"]    = Filter.Header;
globalThis["Filter$Separator"] = Filter.Separator;
globalThis["Filter$Select"]    = Filter.Select;
globalThis["Filter$Text"]      = Filter.Text;
globalThis["Filter$CheckBox"]  = Filter.CheckBox;
globalThis["Filter$TriState"]  = Filter.TriState;
globalThis["Filter$Group"]     = Filter.Group;
globalThis["Filter$Sort"]           = Filter.Sort;
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
        const url = this._url;
        return {
            url() { return new HttpUrl(url); },
            header(name) { return null; },
            method() { return "GET"; },
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
    constructor(map = {}) {
        this._map = {};

        for (const k in map) {
            this._map[k.toLowerCase()] = map[k];
        }
    }

    get(name) {
        return this._map[name.toLowerCase()] ?? null;
    }

    newBuilder() {
        const b = new Headers.Builder();
        for (const k in this._map) {
            b._map[k] = this._map[k];
        }
        return b;
    }

    set(name, value) {
        this._map[name.toLowerCase()] = String(value);
    }

    has(name) {
        return name.toLowerCase() in this._map;
    }

    delete(name) {
        delete this._map[name.toLowerCase()];
    }

    forEach(callback, thisArg = undefined) {
        for (const key in this._map) {
            callback.call(thisArg, this._map[key], key, this);
        }
    }

    entries() {
        return Object.entries(this._map)[Symbol.iterator]();
    }

    keys() {
        return Object.keys(this._map)[Symbol.iterator]();
    }

    values() {
        return Object.values(this._map)[Symbol.iterator]();
    }

    [Symbol.iterator]() {
        return this.entries();
    }

    toFetchHeaders() {
        return { ...this._map };
    }

    static Builder = class HeadersBuilder {
        constructor() {
            this._map = {};
        }

        push(name, value) {
            return this.add(name, value);
        }

        removeAll(name) {
            delete this._map[name.toLowerCase()];
            return this;
        }

        add(name, value) {
            this._map[name.toLowerCase()] = String(value);
            return this;
        }

        set(name, value) {
            this._map[name.toLowerCase()] = String(value);
            return this;
        }

        build() {
            return new Headers(this._map);
        }
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

(function() {
    const _token = { _ctor: null };

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
    globalThis["PreferencesKt$get$1"] = _PrefsInlined;
    globalThis.PreferencesKt = globalThis.PreferencesKt ?? {
        getPreferences(context, name) {
            const app = globalThis.__appInstance ||= new Application();
            return app.getSharedPreferences(name ?? "prefs", 0);
        },
    };
})();

globalThis.Enum = class Enum {
    constructor(name, ordinal) {
        this.name = name;
        this.ordinal = ordinal;
    }

    toString() {
        return this.name;
    }
};

globalThis.EnumEntriesKt = {
    enumEntries(values) {
        return values;
    }
};

globalThis.Enum_2 = Enum;
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
                    encodeToString(serializer, obj)     { return JSON.stringify(obj); },

                    // Application / SharedPreferences
                    getSharedPreferences(name, mode) { return new SharedPreferences(name); },

                    // OkHttpClient
                    newCall(request) { return new OkHttpCall(request); },
                };
            }
        };
    }
};
// RequestsKt
globalThis.RequestsKt = {
    GET(url, headers, cache) {
        const h = typeof headers?.build === 'function' ? headers.build() : headers;
        return new Request.Builder().url(url).headers(h ?? new Headers()).cacheControl(cache ?? null).build();
    },
    GET$default(url, headers, cache, flags, mask) {
        const h = typeof headers?.build === 'function' ? headers.build() : headers;
        return new Request.Builder().url(url).headers(h ?? new Headers()).cacheControl(cache ?? null).build();
    },
    POST$default(url, headers, body) {
        const h = typeof headers?.build === 'function' ? headers.build() : headers;
        return new Request.Builder().url(url).headers(h ?? new Headers()).post(body).build();
    },
};

globalThis.GET  = (url, headers, cache) => RequestsKt.GET(url, headers, cache);
globalThis.POST = (url, headers, body, cache) =>
    new Request.Builder().url(url).headers(headers ?? new Headers()).post(body).build();

// HttpUrl
globalThis.HttpUrl = class HttpUrl {
    constructor(url) { this._url = url; }
    toString()   { return this._url; }
    fragment()   {
        const m = this._url.match(/#(.*)$/);
        return m ? m[1] : null;
    }
    host()       { try { return new URL(this._url).hostname; } catch { return ""; } }
    encodedPath(){ try { return new URL(this._url).pathname; } catch { return "/"; } }
    newBuilder() { return new HttpUrl.Builder(this._url); }

    pathSegments() {
        try {
            const segs = new URL(this._url).pathname
                .split("/")
                .filter(s => s.length > 0);
            segs.get = (i) => segs[i];
            return segs;
        } catch {
            const segs = this._url.split("/").filter(s => s.length > 0);
            segs.get = (i) => segs[i];
            return segs;
        }
    }

    static Builder = class HttpUrlBuilder {
        constructor(base = "") { this._url = base; }
        addQueryParameter(k, v) {
            if (v === null || v === undefined) return this;
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
    constructor(useCloudflare = false) {
        this._useCloudflare = useCloudflare;
    }
    newCall(request) {
        return new _Call(request, this._useCloudflare);
    }
};
globalThis._Call = class _Call {
    constructor(req, useCloudflare = false) {
        this._req = req;
        this._useCloudflare = useCloudflare;
    }

    execute() {
        const url    = this._req.url?.toString?.() ?? String(this._req.url);
        const method = this._req.method ?? "GET";
        const headers = this._req.headers?.toFetchHeaders?.() ?? {};
        const body   = this._req.body ?? undefined;

        const now = Date.now();
        const elapsed = now - globalThis.__lastFetchTime;
        if (elapsed < 1000 && globalThis.__lastFetchTime !== 0) {
            __native_sleep(1000 - elapsed);
        }
        globalThis.__lastFetchTime = Date.now();

        const result = fetchSync(url, { method, headers, body });
        return new _SandboxResponse(result.text, result.status, url);
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

globalThis.OkHttpExtensionsKt = {
    await(call, continuation) {
        const req = call._req;
        const useCloudflare = call._useCloudflare ?? false;
        const url     = req?.url?.toString?.() ?? String(req?.url ?? "");
        const method  = req?.method ?? "GET";
        const headers = req?.headers?.toFetchHeaders?.() ?? {};
        const body    = req?.body ?? undefined;

        const result = fetchSync(url, { method, headers, body });

        if (useCloudflare && (result.status === 403 || result.status === 503)) {
            if (typeof headless === "undefined" || !headless.available) {
                throw new Error(`Cloudflare challenge on ${url} but headless is not available`);
            }
            const cfResult = headless.fetchSync(url, { waitFor: "network_idle", block: ["images", "fonts"] });
            if (cfResult?.cookies?.length) {
                for (const c of cfResult.cookies) _cookieStore.set(c.name, c.value);
                state?.set?.("cookies", Object.fromEntries(_cookieStore));
            }
            const mergedHeaders = _mergeCloudfareCookies(headers, cfResult?.cookies ?? []);
            const retry = fetchSync(url, { method, headers: mergedHeaders, body });
            return new _SandboxResponse(retry.text, retry.status, url);
        }

        return new _SandboxResponse(result.text, result.status, url);
    }
};

globalThis.RequestBody = {
    Companion: {
        create(_mediaType, content) {
            return content;
        }
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

globalThis.FunctionReferenceImpl = class FunctionReferenceImpl {
    constructor(
        arity,
        owner,
        name,
        signature,
        flags
    ) {
        this.arity = arity;
        this.owner = owner;
        this.name = name;
        this.signature = signature;
        this.flags = flags;
    }
};
globalThis.FunctionReference = class FunctionReference {};

globalThis.ContinuationImpl = class ContinuationImpl {
    constructor(completion) {
        this.completion = completion || null;
        this.label = 0;
    }

    invokeSuspend(result) {
        return result;
    }

    resumeWith(result) {
        let current = this;
        let param = result;

        while (current) {
            try {
                const outcome = current.invokeSuspend(param);

                if (outcome === COROUTINE_SUSPENDED) {
                    return outcome;
                }

                param = outcome;
            } catch (e) {
                param = e;
            }

            current = current.completion;
        }

        return param;
    }

    create(value, completion) {
        return this;
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

async function _isCloudflareChallenged(res) {
    if (res.status !== 403 && res.status !== 503) return false;
    const text = await res.text();
    return text.includes("challenge-error-title") || text.includes("challenge-error-text");
}

async function _resolveCloudflare(url) {
    const result = await headless.fetch(url, {
        waitFor: "network_idle",
        timeoutMs: 30000,
        javascript: `({ userAgent: navigator.userAgent })`,
    });

    if (result?.cookies?.length) {
        for (const c of result.cookies) _cookieStore.set(c.name, c.value);
        state?.set?.("cookies", Object.fromEntries(_cookieStore));
    }

    return {
        cookies: result?.cookies ?? [],
        userAgent: result?.result?.userAgent ?? null,
    };
}

function _mergeCloudfareCookies(existingHeaders, cookies) {
    let out;
    if (existingHeaders instanceof Headers) {
        out = new Headers(existingHeaders._map); // copy the internal map
    } else {
        out = new Headers(existingHeaders ?? {});
    }

    if (cookies && cookies.length) {
        const cookieStr = cookies.map(c => `${c.name}=${c.value}`).join("; ");
        const existing = out.get("cookie") ?? "";
        out.set("cookie", existing ? `${existing}; ${cookieStr}` : cookieStr);
    }
    return out;
}

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

// Override in HttpSource to unwrap Headers and HttpUrl properly
globalThis._origDoRequest = Manga.prototype._doRequest;

//  HttpSource

globalThis._SandboxManga = Manga;
class HttpSource extends _SandboxManga {

    get lang()      { return "en"; }
    get name()      { return "Unknown"; }
    get versionId() { return 1; }

    headersBuilder() {
        return new Headers.Builder()
            .add("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/114.0.0.0 Safari/537.36");
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
                    options: filter.values.map((v, i) => ({
                        label: typeof v === "string" ? v : v?.toString?.() ?? String(i),
                        value: String(i),
                    })),
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
                        options: state.map(f => ({ label: f.name, value: f.name })),
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
                        label: typeof v === "string" ? v : String(v),
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

        // Only fall back to popular if there's truly nothing to search by
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

    async getImageRequestHeaders(imageUrl) {
        const page = new Page(0, imageUrl, imageUrl);
        const req = this.imageRequest(page);
        const r = req instanceof _Call ? req._req : req;
        const h = new Headers(r.headers?.toFetchHeaders?.() ?? r.headers ?? {});

        if (!h.has("referer")) {
            h.set("referer", this.getBaseUrl() + "/");
        }

        return h.toFetchHeaders();
    }

    //  internal 

    async _doRequest(req) {
        const r = req instanceof _Call ? req._req : req;
        const useCloudflare = req instanceof _Call
            ? req._useCloudflare
            : (this.client?._useCloudflare ?? true);

        const url    = r.url?.toString?.() ?? String(r.url);
        const method = r.method ?? "GET";
        const body   = r.body ?? undefined;

        const origin = new URL(url).origin;
        const cfState = useCloudflare ? (_cfStateByOrigin.get(origin) ?? null) : false;

        // Helper: build headers with cookie jar merged in
        const buildHeaders = (base, extraCookies = null) => {
            const h = base instanceof Headers
                ? new Headers(base)  // clone
                : new Headers(base instanceof Object ? base : {});

            // Merge cookie jar
            if (_cookieStore.size > 0) {
                const stored = [..._cookieStore.entries()]
                    .map(([k, v]) => `${k}=${v}`).join("; ");
                const existing = h.get("cookie") ?? "";
                h.set("cookie", existing ? `${existing}; ${stored}` : stored);
            }

            // Merge CF cookies if provided
            if (extraCookies) {
                _mergeCloudfareCookies(h, extraCookies);
            }

            return h;
        };

        const baseHeaders = r.headers?.toFetchHeaders?.() ?? r.headers
            ?? this.headers?.toFetchHeaders?.() ?? this.headers ?? {};

        // ── Path A: origin is known CF-protected, go straight to headless ────────
        if (cfState && cfState !== false) {
            const h = buildHeaders(baseHeaders, cfState.cookies);
            h.set("user-agent",    cfState.userAgent);
            h.set("referer",       origin + "/");
            h.set("sec-fetch-site", "same-origin");
            h.set("sec-fetch-mode", "cors");
            h.set("sec-fetch-dest", "empty");

            const res = await headless.fetch(url, {
                method,
                headers: Object.fromEntries(
                    (h.toFetchHeaders?.() ?? h).entries()
                ),
                body,
            });

            if (!res.ok) throw new Error(`HTTP ${res.status}: ${url}`);
            return new _SandboxResponse(res.body ?? res.text ?? "", res.status, url);
        }

        // ── Path B: origin unknown or known-clean, try plain fetch first ─────────
        const headers = buildHeaders(baseHeaders);
        const res  = await fetch(url, {
            method,
            headers: headers.toFetchHeaders?.() ?? headers,
            body,
        });
        const text = await res.text();

        const isCfChallenge = useCloudflare
            && (res.status === 403 || res.status === 503)
            && (text.includes("challenge-error-title") || text.includes("challenge-error-text"));

        if (!isCfChallenge) {
            if (cfState === null) _cfStateByOrigin.set(origin, false);
            _saveCfState();
            if (!res.ok) throw new Error(`HTTP ${res.status}: ${url}`);
            return new _SandboxResponse(text, res.status, url);
        }

        const { cookies, userAgent } = await _resolveCloudflare(url);
        _cfStateByOrigin.set(origin, { cookies, userAgent });
        _saveCfState();

        const retryHeaders = buildHeaders(baseHeaders, cookies);
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

HttpSource.prototype.getHeaders = function() {
    if (!this._headers) {
        const h = this.headersBuilder();
        this._headers = typeof h?.build === 'function' ? h.build() : (h instanceof Headers ? h : new Headers(h));
    }
    return this._headers;
};

let __tachi_captured = null;

globalThis.HttpSource       = HttpSource;
globalThis.ParsedHttpSource = HttpSource;
globalThis.Manga            = HttpSource;
HttpSource.prototype.getNetwork = function() { return _networkHelper; };
_networkHelper.getCloudflareClient = function() { return _makeOkHttpClient(true); };
_networkHelper.getClient            = function() { return _makeOkHttpClient(true); };
_networkHelper.getNonCloudflareClient = function() { return _makeOkHttpClient(false); };

globalThis.__tachi_getCapturedClass = function() {
    if (__tachi_captured) return __tachi_captured;

    const candidates = [];
    for (const key of Object.getOwnPropertyNames(globalThis)) {
        try {
            const v = globalThis[key];
            if (typeof v === "function" && v !== HttpSource && v.prototype instanceof HttpSource) {
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