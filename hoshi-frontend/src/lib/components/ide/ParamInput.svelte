<script lang="ts">
    import { Input } from '@/components/ui/input';
    import { Checkbox } from '@/components/ui/checkbox';
    import { Label } from '@/components/ui/label';
    import type { ParamDef } from './function-signatures.ts';

    let { def, value = $bindable() }: { def: ParamDef; value: unknown } = $props();
</script>

<div class="space-y-1.5 group">
    <Label class="text-xs font-medium text-muted-foreground group-focus-within:text-primary transition-colors">
        {def.name}{def.optional ? ' (optional)' : ''}
    </Label>

    {#if def.type === 'string'}
        <Input bind:value={value as string} placeholder={def.name} class="rounded-sm shadow-sm transition-all focus-visible:ring-1" />
    {:else if def.type === 'number'}
        <Input type="number" bind:value={value as number} class="rounded-sm shadow-sm transition-all focus-visible:ring-1" />
    {:else if def.type === 'boolean'}
        <div class="pt-1">
            <Checkbox bind:checked={value as boolean} class="rounded-sm data-[state=checked]:bg-primary" />
        </div>
    {:else if def.type === 'json'}
        <!-- Fixed to use rounded-sm and standard focus rings[cite: 5] -->
        <textarea
                class="w-full rounded-sm border border-input bg-background px-3 py-2 text-sm shadow-sm font-mono placeholder:text-muted-foreground focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-ring disabled:cursor-not-allowed disabled:opacity-50 transition-all"
                rows="4"
                bind:value={value as string}
                placeholder="{'{\n  \"key\": \"value\"\n}'}"
        ></textarea>
    {/if}
</div>