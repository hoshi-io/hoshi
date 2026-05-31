<script lang="ts">
    import { PlayerState } from "@/app/watch.svelte";
    import Player from "@/components/player/Player.svelte";
    import { primaryMetadata } from "$lib/api/content/types";
    import { appConfig } from "@/stores/config.svelte";
    import ExtensionManager from "@/components/modals/ExtensionManager.svelte";
    import {onDestroy, onMount} from "svelte";

    const playerState = new PlayerState();

    let playerEl = $state<ReturnType<typeof Player> | null>(null);
    let showExtensionManager = $state(false);

    function handlePopState() {
        playerState.destroy();
    }

    onMount(() => {
        window.addEventListener("popstate", handlePopState);
    });

    onDestroy(() => {
        window.removeEventListener("popstate", handlePopState);
        playerState.destroy();
    });
</script>

<svelte:head>
    <title>{playerState.episodeTitle} - {playerState.animeTitle}</title>
</svelte:head>

<div class="flex flex-col h-[100dvh] w-full bg-black relative">
    <div class="w-full h-full lg:absolute lg:inset-0 bg-black flex items-center justify-center relative z-10 shrink-0 shadow-lg lg:shadow-none"
         style="padding-left: env(safe-area-inset-left); padding-right: env(safe-area-inset-right);">
        <Player
                bind:this={playerEl}
                playerState={playerState}
                onPlay={() => playerState.onPlay(playerEl)}
                onManageExtensions={() => showExtensionManager = true}
        />
    </div>
</div>

{#if playerState.animeData}
    <ExtensionManager
            bind:open={showExtensionManager}
            cid={playerState.cid}
            metadata={primaryMetadata(playerState.animeData, appConfig.data?.content?.preferredMetadataProvider)}
            isNsfw={playerState.animeData.content?.nsfw ?? false}
            extensions={playerState.animeData.extensionSources ?? []}
            contentType="anime"
            onSuccess={() => playerState.loadPageData(playerState.cid, playerState.epNumber)}
    />
{/if}