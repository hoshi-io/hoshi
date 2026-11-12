<script lang="ts">
    import { Button } from '@/components/ui/button';
    import { i18n } from '@/stores/i18n.svelte.js';
    import type { SectionLayout } from '@/api/home/types';

    let { title, message, layout = 'carousel', onretry }: {
        title: string;
        message: string;
        layout?: SectionLayout;
        onretry?: () => void;
    } = $props();
</script>

<section class="space-y-1 md:space-y-0">
    <div class="px-1">
        <h2 class="text-lg font-bold tracking-tight text-foreground md:text-2xl">{title}</h2>
    </div>

    <div class="relative overflow-hidden px-1 md:-mx-5 md:px-6">
        {#if layout === 'strip'}
            <div class="grid grid-cols-1 gap-2 py-2 sm:grid-cols-2 lg:grid-cols-3" aria-hidden="true">
                {#each Array(6) as _}
                    <div class="flex items-center gap-3 p-1.5">
                        <div class="aspect-[2/3] w-12 shrink-0 rounded-sm bg-muted/30"></div>
                        <div class="flex-1 space-y-2">
                            <div class="h-3 w-2/3 rounded bg-muted/30"></div>
                            <div class="h-2.5 w-1/3 rounded bg-muted/20"></div>
                        </div>
                    </div>
                {/each}
            </div>
        {:else}
            <div class="-ml-3 flex py-2 md:-ml-5 md:py-6" aria-hidden="true">
                {#each Array(7) as _}
                    <div class="shrink-0 pl-3 md:pl-5 {layout === 'ranked'
                        ? 'basis-[55%] sm:basis-[40%] md:basis-[30%] lg:basis-1/4 xl:basis-1/5 2xl:basis-[18%]'
                        : 'basis-[42%] sm:basis-1/3 md:basis-1/4 lg:basis-1/5 xl:basis-1/6 2xl:basis-[14%]'}">
                        <div class={layout === 'ranked' ? 'pl-9 md:pl-12' : ''}>
                            <div class="aspect-[2/3] w-full rounded-sm bg-muted/25"></div>
                            <div class="mt-2 h-3 w-3/4 rounded bg-muted/20"></div>
                        </div>
                    </div>
                {/each}
            </div>
        {/if}

        <div class="absolute inset-0 flex flex-col items-center justify-center gap-3 bg-gradient-to-r from-background/40 via-background/80 to-background/40 px-6 text-center">
            <p class="max-w-md text-sm text-muted-foreground">{message}</p>
            {#if onretry}
                <Button variant="outline" size="sm" onclick={onretry}>{i18n.t('home.try_again')}</Button>
            {/if}
        </div>
    </div>
</section>
