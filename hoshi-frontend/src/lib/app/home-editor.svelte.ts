import { toast } from 'svelte-sonner';
import type { CoreError } from '@/api/client';
import type { HomeLayout, HomeSection, HomeTab, SectionLayout } from '@/api/home/types';
import { homeState } from '@/app/home.svelte.js';
import { i18n } from '@/stores/i18n.svelte.js';

export const MAX_TABS = 8;
export const MAX_SECTIONS = 20;
export const MAX_LABEL = 64;

function uid(prefix: string) {
    return `${prefix}.${Date.now().toString(36)}${Math.random().toString(36).slice(2, 6)}`;
}

function moveTo<T>(arr: T[], from: number, to: number) {
    if (from < 0 || from >= arr.length || to < 0 || to >= arr.length || from === to) return;
    const [item] = arr.splice(from, 1);
    arr.splice(to, 0, item);
}

/** Draft copy of the layout; nothing is persisted until `save`. */
class HomeEditor {
    draft = $state<HomeLayout | null>(null);
    saving = $state(false);
    dirty = $state(false);

    editing = $derived(this.draft !== null);

    start() {
        if (!homeState.layout) return;
        this.draft = $state.snapshot(homeState.layout) as HomeLayout;
        this.dirty = false;
    }

    cancel() {
        this.draft = null;
        this.dirty = false;
    }

    async save() {
        if (!this.draft) return;
        if (!this.dirty) return this.cancel();
        this.saving = true;
        try {
            await homeState.save($state.snapshot(this.draft) as HomeLayout);
            this.cancel();
        } catch (err) {
            toast.error(i18n.t((err as CoreError)?.key || 'error.home.invalid_layout'));
        } finally {
            this.saving = false;
        }
    }

    async reset() {
        this.saving = true;
        try {
            await homeState.reset();
            this.cancel();
        } catch (err) {
            toast.error(i18n.t((err as CoreError)?.key || 'error.home.invalid_layout'));
        } finally {
            this.saving = false;
        }
    }

    tab(id: string | null | undefined): HomeTab | undefined {
        return this.draft?.tabs.find(t => t.id === id);
    }

    private touch() {
        this.dirty = true;
    }

    addTab(name: string, icon: string): string | null {
        if (!this.draft || this.draft.tabs.length >= MAX_TABS) return null;
        const id = uid('tab');
        this.draft.tabs.push({ id, title: { kind: 'text', text: name.slice(0, MAX_LABEL) }, icon, hero: null, sections: [] });
        this.touch();
        return id;
    }

    removeTab(id: string) {
        if (!this.draft || this.draft.tabs.length <= 1) return;
        this.draft.tabs = this.draft.tabs.filter(t => t.id !== id);
        this.touch();
    }

    renameTab(id: string, name: string) {
        const tab = this.tab(id);
        const text = name.trim().slice(0, MAX_LABEL);
        if (!tab || !text) return;
        tab.title = { kind: 'text', text };
        this.touch();
    }

    setTabIcon(id: string, icon: string) {
        const tab = this.tab(id);
        if (!tab) return;
        tab.icon = icon;
        this.touch();
    }

    moveTab(from: number, to: number) {
        if (!this.draft) return;
        moveTo(this.draft.tabs, from, to);
        this.touch();
    }

    addSection(tabId: string, section: Omit<HomeSection, 'id'>): string | null {
        const tab = this.tab(tabId);
        if (!tab || tab.sections.length >= MAX_SECTIONS) return null;
        const id = uid('section');
        tab.sections.push({ ...section, id });
        this.touch();
        return id;
    }

    removeSection(tabId: string, sectionId: string) {
        const tab = this.tab(tabId);
        if (!tab) return;
        tab.sections = tab.sections.filter(s => s.id !== sectionId);
        if (tab.hero === sectionId) tab.hero = null;
        this.touch();
    }

    moveSection(tabId: string, from: number, to: number) {
        const tab = this.tab(tabId);
        if (!tab) return;
        moveTo(tab.sections, from, to);
        this.touch();
    }

    renameSection(tabId: string, sectionId: string, name: string) {
        const section = this.tab(tabId)?.sections.find(s => s.id === sectionId);
        const text = name.trim().slice(0, MAX_LABEL);
        if (!section || !text) return;
        section.title = { kind: 'text', text };
        this.touch();
    }

    setLayout(tabId: string, sectionId: string, layout: SectionLayout) {
        const section = this.tab(tabId)?.sections.find(s => s.id === sectionId);
        if (!section) return;
        section.layout = layout;
        this.touch();
    }

    toggleHidden(tabId: string, sectionId: string) {
        const section = this.tab(tabId)?.sections.find(s => s.id === sectionId);
        if (!section) return;
        section.hidden = !section.hidden;
        this.touch();
    }

    setHero(tabId: string, sectionId: string | null) {
        const tab = this.tab(tabId);
        if (!tab || tab.hero === sectionId) return;
        tab.hero = sectionId;
        this.touch();
    }
}

export const homeEditor = new HomeEditor();
