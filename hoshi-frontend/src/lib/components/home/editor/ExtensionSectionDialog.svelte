<script lang="ts">
    import { untrack } from 'svelte';
    import * as Dialog from '@/components/ui/dialog';
    import * as Drawer from '@/components/ui/drawer';
    import { layoutState } from '@/stores/layout.svelte.js';
    import { Button } from '@/components/ui/button';
    import { Spinner } from '@/components/ui/spinner';
    import ExtensionFilterFields from '@/components/search/ExtensionFilterFields.svelte';
    import { extensionsApi } from '@/api/extensions/extensions';
    import { homeEditor } from '@/app/home-editor.svelte.js';
    import { labelText } from '@/app/home.svelte.js';
    import { i18n } from '@/stores/i18n.svelte.js';
    import type { HomeSection, HomeTab } from '@/api/home/types';

    let { open = $bindable(false), tab, section }: { open: boolean; tab: HomeTab; section: HomeSection } = $props();

    let schema = $state<Record<string, any>>({});
    let values = $state<Record<string, any>>({});
    let text = $state('');
    let loading = $state(false);
    let failed = $state(false);

    const extensionId = $derived(
        section.source.kind === 'search' && section.source.query.kind === 'extension' ? section.source.query.extension : null
    );

    $effect(() => {
        if (open && extensionId) { const id = extensionId; untrack(() => load(id)); }
    });

    async function load(extId: string) {
        const q = section.source.kind === 'search' && section.source.query.kind === 'extension' ? section.source.query : null;
        text = q?.query ?? '';
        loading = true;
        failed = false;
        try {
            schema = (await extensionsApi.getFilters(extId)).filters as Record<string, any> ?? {};
            // Keep only saved values the extension still understands.
            const saved = (q?.filters ?? {}) as Record<string, any>;
            const next: Record<string, any> = {};
            for (const [key, def] of Object.entries(schema)) {
                if (key in saved) next[key] = saved[key];
                else if (def?.type === 'boolean') next[key] = false;
            }
            values = next;
        } catch {
            schema = {};
            failed = true;
        } finally {
            loading = false;
        }
    }

    function clean(v: Record<string, any>) {
        return Object.fromEntries(Object.entries(v).filter(([, x]) => {
            if (Array.isArray(x)) return x.length > 0;
            if (typeof x === 'string') return x.trim() !== '';
            return x !== null && x !== undefined && x !== false;
        }));
    }

    function clear() {
        text = '';
        values = Object.fromEntries(Object.entries(schema).filter(([, d]) => d?.type === 'boolean').map(([k]) => [k, false]));
    }

    function save() {
        const filters = clean(values);
        homeEditor.setExtensionQuery(
            tab.id, section.id,
            text.trim() || null,
            Object.keys(filters).length > 0 ? filters : null,
        );
        open = false;
    }
</script>

{#snippet body()}
    <div class="space-y-2.5">
        <label for="ext-section-query" class="text-sm font-bold text-foreground/90">{i18n.t('home.edit.search_text')}</label>
        <input
                id="ext-section-query"
                bind:value={text}
                placeholder={i18n.t('home.edit.search_text_placeholder')}
                class="h-11 w-full rounded-sm border border-input bg-background px-3 text-sm outline-none transition-colors focus:border-ring"
        />
    </div>

    {#if loading}
        <div class="flex justify-center py-6"><Spinner class="size-6" /></div>
    {:else if failed}
        <p class="text-sm text-destructive">{i18n.t('home.edit.filters_failed')}</p>
    {:else if Object.keys(schema).length > 0}
        <ExtensionFilterFields {schema} bind:values />
    {:else}
        <p class="text-sm text-muted-foreground">{i18n.t('home.edit.no_filters')}</p>
    {/if}
{/snippet}

{#snippet actions()}
    <Button variant="ghost" onclick={clear}>{i18n.t('search.clear_filters')}</Button>
    <div class="flex gap-2">
        <Button variant="outline" onclick={() => open = false}>{i18n.t('home.edit.cancel')}</Button>
        <Button onclick={save} disabled={loading}>{i18n.t('home.edit.apply')}</Button>
    </div>
{/snippet}

{#if layoutState.isMobile}
    <Drawer.Root bind:open>
        <Drawer.Content class="max-h-[90dvh]">
            <Drawer.Header class="text-left">
                <Drawer.Title class="text-xl font-bold tracking-tight">{labelText(section.title)}</Drawer.Title>
                <Drawer.Description>{i18n.t('home.edit.filters_desc')}</Drawer.Description>
            </Drawer.Header>
            <div class="min-h-0 flex-1 space-y-6 overflow-y-auto px-4 pb-4">
                {@render body()}
            </div>
            <Drawer.Footer class="flex-row items-center justify-between gap-2 border-t border-border/40 pb-safe">
                {@render actions()}
            </Drawer.Footer>
        </Drawer.Content>
    </Drawer.Root>
{:else}
    <Dialog.Root bind:open>
        <Dialog.Content class="flex max-h-[85vh] w-[95vw] flex-col gap-0 p-0 sm:max-w-lg">
            <Dialog.Header class="border-b border-border/40 p-6 pb-4">
                <Dialog.Title class="text-xl font-bold tracking-tight">{labelText(section.title)}</Dialog.Title>
                <Dialog.Description>{i18n.t('home.edit.filters_desc')}</Dialog.Description>
            </Dialog.Header>
            <div class="min-h-0 flex-1 space-y-6 overflow-y-auto p-6">
                {@render body()}
            </div>
            <Dialog.Footer class="flex-row justify-between gap-2 border-t border-border/40 p-4 sm:justify-between">
                {@render actions()}
            </Dialog.Footer>
        </Dialog.Content>
    </Dialog.Root>
{/if}
