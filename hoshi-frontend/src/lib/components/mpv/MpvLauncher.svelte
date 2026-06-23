<script lang="ts">
    import { untrack } from 'svelte';
    import { type } from '@tauri-apps/plugin-os';
    import { extensions } from '@/stores/extensions.svelte.js';
    import { layoutState } from '@/stores/layout.svelte.js';
    import { appConfig } from '@/stores/config.svelte.js';
    import MpvDialog from '@/components/mpv/MpvDialog.svelte';

    let {
        cid,
        epNumber,
        animeTitle,
        epTitle,
        totalEpisodes,
        isNsfw,
        coverImage,
        startTime,
        open = $bindable(),
    }: {
        cid: string;
        epNumber: number;
        animeTitle: string;
        epTitle: string;
        totalEpisodes: number;
        isNsfw: boolean;
        coverImage?: string | null;
        startTime?: number;
        open: boolean;
    } = $props();

    const osType = type();
    const isDesktop = osType !== 'android' && osType !== 'ios';

    if (isDesktop) {
        untrack(() => {
            const extId = layoutState.mpv?.extId ?? extensions.anime[0]?.id;

            if (extId) {
                layoutState.mpv = {
                    cid,
                    epNumber,
                    extId,
                    isDub: false,
                    animeTitle,
                    epTitle,
                    totalEpisodes: totalEpisodes,
                    isNsfw: isNsfw,
                    coverImage: coverImage || "",
                };

                open = false;

                extensions.playWithMpv(cid, epNumber, extId, {
                    animeTitle,
                    episodeTitle: epTitle,
                    totalEpisodes,
                    isNsfw,
                    coverImage,
                    startTime,
                    autoUpdateProgress: appConfig.data?.content?.autoUpdateProgress ?? true,
                    use_hoshi_config: appConfig.data?.mpv?.useHoshiConfig ?? false,
                }).catch(err => {
                    console.error("Failed to launch MPV:", err);
                    layoutState.mpv = null;
                });
            }
        });
    }
</script>

{#if !isDesktop}
    <MpvDialog bind:open {cid} {epNumber} {animeTitle} {epTitle} />
{/if}