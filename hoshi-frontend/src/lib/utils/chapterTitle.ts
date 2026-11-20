import { i18n } from "@/stores/i18n.svelte.js";
import { chapterNum, type ChapterLike } from "@/utils/chapterVariants";

/** The extension's title, unless it only repeats "Chapter N". */
export function cleanChapterTitle(chap: ChapterLike | undefined, num: number): string | null {
    const title = chap?.title?.trim();
    if (!title) return null;
    const numText = i18n.t("reader.chapter_number", { count: num }).trim();
    if (title.toLowerCase() === numText.toLowerCase()) return null;
    if (/^chapter\s+\d+(\.\d+)?$/i.test(title) && Number(title.replace(/[^\d.]/g, "")) === num) return null;
    return title;
}

/** The title already says which chapter it is ("Chapter 1 [Official]", "Ch. 01"). */
export function mentionsNumber(title: string, num: number): boolean {
    const n = String(num).replace(".", "\\.");
    return new RegExp(`(^|[^\\d.])0*${n}(?![\\d]|\\.\\d)`).test(title);
}

/** Main line and optional subtitle for a chapter, without saying the number twice. */
export function chapterLabel(chap: ChapterLike | undefined, num = chap ? chapterNum(chap) : 0) {
    const numText = i18n.t("reader.chapter_number", { count: num });
    const title = cleanChapterTitle(chap, num);
    if (!title) return { title: numText, subtitle: null };
    if (mentionsNumber(title, num)) return { title, subtitle: null };
    return { title, subtitle: numText };
}
