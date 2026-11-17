class Base {
    getFilters()  { return {}; }

    _assertArray(value, label) {
        if (!Array.isArray(value)) {
            throw new Error(`[${this.constructor.name}] ${label} must be an array, got ${typeof value}`);
        }
    }

    _assertString(value, label) {
        if (typeof value !== "string" || value.trim() === "") {
            throw new Error(`[${this.constructor.name}] ${label} must be a non-empty string`);
        }
    }

    _assertNullableString(value, label) {
        if (value !== null && value !== undefined && typeof value !== "string") {
            throw new Error(`[${this.constructor.name}] ${label} must be a string or null`);
        }
    }

    _assertNullableNumber(value, label) {
        if (value !== null && value !== undefined && typeof value !== "number") {
            throw new Error(`[${this.constructor.name}] ${label} must be a number or null`);
        }
    }

    _validateSearchResult(item, index) {
        const ctx = `search result[${index}]`;
        if (typeof item !== "object" || item === null) {
            throw new Error(`[${this.constructor.name}] ${ctx} must be an object`);
        }
        this._assertString(item.id,    `${ctx}.id`);
        this._assertString(item.title, `${ctx}.title`);
        this._assertNullableString(item.image, `${ctx}.image`);
        if (item.url !== undefined)  this._assertNullableString(item.url,  `${ctx}.url`);
        if (item.nsfw !== undefined && typeof item.nsfw !== "boolean") {
            throw new Error(`[${this.constructor.name}] ${ctx}.nsfw must be a boolean`);
        }
    }

    _validateSearchResults(results) {
        this._assertArray(results, "search() return value");
        results.forEach((item, i) => this._validateSearchResult(item, i));
        return results;
    }

    _validateMetadata(meta) {
        if (typeof meta !== "object" || meta === null) {
            throw new Error(`[${this.constructor.name}] getMetadata() must return an object`);
        }
        this._assertString(meta.title,           "metadata.title");
        this._assertNullableString(meta.synopsis, "metadata.synopsis");
        this._assertNullableNumber(meta.eps_or_chapters, "metadata.eps_or_chapters");
        this._assertNullableNumber(meta.rating,   "metadata.rating");
        this._assertNullableNumber(meta.year,     "metadata.year");
        this._assertNullableString(meta.image,    "metadata.image");

        if (meta.genres !== undefined && meta.genres !== null) {
            this._assertArray(meta.genres, "metadata.genres");
            meta.genres.forEach((g, i) => this._assertString(g, `metadata.genres[${i}]`));
        }

        if (meta.nsfw !== undefined && typeof meta.nsfw !== "boolean") {
            throw new Error(`[${this.constructor.name}] metadata.nsfw must be a boolean`);
        }

        if (meta.url !== undefined && meta.url !== null) {
            if (typeof meta.url !== "string" || !/^https?:\/\//i.test(meta.url))
                throw new Error(`[${this.constructor.name}] metadata.url must be an http(s) URL`);
        }

        const trustedIds = ["anilist_id", "mal_id"];
        for (const field of trustedIds) {
            const val = meta[field];
            if (val === undefined || val === null) continue;

            if (typeof val === "number") {
                if (!Number.isInteger(val) || val <= 0) {
                    throw new Error(
                        `[${this.constructor.name}] metadata.${field} must be a positive integer, got ${val}`
                    );
                }
            } else if (typeof val === "string") {
                const parsed = parseInt(val, 10);
                if (isNaN(parsed) || parsed <= 0 || String(parsed) !== val.trim()) {
                    throw new Error(
                        `[${this.constructor.name}] metadata.${field} must be a positive integer string, got "${val}"`
                    );
                }
            } else {
                throw new Error(
                    `[${this.constructor.name}] metadata.${field} must be a number or numeric string, got ${typeof val}`
                );
            }
        }

        if (meta.external_ids !== undefined && meta.external_ids !== null) {
            if (typeof meta.external_ids !== "object" || Array.isArray(meta.external_ids)) {
                throw new Error(
                    `[${this.constructor.name}] metadata.external_ids must be a plain object or null`
                );
            }
        }

        return meta;
    }
}