/**
 * An extension may return several versions of the same chapter number
 * (one per language / scanlator). These helpers keep the rest of the app
 * working with one chapter per number.
 */

export type ChapterLike = {
    id?: string;
    number?: string | number;
    unitNumber?: string | number;
    title?: string;
    language?: string | null;
    scanlators?: string[] | null;
    [key: string]: unknown;
};

export const ALL = "all";

export const chapterNum = (c: ChapterLike) => Number(c.number ?? c.unitNumber);

export const scanlatorLabel = (c: ChapterLike) =>
    c.scanlators?.length ? c.scanlators.join(" & ") : null;

/** Distinct values present in `chapters`, sorted. */
export function chapterFacets(chapters: ChapterLike[]) {
    const languages = new Set<string>();
    const scanlators = new Set<string>();
    for (const c of chapters) {
        if (c.language) languages.add(c.language);
        const label = scanlatorLabel(c);
        if (label) scanlators.add(label);
    }
    const sort = (s: Set<string>) => [...s].sort((a, b) => a.localeCompare(b));
    return { languages: sort(languages), scanlators: sort(scanlators) };
}

export function filterChapters(
    chapters: ChapterLike[],
    { language = ALL, scanlator = ALL }: { language?: string; scanlator?: string },
) {
    return chapters.filter(c =>
        (language === ALL || c.language === language) &&
        (scanlator === ALL || scanlatorLabel(c) === scanlator),
    );
}

/** How well `c` continues reading `ref`: same language first, then a shared scanlator. */
function affinity(c: ChapterLike, ref: ChapterLike | undefined) {
    if (!ref) return 0;
    let score = 0;
    if (ref.language && c.language === ref.language) score += 2;
    if (ref.scanlators?.some(s => c.scanlators?.includes(s))) score += 1;
    return score;
}

/**
 * One chapter per number, sorted by number. `current` (if given) is kept for its
 * number and other numbers pick the version closest to it; ties keep list order.
 */
export function collapseByNumber(chapters: ChapterLike[], current?: ChapterLike): ChapterLike[] {
    const best = new Map<number, { ch: ChapterLike; score: number }>();
    for (const c of chapters) {
        const n = chapterNum(c);
        if (Number.isNaN(n)) continue;
        const score = c === current ? Infinity : affinity(c, current);
        const prev = best.get(n);
        if (!prev || score > prev.score) best.set(n, { ch: c, score });
    }
    return [...best.entries()].sort((a, b) => a[0] - b[0]).map(([, v]) => v.ch);
}

/** Numbers with more than one version need `?v=` so the reader opens the right one. */
export function hasVariants(chapters: ChapterLike[], n: number) {
    let count = 0;
    for (const c of chapters) {
        if (chapterNum(c) === n && ++count > 1) return true;
    }
    return false;
}

export function chapterHref(base: string, cid: string, ext: string, ch: ChapterLike, all: ChapterLike[]) {
    const n = chapterNum(ch);
    const url = `${base}/${cid}/${ext}/${n}`;
    return ch.id && hasVariants(all, n) ? `${url}?v=${encodeURIComponent(ch.id)}` : url;
}
