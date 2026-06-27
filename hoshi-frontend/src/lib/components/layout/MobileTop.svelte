<script lang="ts">
    import { auth } from '@/stores/auth.svelte.js';
    import { ChevronLeft } from 'lucide-svelte';
    import { goto } from '$app/navigation';
    import { layoutState } from '@/stores/layout.svelte.js';
    import * as Avatar from '$lib/components/ui/avatar';
    import { Button } from '$lib/components/ui/button';
    import HistoryMenu from '@/components/history/HistoryMenu.svelte';

    let { showSwitchProfileModal = $bindable(false) } = $props();

    function handleBack() {
        if (layoutState.backUrl) {
            goto(layoutState.backUrl);
        } else {
            window.history.back();
        }
    }
</script>

<header class="lg:hidden flex items-center justify-between border-b border-border bg-background/90 backdrop-blur-md px-4 pb-3 pt-safe gap-3 shrink-0">
    <div class="flex items-center gap-2 sm:gap-3 shrink-0">
        {#if layoutState.showBack}
            <Button variant="ghost" size="icon" class="h-8 w-8 shrink-0 -ml-2 rounded-full" onclick={handleBack}>
                <ChevronLeft class="size-5" />
            </Button>
        {:else}
            <img src="/128x128.png" alt="App Logo" class="h-7 w-7 object-contain opacity-90 pointer-events-none rounded-md" />
        {/if}

        {#if !layoutState.headerAction && layoutState.title}
			<span class="text-lg font-bold tracking-tight text-foreground capitalize truncate">
				{layoutState.title}
			</span>
        {/if}
    </div>

    <div class="flex-1 flex justify-end items-center min-w-0">
        {#if layoutState.headerAction}
            {@render layoutState.headerAction()}
        {/if}
    </div>

    <HistoryMenu />

    {#if auth.user}
        <button onclick={() => showSwitchProfileModal = true} class="shrink-0">
            <Avatar.Root class="size-8 border border-border hover:border-foreground/40 transition-colors active:scale-95">
                <Avatar.Image src={auth.user.avatar} alt={auth.user.username} class="object-cover" />
                <Avatar.Fallback class="bg-muted text-xs font-medium text-muted-foreground">
                    {auth.user.username[0].toUpperCase()}
                </Avatar.Fallback>
            </Avatar.Root>
        </button>
    {/if}
</header>

<style>
    .pt-safe {
        padding-top: calc(max(env(safe-area-inset-top, 0px), 1.5rem) + 0.75rem);
    }
</style>