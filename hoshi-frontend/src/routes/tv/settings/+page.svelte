<script lang="ts">
    import { onMount } from 'svelte';
    import { fade } from 'svelte/transition';
    import { toast } from 'svelte-sonner';
    import {Globe, LayoutTemplate, Link2, MonitorPlay, Puzzle, Settings, User} from 'lucide-svelte';
    import { getVersion } from '@tauri-apps/api/app';

    import { auth } from '@/stores/auth.svelte.js';
    import { appConfig } from '@/stores/config.svelte.js';
    import { i18n } from '@/stores/i18n.svelte.js';
    import { Spinner } from '$lib/components/ui/spinner';
    import TvAccount from '@/tv/components/settings/TvAccount.svelte';
    import TvGeneral from '@/tv/components/settings/TvGeneral.svelte';
    import TvUi from '@/tv/components/settings/TvUi.svelte';
    import TvPlayer from '@/tv/components/settings/TvPlayer.svelte';
    import TvTracker from "@/tv/components/settings/TvTracker.svelte";
    import TvTorrent from "@/tv/components/settings/TvTorrent.svelte";
    import SwitchProfile from "@/components/modals/SwitchProfile.svelte";
    import Account from "@/components/settings/Account.svelte";
    import TvExtensions from "@/tv/components/settings/TvExtensions.svelte";

    type SectionId = 'account' | 'general' | 'ui' | 'player' | 'tracking' | 'torrent' | 'extensions';

    const sections = $derived<{ id: SectionId; label: string; icon: typeof User }[]>([
        { id: 'account', label: i18n.t('settings.account'), icon: User },
        { id: 'general', label: i18n.t('settings.general'), icon: Settings },
        { id: 'ui', label: i18n.t('settings.interface'), icon: LayoutTemplate },
        { id: 'player', label: i18n.t('settings.player'), icon: MonitorPlay },
        { id: 'tracking', label: i18n.t('settings.tracking'), icon: Link2 },
        { id: 'extensions', label: i18n.t('settings.extensions'), icon: Puzzle },
        { id: 'torrent', label: i18n.t('settings.torrent'), icon: Globe }
    ]);

    let active = $state<SectionId>('account');
    let navEl = $state<HTMLElement | null>(null);
    let contentEl = $state<HTMLElement | null>(null);
    let appVersion = $state('');
    let showSwitchProfile = $state(false);

    onMount(() => {
        getVersion()
            .then((v) => (appVersion = v))
            .catch(() => {});
    });

    $effect(() => {
        void active;
        contentEl?.closest('main')?.scrollTo({ top: 0 });
    });

    function focusNavOnMount(node: HTMLElement) {
        const a = document.activeElement;
        if (!a || a === document.body || a.closest('[data-tv-sidebar]')) {
            node.querySelector<HTMLElement>('[data-sn]')?.focus();
        }
    }

    function enterContent() {
        contentEl?.querySelector<HTMLElement>('[data-sn]')?.focus();
    }

    // Back inside a section goes to this page's menu first; Back again reaches the app sidebar.
    function onContentKeydown(e: KeyboardEvent) {
        const isBack =
            e.key === 'Escape' || e.key === 'GoBack' || e.key === 'BrowserBack' || e.keyCode === 4;
        if (!isBack) return;
        e.preventDefault();
        e.stopPropagation();
        navEl?.querySelector<HTMLElement>('[data-sn][aria-current="true"]')?.focus();
    }

    async function handleSaveConfig() {
        if (!appConfig.data) return;
        try {
            await appConfig.update(appConfig.data);
        } catch (err) {
            console.error(err);
            toast.error('Failed to update preferences');
        }
    }
</script>

<svelte:head>
    <title>{i18n.t('settings.title')}</title>
</svelte:head>

<div class="pb-16">
    <h1 class="mb-8 text-4xl font-black tracking-tight">{i18n.t('settings.title')}</h1>

    {#if !auth.user || !appConfig.data}
        <div class="flex h-[50vh] flex-col items-center justify-center gap-4 text-muted-foreground" in:fade={{ duration: 200 }}>
            <Spinner class="size-12 animate-spin text-primary" />
            <p class="text-lg font-bold">{i18n.t('settings.loading')}</p>
        </div>
    {:else}
        <div class="flex items-start gap-8" in:fade={{ duration: 200 }}>
            <!-- Section menu. Focusing an entry previews it; OK moves into its controls. -->
            <nav bind:this={navEl} use:focusNavOnMount class="sticky top-8 flex w-52 shrink-0 flex-col gap-2 self-start">
                {#each sections as section (section.id)}
                    <button
                            type="button"
                            data-sn
                            aria-current={active === section.id}
                            onfocus={() => (active = section.id)}
                            onclick={enterContent}
                            class="flex h-14 items-center gap-3 rounded-lg px-4 text-lg font-bold outline-none transition-colors
                                   focus:bg-primary focus:text-primary-foreground focus:ring-4 focus:ring-primary/40
                                   {active === section.id ? 'bg-primary/15 text-foreground' : 'text-muted-foreground'}"
                    >
                        <section.icon class="size-5 shrink-0" />
                        {section.label}
                    </button>
                {/each}

                {#if appVersion}
                    <p class="px-4 pt-6 text-xs font-bold uppercase tracking-wider text-muted-foreground/50">
                        Version {appVersion}
                    </p>
                {/if}
            </nav>

            <!-- svelte-ignore a11y_no_static_element_interactions -->
            <div bind:this={contentEl} onkeydown={onContentKeydown} data-sn-rows class="min-w-0 flex-1">
                {#key active}
                    <div in:fade={{ duration: 120 }}>
                        {#if active === 'account' && auth.user}
                            <TvAccount user={auth.user} onUpdate={() => auth.restore(true)} onDeleted={() => showSwitchProfile = true}/>
                        {:else if active === 'general'}
                            <TvGeneral
                                    bind:config={appConfig.data.general}
                                    bind:contentConfig={appConfig.data.content}
                                    onSave={handleSaveConfig}
                            />
                        {:else if active === 'ui'}
                            <TvUi bind:config={appConfig.data.ui} onSave={handleSaveConfig} />
                        {:else if active === 'player'}
                            <TvPlayer
                                    bind:playerConfig={appConfig.data.player}
                                    bind:subConfig={appConfig.data.subtitles}
                                    onSave={handleSaveConfig}
                            />
                        {:else if active === 'tracking'}
                            <TvTracker bind:config={appConfig.data.list} onSave={handleSaveConfig} />
                        {:else if active === 'extensions'}
                            <TvExtensions bind:config={appConfig.data.extensions} onSave={handleSaveConfig} />
                        {:else if active === 'torrent'}
                            <TvTorrent bind:torrentConfig={appConfig.data.torrent} onSave={handleSaveConfig} />
                        {/if}
                    </div>
                {/key}
            </div>
        </div>
    {/if}
</div>
<SwitchProfile bind:open={showSwitchProfile} />