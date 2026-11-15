import { i18n } from "@/stores/i18n.svelte";
import { appConfig } from "@/stores/config.svelte";
import type { ViewsConfig } from "@/api/config/types";

/** Per-profile layout choices for the schedule and list pages (stored in the user config). */
export type ViewPrefs = ViewsConfig;

const DEFAULTS: ViewPrefs = {
    scheduleView: "list",
    scheduleDaysAhead: 7,
    timeFormat: "24h",
    weekStart: 1,
    listDensity: "comfortable",
    listBadges: { status: true, progress: true, score: true },
};

class ViewPrefsStore {
    get data(): ViewPrefs {
        const saved = appConfig.data?.views;
        if (!saved) return DEFAULTS;
        return { ...DEFAULTS, ...saved, listBadges: { ...DEFAULTS.listBadges, ...saved.listBadges } };
    }

    async set(patch: Partial<ViewPrefs>) {
        if (!appConfig.data) return;
        const next = { ...this.data, ...patch };
        // Apply locally first so the page reacts at once; the save reconciles.
        appConfig.data = { ...appConfig.data, views: next };
        try {
            await appConfig.update({ views: next });
        } catch (err) {
            console.error("Failed to save view preferences", err);
            appConfig.load().catch(() => {});
        }
    }

    /** Clock time of a timestamp (ms) in the chosen 12h/24h format. */
    formatTime(ms: number): string {
        return new Date(ms).toLocaleTimeString(i18n.locale, {
            hour: "2-digit",
            minute: "2-digit",
            hour12: this.data.timeFormat === "12h",
        });
    }
}

export const viewPrefs = new ViewPrefsStore();
