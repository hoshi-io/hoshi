import type { ContentType } from '@/api/content/types';
import type { BuiltinSection, DiscoverQuery, HomeSection, Label, SectionLayout } from '@/api/home/types';
import type { Extension } from '@/api/extensions/types';

export type PresetGroup = 'forYou' | 'calendar' | 'anilist' | 'mangabaka' | 'extensions';
type Tracker = 'anilist' | 'mangabaka';

export interface Preset {
    key: string;
    group: PresetGroup;
    /** Rendered as a compact chip (genres) instead of a full row. */
    compact?: boolean;
    /** Can be added more than once (each copy gets its own filters). */
    repeatable?: boolean;
    section: Omit<HomeSection, 'id'>;
}

export const TRACKER_NAMES: Record<string, string> = { anilist: 'AniList', mangabaka: 'MangaBaka' };

const GENRES = ['Action', 'Adventure', 'Comedy', 'Drama', 'Fantasy', 'Horror', 'Mystery', 'Romance', 'Sci-Fi', 'Slice of Life', 'Sports', 'Thriller'];

const key = (k: string): Label => ({ kind: 'key', key: k });
const text = (t: string): Label => ({ kind: 'text', text: t });

function builtin(ct: ContentType, title: Label, b: BuiltinSection, layout: SectionLayout = 'carousel'): Omit<HomeSection, 'id'> {
    return { title, layout, contentType: ct, source: { kind: 'builtin', builtin: b } };
}

function search(
    tracker: Tracker, ct: ContentType, title: Label, sort: string,
    opts: { status?: string; genre?: string } = {}, layout: SectionLayout = 'carousel',
): Omit<HomeSection, 'id'> {
    const query: DiscoverQuery = {
        kind: 'tracker', tracker, contentType: ct,
        query: null, sort, genre: opts.genre ?? null, format: null, status: opts.status ?? null, tags: [],
    };
    return { title, layout, contentType: ct, source: { kind: 'search', query } };
}

function anilistPresets(ct: ContentType): Preset[] {
    const g = 'anilist' as const;
    const isAnime = ct === 'anime';
    const list: Preset[] = [
        { key: 'al.trending', group: g, section: search(g, ct, key('home.trending'), 'TRENDING_DESC') },
        { key: 'al.popular', group: g, section: search(g, ct, key('home.popular'), 'POPULARITY_DESC') },
        { key: 'al.top', group: g, section: search(g, ct, key('home.critically_acclaimed'), 'SCORE_DESC', {}, 'ranked') },
    ];
    if (ct !== 'novel') {
        list.push({ key: 'al.seasonal', group: g, section: search(g, ct, key(isAnime ? 'home.simulcast' : 'home.latest'), 'POPULARITY_DESC', { status: 'ongoing' }) });
    }
    if (isAnime) {
        list.push({ key: 'al.upcoming', group: g, section: search(g, ct, key('home.upcoming'), 'POPULARITY_DESC', { status: 'upcoming' }) });
    }
    list.push({ key: 'al.finished', group: g, section: search(g, ct, key('home.recently_finished'), 'END_DATE_DESC', { status: 'completed' }) });
    for (const genre of GENRES) {
        list.push({ key: `al.genre.${genre}`, group: g, compact: true, section: search(g, ct, text(genre), 'SCORE_DESC', { genre }) });
    }
    return list;
}

function mangabakaPresets(ct: ContentType): Preset[] {
    const g = 'mangabaka' as const;
    const list: Preset[] = [
        { key: 'mb.trending', group: g, section: search(g, ct, key('home.trending'), 'trending') },
        { key: 'mb.popular', group: g, section: search(g, ct, key('home.popular'), 'popularity') },
        { key: 'mb.top', group: g, section: search(g, ct, key('home.critically_acclaimed'), 'score', {}, 'ranked') },
        { key: 'mb.latest', group: g, section: search(g, ct, key('home.presets.latest_updates'), 'latest') },
        { key: 'mb.ongoing', group: g, section: search(g, ct, key('home.presets.ongoing'), 'popularity', { status: 'ongoing' }) },
        { key: 'mb.upcoming', group: g, section: search(g, ct, key('home.upcoming'), 'popularity', { status: 'upcoming' }) },
        { key: 'mb.completed', group: g, section: search(g, ct, key('home.presets.completed_series'), 'popularity', { status: 'completed' }) },
    ];
    for (const genre of GENRES) {
        list.push({ key: `mb.genre.${genre}`, group: g, compact: true, section: search(g, ct, text(genre), 'score', { genre }) });
    }
    return list;
}

export function buildPresets(ct: ContentType, installed: Extension[]): Preset[] {
    const isAnime = ct === 'anime';

    const presets: Preset[] = [
        { key: 'continue', group: 'forYou', section: builtin(ct, key(isAnime ? 'home.continue.continue_watching' : 'home.continue.continue_reading'), { type: 'continueWatching' }) },
        { key: 'sequels', group: 'forYou', section: builtin(ct, key('home.presets.sequels'), { type: 'unstartedSequels' }) },
        { key: 'current', group: 'forYou', section: builtin(ct, key(isAnime ? 'home.presets.watching' : 'home.presets.reading'), { type: 'listStatus', status: 'CURRENT' }) },
        { key: 'planning', group: 'forYou', section: builtin(ct, key('home.presets.planning'), { type: 'listStatus', status: 'PLANNING' }) },
        { key: 'paused', group: 'forYou', section: builtin(ct, key('home.presets.paused'), { type: 'listStatus', status: 'PAUSED' }) },
        { key: 'random', group: 'forYou', section: builtin(ct, key('home.presets.random_planning'), { type: 'randomPlanning' }) },
        { key: 'completed', group: 'forYou', section: builtin(ct, key('home.presets.recently_completed'), { type: 'recentlyCompleted' }, 'strip') },
    ];

    if (isAnime) {
        presets.push(
            { key: 'recently_aired', group: 'calendar', section: builtin(ct, key('home.recently_aired.title'), { type: 'recentlyAired' }) },
            { key: 'airing_today', group: 'calendar', section: builtin(ct, key('home.presets.airing_today'), { type: 'airingToday' }) },
            { key: 'new_episodes', group: 'calendar', section: builtin(ct, key('home.presets.new_episodes'), { type: 'newEpisodes' }) },
        );
    }

    presets.push(...anilistPresets(ct));
    if (!isAnime) presets.push(...mangabakaPresets(ct));

    presets.push({ key: 'extensions', group: 'extensions', section: builtin(ct, key('home.presets.extensions'), { type: 'extensions' }) });
    for (const ext of installed.filter(e => e.ext_type === ct)) {
        presets.push({
            key: `ext.${ext.id}`,
            group: 'extensions',
            repeatable: true,
            section: {
                title: text(ext.name),
                layout: 'carousel',
                contentType: ct,
                source: { kind: 'search', query: { kind: 'extension', extension: ext.id, query: null, filters: null } },
            },
        });
    }

    return presets;
}

/** Same content as a preset, ignoring title and layout: used to mark presets already on the tab. */
export function sameSource(a: Omit<HomeSection, 'id'>, b: Omit<HomeSection, 'id'>): boolean {
    return a.contentType === b.contentType && JSON.stringify(a.source) === JSON.stringify(b.source);
}
