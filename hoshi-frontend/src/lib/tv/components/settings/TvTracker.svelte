<script lang="ts">
    import { onDestroy, onMount } from 'svelte';
    import { fade } from 'svelte/transition';
    import { toast } from 'svelte-sonner';
    import { onOpenUrl } from '@tauri-apps/plugin-deep-link';
    import { invoke } from '@tauri-apps/api/core';
    import { BarChart2, Calendar, Plus, Trash2, User } from 'lucide-svelte';

    import { integrationsApi } from '@/api/tracker/tracker.js';
    import type { TrackerInfo } from '@/api/tracker/types.js';
    import type { ListConfig, MergeStrategy } from '@/api/config/types.js';
    import { i18n } from '@/stores/i18n.svelte.js';
    import { Spinner } from '@/components/ui/spinner';
    import TvSettingRow from '@/tv/components/TvSettingRow.svelte';
    import TvSwitch from '@/tv/components/TvSwitch.svelte';
    import TvSegmented from '@/tv/components/TvSegmented.svelte';
    import TvDialog from '@/tv/components/TvDialog.svelte';

    let {
        config = $bindable(),
        onSave
    }: {
        config: ListConfig;
        onSave: () => Promise<void> | void;
    } = $props();

    let trackers = $state<TrackerInfo[]>([]);
    let loading = $state(true);

    let showAddTrackerDialog = $state(false);
    let newTrackerName = $state('');
    let newTrackerDisplayName = $state('');
    let newTrackerToken = $state('');
    let newTrackerUsername = $state('');
    let newTrackerPassword = $state('');
    let newTrackerAuth = $state<any>(null);
    let addingTracker = $state(false);

    // Two-step disconnect (press again within 3s) instead of a confirmation dialog.
    let confirmRemove = $state<string | null>(null);
    let removing = $state<string | null>(null);
    let confirmTimer: ReturnType<typeof setTimeout> | null = null;

    const mergeStrategies = $derived<{ value: MergeStrategy; label: string }[]>([
        { value: 'keepHighest', label: i18n.t('settings.trackers_section.conflict_score') },
        { value: 'keepLocal', label: i18n.t('settings.trackers_section.conflict_local') },
        { value: 'keepRemote', label: i18n.t('settings.trackers_section.conflict_remote') },
        { value: 'keepLatest', label: i18n.t('settings.trackers_section.conflict_updated') },
        { value: 'anilistFirst', label: i18n.t('settings.trackers_section.conflict_anilist') },
        { value: 'malFirst', label: i18n.t('settings.trackers_section.conflict_mal') },
        { value: 'kitsuFirst', label: i18n.t('settings.trackers_section.conflict_kitsu') },
        { value: 'simklFirst', label: i18n.t('settings.trackers_section.conflict_simkl') }
    ]);

    const inputClass =
        'h-12 w-full rounded-md border-2 border-border/60 bg-background px-4 text-lg outline-none ' +
        'focus:border-primary focus:ring-4 focus:ring-primary/40';
    const primaryBtn =
        'flex h-12 w-full items-center justify-center gap-2 rounded-lg bg-primary px-6 text-base font-bold text-primary-foreground outline-none ' +
        'focus:ring-4 focus:ring-primary/50 focus:ring-offset-2 focus:ring-offset-card disabled:opacity-50';
    const secondaryBtn =
        'flex h-12 w-full items-center justify-center rounded-lg border-2 border-border/60 px-6 text-base font-bold outline-none ' +
        'focus:border-primary focus:ring-4 focus:ring-primary/40';

    // Client IDs of the TV-specific clients, whose redirect URL is hoshi://auth (implicit grant).
    const TV_CLIENT_IDS: Record<string, string> = {
        anilist: '55029'
    };

    // Flows where the WebView catches the redirect itself: one login button, no token field.
    const autoLogin = $derived(
        newTrackerAuth?.oauthFlow === 'pkce' || !!TV_CLIENT_IDS[newTrackerName.toLowerCase()]
    );

    function generateVerifier() {
        const charset = 'ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-._~';
        let res = '';
        const randomValues = new Uint8Array(64);
        window.crypto.getRandomValues(randomValues);
        for (let i = 0; i < 64; i++) res += charset[randomValues[i] % charset.length];
        return res;
    }

    onMount(() => {
        loadTrackers();

        let unlistenAuth: (() => void) | undefined;
        onOpenUrl((urls) => {
            for (const url of urls) {
                if (url.startsWith('hoshi://auth')) {
                    const code = new URL(url).searchParams.get('code');
                    if (code) {
                        finalizeAuth(code);
                        break;
                    }
                }
            }
        }).then((un) => (unlistenAuth = un));

        return () => unlistenAuth?.();
    });

    onDestroy(() => {
        if (confirmTimer) clearTimeout(confirmTimer);
    });

    async function loadTrackers() {
        loading = true;
        try {
            const TRACKER_ORDER = ['anilist', 'mal', 'kitsu'];
            const all = (await integrationsApi.getAll()) || [];

            trackers = all
                .filter((t) => t.name.toLowerCase())
                .sort((a, b) => {
                    const ai = TRACKER_ORDER.indexOf(a.name.toLowerCase());
                    const bi = TRACKER_ORDER.indexOf(b.name.toLowerCase());
                    if (a.connected !== b.connected) return a.connected ? -1 : 1;
                    return (ai === -1 ? 999 : ai) - (bi === -1 ? 999 : bi);
                });
        } catch (error: any) {
            toast.error(i18n.t(error.key));
        } finally {
            loading = false;
        }
    }

    async function handleToggleSync(trackerName: string, enabled: boolean) {
        try {
            await integrationsApi.setSyncEnabled(trackerName, enabled);
            const index = trackers.findIndex((t) => t.name === trackerName);
            if (index !== -1) trackers[index].syncEnabled = enabled;
        } catch (error) {
            toast.error(i18n.t('errors.network'));
            await loadTrackers();
        }
    }

    function openAddTrackerDialog(tracker: TrackerInfo) {
        newTrackerName = tracker.name;
        newTrackerDisplayName = tracker.displayName;
        newTrackerAuth = tracker.auth;
        newTrackerToken = '';
        newTrackerUsername = '';
        newTrackerPassword = '';
        showAddTrackerDialog = true;
    }

    async function handleAuthStart() {
        if (newTrackerAuth?.oauthFlow === 'pkce') {
            const verifier = generateVerifier();
            localStorage.setItem(`${newTrackerName}_verifier`, verifier);

            const params = new URLSearchParams({
                client_id: newTrackerAuth.clientId,
                response_type: 'code',
                code_challenge: verifier,
                code_challenge_method: 'plain',
                redirect_uri: 'hoshi://auth'
            });
            const authUrl = `${newTrackerAuth.authUrl}?${params.toString()}`;

            // TV has no browser: log in inside the in-app WebView (AuthWebViewPlugin) and let it
            // catch the hoshi://auth redirect instead of relying on the deep link.
            let result: string | null;
            try {
                result = await invoke<string | null>('auth_webview_open', {
                    url: authUrl,
                    redirectPrefixes: ['hoshi://auth']
                });
            } catch (error: any) {
                toast.error(typeof error === 'string' ? error : i18n.t('errors.auth_error'));
                console.log(error);
                return;
            }

            if (!result) {
                localStorage.removeItem(`${newTrackerName}_verifier`);
                return;
            }

            const code = new URL(result).searchParams.get('code');
            if (code) await finalizeAuth(code);
            else toast.error(i18n.t('errors.auth_error'));
        } else {
            // Token flow (AniList, Simkl). With a TV client whose redirect is hoshi://auth the token
            // is caught automatically; without one the person copies it and pastes it in the field.
            const tvClientId = TV_CLIENT_IDS[newTrackerName.toLowerCase()];
            try {
                const result = await invoke<string | null>('auth_webview_open', {
                    url: `${newTrackerAuth.authUrl}?client_id=${tvClientId || newTrackerAuth.clientId}&response_type=token`,
                    redirectPrefixes: tvClientId ? ['hoshi://auth'] : []
                });

                if (tvClientId && result) {
                    // Implicit grant: hoshi://auth#access_token=...&token_type=Bearer&...
                    const token = new URLSearchParams(new URL(result).hash.slice(1)).get('access_token');
                    if (token) await finalizeAuth(token);
                    else toast.error(i18n.t('errors.auth_error'));
                }
            } catch (error: any) {
                toast.error(typeof error === 'string' ? error : i18n.t('errors.auth_error'));
                console.log(error);
            }
        }
    }

    async function finalizeAuth(code: string) {
        addingTracker = true;
        try {
            await integrationsApi.add({
                trackerName: newTrackerName,
                accessToken: code,
                codeVerifier: localStorage.getItem(`${newTrackerName}_verifier`) || undefined
            });
            localStorage.removeItem(`${newTrackerName}_verifier`);
            showAddTrackerDialog = false;
            await loadTrackers();
        } catch (error: any) {
            toast.error(typeof error === 'string' ? error : i18n.t('errors.auth_error'));
            console.log(error);
        } finally {
            addingTracker = false;
        }
    }

    async function handleAddTracker(e: Event) {
        e.preventDefault();
        const payload: any = { trackerName: newTrackerName };

        if (newTrackerAuth?.oauthFlow === 'password') {
            if (!newTrackerUsername || !newTrackerPassword) return;
            payload.username = newTrackerUsername;
            payload.password = newTrackerPassword;
        } else {
            if (!newTrackerToken) return;
            payload.accessToken = newTrackerToken;
        }

        addingTracker = true;
        try {
            await integrationsApi.add(payload);
            showAddTrackerDialog = false;
            await loadTrackers();
        } catch (error: any) {
            toast.error(typeof error === 'string' ? error : i18n.t('errors.connect_error'));
        } finally {
            addingTracker = false;
        }
    }

    async function onRemoveClick(name: string) {
        if (confirmRemove !== name) {
            confirmRemove = name;
            if (confirmTimer) clearTimeout(confirmTimer);
            confirmTimer = setTimeout(() => (confirmRemove = null), 3000);
            return;
        }
        if (confirmTimer) clearTimeout(confirmTimer);
        confirmRemove = null;
        removing = name;
        try {
            await integrationsApi.remove(name);
            await loadTrackers();
        } catch (error) {
            toast.error(i18n.t('errors.network'));
        } finally {
            removing = null;
        }
    }

    function formatDate(timestamp: number | null | undefined): string {
        if (!timestamp) return 'Never';
        return new Date(timestamp).toLocaleDateString(undefined, {
            month: 'short',
            day: 'numeric',
            hour: '2-digit',
            minute: '2-digit'
        });
    }
</script>

<div>
    <header class="space-y-1 pb-2">
        <h2 class="text-3xl font-black tracking-tight">{i18n.t('settings.trackers_section.trackers_title')}</h2>
        <p class="text-base text-muted-foreground">{i18n.t('settings.trackers_section.trackers_desc')}</p>
    </header>

    <TvSettingRow
            label={i18n.t('settings.trackers_section.sync_startup')}
            description={i18n.t('settings.trackers_section.sync_startup_desc')}
    >
        <TvSwitch
                label={i18n.t('settings.trackers_section.sync_startup')}
                bind:checked={config.syncOnStartup}
                onchange={() => onSave()}
        />
    </TvSettingRow>

    <TvSettingRow
            stacked
            label={i18n.t('settings.trackers_section.conflict_strategy')}
            description={i18n.t('settings.trackers_section.conflict_strategy_desc')}
    >
        <TvSegmented
                label={i18n.t('settings.trackers_section.conflict_strategy')}
                bind:value={config.mergeStrategy}
                options={mergeStrategies}
                onchange={() => onSave()}
        />
    </TvSettingRow>

    {#if loading}
        <div in:fade class="flex justify-center border-b border-border/40 py-12">
            <Spinner class="size-10 text-primary" />
        </div>
    {:else}
        <div in:fade class="divide-y divide-border/40 border-b border-border/40">
            {#each trackers as tracker (tracker.name)}
                <div class="flex items-center gap-5 py-5">
                    <div class="relative shrink-0">
                        <div class="flex size-16 items-center justify-center overflow-hidden rounded-md border border-border bg-muted/10 shadow-sm">
                            {#if tracker.connected && tracker.avatarUrl}
                                <img src={tracker.avatarUrl} alt={tracker.displayName_user || tracker.displayName} class="size-full object-cover" />
                            {:else}
                                <img src={tracker.iconUrl} alt={tracker.displayName} class="size-full bg-muted/20 object-contain p-2" />
                            {/if}
                        </div>
                        {#if tracker.connected && tracker.avatarUrl}
                            <div class="absolute -bottom-1 -right-1 flex size-6 items-center justify-center overflow-hidden rounded-md border border-border bg-background p-0.5 shadow-sm">
                                <img src={tracker.iconUrl} alt="" class="size-full object-contain" />
                            </div>
                        {/if}
                    </div>

                    <div class="min-w-0 flex-1 space-y-1">
                        <p class="truncate text-xl font-bold capitalize">
                            {tracker.connected && tracker.displayName_user ? tracker.displayName_user : tracker.displayName}
                        </p>

                        {#if tracker.connected}
                            <div class="flex flex-wrap items-center gap-x-4 gap-y-1 text-sm text-muted-foreground">
                                <span class="flex min-w-0 items-center gap-1.5">
                                    <User class="size-4 shrink-0" />
                                    <span class="truncate">{tracker.trackerUserId}</span>
                                </span>
                                {#if tracker.totalEntries !== null && tracker.totalEntries !== undefined}
                                    <span class="flex items-center gap-1.5">
                                        <BarChart2 class="size-4" />
                                        <strong>{tracker.totalEntries}</strong>
                                    </span>
                                {/if}
                                {#if tracker.lastSyncedAt}
                                    <span class="flex items-center gap-1.5">
                                        <Calendar class="size-4" />
                                        Synced: {formatDate(tracker.lastSyncedAt)}
                                    </span>
                                {/if}
                            </div>
                        {/if}
                    </div>

                    <div class="flex shrink-0 items-center gap-4">
                        {#if tracker.connected}
                            {#if tracker.syncEnabled !== null}
                                <div class="flex items-center gap-3 text-sm font-semibold">
                                    <span>Sync</span>
                                    <TvSwitch
                                            label={`Sync ${tracker.displayName}`}
                                            checked={tracker.syncEnabled}
                                            onchange={(v) => handleToggleSync(tracker.name, v)}
                                    />
                                </div>
                            {/if}

                            <button
                                    type="button"
                                    data-sn
                                    disabled={removing === tracker.name}
                                    onclick={() => onRemoveClick(tracker.name)}
                                    aria-label={i18n.t('settings.trackers_section.disconnect')}
                                    class="flex h-12 scroll-my-24 items-center gap-2 rounded-lg border-2 px-4 text-base font-bold outline-none
                                           focus:ring-4 focus:ring-destructive/50 focus:ring-offset-2 focus:ring-offset-background disabled:opacity-50
                                           {confirmRemove === tracker.name
                                        ? 'border-destructive bg-destructive text-destructive-foreground'
                                        : 'border-border/60 text-destructive'}"
                            >
                                <Trash2 class="size-5" />
                                {#if confirmRemove === tracker.name}
                                    {i18n.t('settings.trackers_section.disconnect')}
                                {/if}
                            </button>
                        {:else}
                            <button
                                    type="button"
                                    data-sn
                                    onclick={() => openAddTrackerDialog(tracker)}
                                    class="flex h-12 scroll-my-24 items-center gap-2 rounded-lg border-2 border-border/60 px-5 text-base font-bold outline-none
                                           focus:border-primary focus:ring-4 focus:ring-primary/40"
                            >
                                <Plus class="size-5" />
                                {i18n.t('settings.trackers_section.connect')}
                            </button>
                        {/if}
                    </div>
                </div>
            {/each}
        </div>
    {/if}
</div>

{#if showAddTrackerDialog}
    <TvDialog
            title="{i18n.t('settings.trackers_section.connect')} {newTrackerDisplayName}"
            description={i18n.t('settings.trackers_section.connect_tracker_desc', { name: newTrackerDisplayName })}
            onclose={() => (showAddTrackerDialog = false)}
    >
        {#if autoLogin}
            <div class="space-y-4">
                <p class="text-center text-sm text-muted-foreground">
                    {i18n.t('settings.trackers_section.pkce_redirect_notice')}
                </p>
                <button type="button" data-sn onclick={handleAuthStart} disabled={addingTracker} class={primaryBtn}>
                    {#if addingTracker}<Spinner class="size-4" />{/if}
                    {i18n.t('settings.trackers_section.login_to_service', { name: newTrackerDisplayName })}
                </button>
            </div>
        {:else if newTrackerAuth?.oauthFlow === 'password'}
            <form onsubmit={handleAddTracker} class="space-y-4">
                <div class="space-y-2">
                    <label for="tv-tracker-user" class="text-base font-bold">{i18n.t('settings.trackers_section.email_or_username')}</label>
                    <input id="tv-tracker-user" data-sn type="text" bind:value={newTrackerUsername} required class={inputClass} />
                </div>
                <div class="space-y-2">
                    <label for="tv-tracker-pass" class="text-base font-bold">{i18n.t('settings.account_section.new_password')}</label>
                    <input id="tv-tracker-pass" data-sn type="password" autocomplete="off" bind:value={newTrackerPassword} required class={inputClass} />
                </div>
                <button type="submit" data-sn disabled={addingTracker} class={primaryBtn}>
                    {#if addingTracker}<Spinner class="size-4" />{/if}
                    {i18n.t('settings.trackers_section.connect_tracker')}
                </button>
            </form>
        {:else}
            <form onsubmit={handleAddTracker} class="space-y-4">
                <div class="space-y-2">
                    <label for="tv-tracker-token" class="text-base font-bold">{i18n.t('settings.trackers_section.token')}</label>
                    <input
                            id="tv-tracker-token"
                            data-sn
                            type="password"
                            autocomplete="off"
                            placeholder={i18n.t('settings.trackers_section.paste_token')}
                            bind:value={newTrackerToken}
                            required
                            class={inputClass}
                    />
                </div>
                <button type="button" data-sn onclick={handleAuthStart} class={secondaryBtn}>
                    {i18n.t('settings.trackers_section.get_token', { name: newTrackerDisplayName })}
                </button>
                <button type="submit" data-sn disabled={addingTracker} class={primaryBtn}>
                    {#if addingTracker}<Spinner class="size-4" />{/if}
                    {i18n.t('settings.trackers_section.connect_tracker')}
                </button>
            </form>
        {/if}

        <button type="button" data-sn onclick={() => (showAddTrackerDialog = false)} class={secondaryBtn}>
            {i18n.t('content.cancel')}
        </button>
    </TvDialog>
{/if}