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

if (!Array.prototype.clone) {
    Array.prototype.clone = function() { return [...this]; };
}
if (!Number.prototype.ordinal) {
    Number.prototype.ordinal = function() { return this.valueOf(); };
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
let lastValidUrlString = "";
Object.defineProperty(String.prototype, 'length_val', {
    get: function() {
        if (this.startsWith("http://") || this.startsWith("https://")) {
            lastValidUrlString = this.toString();
            return this.length;
        }
        if (this.toString() === "" && lastValidUrlString !== "") {
            return lastValidUrlString.length;
        }

        return this.length;
    },
    enumerable: false,
    configurable: true
});

String.prototype.compareTo = function(other) {
    if (this < other) return -1;
    if (this > other) return 1;
    return 0;
};

String.prototype.substringBefore = function(delimiter) {
    const idx = this.indexOf(delimiter);
    return idx === -1 ? this.toString() : this.slice(0, idx);
};

String.prototype.substringAfter = function(delimiter) {
    const idx = this.indexOf(delimiter);
    return idx === -1 ? "" : this.slice(idx + delimiter.length);
};

Object.defineProperty(Array.prototype, 'length_val', {
    get() { return this.length; },
    enumerable: false,
    configurable: true,
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

Object.defineProperty(Boolean, "valueOf", {
    value(v) {
        return !!v;
    },
    writable: true,
    configurable: true
});

if (!Array.prototype.toArray) {
    Array.prototype.toArray = function(target) { return [...this]; };
}

Set.prototype.contains = function(value) {
    return this.has(value) ? 1 : 0;
};

Array.prototype.contains = function(value) {
    return this.includes(value) ? 1 : 0;
};

String.prototype.contains = function(value) {
    return this.includes(value) ? 1 : 0;
};

Function.prototype.invoke = function(...args) {
    return this(...args);
};

String.prototype.getBytes = function(charset) {
    const str = this;
    const bytes = [];
    for (let i = 0; i < str.length; i++) {
        const code = str.charCodeAt(i);
        if (code < 128) {
            bytes.push(code);
        } else if (code < 2048) {
            bytes.push((code >> 6) | 192);
            bytes.push((code & 63) | 128);
        } else {
            bytes.push((code >> 12) | 224);
            bytes.push(((code >> 6) & 63) | 128);
            bytes.push((code & 63) | 128);
        }
    }
    return bytes;
};

Map.prototype.put = function(k, v) { this.set(k, v); return null; };
Map.prototype.containsKey = function(k) { return this.has(k); };
Map.prototype.remove = function(k) { const v = this.get(k); this.delete(k); return v ?? null; };
Map.prototype.getOrDefault = function(k, def) { return this.has(k) ? this.get(k) : def; };

function _serializeBody(body) {
    if (body?._contentType) {
        return {
            body: body._body,
            contentType: body._contentType,
        };
    }

    if (body instanceof FormBody) {
        return {
            body: body.toString(),
            contentType: "application/x-www-form-urlencoded",
        };
    }

    return {
        body,
        contentType: null,
    };
}

globalThis.Character = {
    isLetterOrDigit(ch) {
        if (typeof ch === "number") {
            ch = String.fromCharCode(ch);
        }
        return /[a-zA-Z0-9]/.test(ch) ? 1 : 0;
    },
    isLetter(ch) {
        if (typeof ch === "number") ch = String.fromCharCode(ch);
        return /[a-zA-Z]/.test(ch) ? 1 : 0;
    },
    isDigit(ch) {
        if (typeof ch === "number") ch = String.fromCharCode(ch);
        return /[0-9]/.test(ch) ? 1 : 0;
    },
    isWhitespace(ch) {
        if (typeof ch === "number") ch = String.fromCharCode(ch);
        return /\s/.test(ch) ? 1 : 0;
    },
    isUpperCase(ch) {
        if (typeof ch === "number") ch = String.fromCharCode(ch);
        return ch === ch.toUpperCase() && /[a-zA-Z]/.test(ch) ? 1 : 0;
    },
    isLowerCase(ch) {
        if (typeof ch === "number") ch = String.fromCharCode(ch);
        return ch === ch.toLowerCase() && /[a-zA-Z]/.test(ch) ? 1 : 0;
    },
    toUpperCase(ch) {
        if (typeof ch === "number") return String.fromCharCode(ch).toUpperCase().charCodeAt(0);
        return ch.toUpperCase();
    },
    toLowerCase(ch) {
        if (typeof ch === "number") return String.fromCharCode(ch).toLowerCase().charCodeAt(0);
        return ch.toLowerCase();
    },
    toString(ch) {
        if (typeof ch === "number") return String.fromCharCode(ch);
        return String(ch);
    },
};

globalThis.Charsets = {
    UTF_8: "UTF-8",
    UTF_16: "UTF-16",
    US_ASCII: "US-ASCII",
    ISO_8859_1: "ISO-8859-1",
};

globalThis.IntRange = class IntRange {
    constructor(start, endInclusive) {
        this.first = start;
        this.last = endInclusive;
    }

    *[Symbol.iterator]() {
        for (let i = this.first; i <= this.last; i++) {
            yield i;
        }
    }
};

globalThis.StringsKt = {

    split$default(str, delimiters, ignoreCase, limit, mask, marker) {
        if (str == null) return _makeKotlinList([]);
        const sep = Array.isArray(delimiters) ? delimiters[0] : delimiters;
        const parts = str.split(sep);
        const result = (limit && limit > 0) ? parts.slice(0, limit) : parts;
        return _makeKotlinList(result);
    },
    substringBeforeLast$default(str, delimiter, missingDelimiterValue, mask, marker) {
        if (str == null) return str;

        if ((mask & 2) !== 0) {
            missingDelimiterValue = str;
        }

        const idx = String(str).lastIndexOf(String(delimiter));

        if (idx < 0) {
            return missingDelimiterValue;
        }

        return String(str).substring(0, idx);
    },

    trimStart(str) {
        if (str == null) return str;
        return String(str).replace(/^\s+/, "");
    },

    startsWith(str, prefix, startIndex = 0) {
        return str.startsWith(prefix, startIndex) ? 1 : 0;
    },

    "substringAfterLast$default"(str, delimiter, missingDelimiterValue, mask, marker) {
        if (mask & 2) missingDelimiterValue = str;
        const idx = str.lastIndexOf(delimiter);
        return idx === -1 ? missingDelimiterValue : str.slice(idx + delimiter.length);
    },
    substringAfterLast(str, delimiter, missingDelimiterValue) {
        if (missingDelimiterValue === undefined) missingDelimiterValue = str;
        const idx = str.lastIndexOf(delimiter);
        return idx === -1 ? missingDelimiterValue : str.slice(idx + delimiter.length);
    },

    startsWith$default(str, prefix, ignoreCase, mask, marker) {
        if ((mask & 4) !== 0) {
            ignoreCase = 0;
        }

        const isCaseIgnored = ignoreCase !== 0;

        if (isCaseIgnored) {
            return str.toLowerCase().startsWith(prefix.toLowerCase()) ? 1 : 0;
        } else {
            return str.startsWith(prefix) ? 1 : 0;
        }
    },

    endsWith(str, suffix, ignoreCase = false) {
        if (ignoreCase) {
            return str.toLowerCase().endsWith(suffix.toLowerCase()) ? 1 : 0;
        }
        return str.endsWith(suffix) ? 1 : 0;
    },

    contains$default(str, other, ignoreCase, mask, marker) {
        if (str == null) return 0;
        if (mask & 1) ignoreCase = false;
        if (ignoreCase) {
            return str.toLowerCase().includes(String(other).toLowerCase()) ? 1 : 0;
        }
        return str.includes(String(other)) ? 1 : 0;
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
        if (str == null || typeof str !== "string") return 0;
        const s = str.trim();
        if (/^[+-]?\d+$/.test(s)) {
            const n = parseInt(s, 10);
            return {
                _value: n,
                intValue() { return n; },
                longValue() { return n; },
                floatValue() { return n; },
                doubleValue() { return n; },
                toString() { return String(n); },
                valueOf() { return n; },
            };
        }
        return 0;
    },

    removeSuffix(str, suffix) {
        if (str == null) return str;
        return str.endsWith(suffix) ? str.slice(0, -suffix.length) : str;
    },

    substringBefore$default(str, delimiter, missingDelimiterValue, mask, marker) {
        if (mask & 2) {
            missingDelimiterValue = str;
        }

        const idx = str.indexOf(delimiter);

        return idx === -1
            ? missingDelimiterValue
            : str.slice(0, idx);
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

    replace$default(str, oldValue, newValue, ignoreCase, mask, marker) {
        if (str == null) return str;

        str = String(str);
        oldValue = String(oldValue);
        newValue = String(newValue);

        if ((mask & 4) !== 0) {
            ignoreCase = false;
        }

        if (ignoreCase) {
            const escapedOld = oldValue.replace(/[.*+?^${}()|[\]\\]/g, '\\$&');
            return str.replace(new RegExp(escapedOld, 'gi'), newValue);
        }

        return str.split(oldValue).join(newValue);
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
    },

    contains(str, other, ignoreCase) {
        if (str == null) return 0;
        ignoreCase = ignoreCase ? true : false;
        if (ignoreCase) {
            return str.toLowerCase().includes(String(other).toLowerCase()) ? 1 : 0;
        }
        return str.includes(String(other)) ? 1 : 0;
    },

    substringAfter$default(str, delimiter, missingDelimiterValue, mask, marker) {
        if (str == null) return str;

        if ((mask & 2) !== 0) {
            missingDelimiterValue = str;
        }

        const idx = str.indexOf(delimiter);

        return idx === -1
            ? missingDelimiterValue
            : str.slice(idx + delimiter.length);
    },

    substringAfter(str, delimiter, missingDelimiterValue) {
        if (str == null) return str;
        const idx = str.indexOf(delimiter);
        return idx === -1
            ? (missingDelimiterValue ?? str)
            : str.slice(idx + delimiter.length);
    },
};

globalThis.kotlin = globalThis.kotlin || {};
globalThis.kotlin.text = globalThis.kotlin.text || {};

globalThis.kotlin.text.StringsKt = globalThis.StringsKt;

globalThis.ArrayDeque = class ArrayDeque {
    constructor(initial) {
        this._data = [];

        if (initial != null) {
            if (Array.isArray(initial)) {
                this._data.push(...initial);
            } else if (typeof initial[Symbol.iterator] === "function") {
                this._data.push(...initial);
            }
        }
    }

    add(v)            { this._data.push(v); return true; }
    addLast(v)        { this._data.push(v); }
    addFirst(v)       { this._data.unshift(v); }

    offer(v)          { this._data.push(v); return true; }
    offerLast(v)      { return this.offer(v); }
    offerFirst(v)     { this._data.unshift(v); return true; }

    push(v)           { this._data.unshift(v); }
    pop()             { return this._data.shift(); }

    remove()          { return this._data.shift(); }
    removeFirst()     { return this._data.shift(); }
    removeLast()      { return this._data.pop(); }

    poll()            { return this._data.shift() ?? null; }
    pollFirst()       { return this.poll(); }
    pollLast() {
        return this._data.length ? this._data.pop() : null;
    }

    getFirst()        { return this._data[0]; }
    getLast()         { return this._data[this._data.length - 1]; }

    peek()            { return this._data[0] ?? null; }
    peekFirst()       { return this.peek(); }
    peekLast() {
        return this._data.length
            ? this._data[this._data.length - 1]
            : null;
    }

    clear()           { this._data.length = 0; }
    size()            { return this._data.length; }
    isEmpty()         { return this._data.length === 0; }

    contains(v)       { return this._data.includes(v); }

    iterator() {
        let i = 0;
        const arr = this._data;
        return {
            hasNext() { return i < arr.length; },
            next()    { return arr[i++]; }
        };
    }

    [Symbol.iterator]() {
        return this._data[Symbol.iterator]();
    }

    toArray() {
        return this._data.slice();
    }
};

globalThis.ReentrantLock = class ReentrantLock {
    constructor(fair = false) {
        this._fair = !!fair;
        this._holdCount = 0;
        this._owner = null;
    }

    lock() {
        const me = Symbol.for("__js_thread__");

        if (this._owner === null || this._owner === me) {
            this._owner = me;
            this._holdCount++;
            return;
        }

        throw new Error("ReentrantLock shim does not support contention");
    }

    unlock() {
        if (this._holdCount > 0) {
            this._holdCount--;

            if (this._holdCount === 0) {
                this._owner = null;
            }
        }
    }

    tryLock() {
        try {
            this.lock();
            return true;
        } catch {
            return false;
        }
    }

    isLocked() {
        return this._holdCount > 0;
    }

    getHoldCount() {
        return this._holdCount;
    }

    isFair() {
        return this._fair;
    }

    newCondition() {
        return {};
    }
};

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

function _unwrapCollection(col) {
    if (col == null) return [];
    if (Array.isArray(col)) return col;
    // Unwrap backing fields commonly generated by Kotlinx serialization or Kotlin/JS
    if (col._a && Array.isArray(col._a)) return col._a;
    if (col._arr && Array.isArray(col._arr)) return col._arr;
    if (col._map && typeof col._map === 'object') return Object.values(col._map);
    if (col[Symbol.iterator]) return Array.from(col);
    return [];
}


globalThis.CollectionsKt = {
    build(builder) {
        return builder;
    },
    reversed(collection) {
        const arr = [..._unwrapCollection(collection)];
        return _makeKotlinList(arr.reverse());
    },
    joinToString$default(collection, separator, prefix, postfix, limit, truncated, transform, flags, marker) {
        if (!collection) return "";
        if (!flags || (flags & 1))  separator = ", ";
        if (!flags || (flags & 2))  prefix    = "";
        if (!flags || (flags & 4))  postfix   = "";
        if (!flags || (flags & 8))  limit     = -1;
        if (!flags || (flags & 16)) truncated = "...";
        if (!flags || (flags & 32)) transform = null;

        let items = _unwrapCollection(collection);
        if (typeof _wrapKotlinObject === 'function') {
            items = items.map(_wrapKotlinObject);
        }

        let over = false;
        if (limit >= 0 && items.length > limit) { items = items.slice(0, limit); over = true; }
        const parts = items.map(x => {
            return transform ? transform(x) : String(x);
        });
        if (over) parts.push(truncated);
        return prefix + parts.join(separator) + postfix;
    },
    flatten(collections) {
        const out = [];

        for (const collection of _unwrapCollection(collections)) {
            if (collection == null) continue;

            for (const item of _unwrapCollection(collection)) {
                out.push(item);
            }
        }

        return _makeKotlinList(out);
    },
    last(collection) {
        const arr = _unwrapCollection(collection);
        if (arr.length === 0) {
            throw new Error("NoSuchElementException");
        }
        return arr[arr.length - 1];
    },
    throwIndexOverflow() { throw new RangeError("Index overflow"); },
    listOf(...args) { return args.length === 1 && Array.isArray(args[0]) ? args[0] : Array.from(args); },
    toList(collection) { return _unwrapCollection(collection); },
    listOfNotNull(arr) {
        if (Array.isArray(arr)) {
            return arr.filter(x => x !== null && x !== undefined && x !== 0);
        }
        const unwrapped = _unwrapCollection(arr);
        if (unwrapped.length > 0) return unwrapped.filter(x => x !== null && x !== undefined && x !== 0);
        return arr != null && arr !== 0 ? [arr] : [];
    },
    collectionSizeOrDefault(collection, default_) {
        if (!collection) return 0;
        if (collection.length !== undefined) return collection.length;
        if (collection.size !== undefined) return typeof collection.size === 'function' ? collection.size() : collection.size;
        return _unwrapCollection(collection).length ?? default_;
    },
    addAll(collection, elements) {
        const target = collection?._a ?? collection?._arr ?? collection;
        const items = _unwrapCollection(elements);
        for (const item of items) {
            if (typeof target.push === 'function') {
                target.push(item);
            } else if (typeof target.add === 'function') {
                target.add(item);
            }
        }
        return true;
    },
    randomOrNull(collection, random) {
        const arr = _unwrapCollection(collection);
        if (arr.length === 0) return 0;
        return arr[Math.floor(Math.random() * arr.length)] ?? 0;
    },

    drop(collection, n) {
        const arr = _unwrapCollection(collection);
        return _makeKotlinList(arr.slice(Math.max(0, n)));
    },

    take(collection, n) {
        return _unwrapCollection(collection).slice(0, Math.max(0, n));
    },

    dropLast(collection, n) {
        const arr = _unwrapCollection(collection);
        return arr.slice(0, Math.max(0, arr.length - n));
    },

    takeLast(collection, n) {
        const arr = _unwrapCollection(collection);
        return arr.slice(Math.max(0, arr.length - n));
    },

    emptyMap:     () => new LinkedHashMap(),
    emptySet:     () => new Set(),
    setOf:        (...args) => new Set(args),
    mapOf:        (...args) => new LinkedHashMap(args),
    plus:         (a, b)   => [..._unwrapCollection(a), ..._unwrapCollection(b)],
    single:       (list)   => { const arr = _unwrapCollection(list); if (arr.length !== 1) throw new Error("Expected single element"); return arr[0]; },
    firstOrNull:  (list, pred) => {
        const arr = _unwrapCollection(list);
        return pred ? (arr.find(pred) ?? null) : (arr[0] ?? null);
    },
    filter:       (list, pred) => _unwrapCollection(list).filter(pred),
    map:          (list, fn)   => _unwrapCollection(list).map(fn),
    forEach:      (list, fn)   => _unwrapCollection(list).forEach(fn),

    emptyList:     () => _mutableList(),
    toMutableList: (col) => _mutableList(_unwrapCollection(col)),
    mutableListOf: (...args) => _mutableList(args.length === 1 && Array.isArray(args[0]) ? args[0] : args),
    createListBuilder: () => _mutableList(),
    getOrNull(collection, index) {
        const arr = _unwrapCollection(collection);
        return (index >= 0 && index < arr.length) ? arr[index] : null;
    },

    first(collection, predicate) {
        const arr = _unwrapCollection(collection);
        if (predicate) {
            const found = arr.find(predicate);
            if (found === undefined) throw new Error("NoSuchElementException");
            return found;
        }
        if (arr.length === 0) throw new Error("NoSuchElementException");
        return arr[0];
    },
};

function _mutableList(items = []) {
    const arr = Array.isArray(items) ? [...items] : [...items];
    arr.add    = (item) => { arr.push(item); return true; };
    arr.addAll = (col)  => {
        const els = _unwrapCollection(col);
        for (const e of els) arr.push(e);
        return true;
    };
    arr.remove = (item) => {
        const i = arr.indexOf(item);
        if (i >= 0) { arr.splice(i, 1); return true; }
        return false;
    };
    arr.isEmpty   = () => arr.length === 0;

    Object.defineProperty(arr, 'size', {
        get() { return arr.length; },
        configurable: true
    });
    arr.get_size = () => arr.length;

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

        // Kotlin LinkedHashMap(capacity)
        if (typeof init === "number" || init == null) {
            return;
        }

        // LinkedHashMap(existingMap)
        if (init instanceof Map) {
            for (const [k, v] of init.entries()) {
                this.set(k, v);
            }
            return;
        }

        // Iterable<Entry<K,V>>
        if (Symbol.iterator in Object(init)) {
            for (const [k, v] of init) {
                this.set(k, v);
            }
        }
    }

    get(key) { return super.get(key) ?? null; }
    put(key, value) { this.set(key, value); return null; }
    containsKey(key) { return this.has(key) ? 1 : 0; }
    containsValue(val) {
        for (const v of super.values()) {
            if (v === val) return 1;
        }
        return 0;
    }
    remove(key) {
        const v = this.get(key);
        this.delete(key);
        return v;
    }
    isEmpty() { return this.size === 0 ? 1 : 0; }
    size() { return super.size; }
    entrySet() {
        const entries = [...super.entries()].map(([k, v]) => ({
            getKey: () => k,
            getValue: () => v,
        }));

        entries.iterator = function() {
            let i = 0;
            return {
                hasNext: () => (i < entries.length ? 1 : 0),
                next: () => entries[i++],
            };
        };

        return entries;
    }
    keySet() {
        return [...super.keys()];
    }

    values() {
        return [...super.values()];
    }

    putAll(map) {
        if (map instanceof Map) {
            for (const [k, v] of map) {
                this.set(k, v);
            }
        }
        return this;
    }
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

globalThis.FormBody_Builder = class FormBody_Builder {
    constructor() {
        this._params = [];
    }

    add(key, value) {
        this._params.push([String(key), String(value)]);
        return this;
    }

    push(key, value) {
        this._params.push([String(key), String(value)]);
        return this;
    }

    build() {
        return new FormBody(this._params);
    }
};

globalThis.FormBody = class FormBody {
    constructor(params = []) {
        this._params = params;
    }

    toRequestBody() {
        return this;
    }

    contentType() {
        return "application/x-www-form-urlencoded";
    }

    toString() {
        return this._params
            .map(([k, v]) =>
                encodeURIComponent(k) + "=" + encodeURIComponent(v))
            .join("&");
    }
};

globalThis.ArrayList = class ArrayList {
    constructor() { this._a = []; }
    push(item) {
        if (item !== undefined) this._a.push(item);
        return this;
    }
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

    forEach(cb)        { this._a.forEach(cb); }
    filter(cb)         { return this._a.filter(cb); }

    contains(item) {
        return this._a.includes(item) ? 1 : 0;
    }
    [Symbol.iterator](){ return this._a[Symbol.iterator](); }

    get length() { return this._a.length; }
    get length_val() { return this._a.length; }
};

globalThis.HashSet = class HashSet {
    constructor(iterable) {
        this._s = new Set();
        if (iterable) {
            for (const item of iterable) {
                this._s.add(item);
            }
        }
    }

    push(item) {
        this._s.add(item);
        return this;
    }

    add(item) {
        this._s.add(item);
        return this;
    }

    remove(item) {
        return this._s.delete(item) ? 1 : 0;
    }

    contains(item) {
        return this._s.has(item) ? 1 : 0;
    }

    size() {
        return this._s.size;
    }

    isEmpty() {
        return this._s.size === 0 ? 1 : 0;
    }

    clear() {
        this._s.clear();
    }

    toArray() {
        return Array.from(this._s);
    }

    forEach(fn) {
        this._s.forEach(fn);
    }

    [Symbol.iterator]() {
        return this._s[Symbol.iterator]();
    }

    get length_val() {
        return this._s.size;
    }

    get size_val() {
        return this._s.size;
    }

    addAll(items) {
        for (const item of items) {
            this._s.add(item);
        }
        return this;
    }

    removeAll(items) {
        for (const item of items) {
            this._s.delete(item);
        }
        return this;
    }

    containsAll(items) {
        for (const item of items) {
            if (!this._s.has(item)) return 0;
        }
        return 1;
    }

    toList() {
        return Array.from(this._s);
    }
};

globalThis.kotlin = Object.assign(globalThis.kotlin ?? {}, { Unit: { INSTANCE: undefined } });
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
    forLanguageTag(tag) { return tag; },
};

globalThis.Collator = {
    getInstance(locale) {
        return {
            locale,
            compare(a, b) {
                return a.localeCompare(b, locale);
            },
        };
    },
};

globalThis.PropertyResourceBundle = class PropertyResourceBundle {
    constructor(reader) {
        this._data = {};
        // Can't load .properties files in JS, fallback to empty
        // Intl will return [key] for missing keys which is fine
    }

    containsKey(key) {
        return 0; // always miss, fall through to [key] fallback
    }

    getString(key) {
        return `[${key}]`;
    }
};

globalThis.InputStreamReader = class InputStreamReader {
    constructor(stream, encoding) {}
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
        this._annotations = [];
        this._elementAnnotations = [];
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

    pushAnnotation(annotation) {
        this._annotations.push(annotation);
    }

    getAnnotations() {
        return this._annotations;
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

    decodeLongElement(descriptor, index) {
        let value;

        if (Array.isArray(this._json)) {
            value = this._json[index];
        } else {
            value = this._json[descriptor.getElementName(index)];
        }

        return value;
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
    lazy(modeOrInitializer, initializer) {
        const init = typeof modeOrInitializer === 'function'
            ? modeOrInitializer
            : initializer;

        let value;
        let initialized = false;
        return {
            getValue() {
                if (!initialized) {
                    value = init();
                    initialized = true;
                    console.log("lazy initialized:", typeof value, value?.constructor?.name);
                }
                return value;
            }
        };
    }
};

globalThis.ContextKt_special__inlined_get_1 = class ContextKt_special__inlined_get_1 {
    getType() { return null; }
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

globalThis.Lambda = class Lambda {
    constructor(arity) {
        this.arity = arity ?? 0;
    }

    // invoke() is overridden by every concrete lambda subclass;
    // the base just forwards up to 4 positional args
    invoke(p0, p1, p2, p3) {
        return undefined;
    }

    toString() {
        return `Lambda/${this.arity}`;
    }
};

// Kotlin also emits FunctionN base types; alias them all to Lambda
globalThis.Function0  = Lambda;
globalThis.Function1  = Lambda;
globalThis.Function2  = Lambda;
globalThis.Function3  = Lambda;
globalThis.Function4  = Lambda;
globalThis.FunctionN  = Lambda;

globalThis.SetsKt = {
    emptySet() {
        return new Set();
    },

    contains(set, value) {
        if (set == null) return 0;
        // Works for native Set and any array-like
        if (set instanceof Set) return set.has(value) ? 1 : 0;
        if (Array.isArray(set)) return set.includes(value) ? 1 : 0;
        return 0;
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

globalThis.Long = {
    valueOf(value, ignored) {
        const n = typeof value === "number" ? value : parseInt(value) || 0;
        return {
            _value: n,
            longValue() { return n; },
            intValue() { return n; },
            floatValue() { return n; },
            doubleValue() { return n; },
            toString() { return String(n); },
            valueOf() { return n; },
            selectFirst() { return 0; },  // guard against register reuse
            text() { return ""; },
            attr() { return ""; },
        };
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
        SECONDS:      { toMillis(v) { return v * 1000; },    toSeconds(v) { return v; } },
        MINUTES:      { toMillis(v) { return v * 60000; },   toSeconds(v) { return v * 60; } },
        HOURS:        { toMillis(v) { return v * 3600000; },  toSeconds(v) { return v * 3600; } },
        MILLISECONDS: { toMillis(v) { return v; },            toSeconds(v) { return Math.floor(v / 1000); } },
        DAYS:         { toMillis(v) { return v * 86400000; }, toSeconds(v) { return v * 86400; } },
    },
};

globalThis.Calendar = java.util.Calendar;
globalThis.TimeUnit = java.util.concurrent.TimeUnit;

globalThis.SimpleDateFormat = class SimpleDateFormat {
    constructor(pattern, locale) {
        this._pattern = pattern;
        this._locale = locale;
    }

    parse(str, pos) {
        if (!str) return null;
        const trimmed = str.trim();
        if (!trimmed) return null;
        const ms = Date.parse(trimmed);
        if (pos) pos.setIndex(str.length);
        return isNaN(ms) ? null : new Date(ms);
    }

    _parseWithPattern(str, pattern) {
        // Map Java pattern tokens to regex + capture groups
        const tokenMap = [
            { token: 'yyyy', re: '(\\d{4})',   key: 'year' },
            { token: 'yy',   re: '(\\d{2})',   key: 'year2' },
            { token: 'MMMM', re: '([A-Za-z]+)', key: 'monthName' },
            { token: 'MMM',  re: '([A-Za-z]+)', key: 'monthShort' },
            { token: 'MM',   re: '(\\d{1,2})', key: 'month' },
            { token: 'M',    re: '(\\d{1,2})', key: 'month' },
            { token: 'dd',   re: '(\\d{1,2})', key: 'day' },
            { token: 'd',    re: '(\\d{1,2})', key: 'day' },
            { token: 'HH',   re: '(\\d{1,2})', key: 'hour' },
            { token: 'H',    re: '(\\d{1,2})', key: 'hour' },
            { token: 'hh',   re: '(\\d{1,2})', key: 'hour' },
            { token: 'h',    re: '(\\d{1,2})', key: 'hour' },
            { token: 'mm',   re: '(\\d{1,2})', key: 'minute' },
            { token: 'ss',   re: '(\\d{1,2})', key: 'second' },
            { token: 'a',    re: '(AM|PM|am|pm)', key: 'ampm' },
        ];

        const MONTHS_LONG  = ['january','february','march','april','may','june','july','august','september','october','november','december'];
        const MONTHS_SHORT = ['jan','feb','mar','apr','may','jun','jul','aug','sep','oct','nov','dec'];

        let reStr = pattern.replace(/[.*+?^${}()|[\]\\]/g, '\\$&');
        const keys = [];

        // Replace escaped pattern chars first
        for (const { token, re, key } of tokenMap) {
            const escaped = token.replace(/[.*+?^${}()|[\]\\]/g, '\\$&');
            if (reStr.includes(escaped)) {
                reStr = reStr.replace(escaped, re);
                keys.push(key);
            }
        }

        const match = new RegExp('^' + reStr + '$', 'i').exec(str.trim());
        if (!match) return null;

        const groups = {};
        keys.forEach((key, i) => { groups[key] = match[i + 1]; });

        let year   = parseInt(groups.year   ?? groups.year2 ?? '1970');
        if (groups.year2) year += year < 70 ? 2000 : 1900;
        let month  = 0;
        let day    = parseInt(groups.day ?? '1');
        let hour   = parseInt(groups.hour ?? '0');
        let minute = parseInt(groups.minute ?? '0');
        let second = parseInt(groups.second ?? '0');

        if (groups.monthName)  month = MONTHS_LONG.indexOf(groups.monthName.toLowerCase());
        else if (groups.monthShort) month = MONTHS_SHORT.indexOf(groups.monthShort.toLowerCase());
        else if (groups.month) month = parseInt(groups.month) - 1;

        if (groups.ampm) {
            const pm = groups.ampm.toLowerCase() === 'pm';
            if (pm && hour < 12) hour += 12;
            if (!pm && hour === 12) hour = 0;
        }

        return new Date(year, month, day, hour, minute, second);
    }

    format(date) {
        return (date instanceof Date ? date : new Date(date)).toISOString();
    }

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
        return { value: m[0], groupValues: m, destructured: { component1: () => m[1] }, getValue() { return this; }, };
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

    test(str) { return this._re.test(str); }

    static find$default(regex, input, startIndex, flags, marker) {
        if (flags & 1) startIndex = 0;
        return regex.find(input, startIndex);
    }
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

    setVisible(v) {
        this._visible = v;
        return this;
    }

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

    setKey(v)          { this._key = v; return this; }
    setTitle(v)        { this._title = v; return this; }
    setSummary(v)      { this._summary = v; return this; }
    setDefaultValue(v) { this._default = v; return this; }
    setDialogTitle(v) { this._dialogTitle = v; return this; }
    setDialogMessage(v) { this._dialogMessage = v; return this; }

    setOnPreferenceChangeListener(l) {
        this._changeListener = l;
        return this;
    }

    setOnBindEditTextListener(l) {
        this._bindListener = l;
        return this;
    }

    _toManifest() {
        return {
            key: this._key ?? "",
            label: this._title ?? this._key ?? "",
            type: "string",
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

    build() {
        return _makeOkHttpClient(
            this._useCloudflare,
            [...this._interceptors],
            [...this._networkInterceptors]
        );
    }
});

const _makeKotlinList = (arr) => {
    arr.iterator = function() {
        let i = 0;
        return {
            hasNext() { return i < arr.length ? 1 : 0; },
            next() { return arr[i++]; }
        };
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

globalThis._JsoupNull = new Proxy({}, {
    get(_, prop) {
        if (prop === "then") return undefined;
        return (...args) => _JsoupNull;
    }
});

globalThis.JsoupDocument = class JsoupDocument {
    constructor(html, baseUri = "") {
        this._html = html;
        this._$ = parseHTML(html);
        this._baseUri = baseUri || "";
    }

    select(selector)              { return new JsoupElements(this._$(selector), this._baseUri); }
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
        if (!selector) return 0;
        const result = this.select(selector).first();
        return result === _JsoupNull ? 0 : result;
    }

    location() {
        return this._baseUri;
    }

    wholeText() { return this._$("body").text(); }

    absUrl(attributeKey) {
        return this.body()?.absUrl(attributeKey) ?? "";
    }
};

globalThis.JsoupElements = class JsoupElements {
    constructor(raw, baseUri = "") {
        this._baseUri = baseUri;
        this._els = raw.map(item => new JsoupElement(item._raw, this._baseUri));
    }

    location() {
        return this._baseUri;
    }

    first()  { return this._els.length === 0 ? _JsoupNull : this._els[0]; }
    get(i)   { return this._els[i] ?? _JsoupNull; }
    last()   { return this._els[this._els.length - 1] ?? _JsoupNull; }
    size()      { return this._els.length; }
    isEmpty()   { return this._els.length === 0; }
    text()      { return this._els.map(el => el.text()).join(""); }
    html()      { return this._els[0]?.html() ?? ""; }
    outerHtml() { return this._els[0]?.outerHtml() ?? ""; }
    attr(name)  { return this._els[0]?.attr(name) ?? null; }
    select(sel) {
        return this._els[0] ? this._els[0].select(sel) : new JsoupElements([], this._baseUri);
    }
    forEach(fn) { this._els.forEach(fn); }
    map(fn)     { return this._els.map(fn); }
    filter(fn)  { return this._els.filter(fn); }

    eachText() {
        return this._els.map(el => el.text());
    }

    selectFirst(selector) {
        if (!selector) return 0;
        const result = this.select(selector).first();
        return result === _JsoupNull ? 0 : result;
    }

    absUrl(attributeKey) {
        return this._els[0]?.absUrl(attributeKey) ?? "";
    }

    get length_val() { return this.size(); }
    get length() {
        return this._els.length;
    }
    get size_val()   { return this.size(); }

    ownText() { return this._els[0]?.ownText() ?? ""; }

    [Symbol.iterator]() { return this._els[Symbol.iterator](); }
};

globalThis.JsoupElement = class JsoupElement {
    constructor(raw, baseUri = "") {
        this._raw = raw;
        this._baseUri = baseUri;
    }

    text()        { return this._raw.text; }
    html()        { return this._raw.html; }
    outerHtml()   { return this._raw.outer; }
    attr(name) {
        if (name.startsWith("abs:")) {
            let realAttr = name.slice(4);
            if (realAttr === "img") realAttr = "src"; // translator bug workaround
            const val = this._raw.attrs?.[realAttr] ?? null
            if (!val) return "";
            if (/^https?:\/\//i.test(val)) return val;
            try {
                return new URL(val, this._baseUri).href;
            } catch(e) {
                return val;
            }
        }
        return this._raw.attrs?.[name] ?? null;
    }

    eachText() {
        const $ = parseHTML(this._raw.html);
        const results = [];
        $(this._raw.tag ?? "*").each((_, el) => {
            results.push($(el).text());
        });
        return results;
    }
    hasAttr(name) {
        return (name in (this._raw.attrs ?? {})) ? 1 : 0;
    }
    id()          { return this.attr("id") ?? ""; }
    className()   { return this.attr("class") ?? ""; }
    tagName()     { return this._raw.tag ?? ""; }

    select(selector) {
        const $ = parseHTML(this._raw.html);
        return new JsoupElements($(selector), this._baseUri);
    }

    ownText() {
        return this._raw.own_text ?? "";
    }

    wholeText() { return this._raw.text; }

    selectFirst(selector) {
        if (!selector) return 0;
        const result = this.select(selector).first();
        return result === _JsoupNull ? 0 : result;
    }


    data() { return this._raw.text; }

    absUrl(attributeKey) {
        let relUrl = this.attr(attributeKey);
        if (!relUrl) return "";
        relUrl = relUrl.trim();

        let fullUrl = relUrl;
        if (this._baseUri && !/^[a-z][a-z0-9+.-]*:/i.test(relUrl)) {
            try {
                fullUrl = new URL(relUrl, this._baseUri).href;
            } catch (e) {
                fullUrl = relUrl;
            }
        }

        if (attributeKey === "href" || attributeKey === "src") {
            globalThis.__lastExtractedUrl = fullUrl;
        }

        return fullUrl;
    }
};

globalThis["JsoupExtensionsKt"] = {
    ["asJsoup$default"]: function(body, baseUri, charset, flags) {
        const html = typeof body?.string === "function" ? body.string() : body._text;
        return new JsoupDocument(html, baseUri);
    },
    ["asJsoup"]: function(body, baseUri, charset) {
        return new JsoupDocument(body.string(), baseUri);
    },
};

globalThis.Jsoup = {
    parseBodyFragment(html, baseUri = "") {
        return new JsoupDocument(html, baseUri);
    },
    parse(html, baseUri = "") {
        return new JsoupDocument(html, baseUri);
    },
};




globalThis.RateLimitInterceptorKt = {
    rateLimit(builder, permits, period, timeUnit) {
        // Convert the given period into milliseconds using our TimeUnit helper
        let periodMs = 1000; // default fallback to 1 second
        if (timeUnit && typeof timeUnit.toMillis === 'function') {
            periodMs = timeUnit.toMillis(period);
        } else if (typeof period === 'number') {
            periodMs = period * 1000;
        }

        // Calculate a safe minimum spacing delay between individual requests
        const delayBetweenRequests = Math.ceil(periodMs / (permits || 1));

        // Inject the timing constraints into the custom client builder instance config
        if (builder) {
            builder._rateLimitDelay = delayBetweenRequests;
        }

        return builder;
    },

    // Handle standard structural compiler variants ($default variations)
    rateLimit$default(builder, permits, period, timeUnit, mask, obj) {
        // Handle Kotlin default arguments bitmask mapping
        if ((mask & 2) !== 0) period = 1;
        if ((mask & 4) !== 0) timeUnit = globalThis.TimeUnit?.SECONDS;

        return this.rateLimit(builder, permits, period, timeUnit);
    }
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

globalThis.Exception = class Exception extends Error {
    constructor(msg) { super(msg ?? "Exception"); this.name = "Exception"; }
};

globalThis.Null = class Null extends Error {
    constructor(msg) { super(msg ?? "null"); this.name = "Null"; }
};

globalThis.NullPointerException = class NullPointerException extends Error {
    constructor(msg) { super(msg ?? "NullPointerException"); this.name = "NullPointerException"; }
};

globalThis.IllegalStateException = class IllegalStateException extends Error {
    constructor(msg) { super(msg ?? "IllegalStateException"); this.name = "IllegalStateException"; }
};

globalThis.IllegalArgumentException = class IllegalArgumentException extends Error {
    constructor(msg) { super(msg ?? "IllegalArgumentException"); this.name = "IllegalArgumentException"; }
};

globalThis.RuntimeException = class RuntimeException extends Error {
    constructor(msg) { super(msg ?? "RuntimeException"); this.name = "RuntimeException"; }
};

globalThis.UnsupportedOperationException = class UnsupportedOperationException extends Error {
    constructor(msg) { super(msg ?? "UnsupportedOperationException"); this.name = "UnsupportedOperationException"; }
};

globalThis.IndexOutOfBoundsException = class IndexOutOfBoundsException extends Error {
    constructor(msg) { super(msg ?? "IndexOutOfBoundsException"); this.name = "IndexOutOfBoundsException"; }
};

globalThis.NoSuchElementException = class NoSuchElementException extends Error {
    constructor(msg) { super(msg ?? "NoSuchElementException"); this.name = "NoSuchElementException"; }
};

// 1. Primitives first
globalThis.JsonPrimitive = class JsonPrimitive {
    constructor(value) { this._value = value; }
    toString()     { return String(this._value); }
    toJsonString() { return JSON.stringify(this._value); }
};

// 2. Then collections
globalThis.JsonArray = class JsonArray {
    constructor(arr) { this._arr = arr ?? []; }
    get(i)         { return this._arr[i] ?? null; }
    [Symbol.iterator]() { return this._arr[Symbol.iterator](); }
};

globalThis.JsonObject = class JsonObject {
    constructor(map) { this._map = map ?? {}; }
    get(key)       { return this._map[key] ?? null; }
};

function deepSerialize(value) {
    if (value == null) return null;

    if (value instanceof JsonObject || (value && typeof value === 'object' && '_map' in value)) {
        const out = {};
        const mapSource = value._map ?? value;
        for (const [k, v] of Object.entries(mapSource)) {
            if (k === '_map') continue; // Don't serialize the property name itself
            out[k] = deepSerialize(v);
        }
        return out;
    }

    if (value instanceof JsonArray || (value && typeof value === 'object' && '_arr' in value)) {
        const arrSource = value._arr ?? value;
        return arrSource.map(deepSerialize);
    }

    if (value instanceof JsonPrimitive) {
        return value._value;
    }

    if (Array.isArray(value)) {
        return value.map(deepSerialize);
    }

    if (typeof value === 'object') {
        const out = {};
        for (const [k, v] of Object.entries(value)) {
            out[k] = deepSerialize(v);
        }
        return out;
    }

    return value;
}

// 4. NOW add the methods that use deepSerialize
Object.assign(JsonArray.prototype, {
    toString()     { return JSON.stringify(deepSerialize(this)); },
    toJsonString() { return JSON.stringify(deepSerialize(this)); },
});

Object.assign(JsonObject.prototype, {
    toString()     { return JSON.stringify(deepSerialize(this)); },
    toJsonString() { return JSON.stringify(deepSerialize(this)); },
    toJsonRequestBody() {
        return {
            contentType: "application/json",
            content: JSON.stringify(deepSerialize(this)),
        };
    },
});

JsonObject.Companion = {
    serializer() {
        return {
            descriptor: { serialName: "JsonObject" },
            serialize: (encoder, value) => { /* ... */ },
            deserialize: (decoder) => { return new JsonObject(); }
        };
    }
};

// 5. Builders
globalThis.JsonArrayBuilder = class JsonArrayBuilder {
    constructor() { this._arr = []; }
    add(value) { this._arr.push(value); }
    build() { return new JsonArray(this._arr); }
};

globalThis.JsonObjectBuilder = class JsonObjectBuilder {
    constructor() { this._map = {}; }
    put(key, value) {
        this._map[key] = value instanceof JsonPrimitive ? value._value : value;
    }
    build() { return new JsonObject(this._map); }
};

function callBlock(block, arg) {
    if (typeof block === 'function') {
        block(arg);
    } else {
        block.invoke(arg);
    }
}

globalThis.JsonElementBuildersKt = {
    put(builder, key, value) {
        if (builder instanceof JsonObjectBuilder) {
            builder.put(key, value);
        }
    },
    buildJsonObject(block) {
        const builder = new JsonObjectBuilder();
        callBlock(block, builder);
        return builder.build();
    },
    putJsonArray(builder, key, block) {
        const arr = new JsonArrayBuilder();
        callBlock(block, arr);
        builder.put(key, arr.build());
    },
    addJsonObject(builder, block) {
        const obj = new JsonObjectBuilder();
        callBlock(block, obj);
        builder.add(obj.build());
    },
    buildJsonArray(block) {
        const arr = new JsonArrayBuilder();
        callBlock(block, arr);
        return arr.build();
    },
};

JsonArray.Companion = {
    serializer() {
        return { descriptor: { serialName: "JsonArray" } };
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
        encodeToString(serializer, obj) {
            return obj?.toJsonString
                ? obj.toJsonString()
                : JSON.stringify(deepSerialize(obj));
        }
    },
    decodeFromString(deserializer, str) {
        const parsed = JSON.parse(str);
        const data = parsed?.data ?? parsed;
        const decoder = new JsonDecoder(data, null);
        return deserializer.deserialize(decoder);
    },
    encodeToString(serializer, obj) {
        return obj?.toJsonString
            ? obj.toJsonString()
            : JSON.stringify(deepSerialize(obj));
    }
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

globalThis.CheckBoxPreference = class CheckBoxPreference {
    constructor(context = null) {
        this.context = context;
        this._changeListener = null;
        this.key = "";
        this.title = "";
        this.summary = "";
        this.summaryOn = "";
        this.summaryOff = "";
        this.defaultValue = false;
        this.checked = false;
    }

    setOnPreferenceChangeListener(listener) {
        this._changeListener = listener;
        return this;
    }

    getOnPreferenceChangeListener() {
        return this._changeListener;
    }

    setKey(v) { this.key = v; }
    getKey() { return this.key; }

    setTitle(v) { this.title = v; }
    getTitle() { return this.title; }

    setSummary(v) { this.summary = v; }
    getSummary() { return this.summary; }

    setSummaryOn(v) { this.summaryOn = v; }
    setSummaryOff(v) { this.summaryOff = v; }

    setDefaultValue(v) { this.defaultValue = !!v; }
    getDefaultValue() { return this.defaultValue; }

    setChecked(v) { this.checked = !!v; }
    isChecked() { return this.checked ? 1 : 0; }
};

globalThis["Filter_Header"]    = Filter.Header;
globalThis["AnimeFilter_Header"] = Filter.Header;
globalThis["Filter$Header"] = Filter.Header;

globalThis["Filter_Separator"] = Filter.Separator;
globalThis["Filter$Separator"] = Filter.Separator;
globalThis["AnimeFilter_Separator"] = Filter.Separator;

globalThis["Filter_Select"]    = Filter.Select;
globalThis["Filter$Select"]    = Filter.Select;
globalThis["AnimeFilter_Select"] = Filter.Select;

globalThis["Filter_Text"]      = Filter.Text;
globalThis["Filter$Text"]      = Filter.Text;
globalThis["AnimeFilter_Text"] = Filter.Text;

globalThis["Filter_CheckBox"]  = Filter.CheckBox;
globalThis["Filter$CheckBox"]  = Filter.CheckBox;
globalThis["AnimeFilter_CheckBox"] = Filter.CheckBox;

globalThis["Filter_TriState"]  = Filter.TriState;
globalThis["Filter$TriState"]  = Filter.TriState;
globalThis["AnimeFilter_TriState"] = Filter.TriState;

globalThis["Filter_Group"]     = Filter.Group;
globalThis["Filter$Group"]     = Filter.Group;
globalThis["AnimeFilter_Group"] = Filter.Group;

globalThis["Filter_Sort"]      = Filter.Sort;
globalThis["Filter$Sort"]      = Filter.Sort;
globalThis["AnimeFilter_Sort"] = Filter.Sort;

globalThis["Filter_Sort_Selection"] = Filter.Sort.Selection;
globalThis["Filter$Sort$Selection"] = Filter.Sort.Selection;
globalThis["AnimeFilter_Sort_Selection"] = Filter.Sort.Selection;

globalThis.CacheControl_Builder = class CacheControl_Builder {
    constructor() {
        this._maxAgeSeconds = -1;
        this._noCache = false;
        this._noStore = false;
        this._onlyIfCached = false;
    }

    maxAge(value, unit) {
        // unit is a TimeUnit-like object or string; normalize to seconds
        if (unit && typeof unit.toSeconds === "function") {
            this._maxAgeSeconds = unit.toSeconds(value);
        } else if (typeof unit === "string") {
            switch (unit.toUpperCase()) {
                case "SECONDS":      this._maxAgeSeconds = value; break;
                case "MINUTES":      this._maxAgeSeconds = value * 60; break;
                case "HOURS":        this._maxAgeSeconds = value * 3600; break;
                case "DAYS":         this._maxAgeSeconds = value * 86400; break;
                default:             this._maxAgeSeconds = value; break;
            }
        } else {
            this._maxAgeSeconds = value;
        }
        return this;
    }

    noCache()      { this._noCache = true;      return this; }
    noStore()      { this._noStore = true;       return this; }
    onlyIfCached() { this._onlyIfCached = true;  return this; }

    build() {
        return {
            maxAgeSeconds: this._maxAgeSeconds,
            noCache:       this._noCache,
            noStore:       this._noStore,
            onlyIfCached:  this._onlyIfCached,
            toString() {
                const parts = [];
                if (this.maxAgeSeconds >= 0) parts.push(`max-age=${this.maxAgeSeconds}`);
                if (this.noCache)            parts.push("no-cache");
                if (this.noStore)            parts.push("no-store");
                if (this.onlyIfCached)       parts.push("only-if-cached");
                return parts.join(", ");
            },
        };
    }
}

globalThis.CacheControl = {
    Builder: CacheControl_Builder,
};

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
globalThis.Headers_Builder = Headers.Builder;

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

    const _PrefsInlined = function PreferencesKt_getPreferences__inlined_get_1() {
        this._token = _makePrefsTypeToken();
    };
    _PrefsInlined.prototype.getType  = function() { return _makePrefsTypeToken(); };
    _PrefsInlined.prototype.invoke   = function() { return _makePrefsTypeToken(); };
    _PrefsInlined.prototype.toString = function() { return "PreferencesKt_get_1"; };

    globalThis["PreferencesKt_getPreferences__inlined_get_1"] = _PrefsInlined;
    globalThis["PreferencesKt_get_1"] = _PrefsInlined;
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
        this._ordinal = ordinal ?? 0;
    }
    ordinal()  { return this._ordinal; }
    name()     { return this.name; }
    toString() { return this.name; }
};

globalThis.HelperKt_special__inlined_get_1 = class HelperKt_special__inlined_get_1 {
    getType() { return null; }
};

globalThis.DurationUnit = {
    NANOSECONDS: 'NANOSECONDS',
    MICROSECONDS: 'MICROSECONDS',
    MILLISECONDS: 'MILLISECONDS',
    SECONDS: 'SECONDS',
    MINUTES: 'MINUTES',
    HOURS: 'HOURS',
    DAYS: 'DAYS',
};

globalThis.Duration = class Duration {
    static Companion = {
        getZERO() {
            return 0;
        },

        getINFINITE() {
            return Number.MAX_SAFE_INTEGER;
        }
    };
};
globalThis.DurationKt = {
    toDuration(value, unit) {
        return {
            value,
            unit,

            inWholeMilliseconds() {
                switch (unit) {
                    case DurationUnit.SECONDS: return value * 1000;
                    case DurationUnit.MINUTES: return value * 60_000;
                    case DurationUnit.HOURS: return value * 3_600_000;
                    case DurationUnit.DAYS: return value * 86_400_000;
                    default: return value;
                }
            },

            toString() {
                return `${value} ${unit}`;
            }
        };
    }
};

globalThis.EnumEntriesKt = {
    enumEntries(values) {
        return values;
    }
};

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

globalThis.RequestsKt = {
    GET(url, headers, cache) {
        if ((!url || url === "") && globalThis.__lastExtractedUrl) {
            url = globalThis.__lastExtractedUrl;
        }
        globalThis.__lastExtractedUrl = "";

        const h = typeof headers?.build === 'function' ? headers.build() : headers;
        return new Request.Builder().url(url).headers(h ?? new Headers()).cacheControl(cache ?? null).build();
    },
    GET$default(url, headers, cache, flags, mask) {
        if ((!url || url === "") && globalThis.__lastExtractedUrl) {
            url = globalThis.__lastExtractedUrl;
        }
        globalThis.__lastExtractedUrl = "";

        const h = typeof headers?.build === 'function' ? headers.build() : headers;
        return new Request.Builder().url(url).headers(h ?? new Headers()).cacheControl(cache ?? null).build();
    },
    POST$default(url, headers, body) {
        if ((!url || url === "") && globalThis.__lastExtractedUrl) {
            url = globalThis.__lastExtractedUrl;
        }
        globalThis.__lastExtractedUrl = "";

        const h = typeof headers?.build === 'function' ? headers.build() : headers;
        return new Request.Builder().url(url).headers(h ?? new Headers()).post(body).build();
    },

    POST(url, headers, body) {
        if ((!url || url === "") && globalThis.__lastExtractedUrl) {
            url = globalThis.__lastExtractedUrl;
        }
        globalThis.__lastExtractedUrl = "";

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

    queryParameter(name) {
        try {
            return new URL(this._url).searchParams.get(name);
        } catch {
            return null;
        }
    }


    pathSegments() {
        const makeList = (segs) => {
            segs.get = (i) => segs[i];
            segs.contains = (v) => segs.includes(v) ? 1 : 0;
            segs.isEmpty = () => segs.length === 0 ? 1 : 0;
            segs.size = () => segs.length;

            segs.iterator = () => {
                let i = 0;
                return {
                    hasNext: () => i < segs.length ? 1 : 0,
                    next: () => segs[i++]
                };
            };

            return segs;
        };

        try {
            return makeList(
                new URL(this._url).pathname
                    .split("/")
                    .filter(s => s.length > 0)
            );
        } catch {
            return makeList(
                this._url
                    .split("/")
                    .filter(s => s.length > 0)
            );
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
        addPathSegment(v)  { this._url += `/${v}`; return this; }
        addPathSegments(v) {
            const parts = v.split("/").filter(p => p.length > 0);
            for (const part of parts) this._url += `/${part}`;
            return this;
        }
        removeAllQueryParameters(k) {
            try {
                const u = new URL(this._url);
                u.searchParams.delete(k);
                this._url = u.toString();
            } catch {}
            return this;
        }
        setQueryParameter(k, v) {
            try {
                const u = new URL(this._url);
                u.searchParams.set(k, v);
                this._url = u.toString();
            } catch {
                this.addQueryParameter(k, v);
            }
            return this;
        }
        fragment(v) { this._url += `#${v}`; return this; }
        build()     { return new HttpUrl(this._url); }
        toString()  { return this._url; }
        setEncodedQueryParameter(k, v) {
            try {
                const u = new URL(this._url);
                // delete all existing occurrences then re-add raw
                u.searchParams.delete(k);
                this._url = u.toString();
            } catch {}
            const sep = this._url.includes("?") ? "&" : "?";
            this._url += `${sep}${k}=${v}`;
            return this;
        }

        addEncodedQueryParameter(k, v) {
            if (v === null || v === undefined) return this;
            const sep = this._url.includes("?") ? "&" : "?";
            this._url += `${sep}${k}=${v}`;
            return this;
        }

        addEncodedPathSegments(v) {
            // v is already percent-encoded; append verbatim after stripping a
            // leading slash so we don't double-slash
            const segment = String(v).replace(/^\//, "");
            this._url = this._url.replace(/\/$/, "") + "/" + segment;
            return this;
        }
    };
};

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


HttpUrl.Companion = {
    get(url) {
        return new HttpUrl(url?.toString?.() ?? url);
    },

    parse(url) {
        if (!url) return new HttpUrl("");
        try {
            new URL(url.toString());
            return new HttpUrl(url.toString());
        } catch {
            return new HttpUrl("");
        }
    }
};

// String.toHttpUrl() extension
globalThis.toHttpUrl = (str) => new HttpUrl(str.toString());

globalThis.OkHttpClient = class OkHttpClient {
    constructor(useCloudflare = false) {
        this._useCloudflare = useCloudflare;
    }
    newCall(request) {
        return new globalThis._Call(request, false, this);
    }
};
globalThis._Call = class _Call {
    constructor(req, useCloudflare = false, clientInstance = null) {
        this._req = req;
        this._useCloudflare = useCloudflare;
        this._client = clientInstance;
    }

    execute() {
        let url = this._req.url?.toString?.() ?? String(this._req.url);
        const method = this._req.method ?? "GET";
        const headers = this._req.headers?.toFetchHeaders?.() ?? {};
        const body = _serializeBody(this._req.body ?? undefined);

        if ((!url || url === "") && globalThis.__lastExtractedUrl) {
            url = globalThis.__lastExtractedUrl;
        }
        globalThis.__lastExtractedUrl = "";

        const targetedDelay = this._client?._rateLimitDelay ?? 1000;

        const now = Date.now();
        const elapsed = now - (globalThis.__lastFetchTime ?? 0);

        if (elapsed < targetedDelay && globalThis.__lastFetchTime !== 0) {
            const sleepTime = targetedDelay - elapsed;
            if (typeof __native_sleep === 'function') {
                __native_sleep(sleepTime);
            }
        }
        globalThis.__lastFetchTime = Date.now();

        const result = fetchSync(url, { method, headers, body });
        if (result.cookies) {
            for (const [k, v] of Object.entries(result.cookies)) {
                _cookieStore.set(k, v);
            }
            state?.set?.("cookies", Object.fromEntries(_cookieStore));
        }
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
        if (result.cookies) {
            for (const [k, v] of Object.entries(result.cookies)) {
                _cookieStore.set(k, v);
            }
            state?.set?.("cookies", Object.fromEntries(_cookieStore));
        }

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
        create(content, mediaType) {
            return {
                _body: content,
                _contentType: mediaType?._type ?? null,
            };
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

globalThis.MapsKt = {
    mapCapacity(expectedSize) {
        expectedSize = Number(expectedSize) || 0;

        if (expectedSize < 3) {
            return expectedSize + 1;
        }

        if (expectedSize < 1073741824) {
            return expectedSize + Math.floor(expectedSize / 3);
        }

        return 2147483647;
    },

    mutableMapOf(...args) {
        const map = new Map();
        if (args.length === 1 && Array.isArray(args[0])) {
            for (const pair of args[0]) {
                if (pair && pair.first !== undefined) {
                    map.set(pair.first, pair.second);
                } else if (Array.isArray(pair)) {
                    map.set(pair[0], pair[1]);
                }
            }
        }
        return map;
    },

    withDefault(map, defaultFn) {
        return new Proxy(map, {
            get(target, prop, receiver) {
                if (prop in target) {
                    return Reflect.get(target, prop, receiver);
                }

                if (typeof prop === "string") {
                    return defaultFn(prop);
                }

                return undefined;
            }
        });
    },

    mapOf(...pairs) {
        const map = new LinkedHashMap();
        for (const pair of pairs) {
            if (pair == null) continue;
            if (Array.isArray(pair)) {
                map.put(pair[0], pair[1]);
            } else if (pair.first !== undefined && pair.second !== undefined) {
                map.put(pair.first, pair.second);
            } else if (typeof pair.getFirst === "function") {
                map.put(pair.getFirst(), pair.getSecond());
            }
        }
        return map;
    },

    toMap(source) {
        if (source == null) {
            return new LinkedHashMap();
        }

        if (source instanceof Map) {
            return new LinkedHashMap(source);
        }

        const map = new LinkedHashMap();

        for (const entry of source) {
            if (Array.isArray(entry)) {
                map.put(entry[0], entry[1]);
            } else if (entry?.getFirst && entry?.getSecond) {
                map.put(entry.getFirst(), entry.getSecond());
            } else if (entry?.first !== undefined && entry?.second !== undefined) {
                map.put(entry.first, entry.second);
            }
        }

        return map;
    },

    toMutableMap(source) {
        return source instanceof Map
            ? new LinkedHashMap(source)
            : this.toMap(source);
    },

    toList(source) {
        if (source == null) return [];
        if (source instanceof LinkedHashMap) {
            return Array.from(source.entries()).map(([k, v]) => TuplesKt.to(k, v));
        }
        if (source instanceof Map) {
            return Array.from(source.entries()).map(([k, v]) => TuplesKt.to(k, v));
        }
        if (Array.isArray(source)) return source;
        return Array.from(source);
    },
};

globalThis.LinkedHashSet = class LinkedHashSet extends Set {
    constructor(init) {
        super();

        if (init == null || typeof init === "number") {
            return;
        }

        if (Symbol.iterator in Object(init)) {
            for (const v of init) {
                this.add(v);
            }
        }
    }

    add(value) {
        super.add(value);
        return this;
    }

    contains(value) {
        return this.has(value) ? 1 : 0;
    }

    remove(value) {
        const existed = this.has(value);
        this.delete(value);
        return existed ? 1 : 0;
    }

    isEmpty() {
        return this.size === 0 ? 1 : 0;
    }

    size() {
        return super.size;
    }

    iterator() {
        const arr = [...this];
        let i = 0;

        return {
            hasNext: () => (i < arr.length ? 1 : 0),
            next: () => arr[i++],
        };
    }

    addAll(collection) {
        for (const v of collection) {
            this.add(v);
        }
        return this;
    }

    clear() {
        super.clear();
    }

    toArray() {
        return [...this];
    }
};

globalThis.HashSet = globalThis.LinkedHashSet;

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
Array.prototype.toHosterList = function() {
    return Hoster.Companion.toHosterList(this);
};

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