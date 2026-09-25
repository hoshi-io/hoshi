<script lang="ts">
    import DesktopTitlebar from "@/components/layout/DesktopTitlebar.svelte";
    import {themeManager} from "@/stores/theme.svelte.js";
    themeManager;

    let { children } = $props();
</script>

<div class="relative flex h-screen w-screen flex-col overflow-hidden bg-background">
    <DesktopTitlebar title="Hoshi IDE" iconSrc="/128x128.png" />
    <div class="flex-1 overflow-hidden pt-8">
        {@render children()}
    </div>
</div>