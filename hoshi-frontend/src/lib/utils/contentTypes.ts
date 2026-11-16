import type { ContentType } from "@/api/content/types";
import { appConfig } from "@/stores/config.svelte";

export const ALL_CONTENT_TYPES: ContentType[] = ["anime", "manga", "novel"];

/** Types the profile wants to discover (General settings). Empty or missing means all. */
export function enabledContentTypes(): ContentType[] {
    const saved = appConfig.data?.general?.contentTypes;
    return saved?.length ? ALL_CONTENT_TYPES.filter(t => saved.includes(t)) : ALL_CONTENT_TYPES;
}

export function isContentTypeEnabled(type: ContentType | null | undefined): boolean {
    return !type || enabledContentTypes().includes(type);
}
