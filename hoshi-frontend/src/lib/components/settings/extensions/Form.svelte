<script lang="ts">
    import { Input } from "$lib/components/ui/input";
    import { Label } from "$lib/components/ui/label";
    import { Switch } from "$lib/components/ui/switch";
    import * as Select from "$lib/components/ui/select";
    import { Spinner } from "$lib/components/ui/spinner";

    import type { CardExt } from "@/app/marketplace.svelte.js";
    import { i18n } from "@/stores/i18n.svelte.js";

    let {
        ext,
        settings = $bindable(),
        isSaving = false,
        onChange
    }: {
        ext: CardExt;
        settings: Record<string, any>;
        isSaving?: boolean;
        onChange: () => void;
    } = $props();

    const fieldId = (key: string) => `ext-${ext.id}-${key}`;
</script>

{#if settings}
    <div class="space-y-4">
        <div class="grid gap-x-4 gap-y-4 sm:grid-cols-2">
            {#each ext.setting_definitions ?? [] as def (def.key)}
                {@const label = def.label || def.key}

                {#if def.type === 'boolean'}
                    <div class="flex items-center justify-between gap-3 rounded-lg border border-border/50 bg-background px-3 py-2.5">
                        <Label for={fieldId(def.key)} class="cursor-pointer text-sm font-bold">{label}</Label>
                        <Switch
                                id={fieldId(def.key)}
                                checked={!!settings[def.key]}
                                onCheckedChange={(v) => {
                                    settings[def.key] = v;
                                    onChange();
                                }}
                        />
                    </div>

                {:else}
                    <div class="space-y-1.5 {def.type === 'multiselect' ? 'sm:col-span-2' : ''}">
                        <Label for={fieldId(def.key)} class="text-sm font-bold">{label}</Label>

                        {#if def.type === 'string'}
                            <Input
                                    id={fieldId(def.key)}
                                    type="text"
                                    value={settings[def.key]}
                                    onchange={(e) => {
                                        settings[def.key] = (e.target as HTMLInputElement).value;
                                        onChange();
                                    }}
                                    placeholder={`${i18n.t('settings.extension_section.enter')} ${label}`}
                                    class="rounded-lg bg-background"
                            />
                        {:else if def.type === 'number'}
                            <Input
                                    id={fieldId(def.key)}
                                    type="number"
                                    value={settings[def.key]}
                                    onchange={(e) => {
                                        settings[def.key] = Number((e.target as HTMLInputElement).value);
                                        onChange();
                                    }}
                                    class="rounded-lg bg-background"
                            />
                        {:else if def.type === 'select' && def.options}
                            <Select.Root
                                    type="single"
                                    value={String(settings[def.key] || '')}
                                    onValueChange={(v) => {
                                        settings[def.key] = v;
                                        onChange();
                                    }}
                            >
                                <Select.Trigger id={fieldId(def.key)} class="w-full rounded-lg border-border bg-background">
                                    {def.options.find((o) => String(o.value) === String(settings[def.key]))?.label
                                    || i18n.t('settings.extension_section.select_option')}
                                </Select.Trigger>
                                <Select.Content>
                                    {#each def.options as option (option.value)}
                                        <Select.Item value={String(option.value)}>{option.label}</Select.Item>
                                    {/each}
                                </Select.Content>
                            </Select.Root>
                        {:else if def.type === 'multiselect' && def.options}
                            <div class="flex flex-wrap gap-2 pt-1">
                                {#each def.options as option (option.value)}
                                    {@const selected = (settings[def.key] || []).includes(option.value)}
                                    <button
                                            type="button"
                                            aria-pressed={selected}
                                            class="rounded-lg border px-3 py-1.5 text-xs font-bold transition-colors
                                                   {selected
                                                ? 'border-primary bg-primary text-primary-foreground'
                                                : 'border-border/60 bg-background hover:bg-muted'}"
                                            onclick={() => {
                                                const current: string[] = settings[def.key] || [];
                                                settings[def.key] = selected
                                                    ? current.filter((v) => v !== option.value)
                                                    : [...current, option.value];
                                                onChange();
                                            }}
                                    >
                                        {option.label}
                                    </button>
                                {/each}
                            </div>
                        {:else}
                            <p class="text-xs text-muted-foreground">
                                {i18n.t('settings.extension_section.unsupported_type')} {def.type}
                            </p>
                        {/if}
                    </div>
                {/if}
            {/each}
        </div>

        <div class="flex h-4 items-center justify-end gap-1.5 text-xs text-muted-foreground">
            {#if isSaving}
                <Spinner class="h-3 w-3 animate-spin" />
                {i18n.t('settings.extension_section.saving')}
            {/if}
        </div>
    </div>
{/if}