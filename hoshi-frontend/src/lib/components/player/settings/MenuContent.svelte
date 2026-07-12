<script lang="ts">
    import { Gauge, AudioLines, Captions, PuzzleIcon, Mic2, Server, Settings2 } from 'lucide-svelte';
    import { Switch } from "@/components/ui/switch";
    import type { PlayerController } from '../PlayerController.svelte.js';
    import type { PlayerState } from "@/app/watch.svelte.js";
    import { type SubtitleSettings } from '../subtitles/SubtitleSettings.svelte.js';
    import { appConfig } from "@/stores/config.svelte.js";
    import { i18n } from "@/stores/i18n.svelte.js";
    import Row from './Row.svelte';
    import OptionList from './OptionList.svelte';
    import SubAppearanceSection from '@/components/player/settings/SubAppearanceSection.svelte';

    interface Props {
        ctrl:               PlayerController;
        playerState:        PlayerState;
        subtitleSettings:   SubtitleSettings;
        onManageExtensions: () => void;
        onClose:            () => void;
    }

    let { ctrl, playerState, subtitleSettings, onManageExtensions, onClose }: Props = $props();

    let activeSection = $state<string | null>(null);

    export function resetSection() { activeSection = null; }

    $effect(() => {
        const subPref = appConfig.data?.player?.preferredSubLang;
        if (subPref && ctrl.subtitleTracks.length > 0 && !ctrl.subAutoApplied) {
            ctrl.subAutoApplied = true;
            const id = findPreferred(ctrl.subtitleTracks, subPref);
            if (id !== null) ctrl.setSubtitleTrack(String(id));
        }
    });

    $effect(() => {
        const audioPref = appConfig.data?.player?.preferredDubLang;
        if (audioPref && ctrl.audioTracks.length > 1 && !ctrl.audioAutoApplied) {
            ctrl.audioAutoApplied = true;
            const id = findPreferred(ctrl.audioTracks, audioPref);
            if (id !== null) ctrl.setAudioTrack(Number(id));
        }
    });

    function findPreferred(
        tracks: { id: string | number; srclang?: string; label: string }[],
        preference: string
    ): string | number | null {
        const langs = preference.split(',').map(l => l.trim().toLowerCase());
        for (const lang of langs) {
            const match = tracks.find(t =>
                t.srclang?.toLowerCase() === lang ||
                t.label.toLowerCase() === lang
            );
            if (match) return match.id;
        }
        return null;
    }

    const allSections = $derived([
        {
            id:       'extension',
            label:    i18n.t('watch.select_extension'),
            icon:     PuzzleIcon,
            options:  playerState.extensionItems.map(i => ({ id: i.value, label: i.label })),
            current:  playerState.selectedExtension ?? '',
            onSelect: (id: string) => playerState.selectExtension(id),
            visible:  true,
            isExtGroup: true,
        },
        {
            id:       'server',
            label:    i18n.t('watch.server'),
            icon:     Server,
            options:  playerState.serverItems.map(i => ({ id: i.value, label: i.label })),
            current:  playerState.selectedServer ?? '',
            onSelect: (id: string) => { playerState.selectedServer = id; playerState.loadPlay(); },
            visible:  playerState.servers.length > 1,
            isExtGroup: true,
        },
        {
            id:       'quality',
            label:    i18n.t('player.quality'),
            icon:     Gauge,
            options:  ctrl.qualityLevels,
            current:  ctrl.currentQuality,
            onSelect: (id: string) => ctrl.setQuality(id),
            visible:  ctrl.qualityLevels.length > 1,
            isExtGroup: false,
        },
        {
            id:       'audio',
            label:    i18n.t('player.audio'),
            icon:     AudioLines,
            options:  ctrl.audioTracks,
            current:  ctrl.currentAudio,
            onSelect: (id: string) => ctrl.setAudioTrack(id),
            visible:  ctrl.audioTracks.length > 1,
            isExtGroup: false,
        },
    ]);

    const visibleSections  = $derived(allSections.filter(s => s.visible));
    const visibleExtGroup  = $derived(visibleSections.filter(s => s.isExtGroup));
    const visiblePlayGroup = $derived(visibleSections.filter(s => !s.isExtGroup));
    const showDivider      = $derived(visibleExtGroup.length > 0 && visiblePlayGroup.length > 0);
    const currentSection   = $derived(allSections.find(s => s.id === activeSection) ?? null);
    const showSubtitles    = $derived(activeSection === 'subtitles');

    const currentSubtitleLabel = $derived(
        ctrl.currentSubtitle === '-1'
            ? i18n.t('player.cc_off')
            : ctrl.subtitleTracks.find(t => t.id === ctrl.currentSubtitle)?.label ?? ''
    );
</script>

{#if showSubtitles}
    <SubAppearanceSection
            settings={subtitleSettings}
            tracks={ctrl.subtitleTracks}
            currentTrack={ctrl.currentSubtitle}
            onSelectTrack={(id) => ctrl.setSubtitleTrack(id)}
            onBack={() => activeSection = null}
    />

{:else if currentSection === null}
    <div class="flex flex-col py-1">

        {#each visibleExtGroup as section (section.id)}
            {@const label = section.options.find(o => o.id === section.current)?.label ?? ''}
            <Row
                    icon={section.icon}
                    label={section.label}
                    value={label}
                    onclick={() => activeSection = section.id}
            />
        {/each}

        {#if playerState.supportsDub}
            <Row
                    icon={Mic2}
                    label={i18n.t('watch.dub')}
                    onclick={() => { playerState.isDub = !playerState.isDub; playerState.loadPlay(); }}
            >
                {#snippet children()}
                    <Switch
                            checked={playerState.isDub}
                            disabled={playerState.isLoadingPlay}
                            class="pointer-events-none"
                    />
                {/snippet}
            </Row>
        {/if}

        <Row
                icon={Settings2}
                label={i18n.t('content.extension_manager.manage_extensions_title')}
                onclick={() => { onClose(); onManageExtensions(); }}
        />

        {#if showDivider}
            <div class="divider"></div>
        {/if}

        {#each visiblePlayGroup as section (section.id)}
            {@const label = section.options.find(o => o.id === section.current)?.label ?? ''}
            <Row
                    icon={section.icon}
                    label={section.label}
                    value={label}
                    onclick={() => activeSection = section.id}
            />
        {/each}

        {#if ctrl.subtitleTracks.length > 0}
            <Row
                    icon={Captions}
                    label="Subtitles"
                    value={currentSubtitleLabel}
                    onclick={() => activeSection = 'subtitles'}
            />
        {/if}
    </div>

{:else}
    <OptionList
            label={currentSection.label}
            options={currentSection.options}
            current={currentSection.current}
            onSelect={currentSection.onSelect}
            onBack={() => activeSection = null}
    />
{/if}

<style>
    .divider {
        height:     1px;
        background: hsl(var(--border));
        margin:     0.25rem 0.75rem;
        opacity:    0.6;
    }
</style>