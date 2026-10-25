<script lang="ts">
    import { tick } from 'svelte';
    import { fade } from 'svelte/transition';
    import { goto } from '$app/navigation';
    import { toast } from 'svelte-sonner';
    import { Check, ChevronLeft, ChevronRight, Pencil } from 'lucide-svelte';
    import { Spinner } from '$lib/components/ui/spinner';

    import { i18n } from '@/stores/i18n.svelte.js';
    import { themeManager } from '@/stores/theme.svelte.js';
    import { completeSetup, DEFAULT_SETUP_PREFS, type SetupPrefs } from '@/utils/setup.js';
    import type { CoreError } from '@/api/client';
    import LanguageSelector from '@/components/LanguageSelector.svelte';
    import TvChoice from '@/tv/components/TvChoice.svelte';
    import TvSettingRow from '@/tv/components/TvSettingRow.svelte';

    // Typing on a TV is painful, so the whole flow works with OK presses on the defaults.
    const DEFAULT_USERNAME = 'Guest';

    const availableSteps = ['profile', 'appearance', 'content'] as const;

    let currentIndex = $state(0);
    const currentStepId = $derived(availableSteps[currentIndex]);
    const isLast = $derived(currentIndex === availableSteps.length - 1);

    let isSaving = $state(false);

    let username = $state(DEFAULT_USERNAME);
    let editing = $state(false);
    let nameButton: HTMLButtonElement | null = $state(null);
    let nameInput: HTMLInputElement | null = $state(null);

    // NSFW toggles are intentionally not asked on TV; they keep the safe defaults.
    let preferredMetadataProvider = $state<SetupPrefs['preferredMetadataProvider']>(DEFAULT_SETUP_PREFS.preferredMetadataProvider);
    let titleLanguage = $state<SetupPrefs['titleLanguage']>(DEFAULT_SETUP_PREFS.titleLanguage);
    let defaultHomeSection = $state<SetupPrefs['defaultHomeSection']>(DEFAULT_SETUP_PREFS.defaultHomeSection);

    const themes = [
        { value: 'light', label: 'Light' },
        { value: 'dark', label: 'Dark' },
        { value: 'oled', label: 'OLED' }
    ];

    const colorPresets = [
        { name: 'Purple', value: '#a855f7' },
        { name: 'Blue', value: '#3b82f6' },
        { name: 'Cyan', value: '#06b6d4' },
        { name: 'Green', value: '#22c55e' },
        { name: 'Amber', value: '#f59e0b' },
        { name: 'Orange', value: '#f97316' },
        { name: 'Rose', value: '#f43f5e' },
        { name: 'Pink', value: '#ec4899' }
    ];

    const metadataOptions: { value: SetupPrefs['preferredMetadataProvider']; label: string }[] = [
        { value: 'anilist', label: 'AniList' },
        { value: 'myanimelist', label: 'MyAnimeList' },
        { value: 'kitsu', label: 'Kitsu' },
        { value: 'simkl', label: 'Simkl' }
    ];

    const languageOptions = $derived<{ value: SetupPrefs['titleLanguage']; label: string }[]>([
        { value: 'romaji', label: i18n.t('setup.content.romaji') },
        { value: 'english', label: i18n.t('setup.content.english') },
        { value: 'native', label: i18n.t('setup.content.native') },
        { value: 'chinese', label: i18n.t('setup.content.chinese') }
    ]);

    const sectionOptions = $derived<{ value: SetupPrefs['defaultHomeSection']; label: string }[]>([
        { value: 'anime', label: i18n.t('setup.content.anime') },
        { value: 'manga', label: i18n.t('setup.content.manga') },
        { value: 'novel', label: i18n.t('setup.content.novel') }
    ]);

    // ---- username editing: an input would trap D-pad focus, so it's only shown on demand ----

    async function startEdit() {
        editing = true;
        await tick();
        nameInput?.focus();
        nameInput?.select();
    }

    async function stopEdit() {
        if (!editing) return;
        if (!username.trim()) username = DEFAULT_USERNAME;
        editing = false;
        await tick();
        nameButton?.focus();
    }

    function onNameKeydown(e: KeyboardEvent) {
        const exit =
            e.key === 'Enter' || e.key === 'Escape' || e.key === 'GoBack' || e.keyCode === 4 ||
            // spatial nav ignores inputs, so Up/Down also leave the field
            e.key === 'ArrowUp' || e.key === 'ArrowDown';
        if (!exit) return;
        e.preventDefault();
        e.stopPropagation();
        stopEdit();
    }

    // ---- steps ----

    async function focusNext() {
        await tick();
        document.querySelector<HTMLElement>('[data-next]')?.focus();
    }

    function nextStep() {
        if (currentIndex < availableSteps.length - 1) {
            currentIndex++;
            focusNext();
        }
    }

    function prevStep() {
        if (currentIndex > 0) {
            currentIndex--;
            focusNext();
        }
    }

    function onWindowKeydown(e: KeyboardEvent) {
        const isBack =
            e.key === 'Escape' || e.key === 'GoBack' || e.key === 'BrowserBack' || e.keyCode === 4;
        if (!isBack || editing) return;

        // First step: do nothing, so Back exits the app.
        if (currentIndex > 0) {
            e.preventDefault();
            prevStep();
        }
    }

    async function finishSetup() {
        if (isSaving) return;
        isSaving = true;
        try {
            await completeSetup({
                username: username.trim() || DEFAULT_USERNAME,
                avatarFile: null,
                prefs: {
                    ...DEFAULT_SETUP_PREFS,
                    preferredMetadataProvider,
                    titleLanguage,
                    defaultHomeSection
                }
            });
            goto('/tv');
        } catch (err) {
            toast.error(i18n.t((err as CoreError).key));
        } finally {
            isSaving = false;
        }
    }

    const navButton =
        'flex h-14 items-center gap-2 rounded-lg px-9 text-lg font-bold outline-none ' +
        'transition-[transform,background-color] duration-150 focus:scale-105 focus:ring-4 focus:ring-primary/70';
</script>

<svelte:head><title>{i18n.t('setup.title')}</title></svelte:head>
<svelte:window onkeydown={onWindowKeydown} />

<div class="mx-auto flex min-h-[calc(100dvh-4rem)] w-full max-w-5xl flex-col">
    <header class="mb-6 flex shrink-0 items-end justify-between gap-6">
        <div class="space-y-1">
            <h1 class="text-4xl font-black tracking-tight">{i18n.t('setup.welcome_app')}</h1>
            {#if currentStepId === 'appearance'}
                <p class="text-lg text-muted-foreground">{i18n.t('setup.appearance.description')}</p>
            {:else if currentStepId === 'content'}
                <p class="text-lg text-muted-foreground">{i18n.t('setup.content.description')}</p>
            {/if}
        </div>

        <!-- Indicator only; steps are not clickable on TV -->
        <div class="flex shrink-0 items-center gap-4" aria-hidden="true">
            <div class="flex items-center gap-2">
                {#each availableSteps as _, i}
                    <span class="block h-2 rounded-sm transition-all duration-300 {currentIndex >= i ? 'w-10 bg-primary' : 'w-3 bg-muted'}"></span>
                {/each}
            </div>
            <span class="text-base font-bold tabular-nums text-muted-foreground">{currentIndex + 1} / {availableSteps.length}</span>
        </div>
    </header>

    <main class="flex-1">
        {#key currentStepId}
            <div
                    class="rounded-xl border border-border/40 bg-card/40 px-8 [&>div:last-child]:border-b-0"
                    in:fade={{ duration: 120 }}
            >
                {#if currentStepId === 'profile'}
                    <TvSettingRow label={i18n.t('setup.profile.username')}>
                        {#if editing}
                            <input
                                    bind:this={nameInput}
                                    bind:value={username}
                                    data-sn
                                    onkeydown={onNameKeydown}
                                    onblur={stopEdit}
                                    class="h-14 w-80 rounded-lg border-2 border-primary bg-secondary/50 px-4 text-xl outline-none"
                                    placeholder={i18n.t('setup.profile.username_placeholder')}
                            />
                        {:else}
                            <button
                                    bind:this={nameButton}
                                    type="button"
                                    data-sn
                                    onclick={startEdit}
                                    class="flex h-14 w-80 items-center justify-between rounded-lg bg-secondary/50 px-4 text-xl font-semibold outline-none
                                           transition-transform duration-150 focus:scale-105 focus:ring-4 focus:ring-primary/70"
                            >
                                <span class="truncate">{username}</span>
                                <Pencil class="size-5 shrink-0 opacity-60" />
                            </button>
                        {/if}
                    </TvSettingRow>

                    <TvSettingRow label={i18n.t('setup.appearance.language')} stacked>
                        <LanguageSelector />
                    </TvSettingRow>
                {/if}

                {#if currentStepId === 'appearance'}
                    <TvSettingRow label={i18n.t('setup.appearance.theme')} stacked>
                        <TvChoice
                                items={themes}
                                value={themeManager.theme}
                                label={i18n.t('setup.appearance.theme')}
                                onselect={(v) => themeManager.setTheme(v)}
                        />
                    </TvSettingRow>

                    <TvSettingRow label={i18n.t('setup.appearance.accent_color')} stacked>
                        <div class="flex flex-wrap gap-4 py-1">
                            {#each colorPresets as preset}
                                <button
                                        type="button"
                                        data-sn
                                        aria-label={preset.name}
                                        onclick={() => themeManager.setAccentColor(preset.value)}
                                        class="flex size-12 items-center justify-center rounded-full outline-none
                                               transition-transform duration-150 focus:scale-125 focus:ring-4 focus:ring-foreground/80"
                                        style="background-color: {preset.value}"
                                >
                                    {#if themeManager.accentColor?.toLowerCase() === preset.value.toLowerCase()}
                                        <Check class="size-6 text-white drop-shadow-md" />
                                    {/if}
                                </button>
                            {/each}
                        </div>
                    </TvSettingRow>
                {/if}

                {#if currentStepId === 'content'}
                    <TvSettingRow label={i18n.t('setup.content.metadata_provider')} stacked>
                        <TvChoice items={metadataOptions} bind:value={preferredMetadataProvider} label={i18n.t('setup.content.metadata_provider')} />
                    </TvSettingRow>

                    <TvSettingRow label={i18n.t('setup.content.title_language')} stacked>
                        <TvChoice items={languageOptions} bind:value={titleLanguage} label={i18n.t('setup.content.title_language')} />
                    </TvSettingRow>

                    <TvSettingRow label={i18n.t('setup.content.default_home_section')} stacked>
                        <TvChoice items={sectionOptions} bind:value={defaultHomeSection} label={i18n.t('setup.content.default_home_section')} />
                    </TvSettingRow>
                {/if}
            </div>
        {/key}
    </main>

    <footer class="mt-6 flex shrink-0 items-center justify-between">
        <div>
            {#if currentIndex > 0}
                <button data-sn type="button" onclick={prevStep} class="{navButton} bg-muted/30 text-foreground">
                    <ChevronLeft class="size-5" /> {i18n.t('setup.navigation.back')}
                </button>
            {/if}
        </div>

        {#if !isLast}
            <button data-sn data-next data-autofocus type="button" onclick={nextStep} class="{navButton} bg-primary text-primary-foreground">
                {i18n.t('setup.navigation.next')} <ChevronRight class="size-5" />
            </button>
        {:else}
            <button data-sn data-next data-autofocus type="button" onclick={finishSetup} disabled={isSaving} class="{navButton} bg-primary text-primary-foreground">
                {#if isSaving}
                    <Spinner class="size-5 animate-spin" /> {i18n.t('setup.navigation.saving')}
                {:else}
                    {i18n.t('setup.navigation.finish')} <Check class="size-5" />
                {/if}
            </button>
        {/if}
    </footer>
</div>