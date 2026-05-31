<script lang="ts">
    import * as Tabs from "$lib/components/ui/tabs";
    import { Switch } from "$lib/components/ui/switch";
    import { Label } from "$lib/components/ui/label";
    import { Input } from "$lib/components/ui/input";
    import type { MpvConfig, PlayerConfig } from "@/api/config/types";
    import { i18n } from "@/stores/i18n.svelte.js";
    import { toast } from "svelte-sonner";
    import { ExternalLink, CheckCircle2, Loader2, Circle, Download, Play, Cpu } from "lucide-svelte";
    import { platform } from "@tauri-apps/plugin-os";
    import ResponsiveSelect from "@/components/ResponsiveSelect.svelte";
    import * as Kbd from "$lib/components/ui/kbd";
    import {mpvApi} from "@/api/mpv/mpv";

    let {
        playerConfig = $bindable(),
        mpvConfig = $bindable(),
        onSave
    }: {
        playerConfig: PlayerConfig,
        mpvConfig: MpvConfig,
        onSave: () => Promise<void> | void
    } = $props();

    const os = platform();
    const isAndroid = os === 'android';

    const seekSteps = [
        { value: "5",  label: i18n.t('settings.player_section.seconds', { num: 5 }) },
        { value: "10", label: i18n.t('settings.player_section.seconds', { num: 10 }) },
        { value: "15", label: i18n.t('settings.player_section.seconds', { num: 15 }) },
        { value: "30", label: i18n.t('settings.player_section.seconds', { num: 30 }) }
    ];

    const keyboardShortcuts = [
        { label: i18n.t('player.play') + "/" + i18n.t('player.pause'), keys: ["Space", "K"] },
        { label: i18n.t('player.seek_forward'),  keys: ["→", "L"], shift: "30s" },
        { label: i18n.t('player.seek_backward'), keys: ["←", "J"], shift: "30s" },
        { label: i18n.t('player.volume'),     keys: ["↑", "↓"] },
        { label: i18n.t('player.mute'),       keys: ["M"] },
        { label: i18n.t('player.fullscreen'), keys: ["F"] },
        { label: i18n.t('watch.skip_op') + "/" + i18n.t('watch.skip_ed'), keys: ["S"] },
    ];

    function handleSeekStepChange(val: string) {
        playerConfig.seekStep = parseInt(val);
        onSave();
    }

    type OscId = 'modernz' | 'mpv-osc-modern' | 'hayase-osc';

    interface OscEntry {
        id: OscId;
        name: string;
        description: string;
        repoUrl: string;
        credit: string;
    }

    const OSC_LIST: OscEntry[] = [
        {
            id: 'modernz',
            name: 'ModernZ',
            description: 'A feature-rich, modern OSC with extensive customization options.',
            repoUrl: 'https://github.com/Samillion/ModernZ',
            credit: 'by Samillion',
        },
        {
            id: 'mpv-osc-modern',
            name: 'mpv-osc-modern',
            description: 'Clean Material Design-inspired OSC with a minimal footprint.',
            repoUrl: 'https://github.com/maoiscat/mpv-osc-modern',
            credit: 'by maoiscat',
        },
        {
            id: 'hayase-osc',
            name: 'Hayase OSC',
            description: 'Lightweight and elegant OSC with Lucide icon support.',
            repoUrl: 'https://github.com/nekoxuee/hayase-osc',
            credit: 'by nekoxuee',
        },
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
            toast.success(`${OSC_LIST.find(o => o.id === id)?.name} applied`);
        } catch (e) {
            toast.error(`Failed to download OSC: ${e}`);
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
        {
            name: 'thumbfast.lua',
            description: 'High-performance on-the-fly seekbar thumbnails.',
            repoUrl: 'https://github.com/po5/thumbfast',
        },
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
            toast.success(`${name} enabled`);
        } catch (e) {
            toast.error(`Failed to download script: ${e}`);
            console.error(e);
        } finally {
            downloadingScript = null;
        }
    }
</script>

<div class="space-y-6">
    <div>
        <h2 class="text-2xl font-bold tracking-tight">{i18n.t('settings.player')}</h2>
        <p class="text-sm text-muted-foreground mt-1">{i18n.t('settings.player_section.player_desc')}</p>
    </div>

    <Tabs.Root value="player_general" class="w-full">
        <Tabs.List class="grid w-full max-w-[400px] grid-cols-2 rounded-xl h-11 p-1 bg-muted/50">
            <Tabs.Trigger value="player_general" class="rounded-lg font-bold flex items-center gap-2">
                <Play class="size-4" /> {i18n.t("settings.general")}
            </Tabs.Trigger>
            <Tabs.Trigger value="player_mpv" class="rounded-lg font-bold flex items-center gap-2">
                <Cpu class="size-4" /> MPV
            </Tabs.Trigger>
        </Tabs.List>

        <Tabs.Content value="player_general" class="focus-visible:outline-none mt-0">

            <div class="flex flex-col sm:flex-row sm:items-center justify-between gap-4 py-6 border-b border-border/40">
                <div class="space-y-1 pr-4 flex-1">
                    <Label class="text-base font-bold">{i18n.t('settings.player_section.preferred_sub_lang')}</Label>
                    <p class="text-sm text-muted-foreground">{i18n.t('settings.player_section.preferred_sub_lang_desc')}</p>
                </div>
                <div class="w-full sm:max-w-md">
                    <Input
                            bind:value={playerConfig.preferredSubLang}
                            onchange={onSave}
                            placeholder="en, es, ja"
                            class="rounded-xl h-11"
                    />
                </div>
            </div>

            <div class="flex flex-col sm:flex-row sm:items-center justify-between gap-4 py-6 border-b border-border/40">
                <div class="space-y-1 pr-4 flex-1">
                    <Label class="text-base font-bold">{i18n.t('settings.player_section.preferred_dub_lang')}</Label>
                    <p class="text-sm text-muted-foreground">{i18n.t('settings.player_section.preferred_dub_lang_desc')}</p>
                </div>
                <div class="w-full sm:max-w-md">
                    <Input
                            bind:value={playerConfig.preferredDubLang}
                            onchange={onSave}
                            placeholder="ja, en"
                            class="rounded-xl h-11"
                    />
                </div>
            </div>

            <div class="flex flex-col sm:flex-row sm:items-center justify-between gap-4 py-6 border-b border-border/40">
                <div class="space-y-1 pr-4">
                    <Label class="text-base font-bold">{i18n.t('settings.player_section.seek_step')}</Label>
                    <p class="text-sm text-muted-foreground">{i18n.t('settings.player_section.seek_step_desc')}</p>
                </div>
                <ResponsiveSelect
                        value={playerConfig.seekStep.toString()}
                        items={seekSteps}
                        class="rounded-xl h-11 w-full sm:max-w-md"
                        onValueChange={handleSeekStepChange}
                />
            </div>

            <div class="flex flex-col sm:flex-row sm:items-center justify-between gap-4 py-6 border-b border-border/40">
                <div class="space-y-1 pr-4">
                    <Label class="text-base font-bold" for="autoNext">{i18n.t('settings.player_section.autoplay')}</Label>
                    <p class="text-sm text-muted-foreground">{i18n.t('settings.player_section.autoplay_desc')}</p>
                </div>
                <Switch id="autoNext" bind:checked={playerConfig.autoplayNextEpisode} onCheckedChange={onSave} class="shrink-0" />
            </div>

            <div class="flex flex-col sm:flex-row sm:items-center justify-between gap-4 py-6 border-b border-border/40">
                <div class="space-y-1 pr-4">
                    <Label class="text-base font-bold" for="resumeFromLastPos">{i18n.t('settings.player_section.resume_playback')}</Label>
                    <p class="text-sm text-muted-foreground">{i18n.t('settings.player_section.resume_playback_desc')}</p>
                </div>
                <Switch id="resumeFromLastPos" bind:checked={playerConfig.resumeFromLastPos} onCheckedChange={onSave} class="shrink-0" />
            </div>

            <div class="flex flex-col sm:flex-row sm:items-center justify-between gap-4 py-6 border-b border-border/40">
                <div class="space-y-1 pr-4">
                    <Label class="text-base font-bold" for="autoSkipIntro">{i18n.t('settings.player_section.auto_skip_intro')}</Label>
                    <p class="text-sm text-muted-foreground">{i18n.t('settings.player_section.auto_skip_intro_desc')}</p>
                </div>
                <Switch id="autoSkipIntro" bind:checked={playerConfig.autoSkipIntro} onCheckedChange={onSave} class="shrink-0" />
            </div>

            <div class="flex flex-col sm:flex-row sm:items-center justify-between gap-4 py-6 border-b border-border/40">
                <div class="space-y-1 pr-4">
                    <Label class="text-base font-bold" for="autoSkipOutro">{i18n.t('settings.player_section.auto_skip_outro')}</Label>
                    <p class="text-sm text-muted-foreground">{i18n.t('settings.player_section.auto_skip_outro_desc')}</p>
                </div>
                <Switch id="autoSkipOutro" bind:checked={playerConfig.autoSkipOutro} onCheckedChange={onSave} class="shrink-0" />
            </div>

            <div class="grid grid-cols-1 md:grid-cols-2 gap-x-12 border-t border-border/40 mt-2">
                {#each keyboardShortcuts as shortcut}
                    <div class="flex items-center justify-between py-4 border-b border-border/40">
                        <div class="flex flex-col gap-0.5">
                            <span class="text-sm font-medium">{shortcut.label}</span>
                        </div>
                        <Kbd.Group>
                            {#each shortcut.keys as key, i}
                                <Kbd.Root>{key}</Kbd.Root>
                                {#if i < shortcut.keys.length - 1}
                                    <span class="text-xs text-muted-foreground/50 mx-1">/</span>
                                {/if}
                            {/each}
                        </Kbd.Group>
                    </div>
                {/each}
            </div>
        </Tabs.Content>

        <Tabs.Content value="player_mpv" class="focus-visible:outline-none mt-0">

            <div class="flex flex-col sm:flex-row sm:items-center justify-between gap-4 py-6 border-b border-border/40">
                <div class="space-y-1 pr-4">
                    <Label class="text-base font-bold" for="useMpv">{i18n.t("settings.player_section.use_mpv")}</Label>
                    <p class="text-sm text-muted-foreground">{i18n.t("settings.player_section.use_mpv_desc")}</p>
                </div>
                <Switch id="useMpv" bind:checked={mpvConfig.useMpv} onCheckedChange={onSave} class="shrink-0" />
            </div>

            {#if mpvConfig.useMpv && !isAndroid}
                <div class="flex flex-col sm:flex-row sm:items-center justify-between gap-4 py-6 border-b border-border/40">
                    <div class="space-y-1 pr-4">
                        <Label class="text-base font-bold" for="useHoshiConfig">{i18n.t("settings.player_section.let_hoshi")}</Label>
                        <p class="text-sm text-muted-foreground">
                            {i18n.t("settings.player_section.let_hoshi_desc")}
                        </p>
                    </div>
                    <Switch id="useHoshiConfig" bind:checked={mpvConfig.useHoshiConfig} onCheckedChange={onSave} class="shrink-0" />
                </div>

                {#if mpvConfig.useHoshiConfig}
                    <div class="py-6 border-b border-border/40 space-y-4">
                        <div class="space-y-1">
                            <Label class="text-base font-bold">{i18n.t("settings.player_section.osc")}</Label>
                            <p class="text-sm text-muted-foreground">
                                {i18n.t("settings.player_section.osc_desc")}
                            </p>
                        </div>

                        <div class="grid grid-cols-1 sm:grid-cols-3 gap-3">
                            {#each OSC_LIST as osc}
                                {@const isActive = mpvConfig.activeOsc === osc.id}
                                {@const isDownloading = downloadingOsc === osc.id}

                                <button
                                        class="relative text-left rounded-xl border p-4 transition-all focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-primary
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

                    <!-- Scripts -->
                    <div class="py-6 space-y-4">
                        <div class="space-y-1">
                            <Label class="text-base font-bold">{i18n.t("settings.player_section.scripts")}</Label>
                            <p class="text-sm text-muted-foreground">
                                {i18n.t("settings.player_section.scripts_desc")}
                            </p>
                        </div>

                        <div class="flex flex-col gap-2">
                            {#each KNOWN_SCRIPTS as script}
                                {@const isEnabled = mpvConfig.enabledScripts.includes(script.name)}
                                {@const isDownloading = downloadingScript === script.name}

                                <div class="flex items-center justify-between rounded-xl border border-border/40 bg-card px-4 py-3 gap-4">
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
                                            {isEnabled
                                                ? 'bg-primary/10 text-primary hover:bg-primary/20'
                                                : 'bg-muted text-muted-foreground hover:bg-muted/70'}
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
        </Tabs.Content>
    </Tabs.Root>
</div>