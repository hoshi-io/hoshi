<script lang="ts">
    import * as Tabs from "$lib/components/ui/tabs";
    import { Switch } from "$lib/components/ui/switch";
    import { Label } from "$lib/components/ui/label";
    import { Input } from "$lib/components/ui/input";
    import type { MpvConfig, PlayerConfig } from "@/api/config/types";
    import { i18n } from "@/stores/i18n.svelte.js";
    import { Play, Captions, Cpu } from "lucide-svelte";
    import { platform } from "@tauri-apps/plugin-os";
    import ResponsiveSelect from "@/components/ResponsiveSelect.svelte";
    import * as Kbd from "$lib/components/ui/kbd";
    import PlayerSubtitleSettings from "@/components/settings/PlayerSubtitleSettings.svelte";
    import PlayerMpvSettings from "@/components/settings/PlayerMpvSettings.svelte";

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
</script>

<div class="space-y-6">
    <div>
        <h2 class="text-2xl font-bold tracking-tight">{i18n.t('settings.player')}</h2>
        <p class="text-sm text-muted-foreground mt-1">{i18n.t('settings.player_section.player_desc')}</p>
    </div>

    <Tabs.Root value="player_general" class="w-full">
        <Tabs.List class="grid w-full max-w-[550px] grid-cols-3 rounded-sm h-11 p-1 bg-muted/50">
            <Tabs.Trigger value="player_general" class="rounded-lg font-bold flex items-center gap-2">
                <Play class="size-4" /> {i18n.t("settings.general")}
            </Tabs.Trigger>
            <Tabs.Trigger value="player_subtitles" class="rounded-lg font-bold flex items-center gap-2">
                <Captions class="size-4" /> {i18n.t("player.subtitles")}
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
                    <Input bind:value={playerConfig.preferredSubLang} onchange={onSave} placeholder="en, es, ja" class="rounded-sm h-11" />
                </div>
            </div>

            <div class="flex flex-col sm:flex-row sm:items-center justify-between gap-4 py-6 border-b border-border/40">
                <div class="space-y-1 pr-4 flex-1">
                    <Label class="text-base font-bold">{i18n.t('settings.player_section.preferred_dub_lang')}</Label>
                    <p class="text-sm text-muted-foreground">{i18n.t('settings.player_section.preferred_dub_lang_desc')}</p>
                </div>
                <div class="w-full sm:max-w-md">
                    <Input bind:value={playerConfig.preferredDubLang} onchange={onSave} placeholder="ja, en" class="rounded-sm h-11" />
                </div>
            </div>

            <div class="flex flex-col sm:flex-row sm:items-center justify-between gap-4 py-6 border-b border-border/40">
                <div class="space-y-1 pr-4">
                    <Label class="text-base font-bold">{i18n.t('settings.player_section.seek_step')}</Label>
                    <p class="text-sm text-muted-foreground">{i18n.t('settings.player_section.seek_step_desc')}</p>
                </div>
                <ResponsiveSelect value={playerConfig.seekStep.toString()} items={seekSteps} class="rounded-sm h-11 w-full sm:max-w-md" onValueChange={handleSeekStepChange} />
            </div>

            <div class="flex flex-col sm:flex-row sm:items-center justify-between gap-4 py-6 border-b border-border/40">
                <div class="space-y-1 pr-4">
                    <Label class="text-base font-bold" for="autoNext">{i18n.t('settings.player_section.autoplay')}</Label>
                    <p class="text-sm text-muted-foreground">{i18n.t('settings.player_section.autoplay_desc')}</p>
                </div>
                <Switch id="autoNext" bind:checked={playerConfig.autoplayNextEpisode} onCheckedChange={onSave} class="shrink-0 scale-125" />
            </div>

            <div class="flex flex-col sm:flex-row sm:items-center justify-between gap-4 py-6 border-b border-border/40">
                <div class="space-y-1 pr-4">
                    <Label class="text-base font-bold" for="resumeFromLastPos">{i18n.t('settings.player_section.resume_playback')}</Label>
                    <p class="text-sm text-muted-foreground">{i18n.t('settings.player_section.resume_playback_desc')}</p>
                </div>
                <Switch id="resumeFromLastPos" bind:checked={playerConfig.resumeFromLastPos} onCheckedChange={onSave} class="shrink-0 scale-125" />
            </div>

            <div class="flex flex-col sm:flex-row sm:items-center justify-between gap-4 py-6 border-b border-border/40">
                <div class="space-y-1 pr-4">
                    <Label class="text-base font-bold" for="autoSkipIntro">{i18n.t('settings.player_section.auto_skip_intro')}</Label>
                    <p class="text-sm text-muted-foreground">{i18n.t('settings.player_section.auto_skip_intro_desc')}</p>
                </div>
                <Switch id="autoSkipIntro" bind:checked={playerConfig.autoSkipIntro} onCheckedChange={onSave} class="shrink-0 scale-125" />
            </div>

            <div class="flex flex-col sm:flex-row sm:items-center justify-between gap-4 py-6 border-b border-border/40">
                <div class="space-y-1 pr-4">
                    <Label class="text-base font-bold" for="autoSkipOutro">{i18n.t('settings.player_section.auto_skip_outro')}</Label>
                    <p class="text-sm text-muted-foreground">{i18n.t('settings.player_section.auto_skip_outro_desc')}</p>
                </div>
                <Switch id="autoSkipOutro" bind:checked={playerConfig.autoSkipOutro} onCheckedChange={onSave} class="shrink-0 scale-125" />
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

        <Tabs.Content value="player_subtitles" class="focus-visible:outline-none mt-0">
            <PlayerSubtitleSettings />
        </Tabs.Content>

        <Tabs.Content value="player_mpv" class="focus-visible:outline-none mt-0">
            <PlayerMpvSettings bind:mpvConfig {isAndroid} {onSave} />
        </Tabs.Content>
    </Tabs.Root>
</div>