<script lang="ts">
    import * as Drawer  from "@/components/ui/drawer";
    import { layoutState } from "@/stores/layout.svelte.js";
    import type { PlayerController } from '../../PlayerController.svelte.js';
    import type { SubtitleSettings } from '../../subtitles/SubtitleSettings.svelte.js';
    import MenuContent from "@/components/player/settings/MenuContent.svelte";

    interface Props {
        ctrl:               PlayerController;
        subtitleSettings:   SubtitleSettings;
        open:               boolean;
        extensionItems:     { value: string; label: string }[];
        fullscreenEl: HTMLElement;
        selectedExtension:  string | null;
        servers:            string[];
        serverItems:        { value: string; label: string }[];
        selectedServer:     string | null;
        supportsDub:        boolean;
        isDub:              boolean;
        isLoadingPlay:      boolean;
        onExtensionChange:  (val: string) => void;
        onServerChange:     () => void;
        onDubChange:        (val: boolean) => void;
        onManageExtensions: () => void;
        onClose:            () => void;
    }

    let {
        ctrl,
        subtitleSettings,
        open,
        extensionItems,
        fullscreenEl,
        selectedExtension = $bindable(),
        servers,
        serverItems,
        selectedServer    = $bindable(),
        supportsDub,
        isDub             = $bindable(),
        isLoadingPlay,
        onExtensionChange,
        onServerChange,
        onDubChange,
        onManageExtensions,
        onClose,
    }: Props = $props();

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
                            bind:selectedExtension
                            bind:selectedServer
                            bind:isDub
                            {ctrl}
                            {subtitleSettings}
                            {extensionItems}
                            {servers}
                            {serverItems}
                            {supportsDub}
                            {isLoadingPlay}
                            {onExtensionChange}
                            {onServerChange}
                            {onDubChange}
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
                        bind:selectedExtension
                        bind:selectedServer
                        bind:isDub
                        {ctrl}
                        {subtitleSettings}
                        {extensionItems}
                        {servers}
                        {serverItems}
                        {supportsDub}
                        {isLoadingPlay}
                        {onExtensionChange}
                        {onServerChange}
                        {onDubChange}
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