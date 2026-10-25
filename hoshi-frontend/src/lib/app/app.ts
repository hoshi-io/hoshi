import { goto } from '$app/navigation';
import { auth } from '@/stores/auth.svelte.js';
import { extensions } from '@/stores/extensions.svelte.js';
import { discordApi } from "@/api/discord/discord";
import {listStore} from "@/app/list.svelte.js";
import {i18n} from "@/stores/i18n.svelte";
import {scheduleStore} from "@/app/schedule.svelte.js";
import {layoutState} from "@/stores/layout.svelte.js";

export async function initApp(setTouchDevice: (v: boolean) => void) {
    setTouchDevice(window.matchMedia('(pointer: coarse)').matches);

    await auth.restore();

    if (auth.isAuthenticated) {
        extensions.load();
        listStore.loadData();
        listStore.initSync();
        scheduleStore.load();
    }
}

export function handleNavigation(pathname: string) {
    if (!auth.initialized) return;

    const isTvRoute = pathname.startsWith('/tv');
    const isSetup = pathname.startsWith('/setup') || pathname.startsWith('/tv/setup');
    
    const useTv = isTvRoute || layoutState.isAndroidTv;

    if (!auth.user && !isSetup) {
        goto(useTv ? '/tv/setup' : '/setup');
    } else if (auth.user && isSetup) {
        goto(isTvRoute ? '/tv' : '/');
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