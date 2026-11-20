import { fontStack } from '@/utils/fonts';

async function loadSavedValue(key: string): Promise<string | null> {
    try {
        const { load } = await import('@tauri-apps/plugin-store');
        const store = await load('settings.json', { autoSave: true, defaults: {} });
        const value = await store.get<string>(key);
        return value ?? null;
    } catch {
        return null;
    }
}

function getContrastColor(hexCode: string): string {
    const hex = hexCode.replace('#', '');
    const r = parseInt(hex.substring(0, 2), 16);
    const g = parseInt(hex.substring(2, 4), 16);
    const b = parseInt(hex.substring(4, 6), 16);
    const yiq = ((r * 299) + (g * 587) + (b * 114)) / 1000;
    return (yiq >= 128) ? '#000000' : '#ffffff';
}

async function persistValue(key: string, value: string | null): Promise<void> {
    try {
        const { load } = await import('@tauri-apps/plugin-store');
        const store = await load('settings.json', { autoSave: true, defaults: {} });
        if (value) {
            await store.set(key, value);
        } else {
            await store.delete(key);
        }
    } catch { /* ignore */ }
}

class ThemeManager {
    theme = $state<string>('dark');
    accentColor = $state<string | null>(null);
    font = $state<string>('system');
    /** No animations, transitions, blur, trailers or hover previews. */
    performance = $state(false);
    /** Stronger muted text and borders. */
    contrast = $state(false);

    constructor() {
        this.init();
    }

    async init() {
        const savedTheme = await loadSavedValue('app_theme');
        const savedAccent = await loadSavedValue('app_accent');

        if (savedTheme) this.theme = savedTheme;
        if (savedAccent) this.accentColor = savedAccent;
        const savedFont = await loadSavedValue('app_font');
        if (savedFont) this.font = savedFont;
        this.performance = (await loadSavedValue('app_performance')) === '1';
        this.contrast = (await loadSavedValue('app_contrast')) === '1';

        this.applyToDOM();
    }

    async setPerformance(on: boolean) {
        this.performance = on;
        this.applyToDOM();
        await persistValue('app_performance', on ? '1' : null);
    }

    async setContrast(on: boolean) {
        this.contrast = on;
        this.applyToDOM();
        await persistValue('app_contrast', on ? '1' : null);
    }

    async setTheme(newTheme: string) {
        this.theme = newTheme;
        this.applyToDOM();
        await persistValue('app_theme', newTheme);
    }

    async setAccentColor(newColor: string | null) {
        if (newColor) {
            const isPureBlack = newColor.toLowerCase() === '#000000';
            const isPureWhite = newColor.toLowerCase() === '#ffffff';

            if (this.theme === 'oled' && isPureBlack) {
                newColor = '#3f3f46'; // Fallback to a visible zinc gray or default accent
            } else if (this.theme === 'light' && isPureWhite) {
                newColor = '#71717a';
            }
        }

        this.accentColor = newColor;
        this.applyToDOM();
        await persistValue('app_accent', newColor);
    }

    async setFont(font: string) {
        this.font = font;
        this.applyToDOM();
        await persistValue('app_font', font === 'system' ? null : font);
    }

    applyToDOM() {
        if (typeof document === 'undefined') return;

        const html = document.documentElement;
        const allThemes = ['light', 'dark', 'oled'];

        allThemes.forEach(t => html.classList.remove(t));
        if (this.theme && this.theme !== 'light') {
            html.classList.add(this.theme);
        }

        if (this.accentColor) {
            html.style.setProperty('--app-accent', this.accentColor);
            html.style.setProperty('--app-accent-foreground', getContrastColor(this.accentColor));
        } else {
            html.style.removeProperty('--app-accent');
            html.style.removeProperty('--app-accent-foreground');
        }

        // Both are handled in layout.css, keyed on these attributes.
        html.toggleAttribute('data-performance', this.performance);
        html.toggleAttribute('data-contrast', this.contrast);

        // Tailwind's base font and `font-sans` both read --font-sans.
        if (this.font && this.font !== 'system') {
            html.style.setProperty('--font-sans', fontStack(this.font));
        } else {
            html.style.removeProperty('--font-sans');
        }
    }
}

export const themeManager = new ThemeManager();