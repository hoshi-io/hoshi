<script lang="ts">
    import { onMount } from "svelte";
    import { integrationsApi } from "@/api/tracker/tracker";
    import type { TrackerInfo } from "@/api/tracker/types";
    import { toast } from "svelte-sonner";
    import { fade } from "svelte/transition";
    import { i18n } from '@/stores/i18n.svelte.js';
    import { onOpenUrl } from "@tauri-apps/plugin-deep-link";
    import { openUrl } from "@tauri-apps/plugin-opener";
    import { Trash2, Plus, AlertTriangle, ExternalLink, User, Settings2, BarChart2, Calendar } from "lucide-svelte";
    import * as AlertDialog from "$lib/components/ui/alert-dialog";
    import * as Dialog from "$lib/components/ui/dialog";
    import { Button } from "$lib/components/ui/button";
    import { Input } from "$lib/components/ui/input";
    import { Label } from "$lib/components/ui/label";
    import { Switch } from "$lib/components/ui/switch";
    import { Spinner } from "$lib/components/ui/spinner";
    import type { ListConfig, MergeStrategy } from "@/api/config/types";
    import ResponsiveSelect from "@/components/ResponsiveSelect.svelte";

    let {
        config = $bindable(),
        onSave
    }: {
        config: ListConfig,
        onSave: () => Promise<void> | void
    } = $props();

    let trackers = $state<TrackerInfo[]>([]);
    let loading = $state(true);
    let showRemoveTrackerAlert = $state(false);
    let trackerToRemove = $state<string | null>(null);
    let removingTracker = $state(false);

    let showAddTrackerDialog = $state(false);
    let newTrackerName = $state("");
    let newTrackerDisplayName = $state("");
    let newTrackerToken = $state("");

    let newTrackerUsername = $state("");
    let newTrackerPassword = $state("");
    let newTrackerAuth = $state<any>(null);
    let addingTracker = $state(false);

    const mergeStrategies: { value: MergeStrategy; label: string }[] = [
        { value: 'keepHighest', label: i18n.t("settings.trackers_section.conflict_score") },
        { value: 'keepLocal', label: i18n.t("settings.trackers_section.conflict_local") },
        { value: 'keepRemote', label: i18n.t("settings.trackers_section.conflict_remote") },
        { value: 'keepLatest', label: i18n.t("settings.trackers_section.conflict_updated") },
        { value: 'anilistFirst', label: i18n.t("settings.trackers_section.conflict_anilist") },
        { value: 'malFirst', label: i18n.t("settings.trackers_section.conflict_mal") },
        { value: 'kitsuFirst', label: i18n.t("settings.trackers_section.conflict_kitsu") },
        { value: 'simklFirst', label: i18n.t("settings.trackers_section.conflict_simkl") }
    ];

    function generateVerifier() {
        const charset = 'ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-._~';
        let res = '';
        const randomValues = new Uint8Array(64);
        window.crypto.getRandomValues(randomValues);
        for (let i = 0; i < 64; i++) {
            res += charset[randomValues[i] % charset.length];
        }
        return res;
    }

    onMount(() => {
        loadTrackers();

        let unlistenAuth: (() => void) | undefined;

        const setupAuthListener = async () => {
            unlistenAuth = await onOpenUrl((urls) => {
                console.log("Deep links recibidos:", urls);

                for (const url of urls) {
                    if (url.startsWith("hoshi://auth")) {
                        const code = new URL(url).searchParams.get("code");
                        if (code) {
                            finalizeAuth(code);
                            break;
                        }
                    }
                }
            });
        };

        setupAuthListener();

        return () => {
            if (unlistenAuth) unlistenAuth();
        };
    });

    async function loadTrackers() {
        loading = true;
        try {
            const TRACKER_ORDER = ['anilist', 'mal', 'kitsu'];
            const allTrackers = await integrationsApi.getAll() || [];

            trackers = allTrackers
                .filter(t => t.name.toLowerCase())
                .sort((a, b) => {
                    const ai = TRACKER_ORDER.indexOf(a.name.toLowerCase());
                    const bi = TRACKER_ORDER.indexOf(b.name.toLowerCase());
                    const an = ai === -1 ? 999 : ai;
                    const bn = bi === -1 ? 999 : bi;
                    if (a.connected !== b.connected) return a.connected ? -1 : 1;
                    return an - bn;
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
            const index = trackers.findIndex(t => t.name === trackerName);
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
        newTrackerToken = "";
        newTrackerUsername = "";
        newTrackerPassword = "";
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

            const url = `${newTrackerAuth.authUrl}?${params.toString()}`;
            await openUrl(url);
        } else {
            const url = `${newTrackerAuth.authUrl}?client_id=${newTrackerAuth.clientId}&response_type=token`;
            await openUrl(url);
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
        let payload: any = { trackerName: newTrackerName };

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

    async function handleRemoveTracker() {
        if (!trackerToRemove) return;
        removingTracker = true;
        try {
            await integrationsApi.remove(trackerToRemove);
            await loadTrackers();
        } catch (error) {
            toast.error(i18n.t('errors.network'));
        } finally {
            removingTracker = false;
            showRemoveTrackerAlert = false;
            trackerToRemove = null;
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

<section class="space-y-2 w-full">
    <div class="mb-6">
        <h2 class="text-2xl font-bold tracking-tight">{i18n.t('settings.trackers_section.trackers_title')}</h2>
        <p class="text-sm text-muted-foreground mt-1">{i18n.t('settings.trackers_section.trackers_desc')}</p>
    </div>

    <div class="w-full">
        <div class="flex flex-col sm:flex-row sm:items-center justify-between gap-4 py-6 border-b border-border/40">
            <div class="space-y-1 pr-4">
                <Label class="text-base font-bold cursor-pointer" for="syncOnStartup">{i18n.t("settings.trackers_section.sync_startup")}</Label>
                <p class="text-sm text-muted-foreground">{i18n.t("settings.trackers_section.sync_startup_desc")}</p>
            </div>
            <Switch id="syncOnStartup" bind:checked={config.syncOnStartup} onCheckedChange={onSave} class="shrink-0 scale-125" />
        </div>

        <!--
        <div class="flex flex-col sm:flex-row sm:items-center justify-between gap-4 py-6 border-b border-border/40">
            <div class="space-y-1 pr-4">
                <Label class="text-base font-bold cursor-pointer" for="privateByDefault">{i18n.t("settings.trackers_section.private")}</Label>
                <p class="text-sm text-muted-foreground">{i18n.t("settings.trackers_section.private_desc")}</p>
            </div>
            <Switch id="privateByDefault" bind:checked={config.privateByDefault} onCheckedChange={onSave} class="shrink-0 scale-125" />
        </div>


        <div class="flex flex-col sm:flex-row sm:items-center justify-between gap-4 py-6 border-b border-border/40">
            <div class="space-y-1 pr-4 flex-1">
                <Label class="text-base font-bold" for="syncIntervalSeconds">{i18n.t("settings.trackers_section.sync_interval")}</Label>
                <p class="text-sm text-muted-foreground">{i18n.t("settings.trackers_section.sync_interval_desc")}</p>
            </div>
            <div class="w-full sm:max-w-[200px]">
                <Input
                        id="syncIntervalSeconds"
                        type="number"
                        min="60"
                        bind:value={config.syncIntervalSeconds}
                        onchange={onSave}
                        class="rounded-sm h-11 text-right w-full"
                />
            </div>
        </div>
                -->

        <div class="flex flex-col sm:flex-row sm:items-center justify-between gap-4 py-6">
            <div class="space-y-1 pr-4 flex-1">
                <Label class="text-base font-bold" for="mergeStrategy">{i18n.t("settings.trackers_section.conflict_strategy")}</Label>
                <p class="text-sm text-muted-foreground">{i18n.t("settings.trackers_section.conflict_strategy_desc")}</p>
            </div>
            <div class="w-full sm:max-w-[240px]">
                <ResponsiveSelect
                        items={mergeStrategies}
                        bind:value={config.mergeStrategy}
                        onValueChange={onSave}
                        placeholder="Select strategy..."
                        class="h-11 rounded-sm bg-muted/20 border-transparent hover:bg-muted/30 font-medium text-sm text-foreground transition-colors"
                />
            </div>
        </div>
    </div>
    {#if loading}
        <div in:fade class="flex justify-center py-12 text-muted-foreground border-b border-border/40">
            <Spinner class="h-8 w-8 text-primary" />
        </div>
    {:else}
        <div in:fade class="border-b border-border/40 divide-y divide-border/40">
            {#each trackers as tracker}
                <div class="py-6 space-y-4">
                    <div class="flex items-start justify-between gap-4">
                        <div class="flex items-start gap-4 min-w-0">
                            <div class="relative shrink-0">
                                <div class="h-14 w-14 border border-border shadow-sm rounded-sm overflow-hidden bg-muted/10 flex items-center justify-center">
                                    {#if tracker.connected && tracker.avatarUrl}
                                        <img src={tracker.avatarUrl} alt={tracker.displayName_user || tracker.displayName} class="h-full w-full object-cover" />
                                    {:else}
                                        <img src={tracker.iconUrl} alt={tracker.displayName} class="h-full w-full object-contain p-2 bg-muted/20" />
                                    {/if}
                                </div>

                                {#if tracker.connected && tracker.avatarUrl}
                                    <div class="absolute -bottom-1 -right-1 h-5 w-5 rounded-md bg-background border border-border p-0.5 shadow-sm overflow-hidden flex items-center justify-center">
                                        <img src={tracker.iconUrl} alt="" class="h-full w-full object-contain" />
                                    </div>
                                {/if}
                            </div>

                            <div class="min-w-0 space-y-1">
                                <div class="flex items-center gap-2 flex-wrap">
                                    <Label class="text-base font-bold capitalize text-foreground">
                                        {tracker.connected && tracker.displayName_user ? tracker.displayName_user : tracker.displayName}
                                    </Label>
                                </div>

                                <div class="flex flex-wrap items-center gap-x-4 gap-y-1 text-xs text-muted-foreground">
                                    {#if tracker.connected}
                            <span class="flex items-center gap-1 min-w-0 truncate">
                                <User class="h-3.5 w-3.5 shrink-0 text-muted-foreground/70" />
                                <span class="truncate">{tracker.trackerUserId}</span>
                            </span>

                                        {#if tracker.totalEntries !== null && tracker.totalEntries !== undefined}
                                <span class="flex items-center gap-1 shrink-0">
                                    <BarChart2 class="h-3.5 w-3.5 text-muted-foreground/70" />
                                    <strong>{tracker.totalEntries}</strong>
                                </span>
                                        {/if}

                                        {#if tracker.lastSyncedAt}
                                <span class="flex items-center gap-1 shrink-0">
                                    <Calendar class="h-3.5 w-3.5 text-muted-foreground/70" />
                                    Synced: {formatDate(tracker.lastSyncedAt)}
                                </span>
                                        {/if}

                                        {#if tracker.syncEnabled !== null}
                                <span class="inline-flex items-center gap-1.5 ml-1 px-2 py-0.5 rounded-lg text-[11px] text-foreground font-medium">
                                    <Settings2 class="h-3 w-3 text-muted-foreground/80" />
                                    <span>Sync</span>
                                    <Switch
                                            id={`sync-${tracker.name}`}
                                            checked={tracker.syncEnabled}
                                            onCheckedChange={(v) => handleToggleSync(tracker.name, v)}
                                            class="scale-125 origin-center h-4 w-7"
                                    />
                                </span>
                                        {/if}
                                    {/if}
                                </div>

                                {#if tracker.supportedTypes && tracker.supportedTypes.length > 0}
                                    <div class="flex items-center gap-1.5 pt-0.5 flex-wrap">
                                        {#each tracker.supportedTypes as type}
                                <span class="text-[10px] tracking-wide font-bold uppercase bg-muted/40 text-muted-foreground px-1.5 py-0.5 rounded-md border border-border/30">
                                    {type}
                                </span>
                                        {/each}
                                    </div>
                                {/if}
                            </div>
                        </div>

                        <div class="shrink-0 flex items-center gap-2">
                            {#if tracker.connected}
                                {#if tracker.profileUrl}
                                    <Button variant="ghost" size="icon" class="text-muted-foreground hover:text-foreground rounded-sm h-10 w-10 border border-border/40 bg-muted/5"
                                            onclick={() => openUrl(tracker.profileUrl!)} title="View Profile">
                                        <ExternalLink class="h-4 w-4" />
                                    </Button>
                                {/if}
                                <Button variant="ghost" size="icon" class="text-muted-foreground hover:text-destructive rounded-sm h-10 w-10 border border-border/40 bg-muted/5"
                                        onclick={() => { trackerToRemove = tracker.name; showRemoveTrackerAlert = true; }}>
                                    <Trash2 class="h-4 w-4" />
                                </Button>
                            {:else}
                                <Button variant="outline" class="rounded-sm h-10 font-bold shadow-sm text-sm px-4"
                                        onclick={() => openAddTrackerDialog(tracker)}>
                                    <Plus class="h-4 w-4 mr-1.5" />
                                    {i18n.t('settings.trackers_section.connect')}
                                </Button>
                            {/if}
                        </div>
                    </div>
                </div>
            {/each}
        </div>
    {/if}
</section>

<Dialog.Root bind:open={showAddTrackerDialog}>
    <Dialog.Content class="sm:max-w-md sm:rounded-sm">
        <Dialog.Header>
            <Dialog.Title class="capitalize text-xl font-bold">{i18n.t('settings.trackers_section.connect')} {newTrackerDisplayName}</Dialog.Title>
            <Dialog.Description class="text-base">
                {i18n.t('settings.trackers_section.connect_tracker_desc', { name: newTrackerDisplayName })}
            </Dialog.Description>
        </Dialog.Header>

        <div class="py-4">
            {#if newTrackerAuth?.oauthFlow === 'pkce'}
                <div class="flex flex-col items-center space-y-4">
                    <p class="text-sm text-center text-muted-foreground">
                        {i18n.t('settings.trackers_section.pkce_redirect_notice')}
                    </p>
                    <Button onclick={handleAuthStart} disabled={addingTracker} class="w-full rounded-sm h-11 font-bold">
                        {#if addingTracker}<Spinner class="mr-2 h-4 w-4" />{/if}
                        {i18n.t('settings.trackers_section.login_to_service', { name: newTrackerDisplayName})}
                    </Button>
                </div>
            {:else if newTrackerAuth?.oauthFlow === 'password'}
                <form onsubmit={handleAddTracker} class="space-y-4">
                    <div class="space-y-2">
                        <Label for="username" class="text-base font-bold">{i18n.t('settings.trackers_section.email_or_username')}</Label>
                        <Input id="username" type="text" placeholder="ejemplo@correo.com" bind:value={newTrackerUsername} required class="rounded-sm h-11 w-full" />
                    </div>
                    <div class="space-y-2">
                        <Label for="password" class="text-base font-bold">{i18n.t('settings.account_section.new_password')}</Label>
                        <Input id="password" type="password" placeholder="••••••••" bind:value={newTrackerPassword} required class="rounded-sm h-11 w-full" />
                    </div>
                    <Button type="submit" disabled={addingTracker} class="w-full rounded-sm h-11 font-bold mt-4">
                        {#if addingTracker}<Spinner class="mr-2 h-4 w-4" />{/if}
                        {i18n.t('settings.trackers_section.connect_tracker')}
                    </Button>
                </form>
            {:else}
                <form onsubmit={handleAddTracker} class="space-y-4">
                    <div class="space-y-2">
                        <div class="flex items-center justify-between">
                            <Label for="token" class="text-base font-bold">{i18n.t('settings.trackers_section.token')}</Label>
                            <Button variant="link" size="sm" onclick={handleAuthStart} class="text-sm font-bold text-primary p-0 h-auto">
                                {i18n.t('settings.trackers_section.get_token', { name: newTrackerDisplayName })} <ExternalLink class="h-3.5 w-3.5 ml-1" />
                            </Button>
                        </div>
                        <Input id="token" type="password" placeholder={i18n.t('settings.trackers_section.paste_token')} bind:value={newTrackerToken} required class="rounded-sm h-11 w-full" />
                    </div>
                    <Button type="submit" disabled={addingTracker} class="w-full rounded-sm h-11 font-bold mt-4">
                        {#if addingTracker}<Spinner class="mr-2 h-4 w-4" />{/if}
                        {i18n.t('settings.trackers_section.connect_tracker')}
                    </Button>
                </form>
            {/if}
        </div>
    </Dialog.Content>
</Dialog.Root>

<AlertDialog.Root bind:open={showRemoveTrackerAlert}>
    <AlertDialog.Content class="border-destructive/20 sm:rounded-sm">
        <AlertDialog.Header>
            <AlertDialog.Title class="text-destructive flex items-center gap-2 text-xl">
                <AlertTriangle class="h-6 w-6" /> {i18n.t('settings.trackers_section.disconnect_tracker')}
            </AlertDialog.Title>
            <AlertDialog.Description class="text-base">
                {i18n.t('settings.trackers_section.disconnect_tracker_desc')}
            </AlertDialog.Description>
        </AlertDialog.Header>
        <AlertDialog.Footer class="mt-6">
            <AlertDialog.Cancel class="rounded-sm font-bold">{i18n.t('settings.general_section.cancel')}</AlertDialog.Cancel>
            <AlertDialog.Action class="bg-destructive text-destructive-foreground hover:bg-destructive/90 rounded-sm font-bold" onclick={handleRemoveTracker}>
                {#if removingTracker}<Spinner class="h-4 w-4 mr-2" />{/if} {i18n.t('settings.trackers_section.disconnect')}
            </AlertDialog.Action>
        </AlertDialog.Footer>
    </AlertDialog.Content>
</AlertDialog.Root>