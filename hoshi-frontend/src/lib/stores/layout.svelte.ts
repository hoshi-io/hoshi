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

    // layout / device
    isMobile: false,
    isTouchDevice: false,   // set by the initApp callback (root layout)
    isAndroidTv: false,     // the physical device is an Android TV (plugin check)
    isTV: false,            // the TV layout is showing (derived from the /tv route in root)

    // modals
    switchProfileOpen: false,
    listEditor: null as ListEditorPayload | null,
    listEditorOpen: false,
});

export function openListEditor(payload: ListEditorPayload) {
    layoutState.listEditor = payload;
    layoutState.listEditorOpen = true;
}

export function closeListEditor() {
    layoutState.listEditorOpen = false;
    setTimeout(() => { layoutState.listEditor = null; }, 300);
}