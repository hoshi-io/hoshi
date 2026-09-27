class Torrent extends Base {
    getFilters() { return {}; }

    _validateTorrentResult(item, index) {
        const ctx = `torrent result[${index}]`;
        if (typeof item !== "object" || item === null)
            throw new Error(`[${this.constructor.name}] ${ctx} must be an object`);

        this._assertString(item.id,    `${ctx}.id`);
        this._assertString(item.title, `${ctx}.title`);

        if (item.magnet !== undefined && item.magnet !== null)
            this._assertString(item.magnet, `${ctx}.magnet`);

        if (item.size !== undefined && item.size !== null)
            this._assertNullableString(item.size, `${ctx}.size`);

        if (item.seeders !== undefined && item.seeders !== null)
            this._assertNullableNumber(item.seeders, `${ctx}.seeders`);

        if (item.leechers !== undefined && item.leechers !== null)
            this._assertNullableNumber(item.leechers, `${ctx}.leechers`);

        if (item.isBatch !== undefined && typeof item.isBatch !== "boolean")
            throw new Error(`[${this.constructor.name}] ${ctx}.isBatch must be a boolean`);
    }

    _validateTorrentResults(results) {
        this._assertArray(results, "search() return value");
        results.forEach((item, i) => this._validateTorrentResult(item, i));
        return results;
    }

    _validateMagnet(value) {
        if (typeof value !== "string" || !value.startsWith("magnet:"))
            throw new Error(`[${this.constructor.name}] getMagnet() must return a magnet: URI string`);
        return value;
    }
}