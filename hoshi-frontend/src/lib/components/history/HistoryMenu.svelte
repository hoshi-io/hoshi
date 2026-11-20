<script lang="ts">
    import ClockCounterClockwiseIcon from 'phosphor-svelte/lib/ClockCounterClockwiseIcon';
    import * as Popover from '@/components/ui/popover/index';
    import * as Drawer from '@/components/ui/drawer/index';
    import { layoutState } from '@/stores/layout.svelte.js';
    import { buttonVariants } from '@/components/ui/button';
    import { i18n } from '@/stores/i18n.svelte.js';
    import { cn } from '@/utils/cn';
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
            <ClockCounterClockwiseIcon class="size-5" />
        </Drawer.Trigger>
        <Drawer.Content class="max-h-[85vh]">
            <HistoryList onNavigate={() => (open = false)} />
        </Drawer.Content>
    </Drawer.Root>
{:else}
    <Popover.Root bind:open>
        <Popover.Trigger class={triggerClass} aria-label={i18n.t('history.title')}>
            <ClockCounterClockwiseIcon class="size-5 transition-transform duration-200 ease-out group-hover:-rotate-12" />
        </Popover.Trigger>

        <Popover.Content
                side="right"
                align="start"
                sideOffset={16}
                collisionPadding={16}
                onOpenAutoFocus={(e) => e.preventDefault()}
                class={cn(
                'flex flex-col overflow-hidden p-0 rounded-sm border border-border/50 bg-popover shadow-xl',
                'w-72 max-h-[min(30rem,calc(100dvh-2rem))]',
                'data-[state=open]:duration-200 data-[state=closed]:duration-150',
                'data-[side=right]:slide-in-from-left-2'
            )}
        >
            <HistoryList compact onNavigate={() => (open = false)} />
        </Popover.Content>
    </Popover.Root>
{/if}