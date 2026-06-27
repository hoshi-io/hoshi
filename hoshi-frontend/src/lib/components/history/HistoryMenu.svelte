<script lang="ts">
    import { History } from 'lucide-svelte';
    import * as Popover from '$lib/components/ui/popover/index.js';
    import * as Drawer from '$lib/components/ui/drawer/index.js';
    import { layoutState } from '@/stores/layout.svelte.js';
    import { buttonVariants } from '$lib/components/ui/button';
    import { cn } from '$lib/utils.js';
    import HistoryList from './HistoryList.svelte';

    let open = $state(false);
</script>

{#if layoutState.isMobile}
    <Drawer.Root bind:open>
        <Drawer.Trigger
                class={cn(buttonVariants({ variant: 'ghost', size: 'icon' }), 'rounded-sm')}
        >
            <History class="size-[18px]" />
        </Drawer.Trigger>
        <Drawer.Content>
            <HistoryList onNavigate={() => (open = false)} />
        </Drawer.Content>
    </Drawer.Root>
{:else}
    <Popover.Root bind:open>
        <Popover.Trigger
                class="group size-10 rounded-lg inline-flex items-center justify-center text-muted-foreground hover:bg-secondary hover:text-foreground transition-colors duration-100 data-[state=open]:bg-secondary data-[state=open]:text-foreground"
                aria-label="View history"
        >
            <History
                    class="shrink-0 size-5 transition-transform duration-150 ease-out group-hover:scale-105"
            />
        </Popover.Trigger>

        <Popover.Content
                class="w-80 p-0 shadow-xl border-l-0 rounded-r-xl border-y border-r border-border bg-background/95 backdrop-blur-md transition-all duration-75 data-[state=open]:animate-in data-[state=closed]:animate-out data-[state=open]:fade-in-50 data-[side=right]:slide-in-from-left-2"
                side="right"
                align="start"
                sideOffset={8}
        >
            <HistoryList onNavigate={() => (open = false)} />
        </Popover.Content>
    </Popover.Root>
{/if}