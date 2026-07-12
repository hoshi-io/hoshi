<script lang="ts">
    import * as Drawer  from "@/components/ui/drawer";
    import { layoutState } from "@/stores/layout.svelte.js";
    import type { PlayerController } from '../../PlayerController.svelte.js';
    import type { PlayerState } from "@/app/watch.svelte.js";
    import type { SubtitleSettings } from '../../subtitles/SubtitleSettings.svelte.js';
    import MenuContent from "@/components/player/settings/MenuContent.svelte";

    interface Props {
        ctrl:               PlayerController;
        playerState:        PlayerState;
        subtitleSettings:   SubtitleSettings;
        open:               boolean;
        fullscreenEl:       HTMLElement;
        onManageExtensions: () => void;
        onClose:            () => void;
    }

    let { ctrl, playerState, subtitleSettings, open, fullscreenEl, onManageExtensions, onClose }: Props = $props();

    let isMobile = layoutState.isMobile;

    let menuContent: MenuContent;
    $effect(() => { if (!open) menuContent?.resetSection(); });
</script>

{#if isMobile && fullscreenEl}
    <Drawer.Root
            open={open}
            onOpenChange={(v) => { if (!v) onClose(); }}
    >
        <Drawer.Portal to={fullscreenEl}>
            <Drawer.Overlay class="absolute inset-0 z-[70] bg-black/40 backdrop-blur-sm" />

            <Drawer.Content
                    class="absolute bottom-0 left-0 right-0 mx-auto z-[70]
                       w-full sm:w-[60%] rounded-t-xl
                       bg-popover/90 backdrop-blur-xl border border-border border-b-0
                       shadow-2xl font-sans focus:outline-none
                       [&>[data-vaul-drag-handle]]:hidden"
            >
                <div class="px-2 pt-1 pb-safe-or-8">
                    <MenuContent
                            bind:this={menuContent}
                            {ctrl}
                            {playerState}
                            {subtitleSettings}
                            {onManageExtensions}
                            {onClose}
                    />
                </div>
            </Drawer.Content>
        </Drawer.Portal>
    </Drawer.Root>
{:else}
    {#if open}
        <div
                class="settings-panel absolute bottom-full right-0 mb-2 w-72
                   rounded-sm border border-border bg-popover/95 backdrop-blur-xl
                   shadow-2xl font-sans overflow-hidden z-[70]"
                onclick={(e) => e.stopPropagation()}
        >
            <div class="px-1 py-1">
                <MenuContent
                        bind:this={menuContent}
                        {ctrl}
                        {playerState}
                        {subtitleSettings}
                        {onManageExtensions}
                        {onClose}
                />
            </div>
        </div>
    {/if}
{/if}

<style>
    .pb-safe-or-8 {
        padding-bottom: max(2rem, env(safe-area-inset-bottom));
    }
</style>