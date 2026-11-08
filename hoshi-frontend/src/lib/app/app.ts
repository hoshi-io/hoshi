import { goto } from '$app/navigation';
import { auth } from '@/stores/auth.svelte.js';
import { extensions } from '@/stores/extensions.svelte.js';
import { discordApi } from "@/api/discord/discord";
import { listStore } from "@/app/list.svelte.js";
import { i18n } from "@/stores/i18n.svelte";
import { scheduleStore } from "@/app/schedule.svelte.js";
import { layoutState } from "@/stores/layout.svelte.js";
import { usersApi } from "@/api/users/users";

const DEFAULT_USERNAME = 'User';

export async function initApp(setTouchDevice: (v: boolean) => void) {
    setTouchDevice(window.matchMedia('(pointer: coarse)').matches);

    await auth.restore();

    if (!auth.isAuthenticated) {
        let hasProfiles = false;
        try {
            hasProfiles = (await usersApi.getAll()).users.length > 0;
        } catch (e) {
            console.error("Could not list profiles", e);
        }

        if (hasProfiles) {
            layoutState.switchProfileOpen = true;
            return;
        }
        try {
            await auth.register({ username: DEFAULT_USERNAME }, null);
        } catch (e) {
            console.error("Could not create the default profile", e);
            layoutState.switchProfileOpen = true;
            return;
        }
    }

    if (auth.isAuthenticated) {
        extensions.load();
        listStore.loadData();
        listStore.initSync();
        scheduleStore.load();
    }
}

export function handleNavigation(pathname: string) {
    if (!auth.initialized) return;

    if (pathname.startsWith('/setup') || pathname.startsWith('/tv/setup')) {
        const useTv = pathname.startsWith('/tv') || layoutState.isAndroidTv;
        goto(useTv ? '/tv' : '/', { replaceState: true });
        return;
    }

    if (auth.user) {
        extensions.load();
    }
}

export function handleDiscordActivity(enabled: boolean, text: string) {
    if (!enabled) return;

    discordApi.setActivity({
        title: i18n.t('discord.browsing'),
        details: text,
        isVideo: false,
        isNsfw: false
    }).catch(() => {});
}