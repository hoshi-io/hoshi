function __stremio_buildTorrentClass() {
    const addon = globalThis.__stremioAddon || {};

    function baseUrl() {
        let url = String(__settings.addonUrl || addon.root || "").trim()
            .replace(/\/manifest\.json$/i, "")
            .replace(/\/+$/, "");
        if (url.startsWith("stremio://")) url = "https://" + url.slice("stremio://".length);

        // SDK addons read their config from a URI-encoded JSON path segment.
        const cfg = {};
        for (const key of addon.configKeys || []) {
            const value = __settings[`cfg_${key}`];
            if ((addon.checkboxKeys || []).includes(key)) {
                if (value) cfg[key] = "checked";
            } else if (value !== undefined && value !== null && value !== "") {
                cfg[key] = value;
            }
        }
        if (Object.keys(cfg).length) url += "/" + encodeURIComponent(JSON.stringify(cfg));
        return url;
    }

    function parseSizeBytes(size) {
        const m = /([\d.,]+)\s*([KMGT])i?B/i.exec(size || "");
        if (!m) return null;
        const n = parseFloat(m[1].replace(",", "."));
        const pow = { K: 1, M: 2, G: 3, T: 4 }[m[2].toUpperCase()];
        return Math.round(n * Math.pow(1024, pow));
    }

    function toResult(s) {
        // Torrentio-style text: release name, optional file name, then "👤 52 💾 1.4 GB ⚙️ …"
        const text = String(s.description ?? s.title ?? "");
        const lines = text.split("\n").map(l => l.trim()).filter(Boolean);
        const title = lines[0] || s.behaviorHints?.filename || String(s.name || addon.name || "Stream").replace(/\n/g, " ");

        const seeders = /👤\s*(\d+)/.exec(text);
        const size = /💾\s*([\d.,]+\s*[KMGT]i?B)/i.exec(text);
        const resolution = /\b(2160p|4k|1440p|1080p|720p|576p|480p)\b/i.exec(`${s.name || ""} ${text}`);

        const base = {
            title,
            seeders: seeders ? Number(seeders[1]) : null,
            size: size ? size[1] : null,
            sizeBytes: s.behaviorHints?.videoSize ?? parseSizeBytes(size?.[1]),
            resolution: resolution ? resolution[1].toLowerCase().replace("4k", "2160p") : null,
        };

        if (typeof s.infoHash === "string" && /^[0-9a-f]{40}$/i.test(s.infoHash)) {
            const hash = s.infoHash.toLowerCase();
            const trackers = (s.sources || [])
                .filter(src => typeof src === "string" && src.startsWith("tracker:"))
                .map(src => "&tr=" + encodeURIComponent(src.slice("tracker:".length)))
                .join("");
            const magnet = `magnet:?xt=urn:btih:${hash}&dn=${encodeURIComponent(title)}${trackers}`;
            state.set(`stremio:magnet:${hash}`, magnet);
            return { ...base, id: hash, magnet, infoHash: hash };
        }

        if (typeof s.url === "string" && /^https?:\/\//i.test(s.url)) {
            return { ...base, id: s.url, url: s.url };
        }
        return null;
    }

    return class StremioCompatTorrent extends Torrent {
        getFilters() {
            return {};
        }

        async search(query, filters, page) {
            if (page && page > 1) return [];

            // Older installs have no prefix list: they were checked for kitsu at install.
            const accepts = p => !addon.idPrefixes || addon.idPrefixes.some(x => x.replace(/:$/, "") === p);
            const raw = p => filters?.ids?.[p] ? String(filters.ids[p]).replace(/^.*:/, "") : null;
            const prefix = ["kitsu", "mal"].find(p => accepts(p) && raw(p));
            if (!prefix) return [];

            const [type, id] = filters?.movie
                ? ["movie", `${prefix}:${raw(prefix)}`]
                : ["series", `${prefix}:${raw(prefix)}:${filters?.episode ?? 1}`];

            const url = `${baseUrl()}/stream/${type}/${id}.json`;
            let res;
            try {
                res = await fetch(url);
            } catch (e) {
                throw new Error(`${addon.name || "Addon"} is unavailable: ${e.message}`);
            }
            if (res.status === 401 || res.status === 403)
                throw new Error(`${addon.name || "Addon"} refused the request: configure it on its website and set the URL it gives you in this extension's settings`);
            if (res.status === 429) throw new Error(`${addon.name || "Addon"} is rate limiting requests, try again later`);
            if (!res.ok) throw new Error(`${addon.name || "Addon"} is unavailable (HTTP ${res.status})`);

            const data = await res.json();
            // The same torrent can come back once per matching file; keep the first.
            const seen = new Set();
            const results = (Array.isArray(data?.streams) ? data.streams : [])
                .map(toResult)
                .filter(r => r && !seen.has(r.id) && seen.add(r.id));
            return this._validateTorrentResults(results);
        }

        async getMagnet(id) {
            const hash = String(id).toLowerCase();
            const magnet = state.get(`stremio:magnet:${hash}`) || `magnet:?xt=urn:btih:${hash}`;
            return this._validateMagnet(magnet);
        }
    };
}
