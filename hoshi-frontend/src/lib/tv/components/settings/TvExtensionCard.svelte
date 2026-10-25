<script lang="ts">
    import { Settings2, Trash2, Download, RefreshCw, Check, ArrowRight } from 'lucide-svelte';
    import { Spinner } from '@/components/ui/spinner';
    import { i18n } from '@/stores/i18n.svelte.js';
    import {
        SOURCE_LABELS,
        buildDraftSettings,
        toneOf,
        type CardExt,
        type Source
    } from '@/app/marketplace.svelte.js';
    import TvDialog from '@/tv/components/TvDialog.svelte';
    import TvExtensionForm from './TvExtensionForm.svelte';

    let {
        ext,
        source = 'native',
        mode = 'installed',
        busy = false,
        saving = false,
        installed = false,
        hasUpdate = false,
        fromVersion = null,
        onInstall,
        onUpdate,
        onUninstall,
        onSave
    }: {
        ext: CardExt;
        source?: Source;
        mode?: 'installed' | 'marketplace';
        busy?: boolean;
        saving?: boolean;
        installed?: boolean;
        hasUpdate?: boolean;
        fromVersion?: string | null;
        onInstall?: () => void;
        onUpdate?: () => void;
        onUninstall?: () => void;
        onSave?: (settings: Record<string, any>) => Promise<boolean>;
    } = $props();

    // Keyed by plain string so it doesn't depend on the `Tone` union; unknown types fall back to muted.
    const TONES: Record<string, string> = {
        anime: 'bg-blue-500/10 text-blue-400 border-blue-500/30',
        torrent: 'bg-amber-500/10 text-amber-400 border-amber-500/30'
    };
    const TONE_FALLBACK = 'bg-muted text-muted-foreground border-border';
    const toneClass = (type: string) => TONES[toneOf(type)] ?? TONE_FALLBACK;

    const hasSettings = $derived(mode === 'installed' && (ext.setting_definitions?.length ?? 0) > 0);

    let iconStatus = $state<'loading' | 'loaded' | 'error'>('loading');
    const showIcon = $derived(!!ext.icon && iconStatus !== 'error');

    // Settings dialog
    let configOpen = $state(false);
    let draft = $state<Record<string, any>>({});

    function openConfig() {
        draft = buildDraftSettings(ext);
        configOpen = true;
    }

    async function handleChange() {
        await onSave?.($state.snapshot(draft));
    }

    // Uninstall confirmation (a timed "tap twice" doesn't work with a D-pad, so use a dialog)
    let confirmOpen = $state(false);

    function confirmUninstall() {
        confirmOpen = false;
        onUninstall?.();
    }

    const btn =
        'flex h-12 shrink-0 scroll-my-24 items-center justify-center gap-2 rounded-lg px-5 text-base font-bold outline-none transition-transform duration-150 focus:scale-105 focus:ring-4 aria-disabled:opacity-50';
</script>

<div
        data-ext-card={ext.id}
        class="flex items-center gap-5 rounded-lg border-2 bg-card p-4
               {ext.nsfw ? 'border-destructive/50' : hasUpdate ? 'border-primary/50' : 'border-border/60'}"
>
    <!-- Icon (initials show underneath until the image loads) -->
    <div class="relative flex size-16 shrink-0 items-center justify-center overflow-hidden rounded-md border border-border/50 bg-muted/30">
        {#if !(showIcon && iconStatus === 'loaded')}
            <span class="select-none text-lg font-black text-primary">{ext.name.slice(0, 2).toUpperCase()}</span>
        {/if}
        {#if showIcon}
            <img
                    src={ext.icon}
                    alt=""
                    class="absolute inset-0 size-full object-cover"
                    onload={() => (iconStatus = 'loaded')}
                    onerror={() => (iconStatus = 'error')}
            />
        {/if}
    </div>

    <!-- Info -->
    <div class="min-w-0 flex-1 space-y-1.5">
        <div class="flex flex-wrap items-center gap-x-3 gap-y-1">
            <h4 class="truncate text-xl font-bold">{ext.name}</h4>

            {#if ext.ext_type}
                <span class="rounded border px-2 py-0.5 text-xs font-black uppercase tracking-wider {toneClass(ext.ext_type)}">
                    {ext.ext_type}
                </span>
            {/if}

            {#if ext.nsfw}
                <span class="rounded border border-destructive bg-destructive/10 px-2 py-0.5 text-xs font-black uppercase tracking-wider text-destructive">
                    NSFW
                </span>
            {/if}
        </div>

        <div class="flex flex-wrap items-center gap-x-2 gap-y-0.5 text-sm font-semibold text-muted-foreground">
            {#if hasUpdate && fromVersion}
                <span class="flex items-center gap-1 text-primary">
                    v{fromVersion}<ArrowRight class="size-4" />v{ext.version}
                </span>
            {:else}
                <span>v{ext.version}</span>
            {/if}

            {#if ext.author}
                <span class="opacity-40">•</span>
                <span class="max-w-[16rem] truncate">{ext.author}</span>
            {/if}

            {#if ext.language}
                <span class="opacity-40">•</span>
                <span class="rounded bg-muted px-1.5 text-xs font-bold uppercase tracking-wider">{ext.language}</span>
            {/if}

            {#if source !== 'native'}
                <span class="opacity-40">•</span>
                <span class="text-xs font-bold tracking-wide">{SOURCE_LABELS[source]}</span>
            {/if}
        </div>
    </div>

    <!-- Actions -->
    <div class="flex shrink-0 items-center gap-3">
        {#if mode === 'installed'}
            {#if hasSettings}
                <button
                        type="button"
                        data-sn
                        aria-label={i18n.t('settings.extension_section.extension_config', { name: ext.name })}
                        onclick={openConfig}
                        class="{btn} w-12 bg-secondary text-secondary-foreground focus:ring-primary/70"
                >
                    <Settings2 class="size-5" />
                </button>
            {/if}

            <button
                    type="button"
                    data-sn
                    aria-disabled={busy}
                    aria-label={i18n.t('settings.extension_section.uninstall')}
                    onclick={() => {
                        if (!busy) confirmOpen = true;
                    }}
                    class="{btn} w-12 bg-destructive/15 text-destructive focus:bg-destructive focus:text-destructive-foreground focus:ring-destructive/50"
            >
                {#if busy}<Spinner class="size-5" />{:else}<Trash2 class="size-5" />{/if}
            </button>
        {:else}
            {@const isInstalled = installed && !hasUpdate}
            <button
                    type="button"
                    data-sn
                    aria-disabled={busy || isInstalled}
                    onclick={() => {
                        if (busy || isInstalled) return;
                        if (hasUpdate) onUpdate?.();
                        else onInstall?.();
                    }}
                    class="{btn} {isInstalled
                        ? 'bg-muted/40 text-muted-foreground focus:ring-primary/70'
                        : 'bg-primary text-primary-foreground focus:ring-primary/50'}"
            >
                {#if busy}
                    <Spinner class="size-5" />
                {:else if isInstalled}
                    <Check class="size-5" />
                {:else if hasUpdate}
                    <RefreshCw class="size-5" />
                {:else}
                    <Download class="size-5" />
                {/if}
                {isInstalled
                    ? i18n.t('marketplace.installed')
                    : hasUpdate
                        ? i18n.t('marketplace.update')
                        : i18n.t('marketplace.install')}
            </button>
        {/if}
    </div>
</div>

{#if configOpen}
    <TvDialog
            title={i18n.t('settings.extension_section.extension_config', { name: ext.name })}
            onclose={() => (configOpen = false)}
    >
        <TvExtensionForm {ext} bind:settings={draft} isSaving={saving} onChange={handleChange} />

        <div class="flex justify-end">
            <button
                    type="button"
                    data-sn
                    onclick={() => (configOpen = false)}
                    class="h-12 rounded-lg bg-primary px-8 text-base font-bold text-primary-foreground outline-none focus:ring-4 focus:ring-primary/50"
            >
                {i18n.t('content.close')}
            </button>
        </div>
    </TvDialog>
{/if}

{#if confirmOpen}
    <TvDialog
            title={i18n.t('settings.extension_section.uninstall')}
            description={ext.name}
            onclose={() => (confirmOpen = false)}
    >
        <!-- Cancel is first in the DOM so it gets the initial focus. -->
        <div class="flex justify-end gap-3 pt-2">
            <button
                    type="button"
                    data-sn
                    onclick={() => (confirmOpen = false)}
                    class="h-12 rounded-lg border-2 border-border/60 px-8 text-base font-bold text-foreground/80 outline-none focus:ring-4 focus:ring-primary/60"
            >
                {i18n.t('content.cancel')}
            </button>
            <button
                    type="button"
                    data-sn
                    onclick={confirmUninstall}
                    class="h-12 rounded-lg bg-destructive px-8 text-base font-bold text-destructive-foreground outline-none focus:ring-4 focus:ring-destructive/50"
            >
                {i18n.t('settings.extension_section.confirm_uninstall')}
            </button>
        </div>
    </TvDialog>
{/if}