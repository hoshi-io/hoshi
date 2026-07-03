<script lang="ts">
    import { Switch } from "$lib/components/ui/switch";
    import { Label } from "$lib/components/ui/label";
    import { ExternalLink, CheckCircle2, Loader2, Circle, Download } from "lucide-svelte";
    import { i18n } from "@/stores/i18n.svelte.js";
    import { mpvApi } from "@/api/mpv/mpv";
    import type { MpvConfig } from "@/api/config/types";

    let {
        mpvConfig = $bindable(),
        isAndroid,
        onSave
    }: {
        mpvConfig: MpvConfig,
        isAndroid: boolean,
        onSave: () => Promise<void> | void
    } = $props();

    type OscId = 'modernz' | 'mpv-osc-modern' | 'hayase-osc';
    interface OscEntry {
        id: OscId;
        name: string;
        description: string;
        repoUrl: string;
        credit: string;
    }

    const OSC_LIST: OscEntry[] = [
        { id: 'modernz', name: 'ModernZ', description: 'A feature-rich, modern OSC with extensive customization options.', repoUrl: 'https://github.com/Samillion/ModernZ', credit: 'by Samillion' },
        { id: 'mpv-osc-modern', name: 'mpv-osc-modern', description: 'Clean Material Design-inspired OSC with a minimal footprint.', repoUrl: 'https://github.com/maoiscat/mpv-osc-modern', credit: 'by maoiscat' },
        { id: 'hayase-osc', name: 'Hayase OSC', description: 'Lightweight and elegant OSC with Lucide icon support.', repoUrl: 'https://github.com/nekoxuee/hayase-osc', credit: 'by nekoxuee' },
    ];

    let downloadingOsc = $state<OscId | null>(null);

    async function handleSelectOsc(id: OscId | null) {
        if (!mpvConfig.useHoshiConfig) return;
        if (mpvConfig.activeOsc === id) {
            mpvConfig.activeOsc = null;
            await onSave();
            return;
        }

        downloadingOsc = id;
        try {
            await mpvApi.downloadOsc(id);
            mpvConfig.activeOsc = id;
            await onSave();
        } catch (e) {
            console.error(e);
        } finally {
            downloadingOsc = null;
        }
    }

    interface KnownScript {
        name: string;
        description: string;
        repoUrl: string;
    }

    const KNOWN_SCRIPTS: KnownScript[] = [
        { name: 'thumbfast.lua', description: 'High-performance on-the-fly seekbar thumbnails.', repoUrl: 'https://github.com/po5/thumbfast' },
    ];

    let downloadingScript = $state<string | null>(null);

    async function handleToggleScript(name: string) {
        if (!mpvConfig.useHoshiConfig) return;
        if (mpvConfig.enabledScripts.includes(name)) {
            mpvConfig.enabledScripts = mpvConfig.enabledScripts.filter(s => s !== name);
            await onSave();
            return;
        }

        downloadingScript = name;
        try {
            await mpvApi.downloadKnownScript(name);
            mpvConfig.enabledScripts = [...mpvConfig.enabledScripts, name];
            await onSave();
        } catch (e) {
            console.error(e);
        } finally {
            downloadingScript = null;
        }
    }
</script>

<div class="focus-visible:outline-none mt-0">
    <div class="flex flex-col sm:flex-row sm:items-center justify-between gap-4 py-6 border-b border-border/40">
        <div class="space-y-1 pr-4">
            <Label class="text-base font-bold" for="useMpv">{i18n.t("settings.player_section.use_mpv")}</Label>
            <p class="text-sm text-muted-foreground">{i18n.t("settings.player_section.use_mpv_desc")}</p>
        </div>
        <Switch id="useMpv" bind:checked={mpvConfig.useMpv} onCheckedChange={onSave} class="shrink-0 scale-125" />
    </div>

    {#if mpvConfig.useMpv && !isAndroid}
        <div class="flex flex-col sm:flex-row sm:items-center justify-between gap-4 py-6 border-b border-border/40">
            <div class="space-y-1 pr-4">
                <Label class="text-base font-bold" for="useHoshiConfig">{i18n.t("settings.player_section.let_hoshi")}</Label>
                <p class="text-sm text-muted-foreground">{i18n.t("settings.player_section.let_hoshi_desc")}</p>
            </div>
            <Switch id="useHoshiConfig" bind:checked={mpvConfig.useHoshiConfig} onCheckedChange={onSave} class="shrink-0 scale-125" />
        </div>

        {#if mpvConfig.useHoshiConfig}
            <div class="py-6 border-b border-border/40 space-y-4">
                <div class="space-y-1">
                    <Label class="text-base font-bold">{i18n.t("settings.player_section.osc")}</Label>
                    <p class="text-sm text-muted-foreground">{i18n.t("settings.player_section.osc_desc")}</p>
                </div>

                <div class="grid grid-cols-1 sm:grid-cols-3 gap-3">
                    {#each OSC_LIST as osc}
                        {@const isActive = mpvConfig.activeOsc === osc.id}
                        {@const isDownloading = downloadingOsc === osc.id}

                        <button
                                class="relative text-left rounded-sm border p-4 transition-all focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-primary
                            {isActive ? 'border-primary bg-primary/5' : 'border-border/40 bg-card hover:border-border hover:bg-muted/30'}
                            {isDownloading ? 'opacity-70 pointer-events-none' : ''}"
                                onclick={() => handleSelectOsc(osc.id)}
                                disabled={isDownloading}
                        >
                            <div class="absolute top-3 right-3">
                                {#if isDownloading}
                                    <Loader2 class="h-4 w-4 animate-spin text-muted-foreground" />
                                {:else if isActive}
                                    <CheckCircle2 class="h-4 w-4 text-primary" />
                                {:else}
                                    <Circle class="h-4 w-4 text-muted-foreground/30" />
                                {/if}
                            </div>

                            <p class="font-bold text-sm pr-6">{osc.name}</p>
                            <p class="text-xs text-muted-foreground/60 mb-2">{osc.credit}</p>
                            <p class="text-xs text-muted-foreground leading-relaxed">{osc.description}</p>

                            <a
                                    href={osc.repoUrl}
                                    target="_blank"
                                    rel="noopener noreferrer"
                                    class="inline-flex items-center gap-1 mt-3 text-xs text-primary/70 hover:text-primary transition-colors"
                                    onclick={(e) => e.stopPropagation()}
                            >
                                <ExternalLink class="h-3 w-3" />
                                Docs
                            </a>
                        </button>
                    {/each}
                </div>
            </div>

            <div class="py-6 space-y-4">
                <div class="space-y-1">
                    <Label class="text-base font-bold">{i18n.t("settings.player_section.scripts")}</Label>
                    <p class="text-sm text-muted-foreground">{i18n.t("settings.player_section.scripts_desc")}</p>
                </div>

                <div class="flex flex-col gap-2">
                    {#each KNOWN_SCRIPTS as script}
                        {@const isEnabled = mpvConfig.enabledScripts.includes(script.name)}
                        {@const isDownloading = downloadingScript === script.name}

                        <div class="flex items-center justify-between rounded-sm border border-border/40 bg-card px-4 py-3 gap-4">
                            <div class="space-y-0.5 min-w-0">
                                <p class="text-sm font-bold">{script.name}</p>
                                <p class="text-xs text-muted-foreground">{script.description}</p>
                                <a
                                        href={script.repoUrl}
                                        target="_blank"
                                        rel="noopener noreferrer"
                                        class="inline-flex items-center gap-1 text-xs text-primary/70 hover:text-primary transition-colors"
                                >
                                    <ExternalLink class="h-3 w-3" />
                                    Repository
                                </a>
                            </div>

                            <button
                                    class="shrink-0 flex items-center gap-1.5 text-xs font-bold px-3 py-1.5 rounded-lg transition-all
                                {isEnabled ? 'bg-primary/10 text-primary hover:bg-primary/20' : 'bg-muted text-muted-foreground hover:bg-muted/70'}
                                {isDownloading ? 'opacity-50 pointer-events-none' : ''}"
                                    onclick={() => handleToggleScript(script.name)}
                                    disabled={isDownloading}
                            >
                                {#if isDownloading}
                                    <Loader2 class="h-3 w-3 animate-spin" />
                                {:else if isEnabled}
                                    <CheckCircle2 class="h-3 w-3" />
                                {:else}
                                    <Download class="h-3 w-3" />
                                {/if}
                                {isEnabled ? 'Enabled' : 'Install'}
                            </button>
                        </div>
                    {/each}
                </div>
            </div>
        {/if}
    {/if}
</div>