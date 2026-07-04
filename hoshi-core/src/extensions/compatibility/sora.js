// sora_compat.js
// Wraps a raw Sora module (already eval'd into global scope) into
// something matching the sandbox's Anime class contract.

function __sora_buildAnimeClass() {

    function hrefToQueryGuess(href) {
        // best-effort: last path segment, dashes -> spaces
        const slug = href.split('/').filter(Boolean).pop() || '';
        return slug.replace(/-/g, ' ');
    }

    class SoraCompatAnime extends Anime {

        async getFilters() {
            return {};
        }

        async search(query, filters, page) {
            const raw = await searchResults(query);
            const items = JSON.parse(raw);

            const mapped = items
                .filter(i => i.href)
                .filter(i => !/\/tag\//.test(i.href))
                .map(i => ({
                    id: i.href,
                    title: i.title,
                    url: i.href,
                    image: i.image
                }));

            for (const item of mapped) {
                state.set(`sora:searchmeta:${item.id}`, { title: item.title, image: item.image, query });
            }

            console.log(mapped)

            return mapped;
        }

        async getMetadata(id) {
            const detailsRaw = await extractDetails(id);
            const details = JSON.parse(detailsRaw)[0];

            let cached = state.get(`sora:searchmeta:${id}`);
            if (!cached) {
                // cold cache: try to re-derive a query and re-search once
                const guess = hrefToQueryGuess(id);
                const raw = await searchResults(guess);
                const items = JSON.parse(raw);
                const match = items.find(i => i.href === id);
                cached = match ? { title: match.title, image: match.image } : { title: null, image: null };
            }

            const epCount = details?.animeDetails?.episodes?.length ?? null;

            return {
                title: cached.title,
                synopsis: details.description ?? null,
                image: cached.image,

                eps_or_chapters: epCount,
                rating: null,       // not exposed upstream
                year: details.animeDetails?.year ?? null,

                genres: [],         // not exposed upstream

                anilist_id: null,   // not exposed upstream
                mal_id: null,       // not exposed upstream

                external_ids: {
                    imdb: null       // not exposed upstream
                }
            };
        }

        getStreamingSettings() {
            return {
                episodeServers: [], // always empty/placeholder; real list comes from discovery
                supportsDub: false  // universally false for Sora — no separate toggle
            };
        }

        async listEpisodeServers(episodeId) {
            let entry = state.get(`sora:episode:${episodeId}`);

            if (!entry) {
                // cold cache — re-derive contentId and repopulate via findEpisodes
                const contentId = episodeId.split('###ep')[0];
                await this.findEpisodes(contentId); // repopulates state for every episode of this content
                entry = state.get(`sora:episode:${episodeId}`);
            }

            if (!entry) {
                throw new Error(`Could not resolve episode: ${episodeId}`);
            }

            const href = entry.subHref ?? entry.dubHref ?? entry.href;
            const cacheKey = `sora:streams:${href}`;
            let streams = state.get(cacheKey);

            if (!streams) {
                const raw = await extractStreamUrl(href);
                streams = JSON.parse(raw).streams ?? [];
                state.set(cacheKey, streams);
            }

            return streams.map(s => s.title);
        }

        async findEpisodes(contentId) {
            // Sora buckets by a fixed module-level langType at extraction time.
            // Pull both so dub/sub aren't lost.
            const subRaw = await extractEpisodes(contentId, "sub").catch(() => null);
            const dubRaw = await extractEpisodes(contentId, "dub").catch(() => null);
            // NOTE: the pasted module hardcodes `const langType = 'dub'` and
            // ignores any argument — if you can't edit the module itself,
            // you only get whichever language it's hardcoded to, once.
            // Assuming langType is made a parameter (recommended patch below),
            // merge by episode number:

            const subEps = subRaw ? JSON.parse(subRaw) : [];
            const dubEps = dubRaw ? JSON.parse(dubRaw) : [];

            const byNumber = new Map();

            for (const e of subEps) {
                if (e.number == null) continue;
                byNumber.set(e.number, { number: e.number, subHref: e.href });
            }
            for (const e of dubEps) {
                if (e.number == null) continue;
                const existing = byNumber.get(e.number) || { number: e.number };
                existing.dubHref = e.href;
                byNumber.set(e.number, existing);
            }

            const merged = [...byNumber.values()].sort((a, b) => a.number - b.number);

            // cache both hrefs under a stable synthetic episodeId
            const result = merged.map(e => {
                const episodeId = `${contentId}###ep${e.number}`;
                state.set(`sora:episode:${episodeId}`, {
                    subHref: e.subHref ?? null,
                    dubHref: e.dubHref ?? null
                });
                return {
                    id: episodeId,
                    number: e.number,
                    title: `Episode ${e.number}`,
                    url: null,
                    image: null
                };
            });

            return result;
        }

        async findEpisodeServer(episodeId, server, category = "sub") {
            let entry = state.get(`sora:episode:${episodeId}`);

            if (!entry) {
                const contentId = episodeId.split('###ep')[0];
                await this.findEpisodes(contentId);
                entry = state.get(`sora:episode:${episodeId}`);
            }

            if (!entry) throw new Error(`Could not resolve episode: ${episodeId}`);

            const href = entry.subHref ?? entry.dubHref ?? entry.href;
            const cacheKey = `sora:streams:${href}`;

            let streams = state.get(cacheKey);
            if (!streams) {
                const raw = await extractStreamUrl(href);
                streams = JSON.parse(raw).streams ?? [];
                state.set(cacheKey, streams);
            }

            const match = streams.find(s => s.title === server);
            if (!match) throw new Error(`No stream found for server: ${server}`);

            return {
                headers: match.headers ?? {},
                source: { url: match.streamUrl, subtitles: normalizeSubtitles(match.subtitles), chapters: [] }
            };
        }

    }

    function normalizeSubtitles(subtitleUrl) {
        if (!subtitleUrl) return [];

        const urls = Array.isArray(subtitleUrl)
            ? subtitleUrl
            : String(subtitleUrl).split(',').map(s => s.trim()).filter(Boolean);

        return urls.map((url, i) => ({
            id: `sub-${i}`,
            url,
            language: guessLangFromUrl(url) ?? `Track ${i + 1}`,
            is_default: i === 0
        }));
    }

    function guessLangFromUrl(url) {
        // best-effort: many of these encode lang in filename, e.g. subs_en.vtt, .en.srt
        const m = url.match(/[._-](en|es|fr|de|ar|pt|ja|ko)[._-]/i);
        return m ? m[1].toUpperCase() : null;
    }

    return SoraCompatAnime;
}