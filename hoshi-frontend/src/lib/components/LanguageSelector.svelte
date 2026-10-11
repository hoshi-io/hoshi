<script lang="ts">
    import { i18n } from "@/stores/i18n.svelte.js";
    import type { Language } from "@/stores/i18n.svelte.js";
    import * as Command from "$lib/components/ui/command";
    import * as Popover from "$lib/components/ui/popover";
    import * as Drawer from "$lib/components/ui/drawer";
    import { Button } from "$lib/components/ui/button";
    import { Check, ChevronsUpDown } from "lucide-svelte";

    let {
        compact = false,
        class: className = "",
        onLanguageChange
    }: {
        compact?: boolean;
        class?: string;
        onLanguageChange?: (langCode: Language) => void;
    } = $props();

    let open = $state(false);
    let isDesktop = $state(true);
    
    let flagsReady = $state(false);
    $effect(() => {
        if (!open) {
            flagsReady = false;
            return;
        }
        const id = setTimeout(() => (flagsReady = true), 60);
        return () => clearTimeout(id);
    });

    const flagClass = "h-4 w-6 shrink-0";
    const compactFlagClass = "h-3 w-[1.125rem] shrink-0";

    const availableLanguages = i18n.getAvailableLanguages();
    let selectedLang = $derived(availableLanguages.find(l => l.code === i18n.locale));

    $effect(() => {
        const mediaQuery = window.matchMedia("(min-width: 768px)");
        isDesktop = mediaQuery.matches;

        const handleMediaChange = (e: MediaQueryListEvent) => {
            isDesktop = e.matches;
        };

        mediaQuery.addEventListener("change", handleMediaChange);
        return () => mediaQuery.removeEventListener("change", handleMediaChange);
    });

    function changeLanguage(code: Language) {
        i18n.setLocale(code);
        if (onLanguageChange) onLanguageChange(code);
        open = false;
    }

    function handleSelect(code: Language) {
        return () => changeLanguage(code);
    }
</script>

<!-- `props` are the trigger attributes handed over by Popover/Drawer (aria-*, data-state, handlers...). -->
{#snippet triggerButton(props: Record<string, unknown>)}
    <Button
            {...props}
            variant="outline"
            role="combobox"
            aria-expanded={open}
            class="justify-between {className}"
    >
        {#if selectedLang}
            <span class="flex items-center gap-2 font-bold">
                {#if compact}
                    <selectedLang.icon class={compactFlagClass} />
                    <span class="uppercase text-[10px] tracking-tighter">{selectedLang.code}</span>
                {:else}
                    <selectedLang.icon class={flagClass} />
                    <span class="text-sm">{selectedLang.name}</span>
                {/if}
            </span>
        {:else}
            {i18n.t('settings.select_language')}
        {/if}

        {#if !compact}
            <ChevronsUpDown class="ml-2 h-4 w-4 shrink-0 opacity-50" />
        {/if}
    </Button>
{/snippet}

{#snippet languageList()}
    <Command.Root>
        <Command.Input placeholder={i18n.t('settings.general_section.search_language')} class="h-10" />
        <Command.Empty>{i18n.t('settings.general_section.no_language_found')}</Command.Empty>

        <Command.Group class="max-h-[300px] overflow-y-auto custom-scrollbar">
            {#each availableLanguages as lang (lang.code)}
                {@const isSelected = i18n.locale === lang.code}
                <Command.Item
                        value={lang.name}
                        keywords={[lang.code]}
                        onSelect={handleSelect(lang.code as Language)}
                        class="flex items-center gap-3 cursor-pointer py-2.5 px-3 rounded-sm mx-1 my-0.5"
                >
                    <Check class="h-4 w-4 shrink-0 {isSelected ? 'opacity-100 text-primary' : 'opacity-0'}" />
                    {#if flagsReady}
                        <lang.icon class={flagClass} />
                    {:else}
                        <span class={flagClass} aria-hidden="true"></span>
                    {/if}
                    <span class="font-semibold text-sm {isSelected ? 'text-primary' : ''}">{lang.name}</span>
                </Command.Item>
            {/each}
        </Command.Group>
    </Command.Root>
{/snippet}

{#if isDesktop}
    <Popover.Root bind:open>
        <Popover.Trigger>
            {#snippet child({ props })}
                {@render triggerButton(props)}
            {/snippet}
        </Popover.Trigger>

        <Popover.Content class="min-w-[220px] p-0 rounded-sm shadow-xl border-border/50 overflow-hidden" align="end">
            {@render languageList()}
        </Popover.Content>
    </Popover.Root>
{:else}
    <Drawer.Root bind:open>
        <Drawer.Trigger>
            {#snippet child({ props })}
                {@render triggerButton(props)}
            {/snippet}
        </Drawer.Trigger>

        <Drawer.Content>
            <div class="mt-4 border-t p-0">
                {@render languageList()}
            </div>
        </Drawer.Content>
    </Drawer.Root>
{/if}