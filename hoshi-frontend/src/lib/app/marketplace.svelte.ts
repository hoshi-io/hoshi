import { untrack } from 'svelte';
import { SvelteSet } from 'svelte/reactivity';
import { toast } from 'svelte-sonner';
import type { ExtensionsConfig } from '@/api/config/types';
import { extensionsApi } from '@/api/extensions/extensions';
import type {
    AnyMarketplaceEntry,
    Extension,
    LNReaderMarketplaceEntry,
    NativeMarketplaceEntry,
    SoraMarketplaceEntry
} from '@/api/extensions/types';
import { extensions } from '@/stores/extensions.svelte.js';
import { i18n } from '@/stores/i18n.svelte.js';

export type Source = 'native' | 'lnreader' | 'sora';
export type Tone = 'anime' | 'manga' | 'novel';

export const SOURCE_LABELS: Record<Source, string> = {
    native: 'Hoshi',
    lnreader: 'LNReader',
    sora: 'Sora'
};

export type CardExt = {
    id: string;
    name: string;
    version: string;
    ext_type?: string;
    icon?: string | null;
    author?: string | null;
    language?: string | null;
    nsfw?: boolean;
    settings?: Record<string, any>;
    setting_definitions?: Extension['setting_definitions'];
};

type InstalledExt = (typeof extensions.installed)[number];

export function errorMessage(error: unknown): string {
    if (typeof error === 'string') return error;
    return (error as any)?.message || i18n.t('errors.unknown');
}

export function sourceOfId(id: string): Source {
    if (id.startsWith('lnr_')) return 'lnreader';
    if (id.startsWith('sora_')) return 'sora';
    return 'native';
}

export function toneOf(type: string | undefined | null): Tone {
    const t = (type || '').toLowerCase();
    return t === 'anime' || t === 'manga' || t === 'novel' ? t : 'anime';
}

export function isHttpUrl(value: string): boolean {
    try {
        const u = new URL(value);
        return u.protocol === 'http:' || u.protocol === 'https:';
    } catch {
        return false;
    }
}

export function buildDraftSettings(ext: CardExt): Record<string, any> {
    const draft: Record<string, any> = { ...(ext.settings ?? {}) };
    for (const def of ext.setting_definitions ?? []) {
        if (draft[def.key] === undefined) {
            draft[def.key] = Array.isArray(def.default) ? [...def.default] : def.default;
        }
    }
    return draft;
}

export function sourceFilterItems() {
    return [
        { value: 'all', label: i18n.t('settings.extension_section.all_sources', { default: 'All sources' }) },
        ...(Object.entries(SOURCE_LABELS) as [Source, string][]).map(([value, label]) => ({ value, label }))
    ];
}

function upsertInstalled(ext: InstalledExt) {
    const i = extensions.installed.findIndex((e) => e.id === ext.id);
    extensions.installed =
        i === -1
            ? [...extensions.installed, ext]
            : extensions.installed.map((e, j) => (j === i ? ext : e));
}

export type InstalledGroup = { type: string; tone: Tone; items: InstalledExt[] };

function normalizeType(type: string | undefined): string {
    if (!type) return 'Other';
    return type.charAt(0).toUpperCase() + type.slice(1).toLowerCase();
}

export function useInstalled() {
    let query = $state('');
    let source = $state<'all' | Source>('all');
    const removing = new SvelteSet<string>();
    const saving = new SvelteSet<string>();

    const sections = $derived.by((): InstalledGroup[] => {
        const q = query.toLowerCase().trim();
        const groups: Record<string, InstalledExt[]> = {};

        const sorted = [...extensions.installed].sort((a, b) =>
            (a.name ?? '').localeCompare(b.name ?? '')
        );

        for (const ext of sorted) {
            if (source !== 'all' && sourceOfId(ext.id) !== source) continue;
            if (q && !(ext.name ?? '').toLowerCase().includes(q)) continue;
            (groups[normalizeType(ext.ext_type)] ??= []).push(ext);
        }

        return Object.entries(groups)
            .sort(([a], [b]) => a.localeCompare(b))
            .map(([type, items]) => ({ type, tone: toneOf(type), items }));
    });

    const visibleCount = $derived(sections.reduce((n, s) => n + s.items.length, 0));

    async function uninstall(id: string) {
        removing.add(id);
        try {
            await extensions.uninstall(id);
        } catch (error) {
            toast.error(errorMessage(error));
        } finally {
            removing.delete(id);
        }
    }

    async function saveSettings(id: string, settings: Record<string, any>): Promise<boolean> {
        saving.add(id);
        try {
            const res = await extensionsApi.updateSettings(id, settings);
            if (!res.ok) return false;
            const ext = extensions.installed.find((e) => e.id === id);
            if (ext) ext.settings = { ...settings };
            return true;
        } catch (error) {
            toast.error(errorMessage(error));
            return false;
        } finally {
            saving.delete(id);
        }
    }

    return {
        get query() { return query; },
        set query(v: string) { query = v; },
        get source() { return source; },
        set source(v: 'all' | Source) { source = v; },
        get sections() { return sections; },
        get total() { return extensions.installed.length; },
        get visibleCount() { return visibleCount; },
        get view(): 'loading' | 'empty' | 'noMatch' | 'ready' {
            if (extensions.installed.length === 0) return extensions.loading ? 'loading' : 'empty';
            return visibleCount === 0 ? 'noMatch' : 'ready';
        },
        isRemoving: (id: string) => removing.has(id),
        isSaving: (id: string) => saving.has(id),
        uninstall,
        saveSettings
    };
}
export type InstalledState = ReturnType<typeof useInstalled>;


const LANG_CODES: Record<string, string> = {
    af: 'Afrikaans', ar: 'Arabic', az: 'Azerbaijani', be: 'Belarusian',
    bg: 'Bulgarian', bn: 'Bengali', ca: 'Catalan', cs: 'Czech',
    da: 'Danish', de: 'German', el: 'Greek', en: 'English',
    eo: 'Esperanto', es: 'Spanish', et: 'Estonian', eu: 'Basque',
    fa: 'Persian', fi: 'Finnish', fr: 'French', gl: 'Galician',
    gu: 'Gujarati', he: 'Hebrew', hi: 'Hindi', hr: 'Croatian',
    hu: 'Hungarian', hy: 'Armenian', id: 'Indonesian', is: 'Icelandic',
    it: 'Italian', ja: 'Japanese', ka: 'Georgian', kk: 'Kazakh',
    km: 'Khmer', kn: 'Kannada', ko: 'Korean', lt: 'Lithuanian',
    lv: 'Latvian', mk: 'Macedonian', ml: 'Malayalam', mn: 'Mongolian',
    mr: 'Marathi', ms: 'Malay', my: 'Burmese', ne: 'Nepali',
    nl: 'Dutch', no: 'Norwegian', pa: 'Punjabi', pl: 'Polish',
    pt: 'Portuguese', ro: 'Romanian', ru: 'Russian', si: 'Sinhala',
    sk: 'Slovak', sl: 'Slovenian', sq: 'Albanian', sr: 'Serbian',
    sv: 'Swedish', sw: 'Swahili', ta: 'Tamil', te: 'Telugu',
    th: 'Thai', tl: 'Filipino', tr: 'Turkish', uk: 'Ukrainian',
    ur: 'Urdu', uz: 'Uzbek', vi: 'Vietnamese', zh: 'Chinese',
    'zh-hans': 'Chinese (Simplified)', 'zh-hant': 'Chinese (Traditional)',
    'pt-br': 'Portuguese (Brazil)', 'pt-pt': 'Portuguese (Portugal)',
    'es-419': 'Spanish (Latin America)'
};

export function normalizeLang(lang: string | undefined): string {
    if (!lang) return 'Unknown';
    const l = lang.toLowerCase().trim().replace(/\s*\([^)]*\)\s*$/, '').trim();
    if (['all', 'multi', 'multiple', 'multilingual', 'various', 'universal'].includes(l)) return 'Multi';
    return LANG_CODES[l] ?? l.charAt(0).toUpperCase() + l.slice(1);
}

function normalizeSoraType(raw: string | undefined): string {
    switch (raw) {
        case 'anime': return 'anime';
        case 'novels':
        case 'novel': return 'novel';
        case 'movies/shows':
        case 'movies':
        case 'shows': return 'movie';
        default: return 'other';
    }
}

function isSora(item: AnyMarketplaceEntry): item is SoraMarketplaceEntry {
    return 'sourceName' in item && 'manifestUrl' in item && 'installCount' in item;
}

function isLNReader(item: AnyMarketplaceEntry): item is LNReaderMarketplaceEntry {
    return 'url' in item && 'iconUrl' in item;
}

function getItemLang(item: AnyMarketplaceEntry): string | undefined {
    return 'language' in item ? item.language : item.lang;
}

function getItemName(item: AnyMarketplaceEntry): string {
    return isSora(item) ? item.sourceName : (item.name ?? '');
}

function installedIdOf(item: AnyMarketplaceEntry): string {
    if (isLNReader(item)) return `lnr_${item.id}`;
    if (isSora(item)) return `sora_${item.id}`;
    return item.id;
}

export function sourceOfEntry(item: AnyMarketplaceEntry): Source {
    return isLNReader(item) ? 'lnreader' : isSora(item) ? 'sora' : 'native';
}

export function toCardExt(item: AnyMarketplaceEntry): CardExt {
    if (isLNReader(item)) {
        return {
            ...item,
            ext_type: 'novel',
            icon: item.iconUrl,
            language: item.lang,
            author: 'LNReader'
        } as CardExt;
    }
    if (isSora(item)) {
        return {
            ...item,
            name: item.sourceName,
            icon: item.iconUrl,
            ext_type: normalizeSoraType(item.type),
            language: item.language,
            author: item.author?.name ?? 'Unknown'
        } as CardExt;
    }
    return { ...(item as any), language: getItemLang(item) } as CardExt;
}

function isNewerVersion(repoVer: string | undefined, installedVer: string): boolean {
    if (!repoVer || !installedVer) return false;
    const repo = repoVer.split('.').map((v) => parseInt(v, 10) || 0);
    const inst = installedVer.split('.').map((v) => parseInt(v, 10) || 0);
    for (let i = 0; i < Math.max(repo.length, inst.length); i++) {
        const r = repo[i] || 0;
        const n = inst[i] || 0;
        if (r > n) return true;
        if (r < n) return false;
    }
    return false;
}

function repoLabel(url: string, index: number): string {
    if (!url) return i18n.t('marketplace.new_repo', { default: 'New repository' });
    try {
        const u = new URL(url);
        if (u.hostname === 'raw.githubusercontent.com') {
            const parts = u.pathname.split('/');
            if (parts.length > 2 && parts[2]) return `${parts[1]}/${parts[2]}`;
        }
        return u.hostname;
    } catch {
        return `Repo ${index + 1}`;
    }
}

const CHUNK_SIZE = 24;

type MarketplaceOptions = {
    config: () => ExtensionsConfig;
    onSave: () => Promise<void> | void;
    /** Nothing is fetched until this is true (e.g. the tab was opened). Default: always. */
    enabled?: () => boolean;
};

export type MarketplaceGroup = { lang: string; items: AnyMarketplaceEntry[]; total: number };

export function useMarketplace(opts: MarketplaceOptions) {
    const initial = opts.config().repoUrls as string[] | string | undefined;
    let repos = $state<string[]>(
        Array.isArray(initial) && initial.length > 0 ? [...initial] : initial && !Array.isArray(initial) ? [initial] : ['']
    );
    let activeIndex = $state(0);
    const activeUrl = $derived(repos[activeIndex] ?? '');

    let items = $state<AnyMarketplaceEntry[]>([]);
    let loading = $state(false);
    let error = $state<string | null>(null);
    let loadedUrl = $state('');
    let reqId = 0;

    let query = $state('');
    let lang = $state('All');
    let visibleCount = $state(CHUNK_SIZE);

    const busy = new SvelteSet<string>();

    const languages = $derived(
        Array.from(new Set(items.map((item) => normalizeLang(getItemLang(item))))).sort()
    );

    const langItems = $derived([
        { value: 'All', label: i18n.t('marketplace.all_languages', { default: 'All languages' }) },
        ...languages.map((l) => ({ value: l, label: l }))
    ]);

    const repoItems = $derived(repos.map((url, i) => ({ value: String(i), label: repoLabel(url, i) })));

    const sections = $derived.by((): { lang: string; items: AnyMarketplaceEntry[] }[] => {
        const q = query.toLowerCase().trim();
        const groups: Record<string, AnyMarketplaceEntry[]> = {};
        for (const item of items) {
            const l = normalizeLang(getItemLang(item));
            if (lang !== 'All' && l !== lang) continue;
            if (q && !getItemName(item).toLowerCase().includes(q)) continue;
            (groups[l] ??= []).push(item);
        }
        return Object.entries(groups).map(([l, list]) => ({ lang: l, items: list }));
    });

    $effect(() => {
        sections;
        visibleCount = CHUNK_SIZE;
    });

    const totalCards = $derived(sections.reduce((n, s) => n + s.items.length, 0));
    const hasMore = $derived(visibleCount < totalCards);

    const visibleSections = $derived.by((): MarketplaceGroup[] => {
        let remaining = visibleCount;
        const out: MarketplaceGroup[] = [];
        for (const s of sections) {
            if (remaining <= 0) break;
            out.push({ lang: s.lang, items: s.items.slice(0, remaining), total: s.items.length });
            remaining -= s.items.length;
        }
        return out;
    });

    const updateCount = $derived(items.filter((item) => hasUpdate(item)).length);

    async function load(url: string) {
        const id = ++reqId;
        loading = true;
        error = null;
        loadedUrl = url;
        lang = 'All';
        items = [];

        try {
            const res = await fetch(url);
            if (!res.ok) throw new Error(i18n.t('errors.network'));
            const data = await res.json();
            if (id !== reqId) return; // a newer request took over
            items = Array.isArray(data) ? data : (data.extensions ?? []);
        } catch (e) {
            if (id !== reqId) return;
            error = errorMessage(e);
            toast.error(error);
            items = [];
        } finally {
            if (id === reqId) loading = false;
        }
    }

    function syncActive() {
        const url = activeUrl;
        if (!url) {
            reqId++;
            items = [];
            loadedUrl = '';
            loading = false;
            error = null;
            return;
        }
        if (url !== loadedUrl && isHttpUrl(url)) load(url);
    }

    // First load, once the marketplace is allowed to fetch.
    $effect(() => {
        if (!(opts.enabled?.() ?? true)) return;
        untrack(syncActive);
    });

    function reload() {
        loadedUrl = '';
        syncActive();
    }

    function persist() {
        opts.config().repoUrls = repos.map((u) => u.trim()).filter((u) => u !== '');
        void opts.onSave();
    }

    function selectRepo(index: number) {
        activeIndex = index;
        syncActive();
    }

    function addRepo() {
        repos = [...repos, ''];
        activeIndex = repos.length - 1;
        persist();
        syncActive();
    }

    function removeRepo(index: number) {
        if (repos.length <= 1) {
            repos = [''];
            activeIndex = 0;
        } else {
            repos = repos.filter((_, i) => i !== index);
            if (index < activeIndex || activeIndex >= repos.length) {
                activeIndex = Math.max(0, activeIndex - 1);
            }
        }
        persist();
        syncActive();
    }

    /** Call when a URL input is committed (Enter / blur). */
    function commitRepos() {
        repos = repos.map((u) => u.trim());
        persist();
        syncActive();
    }

    function installedVersion(item: AnyMarketplaceEntry): string | null {
        const id = installedIdOf(item);
        return extensions.installed.find((e) => e.id === id)?.version ?? null;
    }

    function isInstalled(item: AnyMarketplaceEntry): boolean {
        const id = installedIdOf(item);
        return extensions.installed.some((e) => e.id === id);
    }

    function hasUpdate(item: AnyMarketplaceEntry): boolean {
        const v = installedVersion(item);
        return v ? isNewerVersion(item.version, v) : false;
    }

    function applyResult(res: { ok?: boolean; extension?: any }) {
        if (res.ok && res.extension) upsertInstalled(res.extension);
        else if (!res.ok) toast.error(i18n.t('errors.unknown'));
    }

    async function install(item: AnyMarketplaceEntry) {
        const key = installedIdOf(item);
        busy.add(key);
        try {
            if (isLNReader(item)) {
                applyResult(await extensionsApi.installLNReader(item));
            } else if (isSora(item)) {
                applyResult(await extensionsApi.installSora(item));
            } else {
                const manifest = (item as NativeMarketplaceEntry).manifestUrl;
                if (!manifest) {
                    toast.error(i18n.t('marketplace.missing_manifest'));
                    return;
                }
                await extensions.install(manifest);
            }
        } catch (e) {
            toast.error(errorMessage(e));
        } finally {
            busy.delete(key);
        }
    }

    async function update(item: AnyMarketplaceEntry) {
        const key = installedIdOf(item);
        busy.add(key);
        try {
            if (isLNReader(item)) {
                applyResult(await extensionsApi.installLNReader(item));
            } else if (isSora(item)) {
                applyResult(await extensionsApi.installSora(item));
            } else {
                const manifest = (item as NativeMarketplaceEntry).manifestUrl;
                if (!manifest) {
                    toast.error(i18n.t('marketplace.missing_manifest'));
                    return;
                }
                await extensions.update(item.id, manifest);
            }
        } catch (e) {
            toast.error(errorMessage(e));
        } finally {
            busy.delete(key);
        }
    }

    return {
        /* repositories */
        get repos() { return repos; },
        get repoItems() { return repoItems; },
        get activeIndex() { return activeIndex; },
        get activeUrl() { return activeUrl; },
        get loading() { return loading; },
        selectRepo,
        addRepo,
        removeRepo,
        commitRepos,
        reload,

        /* filters */
        get query() { return query; },
        set query(v: string) { query = v; },
        get lang() { return lang; },
        set lang(v: string) { lang = v; },
        get langItems() { return langItems; },

        /* results */
        get view(): 'noRepo' | 'invalid' | 'loading' | 'error' | 'empty' | 'noMatch' | 'ready' {
            if (!activeUrl) return 'noRepo';
            if (!isHttpUrl(activeUrl)) return 'invalid';
            if (items.length > 0) return sections.length === 0 ? 'noMatch' : 'ready';
            if (error && !loading) return 'error';
            if (loading || loadedUrl !== activeUrl) return 'loading';
            return 'empty';
        },
        get hasItems() { return items.length > 0; },
        get error() { return error; },
        get sections() { return visibleSections; },
        get hasMore() { return hasMore; },
        get visibleCount() { return visibleCount; },
        loadMore() { if (hasMore) visibleCount += CHUNK_SIZE; },
        get updateCount() { return updateCount; },

        /* per-entry */
        isInstalled,
        hasUpdate,
        installedVersion,
        isBusy: (item: AnyMarketplaceEntry) => busy.has(installedIdOf(item)),
        install,
        update
    };
}
export type MarketplaceState = ReturnType<typeof useMarketplace>;