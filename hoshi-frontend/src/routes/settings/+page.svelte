<script lang="ts">
    import { auth } from "@/stores/auth.svelte.js";
    import { toast } from "svelte-sonner";
    import { fade, fly } from "svelte/transition";
    import { cubicOut } from "svelte/easing";
    import { goto } from '$app/navigation';
    import {
        User, Link2, Settings, MonitorPlay, Puzzle, BookOpen, LayoutTemplate, Database,
        MessageSquare, ChevronRight, Terminal
    } from "lucide-svelte";
    import { Spinner } from "$lib/components/ui/spinner";
    import * as Avatar from "$lib/components/ui/avatar";
    import SwitchProfile from "$lib/components/modals/SwitchProfile.svelte";
    import { type, version, arch } from "@tauri-apps/plugin-os";
    import Account from "$lib/components/settings/Account.svelte";
    import Tracker from "$lib/components/settings/Tracker.svelte";
    import General from "$lib/components/settings/General.svelte";
    import UI from "$lib/components/settings/UI.svelte";
    import Extensions from "$lib/components/settings/extensions/Extensions.svelte";
    import Player from "$lib/components/settings/Player.svelte";
    import Readers from "$lib/components/settings/Readers.svelte";
    import Discord from "$lib/components/settings/Discord.svelte";
    import LogsViewer from "$lib/components/settings/LogsViewer.svelte";
    import * as Tabs from "$lib/components/ui/tabs";
    import { appConfig } from "@/stores/config.svelte.js";
    import { layoutState } from '@/stores/layout.svelte.js';
    import { i18n } from "@/stores/i18n.svelte.js";
    import { onMount } from "svelte";
    import { page } from "$app/state";
    import {getVersion} from "@tauri-apps/api/app";

    let showSwitchProfile = $state(false);
    let configSaving = $state(false);
    let isDesktop = $state(false);
    let activeTab = $state(page.url.searchParams.get('tab') || 'account');
    let isMobileDetail = $derived(page.url.searchParams.has('tab'));

    let appVersion = $state("");
    let osInfo = $state("");
    let hardwareInfo = $state("");
    let webviewInfo = $state("");

    onMount(() => {
        const mediaQuery = window.matchMedia('(min-width: 768px)');
        isDesktop = mediaQuery.matches;

        const updateLayout = (e: MediaQueryListEvent) => {
            isDesktop = e.matches;
        };

        mediaQuery.addEventListener('change', updateLayout);

        getVersion()
            .then((v) => appVersion = v)
            .catch(console.error);

        try {
            osInfo = `${type()} ${version()} (${arch()})`;
        } catch (err) {
            console.error("Failed to get OS info:", err);
        }

        try {
            const cores = navigator.hardwareConcurrency ? `${navigator.hardwareConcurrency} Cores` : '';
            hardwareInfo = [cores].filter(Boolean).join(' • ');

            const ua = navigator.userAgent;
            if (ua.includes("Edg/")) {
                const match = ua.match(/Edg\/(\d+\.\d+)/);
                webviewInfo = match ? `WebView2 v${match[1]}` : "WebView2";
            } else if (ua.includes("Chrome/")) {
                const match = ua.match(/Chrome\/(\d+\.\d+)/);
                webviewInfo = match ? `Chromium v${match[1]}` : "Chromium";
            } else if (ua.includes("WebKit/")) {
                const match = ua.match(/WebKit\/(\d+\.\d+)/);
                webviewInfo = match ? `WebKit v${match[1]}` : "WebKit";
            } else {
                webviewInfo = "Unknown Engine";
            }
        } catch (err) {
            console.error("Failed to get extra system info:", err);
        }

        return () => mediaQuery.removeEventListener('change', updateLayout);
    });

    onMount(() => {
        const mediaQuery = window.matchMedia('(min-width: 768px)');

        isDesktop = mediaQuery.matches;

        const updateLayout = (e: MediaQueryListEvent) => {
            isDesktop = e.matches;
        };

        mediaQuery.addEventListener('change', updateLayout);

        return () => mediaQuery.removeEventListener('change', updateLayout);
    });

    $effect(() => {
        if (isDesktop) {
            layoutState.title = "Settings";
            layoutState.showBack = true;
            layoutState.backUrl = null;
            layoutState.headerAction = undefined;
        } else {
            if (isMobileDetail) {
                const titles: Record<string, string> = {
                    account: i18n.t('settings.account'),
                    general: i18n.t('settings.general'),
                    ui: i18n.t('settings.interface'),
                    logs: i18n.t('settings.logs.title'),
                    player: i18n.t('settings.player'),
                    readers: i18n.t('settings.readers'),
                    content: i18n.t('settings.content'),
                    extensions: i18n.t('settings.extensions'),
                    tracking: i18n.t('settings.tracking'),
                    discord: i18n.t('settings.discord')
                };
                layoutState.title = titles[activeTab] || i18n.t('settings.title');
                layoutState.showBack = true;
                layoutState.backUrl = null;
                layoutState.headerAction = undefined;
            } else {
                layoutState.title = i18n.t('settings.title');
                layoutState.showBack = true;
                layoutState.backUrl = '/';
                layoutState.headerAction = undefined;
            }
        }
    });

    $effect(() => {
        const urlTab = page.url.searchParams.get('tab');
        if (!isDesktop && !urlTab) {
            activeTab = '';
        } else if (urlTab) {
            activeTab = urlTab;
        }
    });

    async function handleSaveConfig() {
        if (!appConfig.data) return;
        configSaving = true;
        try {
            await appConfig.update(appConfig.data);
        } catch (err) {
            console.error(err);
            toast.error("Failed to update preferences");
        } finally {
            configSaving = false;
        }
    }
</script>

<svelte:head>
    <title>{i18n.t('settings.title')}</title>
</svelte:head>

<main class="bg-background pb-6 md:pb-12 {isMobileDetail ? 'pt-0 md:pt-12' : 'pt-4 md:pt-20'} px-4 md:px-8 lg:pl-32 lg:pr-12 w-full max-w-[2000px] mx-auto {isMobileDetail ? 'space-y-0 md:space-y-8' : 'space-y-4 md:space-y-8'}">

    <header
            in:fly={{ y: -10, duration: 400, easing: cubicOut }}
            class="{isMobileDetail ? 'hidden md:flex' : 'flex'} flex-col md:flex-row md:items-center justify-between gap-6 border-b border-border/40 pb-4 md:pb-8 w-full"
    >
        <div class="flex items-center gap-5">
            <Avatar.Root class="h-12 w-12 md:h-16 md:w-16 border border-border/50 shadow-sm transition-transform duration-300 hover:scale-102">
                {#if auth.user?.avatar}
                    <Avatar.Image src={auth.user.avatar} alt={auth.user.username} class="object-cover" />
                {/if}
                <Avatar.Fallback class="bg-primary/10 text-primary font-black uppercase">
                    {auth.user?.username?.charAt(0) || 'U'}
                </Avatar.Fallback>
            </Avatar.Root>

            <div class="space-y-0.5">
                <h1 class="text-2xl md:text-3xl font-black tracking-tight">{auth.user?.username || 'Account'}</h1>
                <p class="text-xs md:text-sm text-muted-foreground font-medium opacity-70 uppercase tracking-wider">
                    {i18n.t('settings.preferences')}
                </p>
            </div>
        </div>
    </header>

    <section class="w-full">
        {#if !auth.user || !appConfig.data}
            <div in:fade={{ duration: 200 }} class="h-[50vh] flex flex-col items-center justify-center gap-4 text-muted-foreground">
                <Spinner class="h-10 w-10 animate-spin text-primary" />
                <p class="text-sm font-bold animate-pulse">{i18n.t('settings.loading')}</p>
            </div>
        {:else}
            <div in:fly={{ y: 15, duration: 400, delay: 100, easing: cubicOut }} class="w-full">
                <Tabs.Root
                        value={activeTab}
                        onValueChange={(v) => {
                        activeTab = v;
                        if (!isDesktop) goto(`?tab=${v}`);
                        document.querySelector('main')?.scrollTo({ top: 0, behavior: 'smooth' });
                    }}
                        class="flex flex-col md:flex-row gap-8 lg:gap-16 w-full items-start"
                >
                    <Tabs.List class="{isMobileDetail ? 'hidden md:flex' : 'flex'} flex-col justify-start bg-transparent h-auto p-0 gap-1 w-full md:w-64 shrink-0 border-none md:sticky md:top-24 max-h-[calc(100vh-8rem)] overflow-y-auto hide-scrollbar">                        <div class="px-4 pt-2 pb-2 text-[10px] font-black uppercase tracking-widest text-muted-foreground/50 text-left w-full">
                            {i18n.t('settings.section_profile')}
                        </div>
                        <Tabs.Trigger value="account" class="relative px-4 py-2.5 rounded-xl text-base md:text-sm font-bold transition-all duration-200 data-[state=active]:bg-primary/10 data-[state=active]:text-primary data-[state=inactive]:hover:bg-muted/40 w-full flex items-center justify-between group">
                            <div class="flex items-center gap-4 md:gap-3">
                                <User class="h-5 w-5 md:h-4 md:w-4 transition-transform duration-300 group-hover:scale-105" />
                                {i18n.t('settings.account')}
                            </div>
                            <ChevronRight class="h-5 w-5 md:hidden text-muted-foreground opacity-50 transition-transform group-hover:translate-x-0.5" />
                        </Tabs.Trigger>

                        <div class="px-4 pt-6 pb-2 text-[10px] font-black uppercase tracking-widest text-muted-foreground/50 text-left w-full">
                            {i18n.t('settings.section_application')}
                        </div>
                        <Tabs.Trigger value="general" class="relative px-4 py-2.5 rounded-xl text-base md:text-sm font-bold transition-all duration-200 data-[state=active]:bg-primary/10 data-[state=active]:text-primary data-[state=inactive]:hover:bg-muted/40 w-full flex items-center justify-between group">
                            <div class="flex items-center gap-4 md:gap-3">
                                <Settings class="h-5 w-5 md:h-4 md:w-4 transition-transform duration-300 group-hover:rotate-45" />
                                {i18n.t('settings.general')}
                            </div>
                            <ChevronRight class="h-5 w-5 md:hidden text-muted-foreground opacity-50 transition-transform group-hover:translate-x-0.5" />
                        </Tabs.Trigger>
                        <Tabs.Trigger value="ui" class="relative px-4 py-2.5 rounded-xl text-base md:text-sm font-bold transition-all duration-200 data-[state=active]:bg-primary/10 data-[state=active]:text-primary data-[state=inactive]:hover:bg-muted/40 w-full flex items-center justify-between group">
                            <div class="flex items-center gap-4 md:gap-3">
                                <LayoutTemplate class="h-5 w-5 md:h-4 md:w-4 transition-transform duration-300 group-hover:scale-105" />
                                {i18n.t('settings.interface')}
                            </div>
                            <ChevronRight class="h-5 w-5 md:hidden text-muted-foreground opacity-50 transition-transform group-hover:translate-x-0.5" />
                        </Tabs.Trigger>
                        <Tabs.Trigger value="logs" class="relative px-4 py-2.5 rounded-xl text-base md:text-sm font-bold transition-all duration-200 data-[state=active]:bg-primary/10 data-[state=active]:text-primary data-[state=inactive]:hover:bg-muted/40 w-full flex items-center justify-between group">
                            <div class="flex items-center gap-4 md:gap-3">
                                <Terminal class="h-5 w-5 md:h-4 md:w-4 transition-transform duration-300 group-hover:translate-x-0.5" />
                                {i18n.t('settings.logs.title')}
                            </div>
                            <ChevronRight class="h-5 w-5 md:hidden text-muted-foreground opacity-50 transition-transform group-hover:translate-x-0.5" />
                        </Tabs.Trigger>

                        <div class="px-4 pt-6 pb-2 text-[10px] font-black uppercase tracking-widest text-muted-foreground/50 text-left w-full">
                            {i18n.t('settings.section_experience')}
                        </div>
                        <Tabs.Trigger value="player" class="relative px-4 py-2.5 rounded-xl text-base md:text-sm font-bold transition-all duration-200 data-[state=active]:bg-primary/10 data-[state=active]:text-primary data-[state=inactive]:hover:bg-muted/40 w-full flex items-center justify-between group">
                            <div class="flex items-center gap-4 md:gap-3">
                                <MonitorPlay class="h-5 w-5 md:h-4 md:w-4 transition-transform duration-300 group-hover:scale-105" />
                                {i18n.t('settings.player')}
                            </div>
                            <ChevronRight class="h-5 w-5 md:hidden text-muted-foreground opacity-50 transition-transform group-hover:translate-x-0.5" />
                        </Tabs.Trigger>
                        <Tabs.Trigger value="readers" class="relative px-4 py-2.5 rounded-xl text-base md:text-sm font-bold transition-all duration-200 data-[state=active]:bg-primary/10 data-[state=active]:text-primary data-[state=inactive]:hover:bg-muted/40 w-full flex items-center justify-between group">
                            <div class="flex items-center gap-4 md:gap-3">
                                <BookOpen class="h-5 w-5 md:h-4 md:w-4 transition-transform duration-300 group-hover:scale-105" />
                                {i18n.t('settings.readers')}
                            </div>
                            <ChevronRight class="h-5 w-5 md:hidden text-muted-foreground opacity-50 transition-transform group-hover:translate-x-0.5" />
                        </Tabs.Trigger>

                        <div class="px-4 pt-6 pb-2 text-[10px] font-black uppercase tracking-widest text-muted-foreground/50 text-left w-full">
                            {i18n.t('settings.section_integrations')}
                        </div>
                        <Tabs.Trigger value="extensions" class="relative px-4 py-2.5 rounded-xl text-base md:text-sm font-bold transition-all duration-200 data-[state=active]:bg-primary/10 data-[state=active]:text-primary data-[state=inactive]:hover:bg-muted/40 w-full flex items-center justify-between group">
                            <div class="flex items-center gap-4 md:gap-3">
                                <Puzzle class="h-5 w-5 md:h-4 md:w-4 transition-transform duration-300 group-hover:rotate-12" />
                                {i18n.t('settings.extensions')}
                            </div>
                            <ChevronRight class="h-5 w-5 md:hidden text-muted-foreground opacity-50 transition-transform group-hover:translate-x-0.5" />
                        </Tabs.Trigger>
                        <Tabs.Trigger value="tracking" class="relative px-4 py-2.5 rounded-xl text-base md:text-sm font-bold transition-all duration-200 data-[state=active]:bg-primary/10 data-[state=active]:text-primary data-[state=inactive]:hover:bg-muted/40 w-full flex items-center justify-between group">
                            <div class="flex items-center gap-4 md:gap-3">
                                <Link2 class="h-5 w-5 md:h-4 md:w-4 transition-transform duration-300 group-hover:rotate-45" />
                                {i18n.t('settings.tracking')}
                            </div>
                            <ChevronRight class="h-5 w-5 md:hidden text-muted-foreground opacity-50 transition-transform group-hover:translate-x-0.5" />
                        </Tabs.Trigger>

                        {#if isDesktop}
                            <Tabs.Trigger value="discord" class="relative px-4 py-2.5 rounded-xl text-base md:text-sm font-bold transition-all duration-200 data-[state=active]:bg-primary/10 data-[state=active]:text-primary data-[state=inactive]:hover:bg-muted/40 w-full flex items-center justify-between group">
                                <div class="flex items-center gap-4 md:gap-3">
                                    <MessageSquare class="h-5 w-5 md:h-4 md:w-4 transition-transform duration-300 group-hover:scale-105" />
                                    {i18n.t('settings.discord')}
                                </div>
                                <ChevronRight class="h-5 w-5 md:hidden text-muted-foreground opacity-50 transition-transform group-hover:translate-x-0.5" />
                            </Tabs.Trigger>
                        {/if}
                        <div class="mt-8 md:mt-auto pt-4 pb-4 px-4 w-full flex flex-col items-center md:items-start gap-4 text-center md:text-left">
                            <div class="flex flex-col gap-1 text-[10px] font-bold text-muted-foreground/40 uppercase tracking-wider w-full">
                                {#if appVersion}
                                    <span class="text-xs text-primary/80 mb-1">Version {appVersion} ★</span>
                                {/if}
                                {#if osInfo}
                                    <span class="truncate" title={osInfo}>{osInfo}</span>
                                {/if}
                                {#if hardwareInfo}
                                    <span class="truncate" title={hardwareInfo}>{hardwareInfo}</span>
                                {/if}
                                {#if webviewInfo}
                                    <span class="truncate" title={webviewInfo}>{webviewInfo}</span>
                                {/if}
                            </div>
                            <img
                                    src="/chibi.png"
                                    alt="App Mascot"
                                    class="w-20 h-20 object-contain hover:opacity-100 hover:scale-105 transition-all duration-300 drop-shadow-sm will-change-[transform,opacity]"
                            />
                        </div>
                    </Tabs.List>

                    <div class="{isMobileDetail ? 'block' : 'hidden md:block pr-5'} mobile-content-wrapper flex-1 min-w-0 w-full max-w-none">

                        <Tabs.Content value="account" class="focus-visible:outline-none mt-0 w-full">
                            {#if activeTab === 'account'}
                                <div in:fly={{ y: 8, duration: 250, easing: cubicOut }}>
                                    <Account
                                            user={auth.user}
                                            onUpdate={() => auth.restore(true)}
                                            onDeleted={() => showSwitchProfile = true}
                                    />
                                </div>
                            {/if}
                        </Tabs.Content>

                        {#if appConfig.data}
                            <Tabs.Content value="general" class="focus-visible:outline-none mt-0 w-full">
                                {#if activeTab === 'general'}
                                    <div in:fly={{ y: 8, duration: 250, easing: cubicOut }}>
                                        <General bind:config={appConfig.data.general} bind:contentConfig={appConfig.data.content} onSave={handleSaveConfig} />
                                    </div>
                                {/if}
                            </Tabs.Content>

                            <Tabs.Content value="ui" class="focus-visible:outline-none mt-0 w-full">
                                {#if activeTab === 'ui'}
                                    <div in:fly={{ y: 8, duration: 250, easing: cubicOut }}>
                                        <UI bind:config={appConfig.data.ui} onSave={handleSaveConfig} />
                                    </div>
                                {/if}
                            </Tabs.Content>

                            <Tabs.Content value="logs" class="focus-visible:outline-none mt-0 w-full max-h-[75vh] flex flex-col">
                                {#if activeTab === 'logs'}
                                    <div in:fly={{ y: 8, duration: 250, easing: cubicOut }} class="w-full flex-1 flex flex-col min-h-0">
                                        <LogsViewer />
                                    </div>
                                {/if}
                            </Tabs.Content>

                            <Tabs.Content value="player" class="focus-visible:outline-none mt-0 w-full">
                                {#if activeTab === 'player'}
                                    <div in:fly={{ y: 8, duration: 250, easing: cubicOut }}>
                                        <Player
                                                bind:playerConfig={appConfig.data.player}
                                                bind:mpvConfig={appConfig.data.mpv}
                                                onSave={handleSaveConfig}
                                        />
                                    </div>
                                {/if}
                            </Tabs.Content>

                            <Tabs.Content value="readers" class="focus-visible:outline-none mt-0 w-full">
                                {#if activeTab === 'readers'}
                                    <div in:fly={{ y: 8, duration: 250, easing: cubicOut }}>
                                        <Readers bind:mangaConfig={appConfig.data.manga} bind:novelConfig={appConfig.data.novel} onSave={handleSaveConfig} />
                                    </div>
                                {/if}
                            </Tabs.Content>

                            <Tabs.Content value="extensions" class="focus-visible:outline-none mt-0 w-full">
                                {#if activeTab === 'extensions'}
                                    <div in:fly={{ y: 8, duration: 250, easing: cubicOut }}>
                                        <Extensions bind:config={appConfig.data.extensions} onSave={handleSaveConfig} />
                                    </div>
                                {/if}
                            </Tabs.Content>

                            <Tabs.Content value="tracking" class="focus-visible:outline-none mt-0 w-full">
                                {#if activeTab === 'tracking'}
                                    <div in:fly={{ y: 8, duration: 250, easing: cubicOut }}>
                                        <Tracker bind:config={appConfig.data.list} onSave={handleSaveConfig}/>
                                    </div>
                                {/if}
                            </Tabs.Content>

                            {#if isDesktop}
                                <Tabs.Content value="discord" class="focus-visible:outline-none mt-0 w-full">
                                    {#if activeTab === 'discord'}
                                        <div in:fly={{ y: 8, duration: 250, easing: cubicOut }}>
                                            <Discord bind:config={appConfig.data.discord} onSave={handleSaveConfig} />
                                        </div>
                                    {/if}
                                </Tabs.Content>
                            {/if}
                        {/if}
                    </div>
                </Tabs.Root>
            </div>
        {/if}
    </section>
</main>
<SwitchProfile bind:open={showSwitchProfile} />

<style>
    @media (max-width: 768px) {
        :global(.mobile-content-wrapper h2.text-2xl) {
            display: none !important;
        }

        :global(.mobile-content-wrapper .mb-6) {
            margin-bottom: 0.5rem !important;
        }
    }
</style>