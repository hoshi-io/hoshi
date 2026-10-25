<script lang="ts">
    import type { CardExt } from '@/app/marketplace.svelte.js';
    import { i18n } from '@/stores/i18n.svelte.js';
    import { Spinner } from '@/components/ui/spinner';
    import TvSwitch from '@/tv/components/TvSwitch.svelte';

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

    const fieldId = (key: string) => `tv-ext-${ext.id}-${key}`;

    const inputClass =
        'h-12 w-full scroll-my-24 rounded-md border-2 border-border/60 bg-card px-4 text-lg outline-none focus:border-primary focus:ring-4 focus:ring-primary/40';
    const chipBase =
        'h-11 scroll-my-24 rounded-md px-4 text-base font-semibold outline-none transition-[transform,background-color,color] duration-150 focus:scale-105 focus:ring-4 focus:ring-primary/70';
    const chipClass = (on: boolean) =>
        `${chipBase} ${on ? 'bg-primary text-primary-foreground' : 'bg-muted/30 text-foreground/80'}`;
</script>

{#if settings}
    <div class="space-y-5">
        {#each ext.setting_definitions ?? [] as def (def.key)}
            {@const label = def.label || def.key}

            {#if def.type === 'boolean'}
                <div class="flex items-center justify-between gap-4">
                    <span class="text-lg font-bold">{label}</span>
                    <TvSwitch {label} bind:checked={settings[def.key]} onchange={() => onChange()} />
                </div>
            {:else}
                <div class="space-y-2">
                    <label for={fieldId(def.key)} class="text-lg font-bold">{label}</label>

                    {#if def.type === 'string'}
                        <input
                                id={fieldId(def.key)}
                                data-sn
                                type="text"
                                autocomplete="off"
                                value={settings[def.key] ?? ''}
                                placeholder={`${i18n.t('settings.extension_section.enter')} ${label}`}
                                onchange={(e) => {
                                    settings[def.key] = (e.target as HTMLInputElement).value;
                                    onChange();
                                }}
                                class={inputClass}
                        />
                    {:else if def.type === 'number'}
                        <input
                                id={fieldId(def.key)}
                                data-sn
                                type="number"
                                value={settings[def.key]}
                                onchange={(e) => {
                                    settings[def.key] = Number((e.target as HTMLInputElement).value);
                                    onChange();
                                }}
                                class={inputClass}
                        />
                    {:else if def.type === 'select' && def.options}
                        <div id={fieldId(def.key)} role="radiogroup" aria-label={label} class="flex flex-wrap gap-2">
                            {#each def.options as option (option.value)}
                                {@const selected = String(settings[def.key]) === String(option.value)}
                                <button
                                        type="button"
                                        role="radio"
                                        aria-checked={selected}
                                        data-sn
                                        class={chipClass(selected)}
                                        onclick={() => {
                                            settings[def.key] = String(option.value);
                                            onChange();
                                        }}
                                >
                                    {option.label}
                                </button>
                            {/each}
                        </div>
                    {:else if def.type === 'multiselect' && def.options}
                        <div class="flex flex-wrap gap-2">
                            {#each def.options as option (option.value)}
                                {@const selected = (settings[def.key] || []).includes(option.value)}
                                <button
                                        type="button"
                                        aria-pressed={selected}
                                        data-sn
                                        class={chipClass(selected)}
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
                        <p class="text-sm text-muted-foreground">
                            {i18n.t('settings.extension_section.unsupported_type')} {def.type}
                        </p>
                    {/if}
                </div>
            {/if}
        {/each}

        <div class="flex h-5 items-center justify-end gap-2 text-sm text-muted-foreground">
            {#if isSaving}
                <Spinner class="size-4" />
                {i18n.t('settings.extension_section.saving')}
            {/if}
        </div>
    </div>
{/if}