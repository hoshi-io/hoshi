import {
    Us, Es, Jp, Fr, De, It, Ru, Sa, Id, Tr, Cn, Kr, Th, Vn, Pl, Pt, Tw, Ph, In, My
} from "svelte-flag-icons";
import { invoke } from '@tauri-apps/api/core';
import type { Store } from '@tauri-apps/plugin-store';

const VALID_LANGS = new Set([
    'en', 'es', 'ja', 'fr', 'de', 'it', 'ru', 'ar', 'id', 'tr',
    'zh-CN', 'ko-KR', 'th', 'vi', 'pl', 'pt-PT', 'zh-TW', 'tl', 'ms', 'hi'
] as const);

export type Language = typeof VALID_LANGS extends Set<infer T> ? T : never;

let tauriStoreInstance: any = null;

async function getTauriStore(): Promise<Store> {
    if (!tauriStoreInstance) {
        const { load } = await import('@tauri-apps/plugin-store');
        tauriStoreInstance = await load('settings.json', { autoSave: true, defaults: {} });
    }
    return tauriStoreInstance as Store;
}

import { locale as osLocale } from '@tauri-apps/plugin-os';

function matchSupported(tag: string | null | undefined): Language | null {
    if (!tag) return null;

    const norm = tag.replace('_', '-');
    if (VALID_LANGS.has(norm as Language)) return norm as Language;

    const [primary, ...rest] = norm.split('-');
    const lower = primary.toLowerCase();
    const parts = rest.map((p) => p.toLowerCase());

    switch (lower) {
        case 'zh':
            // Traditional: script Hant, or regions TW/HK/MO. Everything else is Simplified.
            return parts.some((p) => ['hant', 'tw', 'hk', 'mo'].includes(p)) ? 'zh-TW' : 'zh-CN';
        case 'ko': return 'ko-KR';
        case 'pt': return 'pt-PT';      // also covers pt-BR, since there's no Brazilian file
        case 'fil': return 'tl';        // Filipino
        case 'in': return 'id';         // legacy Indonesian code
        default:
            return VALID_LANGS.has(lower as Language) ? (lower as Language) : null;
    }
}

async function detectSystemLanguage(): Promise<Language> {
    try {
        const match = matchSupported(await osLocale());
        if (match) return match;
    } catch (err) {
        console.warn("[i18n] Error reading system locale:", err);
    }
    return matchSupported(navigator.language) ?? 'en';
}

async function loadSavedLanguage(): Promise<Language> {
    try {
        const store = await getTauriStore();
        const saved = await store.get<string>('app_lang');
        if (saved && VALID_LANGS.has(saved as Language)) return saved as Language;

        const detected = await detectSystemLanguage();
        await store.set('app_lang', detected);
        return detected;
    } catch (err) {
        console.warn("[i18n] Error loading saved language:", err);
    }
    return 'en';
}

class I18n {
    locale = $state<Language>('en');
    currentData = $state<any>(null);
    fallbackData = $state<any>(null);

    private translationCache = new Map<string, string>();

    constructor() {
        this.init();
    }

    private async loadLocale(lang: Language): Promise<any> {
        return invoke<any>('load_locale', { lang });
    }

    private async init() {
        const lang = await loadSavedLanguage();
        const [enData, currentData] = await Promise.all([
            this.loadLocale('en'),
            this.loadLocale(lang),
        ]);

        this.fallbackData = enData;
        this.currentData = currentData;
        this.locale = lang;
    }

    async setLocale(lang: Language) {
        if (!VALID_LANGS.has(lang)) return;

        this.currentData = await this.loadLocale(lang);
        this.locale = lang;
        this.translationCache.clear();

        try {
            const store = await getTauriStore();
            await store.set('app_lang', lang);
        } catch (err) {
            console.warn("[i18n] Error saving language:", err);
        }
    }

    t(key: string, params?: Record<string, string | number>): string {
        const keyStr = key as string;

        if (!this.currentData) return keyStr;

        let result: string;

        if (this.translationCache.has(keyStr)) {
            result = this.translationCache.get(keyStr)!;
        } else {
            const keys = keyStr.split('.');
            let value: any = this.currentData;
            let fallback: any = this.fallbackData;

            for (const k of keys) {
                value = value?.[k];
                fallback = fallback?.[k];
            }

            result = (typeof value === 'string' && value)
                ? value
                : (typeof fallback === 'string' && fallback)
                    ? fallback
                    : keyStr;

            this.translationCache.set(keyStr, result);
        }

        if (params) {
            let interpolated = result;
            for (const [paramKey, paramValue] of Object.entries(params)) {
                interpolated = interpolated.replace(new RegExp(`{{${paramKey}}}`, 'g'), String(paramValue));
            }
            return interpolated;
        }

        return result;
    }

    getAvailableLanguages() {
        return [
            { code: 'en', name: 'English', icon: Us },
            { code: 'es', name: 'Español', icon: Es },
            { code: 'zh-CN', name: '简体中文', icon: Cn },
            { code: 'hi', name: 'हिन्दी', icon: In },
            { code: 'ar', name: 'العربية', icon: Sa },
            { code: 'pt-PT', name: 'Português (Portugal)', icon: Pt },
            { code: 'ru', name: 'Русский', icon: Ru },
            { code: 'ja', name: '日本語', icon: Jp },
            { code: 'de', name: 'Deutsch', icon: De },
            { code: 'fr', name: 'Français', icon: Fr },
            { code: 'it', name: 'Italiano', icon: It },
            { code: 'ko-KR', name: '한국어', icon: Kr },
            { code: 'tr', name: 'Türkçe', icon: Tr },
            { code: 'vi', name: 'Tiếng Việt', icon: Vn },
            { code: 'id', name: 'Bahasa Indonesia', icon: Id },
            { code: 'th', name: 'ไทย', icon: Th },
            { code: 'pl', name: 'Polski', icon: Pl },
            { code: 'ms', name: 'Bahasa Melayu', icon: My },
            { code: 'tl', name: 'Tagalog', icon: Ph },
            { code: 'zh-TW', name: '繁體中文', icon: Tw }
        ];
    }
}

export const i18n = new I18n();