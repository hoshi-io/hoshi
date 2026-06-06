<script lang="ts">
    import { Input } from "$lib/components/ui/input";
    import { Label } from "$lib/components/ui/label";
    import { Switch } from "$lib/components/ui/switch";
    import * as Select from "$lib/components/ui/select";
    import { Spinner } from "$lib/components/ui/spinner";

    import type { Extension } from "@/api/extensions/types";
    import { i18n } from "@/stores/i18n.svelte.js";

    let {
        ext,
        settings = $bindable(),
        isSaving,
        onSave
    }: {
        ext: Extension;
        settings: Record<string, any>;
        isSaving: boolean;
        onSave: () => void;
    } = $props();
</script>

{#if settings}
    <div class="space-y-4">
        {#each ext.setting_definitions as def}
            <div class="space-y-1.5">
                <Label class="text-sm font-bold">{def.label || def.key}</Label>

                {#if def.type === 'string'}
                    <Input
                            type="text"
                            value={settings[def.key]}
                            onchange={(e) => {
                                settings[def.key] = (e.target as HTMLInputElement).value;
                                onSave();
                            }}
                            placeholder={`${i18n.t('settings.extension_section.enter')} ${def.label || def.key}`}
                            class="rounded-lg bg-background"
                    />
                {:else if def.type === 'number'}
                    <Input
                            type="number"
                            value={settings[def.key]}
                            onchange={(e) => {
                                settings[def.key] = Number((e.target as HTMLInputElement).value);
                                onSave();
                            }}
                            class="rounded-lg bg-background"
                    />
                {:else if def.type === 'boolean'}
                    <div class="flex items-center space-x-2 mt-2">
                        <Switch
                                id={`switch-${ext.id}-${def.key}`}
                                checked={!!settings[def.key]}
                                onCheckedChange={(v) => {
                                    settings[def.key] = v;
                                    onSave();
                                }}
                        />
                        <Label for={`switch-${ext.id}-${def.key}`} class="text-sm font-medium text-muted-foreground cursor-pointer">
                            {i18n.t('settings.extension_section.enabled')}
                        </Label>
                    </div>
                {:else if def.type === 'select' && def.options}
                    <Select.Root
                            type="single"
                            value={String(settings[def.key] || '')}
                            onValueChange={(v) => {
                                settings[def.key] = v;
                                onSave();
                            }}
                    >
                        <Select.Trigger class="w-full bg-background rounded-lg border-border">
                            {def.options.find(o => o.value === String(settings[def.key]))?.label || i18n.t('settings.extension_section.select_option')}
                        </Select.Trigger>
                        <Select.Content>
                            {#each def.options as option}
                                <Select.Item value={String(option.value)}>{option.label}</Select.Item>
                            {/each}
                        </Select.Content>
                    </Select.Root>
                {:else if def.type === 'multiselect' && def.options}
                    <div class="flex flex-wrap gap-2 pt-1">
                        {#each def.options as option}
                            <button
                                    type="button"
                                    class="px-3 py-1.5 text-xs font-bold rounded-lg border transition-colors shadow-sm {(settings[def.key] || []).includes(option.value) ? 'bg-primary text-primary-foreground border-primary' : 'bg-background hover:bg-muted border-border/60'}"
                                    onclick={() => {
                                        let currentVals = settings[def.key] || [];
                                        if (currentVals.includes(option.value)) {
                                            settings[def.key] = currentVals.filter((v: string) => v !== option.value);
                                        } else {
                                            settings[def.key] = [...currentVals, option.value];
                                        }
                                        onSave();
                                    }}
                            >
                                {option.label}
                            </button>
                        {/each}
                    </div>
                {:else}
                    <p class="text-xs text-muted-foreground">{i18n.t('settings.extension_section.unsupported_type')} {def.type}</p>
                {/if}
            </div>
        {/each}

        {#if isSaving}
            <div class="pt-2 flex justify-end items-center text-xs text-muted-foreground gap-1.5 animate-in fade-in duration-150">
                <Spinner class="h-3 w-3 animate-spin" /> Saving changes...
            </div>
        {/if}
    </div>
{/if}