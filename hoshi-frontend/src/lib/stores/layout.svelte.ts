import type { Snippet } from 'svelte';

export type ListEditorPayload = {
    cid: string;
    title: string;
    contentType: string;
    coverImage?: string;
};

export type MpvState = {
    cid: string;
    epNumber: number;
    extId: string;
    server?: string;
    isDub: boolean;
    animeTitle: string;
    epTitle: string;
    totalEpisodes: number;
    isNsfw: boolean;
    coverImage: string;
};

export const layoutState = $state({
    title: "Hoshi",
    showBack: false,
    backUrl: null as string | null,
    headerAction: undefined as Snippet | undefined,
    isMobile: false,
    listEditor: null as ListEditorPayload | null,
    listEditorOpen: false,
    mpv: null as MpvState | null,
});

export function openListEditor(payload: ListEditorPayload) {
    layoutState.listEditor = payload;
    layoutState.listEditorOpen = true;
}

export function closeListEditor() {
    layoutState.listEditorOpen = false;
    setTimeout(() => { layoutState.listEditor = null; }, 300);
}