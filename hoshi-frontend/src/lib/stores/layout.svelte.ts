import type { Snippet } from 'svelte';

export type ListEditorPayload = {
    cid: string;
    title: string;
    contentType: string;
    coverImage?: string;
    headers?: string;
};


export const layoutState = $state({
    title: "Hoshi",
    showBack: false,
    backUrl: null as string | null,
    headerAction: undefined as Snippet | undefined,
    isMobile: false,
    listEditor: null as ListEditorPayload | null,
    listEditorOpen: false,
    isTV: false,
});

export function openListEditor(payload: ListEditorPayload) {
    layoutState.listEditor = payload;
    layoutState.listEditorOpen = true;
}

export function closeListEditor() {
    layoutState.listEditorOpen = false;
    setTimeout(() => { layoutState.listEditor = null; }, 300);
}