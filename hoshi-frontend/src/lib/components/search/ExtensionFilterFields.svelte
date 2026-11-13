<script lang="ts">
    import { Label } from '$lib/components/ui/label';
    import { Switch } from '$lib/components/ui/switch';
    import ResponsiveSelect from '@/components/ResponsiveSelect.svelte';
    import { i18n } from '@/stores/i18n.svelte.js';
    import { formatLabel } from '@/utils/search.js';

    let {
        schema,
        values = $bindable(),
        onchange,
    }: {
        schema: Record<string, any>;
        values: Record<string, any>;
        onchange?: () => void;
    } = $props();

    function toggleMulti(key: string, value: string) {
        const current: string[] = values[key] ?? [];
        values[key] = current.includes(value) ? current.filter(v => v !== value) : [...current, value];
        onchange?.();
    }
</script>

<div class="space-y-4">
    {#each Object.entries(schema) as [key, filterDef] (key)}
        <div class="space-y-2.5">
            {#if filterDef.type === 'select'}
                <Label class="text-sm font-bold text-foreground/90">{filterDef.label || formatLabel(key)}</Label>
                <ResponsiveSelect
                        bind:value={values[key]}
                        items={[
                            { value: '', label: i18n.t('search.any_genre') },
                            ...(filterDef.options || []).map((opt: any) => ({ value: opt.value, label: opt.label })),
                        ]}
                        onValueChange={() => onchange?.()}
                        placeholder={i18n.t('search.any_genre')}
                        class="bg-muted/20 border-none h-11 rounded-sm font-semibold"
                />
            {:else if filterDef.type === 'multiselect'}
                <div class="space-y-3">
                    <Label class="text-sm font-bold text-foreground/90">{filterDef.label || formatLabel(key)}</Label>
                    <div class="flex flex-wrap gap-2 max-h-36 overflow-y-auto p-2 rounded-sm border border-border/40 bg-muted/5 [scrollbar-width:thin]">
                        {#each filterDef.options || [] as opt (opt.value)}
                            {@const isSelected = values[key]?.includes(opt.value)}
                            <button
                                    type="button"
                                    class="px-3 py-1.5 rounded-sm text-xs font-bold transition-all border {isSelected
                                        ? 'bg-primary text-primary-foreground border-primary'
                                        : 'bg-muted/20 text-foreground/70 border-transparent hover:border-border'}"
                                    onclick={() => toggleMulti(key, opt.value)}
                            >
                                {opt.label}
                                {#if isSelected}<span class="ml-1 opacity-70">✕</span>{/if}
                            </button>
                        {/each}
                    </div>
                </div>
            {:else if filterDef.type === 'boolean'}
                <div class="flex items-center space-x-3 pt-2">
                    <Switch id={`filter-${key}`} bind:checked={values[key]} onCheckedChange={() => onchange?.()} />
                    <Label for={`filter-${key}`} class="text-sm font-bold text-foreground/90">{filterDef.label || formatLabel(key)}</Label>
                </div>
            {/if}
        </div>
    {/each}
</div>
