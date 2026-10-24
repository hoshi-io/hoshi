<script lang="ts">
    import { auth } from "@/stores/auth.svelte.js";
    import { toast } from "svelte-sonner";
    import { crossfade, fade, fly } from "svelte/transition";
    import { cubicOut } from "svelte/easing";
    import { goto } from '$app/navigation';
    import {
        User, Link2, Settings, MonitorPlay, Puzzle, BookOpen, LayoutTemplate, Database,
        MessageSquare, ChevronRight, Terminal, Globe
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
    import Torrent from "@/components/settings/Torrent.svelte";
    import PageHeader from "@/components/PageHeader.svelte";

    let showSwitchProfile = $state(false);
    let isDesktop = $state(false);
    let activeTab = $state(page.url.searchParams.get('tab') || 'account');
    let isMobileDetail = $derived(page.url.searchParams.has('tab'));

    let appVersion = $state("");
    let osInfo = $state("");
    let hardwareInfo = $state("");
    let webviewInfo = $state("");

    const prefersReducedMotion =
        typeof matchMedia !== 'undefined' && matchMedia('(prefers-reduced-motion: reduce)').matches;

    const [send, receive] = crossfade({
        duration: prefersReducedMotion ? 0 : 280,
        easing: cubicOut,
        fallback: (node) => fade(node, { duration: 300 })
    });

    type NavItem = { id: string; label: string; icon: typeof User; hover: string; desktopOnly?: boolean };
    type NavGroup = { title: string; desktopOnly?: boolean; items: NavItem[] };

    const navGroups: NavGroup[] = $derived([
        {
            title: i18n.t('settings.section_profile'),
            desktopOnly: true,
            items: [
                { id: 'account', label: i18n.t('settings.account'), icon: User, hover: 'group-hover:scale-105' }
            ]
        },
        {
            title: i18n.t('settings.section_application'),
            items: [
                { id: 'general', label: i18n.t('settings.general'), icon: Settings, hover: 'group-hover:rotate-45' },
                { id: 'ui', label: i18n.t('settings.interface'), icon: LayoutTemplate, hover: 'group-hover:scale-105' },
                { id: 'logs', label: i18n.t('settings.logs.title'), icon: Terminal, hover: 'group-hover:translate-x-0.5' }
            ]
        },
        {
            title: i18n.t('settings.section_experience'),
            items: [
                { id: 'player', label: i18n.t('settings.player'), icon: MonitorPlay, hover: 'group-hover:scale-105' },
                { id: 'readers', label: i18n.t('settings.readers'), icon: BookOpen, hover: 'group-hover:scale-105' }
            ]
        },
        {
            title: i18n.t('settings.section_integrations'),
            items: [
                { id: 'extensions', label: i18n.t('settings.extensions'), icon: Puzzle, hover: 'group-hover:rotate-12' },
                { id: 'tracking', label: i18n.t('settings.tracking'), icon: Link2, hover: 'group-hover:rotate-45' },
                { id: 'torrent', label: i18n.t('settings.torrent'), icon: Globe, hover: 'group-hover:rotate-45' },
                { id: 'discord', label: i18n.t('settings.discord'), icon: MessageSquare, hover: 'group-hover:scale-105', desktopOnly: true }
            ]
        }
    ]);

    const visibleGroups = $derived.by(() => {
        let n = 0;
        return navGroups
            .filter((g) => !g.desktopOnly || isDesktop)
            .map((g) => ({
                ...g,
                items: g.items
                    .filter((i) => !i.desktopOnly || isDesktop)
                    .map((i) => ({ ...i, n: n++ }))
            }))
            .filter((g) => g.items.length > 0);
    });

    const triggerClass =
        "group relative w-full flex items-center justify-between rounded-md md:rounded-sm px-3 py-3 md:py-2.5 " +
        "text-base md:text-[15px] font-semibold text-foreground/75 transition-colors duration-200 " +
        "hover:text-foreground data-[state=active]:md:text-foreground " +
        "data-[state=inactive]:md:hover:bg-muted/40 data-[state=inactive]:active:bg-muted/40 " +
        "focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-primary/40";

    const chipClass =
        "grid shrink-0 place-items-center size-9 md:size-8 rounded-lg bg-muted/60 md:bg-transparent text-muted-foreground " +
        "transition-colors duration-200 " +
        "group-hover:text-foreground " +
        "group-data-[state=active]:md:bg-transparent group-data-[state=active]:md:text-foreground";

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
                    torrent: i18n.t('settings.torrent'),
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
        try {
            await appConfig.update(appConfig.data);
        } catch (err) {
            console.error(err);
            toast.error("Failed to update preferences");
        }
    }
</script>

<svelte:head>
    <title>{i18n.t('settings.title')}</title>
</svelte:head>

<main class="bg-background pb-6 md:pb-12 {isMobileDetail ? 'pt-[max(1.25rem,calc(var(--mobile-top-h,6rem)_-_6rem_+_0.75rem))] md:pt-12' : 'pt-[max(1rem,calc(var(--mobile-top-h,6rem)_-_6rem_+_0.75rem))] md:pt-20'} px-4 md:px-8 lg:pl-32 lg:pr-12 w-full max-w-[2000px] mx-auto {isMobileDetail ? 'space-y-0 md:space-y-8' : 'space-y-4 md:space-y-8'}">

    <PageHeader title={i18n.t('settings.title')} />

    {#if !isMobileDetail}
        <button
                class="md:hidden flex w-full items-center gap-4 rounded-sm border border-border/30 bg-muted/20 p-4 text-left transition-all duration-200 hover:bg-muted/30 active:scale-[0.99]"
                onclick={() => goto('?tab=account')}
        >
            <Avatar.Root class="size-14 ring-2 ring-primary/20 ring-offset-2 ring-offset-background">
                {#if auth.user?.avatar}<Avatar.Image src={auth.user.avatar} alt={auth.user.username} class="object-cover" />{/if}
                <Avatar.Fallback class="bg-primary/10 text-primary font-semibold">
                    {auth.user?.username?.charAt(0)?.toUpperCase() || 'U'}
                </Avatar.Fallback>
            </Avatar.Root>
            <div class="min-w-0 flex-1">
                <p class="truncate font-semibold">{auth.user?.username}</p>
                <p class="text-sm text-muted-foreground">{i18n.t('settings.account')}</p>
            </div>
            <ChevronRight class="size-5 text-muted-foreground/50" />
        </button>
    {/if}

    <section class="w-full">
        {#if !auth.user || !appConfig.data}
            <div in:fade={{ duration: 200 }} class="h-[50vh] flex flex-col items-center justify-center gap-4 text-muted-foreground">
                <Spinner class="h-10 w-10 animate-spin text-primary" />
                <p class="text-sm font-bold animate-pulse">{i18n.t('settings.loading')}</p>
            </div>
        {:else}
            <div in:fade={{ duration: 300 }} class="w-full">
                <Tabs.Root
                        value={activeTab}
                        onValueChange={(v) => {
                        activeTab = v;
                        if (!isDesktop) goto(`?tab=${v}`);
                        document.querySelector('main')?.scrollTo({ top: 0, behavior: 'smooth' });
                    }}
                        class="flex flex-col md:flex-row gap-8 lg:gap-12 w-full items-start"
                >
                    <Tabs.List class="{isMobileDetail ? 'hidden md:flex' : 'flex'} flex-col justify-start bg-transparent h-auto p-0 gap-1 w-full md:w-72 shrink-0 md:sticky md:top-24 md:max-h-[calc(100vh-8rem)] md:overflow-y-auto md:hide-scrollbar">
                        {#each visibleGroups as group, gi (group.title)}
                            <div class="px-3 {gi === 0 ? 'pt-1' : 'pt-6'} pb-2 text-[11px] font-bold uppercase tracking-[0.14em] text-muted-foreground/70 text-left w-full">
                                {group.title}
                            </div>

                            <!-- Mobile: each group is an inset card. Desktop: items sit flat in the sidebar. -->
                            <div class="w-full flex flex-col gap-0 md:gap-0.5 rounded-xl border border-border/40 bg-muted/20 p-1 md:contents">
                                {#each group.items as item (item.id)}
                                    <div class="nav-item w-full" style="--i: {item.n}">
                                        <Tabs.Trigger value={item.id} class={triggerClass}>
                                            {#if activeTab === item.id}
                                                <span
                                                        aria-hidden="true"
                                                        class="absolute inset-0 hidden md:block rounded-sm bg-primary/10 ring-1 ring-inset ring-primary/15"
                                                        in:receive={{ key: 'settings-nav' }}
                                                        out:send={{ key: 'settings-nav' }}
                                                ></span>
                                            {/if}

                                            <div class="relative flex items-center gap-3.5">
                                                <span class={chipClass}>
                                                    <item.icon class="size-[1.1rem] md:size-4 transition-transform duration-300 {item.hover}" />
                                                </span>
                                                {item.label}
                                            </div>
                                            <ChevronRight class="relative size-4 md:hidden text-muted-foreground/50 transition-transform group-hover:translate-x-0.5" />
                                        </Tabs.Trigger>
                                    </div>
                                {/each}
                            </div>
                        {/each}

                        <div class="mt-8 md:mt-auto pt-4 pb-4 px-3 w-full flex flex-col items-center md:items-start gap-4 text-center md:text-left">
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
                            <div class="w-25 h-25 flex items-center justify-center">
                                <img
                                        src="/chibi.png"
                                        alt="App Mascot"
                                        class="max-w-full max-h-full object-scale-down [image-rendering:-webkit-optimize-contrast] hover:opacity-100 hover:-scale-x-105 hover:scale-y-105 transition-all duration-300 drop-shadow-sm -scale-x-100"
                                />
                            </div>
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
                                                bind:subConfig={appConfig.data.subtitles}
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

                            <Tabs.Content value="torrent" class="focus-visible:outline-none mt-0 w-full">
                                {#if activeTab === 'torrent'}
                                    <div in:fly={{ y: 8, duration: 250, easing: cubicOut }}>
                                        <Torrent bind:torrentConfig={appConfig.data.torrent} onSave={handleSaveConfig}/>
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
    .nav-item {
        animation: navIn 380ms cubic-bezier(0.22, 1, 0.36, 1) backwards;
        animation-delay: calc(var(--i) * 28ms);
    }

    @keyframes navIn {
        from { opacity: 0; transform: translateX(-8px); }
    }

    @media (prefers-reduced-motion: reduce) {
        .nav-item { animation: none; }
    }

    @media (max-width: 768px) {
        :global(.mobile-content-wrapper h2.text-2xl) {
            display: none !important;
        }

        :global(.mobile-content-wrapper .mb-6) {
            margin-bottom: 0.5rem !important;
        }
    }
</style>