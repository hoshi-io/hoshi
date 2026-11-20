import { i18n } from "@/stores/i18n.svelte.js";

export type Shortcut = { label: string; keys: string[]; hint?: string };

/** Keys handled by the shared reader layout (Reader.svelte), for both readers. */
export function commonReaderShortcuts(): Shortcut[] {
    return [
        { label: i18n.t("reader.shortcut_prev_chapter"), keys: ["["] },
        { label: i18n.t("reader.shortcut_next_chapter"), keys: ["]"] },
        { label: i18n.t("reader.chapters"), keys: ["C"] },
        { label: i18n.t("reader.settings"), keys: ["S"] },
        { label: i18n.t("reader.shortcut_fullscreen"), keys: ["F"], hint: i18n.t("reader.desktop_only") },
    ];
}
