<script lang="ts">
    import { Badge } from "$lib/components/ui/badge";
    import { Button } from "$lib/components/ui/button";
    import * as Avatar from "$lib/components/ui/avatar";
    import * as Drawer from "$lib/components/ui/drawer";
    import { Spinner } from "@/components/ui/spinner";
    import { Settings2, X, Trash2, Download, RefreshCw } from "lucide-svelte";
    import { i18n } from "@/stores/i18n.svelte.js";
    import Form from "./Form.svelte";
    import type { Extension } from "@/api/extensions/types";

    let {
        ext,
        source,
        mode = "installed",
        isActionLoading = false,
        isMarketplaceInstalled = false,
        isSaving = false,
        hasUpdate = false,
        onAction,
        onUpdate,
        onSave
    }: {
        ext: any;
        source?: string;
        mode?: "installed" | "marketplace";
        isActionLoading?: boolean;
        isMarketplaceInstalled?: boolean;
        isSaving?: boolean;
        hasUpdate?: boolean;
        onAction?: (ext: any) => void;
        onUpdate?: (ext: any) => void;
        onSave?: (id: string, settings: Record<string, any>) => Promise<boolean>;
    } = $props();

    let isExpanded = $state(false);
    let isDrawerOpen = $state(false);
    let draftSettings = $state<Record<string, any>>({});
    let hasInitializedSettings = $state(false);

    $effect(() => {
        if (mode === "installed" && (isExpanded || isDrawerOpen) && !hasInitializedSettings) {
            const initialSettings: Record<string, any> = { ...ext.settings };
            if (ext.setting_definitions) {
                ext.setting_definitions.forEach((def: any) => {
                    if (initialSettings[def.key] === undefined) {
                        initialSettings[def.key] = Array.isArray(def.default)
                            ? [...def.default]
                            : def.default;
                    }
                });
            }
            draftSettings = initialSettings;
            hasInitializedSettings = true;
        }
    });

    async function handleSave() {
        if (!onSave) return;
        const success = await onSave(ext.id, draftSettings);
        if (success) {
            isExpanded = false;
            isDrawerOpen = false;
        }
    }

    function getTypeColor(type: string) {
        const t = (type || "").toLowerCase();
        switch(t) {
            case 'anime': return 'bg-blue-500/10 text-blue-500 border-blue-500/20';
            case 'manga': return 'bg-green-500/10 text-green-500 border-green-500/20';
            case 'novel': return 'bg-purple-500/10 text-purple-500 border-purple-500/20';
            default: return 'bg-muted text-muted-foreground border-border';
        }
    }
</script>

<div class="flex flex-col rounded-xl border {ext.nsfw ? 'border-destructive/60 shadow-destructive/10' : 'border-border/60'} bg-card overflow-hidden transition-all shadow-sm {mode === 'marketplace' ? 'hover:border-primary/40' : ''}">
    <div class="flex items-center p-3 gap-3">
        <Avatar.Root class="relative h-10 w-10 rounded-lg border border-border/50 shrink-0 bg-muted/30 overflow-hidden flex items-center justify-center">
            <div data-fallback class="bg-primary/10 text-primary font-black rounded-lg text-xs w-full h-full flex items-center justify-center absolute inset-0 z-0">
                {ext.name.slice(0, 2).toUpperCase()}
            </div>
            {#if ext.icon}
                <img
                        src={ext.icon}
                        alt={ext.name}
                        class="object-cover w-full h-full absolute inset-0 z-10"
                        onload={(e) => {
                        const fallback = e.currentTarget.parentElement?.querySelector('[data-fallback]');
                        if (fallback) fallback.style.display = 'none';
                    }}
                        onerror={(e) => {
                        e.currentTarget.style.display = 'none';
                    }}
                />
            {/if}
        </Avatar.Root>

        <div class="space-y-0.5 flex-1 min-w-0">
            <div class="flex items-center gap-2">
                <h3 class="font-bold text-sm truncate">{ext.name}</h3>

                <div class="flex items-center gap-1 shrink-0">
                    <Badge variant="outline" class="text-[9px] px-1 uppercase font-black tracking-wider h-4 {getTypeColor(ext.ext_type)}">
                        {ext.ext_type}
                    </Badge>

                    {#if ext.language}
                        <Badge variant="secondary" class="text-[9px] px-1 uppercase font-black tracking-wider h-4 bg-muted/80 text-muted-foreground">
                            {ext.language}
                        </Badge>
                    {/if}

                    {#if ext.nsfw}
                        <Badge variant="outline" class="text-[9px] px-1 uppercase font-black tracking-wider h-4 border-destructive text-destructive bg-destructive/10">
                            NSFW
                        </Badge>
                    {/if}
                    {#if source === 'lnreader'}
                        <Badge variant="outline" class="text-[9px] px-1 uppercase font-black tracking-wider h-4 border-orange-500/40 text-orange-500 bg-orange-500/10">
                            LNR
                        </Badge>
                    {/if}
                    {#if source === 'tachiyomi'}
                        <Badge
                                variant="outline"
                                class="text-[9px] px-1 uppercase font-black tracking-wider h-4 border-blue-500/40 text-blue-500 bg-blue-500/10"
                        >
                            TACHIYOMI
                        </Badge>
                    {/if}
                </div>
            </div>

            <div class="flex items-center gap-1.5 text-[11px] font-semibold text-muted-foreground/80 mt-0.5">
                <span>v{ext.version}</span>
                {#if ext.author}
                    <span class="opacity-50">•</span>
                    <span class="truncate">{ext.author}</span>
                {/if}
            </div>
        </div>

        <div class="flex gap-1.5 shrink-0 items-center">
            {#if mode === "installed"}
                {#if ext.setting_definitions && ext.setting_definitions.length > 0}
                    <Button
                            variant={isExpanded ? "default" : "secondary"}
                            size="sm"
                            class="h-8 w-8 p-0 rounded-lg hidden md:flex"
                            onclick={() => isExpanded = !isExpanded}
                    >
                        {#if isExpanded}<X class="h-4 w-4" />{:else}<Settings2 class="h-4 w-4" />{/if}
                    </Button>

                    <div class="md:hidden">
                        <Drawer.Root open={isDrawerOpen} onOpenChange={(v) => isDrawerOpen = v}>
                            <Drawer.Trigger>
                                {#snippet child({ props })}
                                    <Button {...props} variant="secondary" size="sm" class="h-8 w-8 p-0 rounded-lg">
                                        <Settings2 class="h-4 w-4" />
                                    </Button>
                                {/snippet}
                            </Drawer.Trigger>
                            <Drawer.Content class="h-[85vh] rounded-t-2xl border-border/50">
                                <div class="p-5 overflow-y-auto hide-scrollbar">
                                    <h3 class="font-black text-lg mb-6 tracking-tight flex items-center gap-2 border-b border-border/40 pb-4">
                                        <Settings2 class="w-5 h-5 text-primary" />
                                        {i18n.t('settings.extension_section.extension_config', { name: ext.name})}
                                    </h3>
                                    <Form
                                            {ext}
                                            bind:settings={draftSettings}
                                            {isSaving}
                                            onSave={handleSave}
                                    />
                                </div>
                            </Drawer.Content>
                        </Drawer.Root>
                    </div>
                {/if}

                <Button
                        variant="destructive"
                        size="sm"
                        class="h-8 w-8 p-0 rounded-lg bg-destructive/10 text-destructive hover:bg-destructive hover:text-destructive-foreground"
                        onclick={() => onAction?.(ext.id)}
                        disabled={isActionLoading}
                >
                    {#if isActionLoading}<Spinner class="h-4 w-4 animate-spin" />{:else}<Trash2 class="h-4 w-4" />{/if}
                </Button>
            {:else if mode === "marketplace"}
                {#if isMarketplaceInstalled}
                    {#if hasUpdate}
                        <Button
                                size="sm"
                                class="rounded-lg h-8 px-4 text-xs font-bold shadow-sm bg-primary/90 hover:bg-primary"
                                onclick={() => onUpdate?.(ext)}
                                disabled={isActionLoading}
                        >
                            {#if isActionLoading}
                                <Spinner class="h-3 w-3 mr-1.5 animate-spin" />
                            {:else}
                                <RefreshCw class="h-3 w-3 mr-1.5" />
                            {/if}
                            {i18n.t('marketplace.update')}
                        </Button>
                        {:else}
                        <Button variant="secondary" size="sm" class="rounded-lg h-8 px-4 text-xs font-bold bg-muted/40 text-muted-foreground" disabled>
                            {i18n.t('marketplace.installed')}
                        </Button>
                    {/if}
                    {:else}
                    <Button size="sm" class="rounded-lg h-8 px-4 text-xs font-bold shadow-sm" onclick={() => onAction?.(ext)} disabled={isActionLoading}>
                        {#if isActionLoading}
                            <Spinner class="h-3 w-3 mr-1.5 animate-spin" />
                        {:else}
                            <Download class="h-3 w-3 mr-1.5" />
                        {/if}
                        {i18n.t('marketplace.install')}
                    </Button>
                {/if}
            {/if}
        </div>
    </div>

    {#if mode === "installed" && isExpanded}
        <div class="p-4 border-t border-border/40 bg-muted/10 hidden md:block">
            <Form
                    {ext}
                    bind:settings={draftSettings}
                    {isSaving}
                    onSave={handleSave}
            />
        </div>
    {/if}
</div>