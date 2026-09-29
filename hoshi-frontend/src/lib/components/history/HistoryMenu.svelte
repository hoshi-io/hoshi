<script lang="ts">
    import { History } from 'lucide-svelte';
    import * as Popover from '$lib/components/ui/popover/index.js';
    import * as Drawer from '$lib/components/ui/drawer/index.js';
    import { layoutState } from '@/stores/layout.svelte.js';
    import { buttonVariants } from '$lib/components/ui/button';
    import { i18n } from '@/stores/i18n.svelte.js';
    import { cn } from '$lib/utils.js';
    import HistoryList from './HistoryList.svelte';

    let open = $state(false);

    const triggerClass = cn(
        buttonVariants({ variant: 'ghost', size: 'icon' }),
        'group size-10 rounded-sm text-muted-foreground transition-colors duration-150',
        'hover:bg-secondary hover:text-foreground',
        'data-[state=open]:bg-secondary data-[state=open]:text-foreground'
    );
</script>

{#if layoutState.isMobile}
    <Drawer.Root bind:open>
        <Drawer.Trigger class={triggerClass} aria-label={i18n.t('history.title')}>
            <History class="size-5" />
        </Drawer.Trigger>
        <Drawer.Content class="max-h-[85vh]">
            <HistoryList onNavigate={() => (open = false)} />
        </Drawer.Content>
    </Drawer.Root>
{:else}
    <Popover.Root bind:open>
        <Popover.Trigger class={triggerClass} aria-label={i18n.t('history.title')}>
            <History class="size-5 transition-transform duration-200 ease-out group-hover:-rotate-12" />
        </Popover.Trigger>

        <Popover.Content
                side="right"
                align="start"
                sideOffset={12}
                collisionPadding={16}
                class={cn(
                'flex flex-col p-0 rounded-sm border border-border bg-popover shadow-xl',
                'w-[min(26rem,calc(100vw-6rem))] h-[min(36rem,calc(100dvh-2rem))]',
                'data-[state=open]:duration-200 data-[state=closed]:duration-150',
                'data-[side=right]:slide-in-from-left-2'
            )}
        >
            <HistoryList onNavigate={() => (open = false)} />
        </Popover.Content>
    </Popover.Root>
{/if}