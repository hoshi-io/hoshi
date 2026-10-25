<script lang="ts">
    import { onDestroy } from "svelte";
    import { Badge } from "$lib/components/ui/badge";
    import { Button } from "$lib/components/ui/button";
    import * as Drawer from "$lib/components/ui/drawer";
    import { Spinner } from "@/components/ui/spinner";
    import { Settings2, X, Trash2, Download, RefreshCw, Check, ArrowRight } from "lucide-svelte";
    import { i18n } from "@/stores/i18n.svelte.js";
    import {
        SOURCE_LABELS,
        buildDraftSettings,
        toneOf,
        type CardExt,
        type Tone,
        type Source
    } from "@/app/marketplace.svelte.js";
    import Form from "./Form.svelte";

    let {
        ext,
        source = "native",
        mode = "installed",
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
        mode?: "installed" | "marketplace";
        /** An install / update / uninstall is running */
        busy?: boolean;
        /** Settings are being saved */
        saving?: boolean;
        /** Marketplace: this entry is already installed */
        installed?: boolean;
        /** Marketplace: a newer version than the installed one exists */
        hasUpdate?: boolean;
        /** Marketplace + update: the version currently installed */
        fromVersion?: string | null;
        onInstall?: () => void;
        onUpdate?: () => void;
        onUninstall?: () => void;
        onSave?: (settings: Record<string, any>) => Promise<boolean>;
    } = $props();

    const TONES: Record<Tone, string> = {
        anime: "bg-blue-500/10 text-blue-500 border-blue-500/20",
        manga: "bg-green-500/10 text-green-500 border-green-500/20",
        novel: "bg-purple-500/10 text-purple-500 border-purple-500/20",
        movie: "bg-amber-500/10 text-amber-500 border-amber-500/20",
        other: "bg-muted text-muted-foreground border-border"
    };

    const hasSettings = $derived(mode === "installed" && (ext.setting_definitions?.length ?? 0) > 0);

    let iconStatus = $state<"loading" | "loaded" | "error">("loading");
    const showIcon = $derived(!!ext.icon && iconStatus !== "error");

    let expanded = $state(false);
    let drawerOpen = $state(false);
    let draft = $state<Record<string, any>>({});
    let draftReady = $state(false);

    function ensureDraft() {
        if (draftReady) return;
        draft = buildDraftSettings(ext);
        draftReady = true;
    }

    function toggleExpanded() {
        ensureDraft();
        expanded = !expanded;
    }

    function setDrawer(open: boolean) {
        if (open) ensureDraft();
        drawerOpen = open;
    }

    async function handleChange() {
        if (draftReady) await onSave?.($state.snapshot(draft));
    }

    let confirming = $state(false);
    let confirmTimer: ReturnType<typeof setTimeout> | undefined;

    function askUninstall() {
        if (!confirming) {
            confirming = true;
            confirmTimer = setTimeout(() => (confirming = false), 3000);
            return;
        }
        clearTimeout(confirmTimer);
        confirming = false;
        onUninstall?.();
    }

    onDestroy(() => clearTimeout(confirmTimer));
</script>

<div
        class="flex flex-col overflow-hidden rounded-sm border bg-card shadow-sm transition-colors
               {ext.nsfw ? 'border-destructive/50'
                   : hasUpdate ? 'border-primary/50'
                   : expanded ? 'border-primary/30'
                   : 'border-border/60'}
               {mode === 'marketplace' && !ext.nsfw ? 'hover:border-primary/40' : ''}"
>
    <div class="flex items-center gap-3 p-3.5">
        <!-- Icon (initials sit underneath until the image has loaded) -->
        <div class="relative flex size-11 shrink-0 items-center justify-center overflow-hidden rounded-sm border border-border/50 bg-muted/30">
            {#if !(showIcon && iconStatus === "loaded")}
                <span class="select-none text-xs font-black text-primary">{ext.name.slice(0, 2).toUpperCase()}</span>
            {/if}
            {#if showIcon}
                <img
                        src={ext.icon}
                        alt=""
                        class="absolute inset-0 size-full object-cover"
                        onload={() => (iconStatus = "loaded")}
                        onerror={() => (iconStatus = "error")}
                />
            {/if}
        </div>

        <!-- Info -->
        <div class="min-w-0 flex-1 space-y-1">
            <div class="flex flex-wrap items-center gap-x-2 gap-y-1">
                <h4 class="truncate text-sm font-bold">{ext.name}</h4>

                {#if ext.ext_type}
                    <Badge variant="outline" class="h-4 px-1.5 text-[9px] font-black uppercase tracking-wider {TONES[toneOf(ext.ext_type)]}">
                        {ext.ext_type}
                    </Badge>
                {/if}

                {#if ext.nsfw}
                    <Badge variant="outline" class="h-4 border-destructive bg-destructive/10 px-1.5 text-[9px] font-black uppercase tracking-wider text-destructive">
                        NSFW
                    </Badge>
                {/if}
            </div>

            <div class="flex flex-wrap items-center gap-x-1.5 gap-y-0.5 text-[11px] font-semibold text-muted-foreground/80">
                {#if hasUpdate && fromVersion}
                    <span class="flex items-center gap-1 text-primary">
                        v{fromVersion}<ArrowRight class="size-3" />v{ext.version}
                    </span>
                {:else}
                    <span>v{ext.version}</span>
                {/if}

                {#if ext.author}
                    <span class="opacity-40">•</span>
                    <span class="max-w-[140px] truncate">{ext.author}</span>
                {/if}

                {#if ext.language}
                    <span class="opacity-40">•</span>
                    <span class="rounded-sm bg-muted px-1 text-[10px] font-bold uppercase tracking-wider text-muted-foreground/70">
                        {ext.language}
                    </span>
                {/if}

                {#if source !== "native"}
                    <span class="opacity-40">•</span>
                    <span class="text-[10px] font-bold tracking-wide text-muted-foreground">{SOURCE_LABELS[source]}</span>
                {/if}
            </div>
        </div>

        <!-- Actions -->
        <div class="flex shrink-0 items-center gap-1.5">
            {#if mode === "installed"}
                {#if hasSettings}
                    <!-- Desktop: toggles the inline panel -->
                    <Button
                            variant={expanded ? "default" : "secondary"}
                            size="sm"
                            class="hidden size-8 rounded-lg p-0 md:flex"
                            aria-label={i18n.t('settings.extension_section.extension_config', { name: ext.name })}
                            aria-expanded={expanded}
                            onclick={toggleExpanded}
                    >
                        {#if expanded}<X class="size-4" />{:else}<Settings2 class="size-4" />{/if}
                    </Button>

                    <!-- Small screens: drawer -->
                    <div class="md:hidden">
                        <Drawer.Root open={drawerOpen} onOpenChange={setDrawer}>
                            <Drawer.Trigger>
                                {#snippet child({ props })}
                                    <Button
                                            {...props}
                                            variant="secondary"
                                            size="sm"
                                            class="size-8 rounded-lg p-0"
                                            aria-label={i18n.t('settings.extension_section.extension_config', { name: ext.name })}
                                    >
                                        <Settings2 class="size-4" />
                                    </Button>
                                {/snippet}
                            </Drawer.Trigger>
                            <Drawer.Content class="h-[85vh] rounded-t-2xl border-border/50">
                                <div class="hide-scrollbar overflow-y-auto p-5">
                                    <h3 class="mb-6 flex items-center gap-2 border-b border-border/40 pb-4 text-lg font-black tracking-tight">
                                        <Settings2 class="size-5 text-primary" />
                                        {i18n.t('settings.extension_section.extension_config', { name: ext.name })}
                                    </h3>
                                    {#if draftReady}
                                        <Form {ext} bind:settings={draft} isSaving={saving} onChange={handleChange} />
                                    {/if}
                                </div>
                            </Drawer.Content>
                        </Drawer.Root>
                    </div>
                {/if}

                <Button
                        variant="destructive"
                        size="sm"
                        class="h-8 rounded-sm text-xs font-bold transition-all
                               {confirming
                            ? 'px-3'
                            : 'w-8 bg-destructive/10 p-0 text-destructive hover:bg-destructive hover:text-white'}"
                        aria-label={i18n.t('settings.extension_section.uninstall', { default: 'Uninstall' })}
                        onclick={askUninstall}
                        onblur={() => (confirming = false)}
                        disabled={busy}
                >
                    {#if busy}
                        <Spinner class="size-4 animate-spin" />
                    {:else if confirming}
                        {i18n.t('settings.extension_section.confirm_uninstall', { default: 'Remove?' })}
                    {:else}
                        <Trash2 class="size-4" />
                    {/if}
                </Button>

            {:else if installed && !hasUpdate}
                <Button variant="ghost" size="sm" class="h-8 gap-1.5 rounded-sm px-3 text-xs font-bold text-muted-foreground" disabled>
                    <Check class="size-3.5" />
                    {i18n.t('marketplace.installed')}
                </Button>

            {:else if hasUpdate}
                <Button size="sm" class="h-8 gap-1.5 rounded-sm px-3 text-xs font-bold shadow-sm" onclick={onUpdate} disabled={busy}>
                    {#if busy}<Spinner class="size-3.5 animate-spin" />{:else}<RefreshCw class="size-3.5" />{/if}
                    {i18n.t('marketplace.update', { default: 'Update' })}
                </Button>

            {:else}
                <Button size="sm" class="h-8 gap-1.5 rounded-sm px-3 text-xs font-bold shadow-sm" onclick={onInstall} disabled={busy}>
                    {#if busy}<Spinner class="size-3.5 animate-spin" />{:else}<Download class="size-3.5" />{/if}
                    {i18n.t('marketplace.install')}
                </Button>
            {/if}
        </div>
    </div>

    {#if expanded && draftReady}
        <div class="hidden border-t border-border/40 bg-muted/10 p-4 md:block">
            <Form {ext} bind:settings={draft} isSaving={saving} onChange={handleChange} />
        </div>
    {/if}
</div>