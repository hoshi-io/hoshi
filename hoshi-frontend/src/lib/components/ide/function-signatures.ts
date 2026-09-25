// src/routes/ide/function-signatures.ts

export type ParamType = 'string' | 'number' | 'boolean' | 'json';

export interface ParamDef {
    name: string;
    type: ParamType;
    optional?: boolean;
}

export interface FunctionDef {
    name: string;
    params: ParamDef[];
}

const COMMON: FunctionDef[] = [
    { name: 'getFilters', params: [] },
    {
        name: 'search',
        params: [
            { name: 'query', type: 'string' },
            { name: 'filters', type: 'json' },
            { name: 'page', type: 'number' },
        ],
    },
    { name: 'getMetadata', params: [{ name: 'id', type: 'string' }] },
];

const ANIME: FunctionDef[] = [
    ...COMMON,
    { name: 'getStreamingSettings', params: [] },
    { name: 'findEpisodes', params: [{ name: 'contentId', type: 'string' }] },
    {
        name: 'findEpisodeServer',
        params: [
            { name: 'episodeId', type: 'string' },
            { name: 'server', type: 'string' },
            { name: 'category', type: 'string', optional: true }, // defaults "sub"
        ],
    },
];

const MANGA: FunctionDef[] = [
    ...COMMON,
    { name: 'findChapters', params: [{ name: 'contentId', type: 'string' }] },
    { name: 'findChapterPages', params: [{ name: 'chapterId', type: 'string' }] },
];

const NOVEL: FunctionDef[] = [
    ...COMMON,
    { name: 'findChapters', params: [{ name: 'contentId', type: 'string' }] },
    { name: 'findChapterPages', params: [{ name: 'chapterId', type: 'string' }] }, // returns HTML string
];

export const FUNCTIONS_BY_TYPE: Record<string, FunctionDef[]> = {
    anime: ANIME,
    manga: MANGA,
    novel: NOVEL,
};

const ORDER = [
    'getFilters', 'search', 'getMetadata',
    'findEpisodes', 'findEpisodeServer', 'getStreamingSettings',
    'findChapters', 'findChapterPages',
];

export function sortedFunctions(defs: FunctionDef[]): FunctionDef[] {
    return [...defs].sort((a, b) => ORDER.indexOf(a.name) - ORDER.indexOf(b.name));
}