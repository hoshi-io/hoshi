import type { ContentType } from '@/api/content/types';
import type { ContinueItem } from '@/api/progress/types';
import type { CoreError } from '@/api/client';
import type { DiscoverQuery, HomeLayout, HomeSection, HomeTab, Label } from '@/api/home/types';
import { homeApi } from '@/api/home/home';
import { progressApi } from '@/api/progress/progress';
import { extensions } from '@/stores/extensions.svelte.js';
import { i18n } from '@/stores/i18n.svelte.js';
import { type NormalizedCard, normalizeDiscoverItem } from '@/utils/normalize';

export function labelText(label: Label): string {
    return label.kind === 'key' ? i18n.t(label.key) : label.text;
}

function queryContentType(query: DiscoverQuery): ContentType {
    if (query.kind === 'tracker') return query.contentType;
    const ext = extensions.installed.find(e => e.id === query.extension);
    const type = ext?.ext_type;
    return type === 'manga' || type === 'novel' ? type : 'anime';
}

/** Paged results of one saved search; shared by every section using the same query. */
export class SectionFeed {
    items = $state<NormalizedCard[]>([]);
    loading = $state(false);
    loaded = $state(false);
    error = $state<CoreError | null>(null);
    hasMore = $state(true);
    private page = 0;

    constructor(readonly query: DiscoverQuery) {}

    load() {
        if (this.loaded || this.loading) return Promise.resolve();
        return this.fetchPage(1);
    }

    more() {
        if (!this.hasMore || this.loading || !this.loaded) return Promise.resolve();
        return this.fetchPage(this.page + 1);
    }

    private async fetchPage(page: number) {
        this.loading = true;
        this.error = null;
        try {
            const res = await homeApi.discover(this.query, page);
            const type = queryContentType(this.query);
            const cards = res.map(item => normalizeDiscoverItem(item, type));
            const seen = new Set(page === 1 ? [] : this.items.map(c => c.key));
            const fresh = cards.filter(c => !seen.has(c.key));
            this.items = page === 1 ? fresh : [...this.items, ...fresh];
            this.page = page;
            this.hasMore = fresh.length > 0;
            this.loaded = true;
        } catch (err) {
            console.error('Failed to load home section', err);
            this.error = err as CoreError;
        } finally {
            this.loading = false;
        }
    }
}

class HomeState {
    layout = $state<HomeLayout | null>(null);
    activeTabId = $state<string | null>(null);
    continueItems = $state<ContinueItem[]>([]);
    loading = $state(false);
    error = $state<CoreError | null>(null);

    hasData = $derived(this.layout !== null);
    tabs = $derived(this.layout?.tabs ?? []);
    activeTab = $derived<HomeTab | null>(
        this.tabs.find(t => t.id === this.activeTabId) ?? this.tabs[0] ?? null
    );

    private feeds = new Map<string, SectionFeed>();

    /** `defaultOnly` ignores the user's layout (TV). */
    async load({ defaultOnly = false } = {}) {
        if (!this.hasData) this.loading = true;
        this.error = null;

        try {
            const [layout, progRes] = await Promise.all([
                defaultOnly ? homeApi.getDefaultLayout() : homeApi.getLayout(),
                progressApi.getContinueWatching(20),
            ]);
            this.setLayout(layout);
            this.continueItems = progRes.items || [];
        } catch (err) {
            console.error('Failed to load home layout', err);
            if (!this.hasData) this.error = err as CoreError;
        } finally {
            this.loading = false;
        }
    }

    async save(layout: HomeLayout) {
        this.setLayout(await homeApi.saveLayout(layout));
    }

    async reset() {
        this.setLayout(await homeApi.resetLayout());
    }

    private setLayout(layout: HomeLayout) {
        this.layout = layout;
        if (!layout.tabs.some(t => t.id === this.activeTabId)) {
            this.activeTabId = layout.tabs[0]?.id ?? null;
        }
    }

    feed(query: DiscoverQuery): SectionFeed {
        const key = JSON.stringify(query);
        let feed = this.feeds.get(key);
        if (!feed) {
            feed = new SectionFeed(query);
            this.feeds.set(key, feed);
        }
        return feed;
    }

    heroSection(tab: HomeTab | null): HomeSection | null {
        if (!tab?.hero) return null;
        return tab.sections.find(s => s.id === tab.hero && s.source.kind === 'search') ?? null;
    }

    async refreshContinueWatching() {
        try {
            const progRes = await progressApi.getContinueWatching(20);
            this.continueItems = progRes.items || [];
        } catch (err) {
            console.error('Failed to refresh continue watching items', err);
        }
    }

    getContinueItems(mode: ContentType | null) {
        return mode ? this.continueItems.filter(item => item.contentType === mode) : this.continueItems;
    }
}

export const homeState = new HomeState();
