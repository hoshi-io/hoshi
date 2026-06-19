"use strict";

globalThis.console = {
    _fmt: (args) => args.map(a =>
        (a !== null && typeof a === "object") ? JSON.stringify(a) : String(a)
    ).join(" "),
    log:   function(...args) { __native_log("[LOG] "   + console._fmt(args)); },
    info:  function(...args) { __native_log("[INFO] "  + console._fmt(args)); },
    warn:  function(...args) { __native_log("[WARN] "  + console._fmt(args)); },
    error: function(...args) { __native_log("[ERROR] " + console._fmt(args)); },
    debug: function(...args) { __native_log("[DEBUG] " + console._fmt(args)); },
};

globalThis.crypto = {
    md5:    (data) => __native_crypto_hash("md5",    data),
    sha1:   (data) => __native_crypto_hash("sha1",   data),
    sha256: (data) => __native_crypto_hash("sha256", data),
    sha512: (data) => __native_crypto_hash("sha512", data),

    hmac: {
        sha1:   (keyHex, data) => {
            const r = __native_crypto_hmac("sha1",   keyHex, data);
            if (r && typeof r === "object" && r.error) throw new Error(r.error);
            return r;
        },
        sha256: (keyHex, data) => {
            const r = __native_crypto_hmac("sha256", keyHex, data);
            if (r && typeof r === "object" && r.error) throw new Error(r.error);
            return r;
        },
        sha512: (keyHex, data) => {
            const r = __native_crypto_hmac("sha512", keyHex, data);
            if (r && typeof r === "object" && r.error) throw new Error(r.error);
            return r;
        },
    },

    aes: {
        encrypt: (keyHex, dataHex, opts = {}) => {
            const r = __native_crypto_aes(
                "encrypt", keyHex, dataHex,
                opts.iv ?? null, opts.mode ?? "cbc"
            );
            if (r && typeof r === "object" && r.error) throw new Error(r.error);
            return r;
        },
        decrypt: (keyHex, dataHex, opts = {}) => {
            const r = __native_crypto_aes(
                "decrypt", keyHex, dataHex,
                opts.iv ?? null, opts.mode ?? "cbc"
            );
            if (r && typeof r === "object" && r.error) throw new Error(r.error);
            return r;
        },

        gcm: {
            encrypt: (keyHex, dataHex, opts = {}) => {
                const r = __native_crypto_aes("encrypt", keyHex, dataHex, opts.iv ?? null, "gcm");
                if (r && typeof r === "object" && r.error) throw new Error(r.error);
                return r;
            },
            decrypt: (keyHex, dataHex, opts = {}) => {
                const r = __native_crypto_aes("decrypt", keyHex, dataHex, opts.iv ?? null, "gcm");
                if (r && typeof r === "object" && r.error) throw new Error(r.error);
                return r;
            },
        },
    },

    hex: {
        fromString: (str) => Array.from(str).map(c =>
            c.charCodeAt(0).toString(16).padStart(2, "0")).join(""),
        toString:   (hex) => hex.match(/.{2}/g)
            .map(b => String.fromCharCode(parseInt(b, 16))).join(""),
        fromBase64: (b64) => crypto.hex.fromString(atob(b64)),
        toBase64:   (hex) => btoa(crypto.hex.toString(hex)),
    },
};
globalThis.fetch = async function(url, options) {
    if (typeof url !== "string" || url.length === 0) {
        throw new TypeError("fetch: url must be a non-empty string");
    }

    options = options || {};

    const method      = (options.method || "GET").toUpperCase();
    let body = "";
    let headersObj = options.headers || {};

    if (options.body instanceof FormData) {
        body = options.body.__serialize();
        headersObj["Content-Type"] = "application/x-www-form-urlencoded";
    } else if (options.body !== undefined && options.body !== null) {
        body = String(options.body);
    }

    const headersJson = JSON.stringify(headersObj);

    const rawJson = __native_fetch(url, method, headersJson, body);
    const raw     = JSON.parse(rawJson);

    if (raw.error) {
        throw new TypeError("fetch failed: " + raw.error);
    }

    if (raw.cookies) {
        for (const [k, v] of Object.entries(raw.cookies)) {
            _cookieStore.set(k, v);
        }
        state?.set?.("cookies", Object.fromEntries(_cookieStore));
    }

    const responseBody = raw.body;
    return {
        ok:     raw.ok,
        status: raw.status,
        url:    url,
        headers: {
            get: (_name) => null,
            has: (_name) => false,
        },
        text: async function() { return responseBody; },
        json: async function() {
            try {
                return JSON.parse(responseBody);
            } catch (e) {
                throw new SyntaxError("fetch response is not valid JSON: " + e.message);
            }
        },
        arrayBuffer: async function() {
            const arr = new Uint8Array(responseBody.length);
            for (let i = 0; i < responseBody.length; i++) {
                arr[i] = responseBody.charCodeAt(i) & 0xFF;
            }
            return arr.buffer;
        },
    };
};

globalThis.fetchSync = function(url, options) {
    options = options || {};
    const method = (options.method || "GET").toUpperCase();
    let body = "";
    let headersObj = options.headers || {};

    if (options.body instanceof FormData) {
        body = options.body.__serialize();
        headersObj["Content-Type"] = "application/x-www-form-urlencoded";
    } else if (options.body !== undefined && options.body !== null) {
        body = String(options.body);
    }

    const headersJson = JSON.stringify(headersObj);
    const rawJson = __native_fetch(url, method, headersJson, body);
    const raw = JSON.parse(rawJson);

    if (raw.error) throw new TypeError("fetchSync failed: " + raw.error);

    if (raw.cookies) {
        for (const [k, v] of Object.entries(raw.cookies)) {
            _cookieStore.set(k, v);
        }
        state?.set?.("cookies", Object.fromEntries(_cookieStore));
    }

    return {
        text: raw.body,
        status: raw.status,
        ok: raw.ok,
        cookies: raw.cookies ?? {},
    };
};

globalThis.URL = class URL {
    constructor(input, base) {
        let full = input;
        if (base) {
            if (/^https?:\/\//i.test(input)) {
                full = input;
            } else {
                const b = String(base).replace(/\/$/, "");
                full = input.startsWith("/")
                    ? b.replace(/(https?:\/\/[^/]+).*/, "$1") + input
                    : b + "/" + input.replace(/^\.\//, "");
            }
        }

        const m = String(full).match(
            /^(https?):\/\/([^/?#]+)([^?#]*)(\?[^#]*)?(#.*)?$/i
        );
        if (!m) throw new TypeError("Invalid URL: " + full);

        this.protocol = m[1].toLowerCase() + ":";
        this.host     = m[2];
        this.hostname = m[2].split(":")[0];
        this.port     = m[2].includes(":") ? m[2].split(":")[1] : "";
        this.pathname = m[3] || "/";
        this.search   = m[4] || "";
        this.hash     = m[5] || "";
        this.origin   = this.protocol + "//" + this.host;
        this.href     = full;
    }

    toString()     { return this.href; }
    toJSON()       { return this.href; }

    get searchParams() {
        return new URLSearchParams(this.search);
    }
};

globalThis.parseHTML = function(html) {
    return function $(selector) {
        const rawJson = __native_html_query(html, selector);
        const raw = JSON.parse(rawJson);
        if (raw.error) throw new Error(raw.error);

        const wrap = (item) => ({
            text:  ()    => item.text,
            html:  ()    => item.html,
            outer: ()    => item.outer,
            attr:  (name) => item.attrs[name] ?? null,
            find:  (sel)  => parseHTML(item.html)(sel),
            _raw: item,
        });

        const results = raw.map(wrap);

        results.attr = function(name) { return this.length > 0 ? this[0].attr(name) : null; };
        results.text = function() { return this.map(r => r.text()).join(""); };
        results.html = function() { return this.length > 0 ? this[0].html() : null; };

        return results;
    };
};

globalThis.URLSearchParams = class URLSearchParams {
    constructor(init) {
        this._map = new Map();
        if (typeof init === "string") {
            init.replace(/^\?/, "").split("&").forEach(pair => {
                if (!pair) return;
                const idx = pair.indexOf("=");
                const k   = decodeURIComponent(idx < 0 ? pair : pair.slice(0, idx));
                const v   = decodeURIComponent(idx < 0 ? ""   : pair.slice(idx + 1));
                this._map.set(k, v);
            });
        } else if (init && typeof init === "object") {
            Object.entries(init).forEach(([k, v]) => this._map.set(k, String(v)));
        }
    }

    get(k)      { return this._map.has(k) ? this._map.get(k) : null; }
    getAll(k)   { return this._map.has(k) ? [this._map.get(k)] : []; }
    has(k)      { return this._map.has(k); }
    set(k, v)   { this._map.set(k, String(v)); }
    append(k,v) { this._map.set(k, String(v)); }
    delete(k)   { this._map.delete(k); }
    keys()      { return this._map.keys(); }
    values()    { return this._map.values(); }
    entries()   { return this._map.entries(); }

    toString() {
        const parts = [];
        this._map.forEach((v, k) =>
            parts.push(encodeURIComponent(k) + "=" + encodeURIComponent(v))
        );
        return parts.join("&");
    }
};

(function() {
    const CHARS = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

    globalThis.btoa = function(str) {
        let out = "", i = 0, bits = 0, acc = 0;
        str = String(str);
        for (; i < str.length; i++) {
            acc  = (acc << 8) | (str.charCodeAt(i) & 0xFF);
            bits += 8;
            while (bits >= 6) {
                bits -= 6;
                out += CHARS[(acc >> bits) & 0x3F];
            }
        }
        if (bits > 0) out += CHARS[(acc << (6 - bits)) & 0x3F];
        while (out.length % 4) out += "=";
        return out;
    };

    globalThis.atob = function(b64) {
        b64 = String(b64).replace(/[^A-Za-z0-9+/]/g, "");
        let out = "", bits = 0, acc = 0;
        for (let i = 0; i < b64.length; i++) {
            const idx = CHARS.indexOf(b64[i]);
            if (idx < 0) continue;
            acc  = (acc << 6) | idx;
            bits += 6;
            if (bits >= 8) {
                bits -= 8;
                out += String.fromCharCode((acc >> bits) & 0xFF);
            }
        }
        return out;
    };
})();

if (typeof globalThis.File === "undefined") {
    globalThis.File = class File {
        constructor(bits, name, opts) {
            this.name = name;
            this.size = 0;
            this.type = (opts && opts.type) || "";
        }
    };
}

if (typeof globalThis.FormData === "undefined") {
    globalThis.FormData = class FormData {
        constructor() { this._entries = []; }
        append(k, v) { this._entries.push([k, v]); }
        get(k)       { const e = this._entries.find(([key]) => key === k); return e ? e[1] : null; }
        has(k)       { return this._entries.some(([key]) => key === k); }
        __serialize() {
            return this._entries
                .map(([k, v]) => encodeURIComponent(k) + "=" + encodeURIComponent(v))
                .join("&");
        }
    };
}

if (typeof setTimeout === "undefined") {
    let __timer_id = 1;
    const __timers = new Map();
    globalThis.setTimeout = (fn, ms, ...args) => {
        const id = __timer_id++;
        __timers.set(id, true);
        __native_sleep(Number(ms) || 0);
        if (__timers.has(id)) {
            try {
                fn(...args);
            } finally {
                __timers.delete(id);
            }
        }
        return id;
    };
    globalThis.clearTimeout = (id) => {
        __timers.delete(id);
    };
    globalThis.setInterval = (fn, ms, ...args) => {
        throw new Error(
            "setInterval is not supported in this sandbox. " +
            "Use waitUntil(conditionFn, { interval, timeout }) for polling."
        );
    };

    globalThis.waitUntil = async function(conditionFn, opts = {}) {
        const interval = opts.interval ?? 500;
        const timeout  = opts.timeout  ?? 10_000;
        const deadline = Date.now() + timeout;

        while (Date.now() < deadline) {
            const result = await conditionFn();
            if (result !== undefined && result !== null && result !== false)
                return result;
            await new Promise(r => setTimeout(r, interval));
        }

        throw new Error(`waitUntil: timed out after ${timeout}ms`);
    };

    globalThis.clearInterval = (id) => {
        __timers.delete(id);
    };
}
globalThis.headless = {
    get available() { return !!__headless_available; },

    fetch: async function(url, options = {}) {
        if (!__headless_available) {
            throw new Error("headless.fetch: not available on this platform");
        }

        const opts = {
            method:     options.method     || "GET",
            headers:    options.headers    || {},
            body:       options.body       ?? null,
            wait_for:   _normalizeWaitFor(options.waitFor ?? options.wait_for),
            javascript: options.javascript ?? null,
            block:      _normalizeBlock(options.block  || []),
            capture:    options.capture    || [],
            timeout_ms: options.timeoutMs  ?? options.timeout_ms ?? 15000,
        };

        const rawJson = __native_headless_sync(url, JSON.stringify(opts));
        const raw = JSON.parse(rawJson);

        if (raw.error) {
            throw new Error("headless.fetch failed: " + raw.error);
        }

        return raw;
    },
};

function _normalizeWaitFor(value) {
    if (!value || value === "dom_ready" || value === "domready")  return "dom_ready";
    if (value === "network_idle" || value === "networkidle")       return "network_idle";
    if (typeof value === "object" && value.selector)               return { selector: value.selector };
    if (typeof value === "string")                                 return { selector: value };
    return "dom_ready";
}

function _normalizeBlock(arr) {
    return arr.map(item => {
        if (item === "images")     return "images";
        if (item === "fonts")      return "fonts";
        if (item === "media")      return "media";
        if (item === "stylesheet") return "stylesheet";
        return { pattern: item };
    });
}

globalThis.state = {
    get(key, defaultValue = undefined) {
        const raw = __native_state_get(String(key));
        const val = JSON.parse(raw);
        return (val === null && defaultValue !== undefined) ? defaultValue : val;
    },

    set(key, value) {
        __native_state_set(String(key), JSON.stringify(value ?? null));
    },

    delete(key) {
        __native_state_delete(String(key));
    },

    has(key) {
        return __native_state_has(String(key));
    },

    keys() {
        return JSON.parse(__native_state_keys());
    },

    update(key, fn, defaultValue = {}) {
        const current = this.get(key, defaultValue);
        const next    = fn(current);
        this.set(key, next);
        return next;
    },
};