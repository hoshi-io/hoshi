<script lang="ts">
    import { i18n } from "@/stores/i18n.svelte.js";
    import { themeManager } from "@/stores/theme.svelte.js";
    import { Button } from "$lib/components/ui/button";
    import { Input } from "$lib/components/ui/input";
    import { Label } from "$lib/components/ui/label";
    import { Switch } from "$lib/components/ui/switch";
    import LanguageSelector from "@/components/LanguageSelector.svelte";
    import { auth } from "@/stores/auth.svelte.js";
    import { appConfig } from "@/stores/config.svelte.js";
    import {Check, ChevronRight, ChevronLeft, UserCircle2, Camera, User} from "lucide-svelte";
    import { Spinner } from "$lib/components/ui/spinner";
    import { fly } from "svelte/transition";
    import { toast } from "svelte-sonner";
    import { goto } from "$app/navigation";
    import { layoutState } from "@/stores/layout.svelte.js";
    import type { CoreError } from "@/api/client";
    import ResponsiveSelect from "@/components/ResponsiveSelect.svelte";
    import StarBackground from "@/components/StarBackground.svelte";

    const availableSteps = ['profile', 'appearance', 'content'];

    let isSaving = $state(false);

    let language = $state(i18n.locale);

    let username = $state("");
    let avatarFile = $state<File | null>(null);
    let avatarPreview = $state<string | null>(null);

    let showAdultContent = $state(false);
    let blurAdultContent = $state(true);
    let preferredMetadataProvider = $state<'anilist' | 'myanimelist' | 'kitsu'>('anilist');
    let titleLanguage = $state<'romaji' | 'english' | 'native'>('romaji');
    let defaultHomeSection = $state<'anime' | 'manga' | 'novel'>('anime');

    let extConfig = $state({ repoUrl: appConfig.data?.extensions?.repoUrls || [] });

    const themes = [
        { id: 'light', label: 'Light', classes: 'bg-zinc-50 text-zinc-950 border-zinc-200' },
        { id: 'dark', label: 'Dark', classes: 'bg-zinc-900 text-zinc-50 border-zinc-800' },
        { id: 'oled', label: 'OLED', classes: 'bg-black text-white border-zinc-900' }
    ];

    const colorPresets = [
        { name: 'Purple', value: '#a855f7' },
        { name: 'Blue', value: '#3b82f6' },
        { name: 'Cyan', value: '#06b6d4' },
        { name: 'Green', value: '#22c55e' },
        { name: 'Amber', value: '#f59e0b' },
        { name: 'Orange', value: '#f97316' },
        { name: 'Rose', value: '#f43f5e' },
        { name: 'Pink', value: '#ec4899' },
    ];

    const metadataOptions = [
        { value: "anilist", label: "AniList" },
        { value: "myanimelist", label: "MyAnimeList" },
        { value: "kitsu", label: "Kitsu" },
        { value: "simkl", label: "Simkl" }
    ];

    const languageOptions = $derived([
        { value: "romaji", label: i18n.t('setup.content.romaji') },
        { value: "english", label: i18n.t('setup.content.english') },
        { value: "native", label: i18n.t('setup.content.native') },
        { value: "chinese", label: i18n.t('setup.content.chinese') },
    ]);

    const sectionOptions = $derived([
        { value: "anime", label: i18n.t('setup.content.anime') },
        { value: "manga", label: i18n.t('setup.content.manga') },
        { value: "novel", label: i18n.t('setup.content.novel') }
    ]);

    $effect(() => {
        layoutState.title = "Setup";
        layoutState.showBack = false;
        layoutState.backUrl = null;
    });

    function setPresetColor(color: string) {
        themeManager.setAccentColor(color);
    }

    function handleCustomColor(event: Event) {
        const input = event.target as HTMLInputElement;
        themeManager.setAccentColor(input.value);
    }

    let currentIndex = $state(0);
    let currentStepId = $derived(availableSteps[currentIndex]);
    let direction = $state(1);

    function nextStep() {
        if (currentStepId === 'profile' && !username.trim()) {
            toast.error(i18n.t('setup.profile.validation_error'));
            return;
        }
        if (currentIndex < availableSteps.length - 1) {
            direction = 1;
            currentIndex++;
        }
    }

    function prevStep() {
        if (currentIndex > 0) {
            direction = -1;
            currentIndex--;
        }
    }

    function skipStep() {
        if (currentStepId === 'profile') {
            toast.error(i18n.t('setup.profile.require_profile'));
            return;
        }
        if (currentIndex < availableSteps.length - 1) {
            direction = 1; // Set forward
            currentIndex++;
        } else {
            finishSetup();
        }
    }

    async function finishSetup() {
        if (!username.trim()) {
            currentIndex = availableSteps.indexOf('profile');
            toast.error(i18n.t('setup.profile.validation_error'));
            return;
        }

        isSaving = true;
        try {
            const registerData = { username };
            await auth.register(registerData, avatarFile);

            await appConfig.update({
                general: {
                    showAdultContent,
                    blurAdultContent,
                    needSetup: false
                },
                ui: {
                    sidebarCollapsed: appConfig.data?.ui?.sidebarCollapsed ?? false,
                    disableCardTrailers: appConfig.data?.ui?.disableCardTrailers ?? false,
                    defaultHomeSection,
                    titleLanguage
                },
                content: {
                    preferredMetadataProvider,
                    autoUpdateProgress: appConfig.data?.content?.autoUpdateProgress ?? true,
                    scoreFormat: "point10"
                },
                extensions: extConfig,
            });

            goto("/");
        } catch (err) {
            const error = err as CoreError;
            toast.error(i18n.t(error.key));
        } finally {
            isSaving = false;
        }
    }

    function handleAvatarChange(e: Event) {
        const input = e.target as HTMLInputElement;
        if (input.files && input.files[0]) {
            avatarFile = input.files[0];
            avatarPreview = URL.createObjectURL(avatarFile);
        }
    }
</script>

<svelte:head><title>{i18n.t("setup.title")}</title></svelte:head>

<div class="h-[100dvh] bg-background text-foreground flex flex-col overflow-hidden">
    <StarBackground />

<div class="w-full max-w-3xl mx-auto flex flex-col flex-1 pt-10 pb-6 px-4 sm:px-8 md:px-10 min-h-0">

    <header class="mb-8 shrink-0">
        <h1 class="text-3xl sm:text-4xl font-extrabold tracking-tight text-center mb-6">
            {i18n.t('setup.welcome_app')}
        </h1>
        <div class="flex items-center justify-center gap-2">
            {#each availableSteps as _, i}
                <div class="h-2 rounded-sm transition-all duration-300 {currentIndex >= i ? 'w-10 bg-primary' : 'w-3 bg-muted'}"></div>
            {/each}
        </div>
    </header>

    <main class="flex-1 overflow-y-auto pr-1 pb-6 min-h-0 custom-scrollbar relative">
        {#key currentStepId}
            <div
                    in:fly={{ x: direction * 50, duration: 300, delay: 150 }}
                    out:fly={{ x: -direction * 50, duration: 150 }}
                    class="w-full h-full"
            >

            {#if currentStepId === 'profile'}
                    <div class="flex flex-col items-center gap-5">
                        <div class="relative group">
                            <div class="w-36 h-36 md:w-44 md:h-44 rounded-full overflow-hidden border-[6px] border-background shadow-2xl bg-secondary flex items-center justify-center transition-all duration-500 group-hover:scale-105 group-hover:shadow-primary/20">
                                {#if avatarPreview}
                                    <img src={avatarPreview} alt="Avatar" class="w-full h-full object-cover" />
                                {:else}
                                    <User class="w-20 h-20 text-muted-foreground/40 group-hover:text-primary/60 transition-colors" />
                                {/if}
                            </div>
                            <label for="avatar-upload" class="absolute bottom-2 right-2 bg-primary text-primary-foreground p-3.5 rounded-full cursor-pointer shadow-xl hover:scale-110 transition-transform duration-300">
                                <Camera class="w-5 h-5" />
                            </label>
                            <input type="file" id="avatar-upload" accept="image/*" class="hidden" onchange={handleAvatarChange} />
                        </div>
                        <div class="text-center space-y-1">
                            <p class="text-xl font-bold text-foreground">{i18n.t('setup.profile.avatar')}</p>
                        </div>
                    </div>

                    <div class="p-6 md:p-8 space-y-6">
                        <div class="space-y-3">
                            <Label for="username" class="text-sm font-bold text-foreground flex items-center gap-2">
                                <UserCircle2 class="w-4 h-4 text-primary" />
                                {i18n.t('setup.profile.username')}
                            </Label>
                            <Input
                                    id="username"
                                    bind:value={username}
                                    placeholder={i18n.t('setup.profile.username_placeholder')}
                                    class="h-14 rounded-sm bg-secondary/50 border-border text-lg shadow-inner focus-visible:ring-primary px-4 transition-all"
                            />
                        </div>

                        <div class="space-y-3 pt-2 border-t border-border/40">
                            <Label class="text-sm font-bold">{i18n.t('setup.appearance.language')}</Label>
                            <LanguageSelector
                                    class="w-full h-12 rounded-sm bg-muted/20"
                                    onLanguageChange={(code) => { language = code; i18n.setLocale(code); }}
                            />
                        </div>
                    </div>
            {/if}

            {#if currentStepId === 'appearance'}
                    <div class="text-center space-y-2">
                        <h2 class="text-2xl font-bold">{i18n.t('setup.appearance.title')}</h2>
                        <p class="text-muted-foreground">{i18n.t('setup.appearance.description')}</p>
                    </div>

                    <div class="p-6 space-y-6">
                        <div class="space-y-4">
                            <Label class="text-base font-bold">{i18n.t('setup.appearance.theme')}</Label>
                            <div class="grid grid-cols-3 gap-3">
                                {#each themes as theme}
                                    <button
                                            onclick={() => themeManager.setTheme(theme.id)}
                                            class="relative flex items-center justify-center h-14 rounded-sm border-2 font-bold {theme.classes} {themeManager.theme === theme.id ? 'ring-2 ring-primary border-transparent' : 'opacity-70 border-transparent'} transition-all"
                                    >
                                        {theme.label}
                                        {#if themeManager.theme === theme.id}
                                            <div class="absolute top-1.5 right-1.5 bg-primary rounded-full p-0.5">
                                                <Check class="size-3 text-primary-foreground" />
                                            </div>
                                        {/if}
                                    </button>
                                {/each}
                            </div>
                        </div>

                        <!-- Accent color -->
                        <div class="space-y-4 pt-4 border-t border-border/40">
                            <Label class="text-base font-bold">{i18n.t('setup.appearance.accent_color')}</Label>
                            <div class="flex flex-wrap items-center gap-3">
                                <div class="relative flex items-center gap-3 bg-muted/20 p-2 rounded-sm border border-border/50">
                                    <Input
                                            type="color"
                                            value={themeManager.accentColor || '#ffffff'}
                                            onchange={handleCustomColor}
                                            class="w-10 h-10 p-0 rounded-sm border-none cursor-pointer bg-transparent shrink-0"
                                    />
                                    <span class="text-xs font-mono font-bold pr-2 uppercase opacity-70">{themeManager.accentColor}</span>
                                </div>
                                <div class="h-8 w-px bg-border/40 mx-1 hidden sm:block"></div>
                                <div class="flex flex-wrap gap-2">
                                    {#each colorPresets as preset}
                                        <button
                                                type="button"
                                                onclick={() => setPresetColor(preset.value)}
                                                class="size-10 rounded-full border-2 border-background shadow-sm transition-transform active:scale-90 flex items-center justify-center"
                                                style="background-color: {preset.value}"
                                                title={preset.name}
                                        >
                                            {#if themeManager.accentColor?.toLowerCase() === preset.value.toLowerCase()}
                                                <Check class="size-5 text-white drop-shadow-md" />
                                            {/if}
                                        </button>
                                    {/each}
                                </div>
                            </div>
                        </div>
                    </div>
            {/if}

            {#if currentStepId === 'content'}
                    <div class="text-center space-y-2">
                        <h2 class="text-2xl font-bold">{i18n.t('setup.content.title')}</h2>
                        <p class="text-muted-foreground">{i18n.t('setup.content.description')}</p>
                    </div>

                    <div class="rounded-sm p-6 space-y-6">
                        <div class="space-y-3">
                            <Label class="text-base font-bold">{i18n.t('setup.content.metadata_provider')}</Label>
                            <ResponsiveSelect
                                    bind:value={preferredMetadataProvider}
                                    items={metadataOptions}
                                    label={i18n.t('setup.content.metadata_provider')}
                            />
                        </div>

                        <div class="space-y-3 pt-4 border-t border-border/40">
                            <Label class="text-base font-bold">{i18n.t('setup.content.title_language')}</Label>
                            <ResponsiveSelect
                                    bind:value={titleLanguage}
                                    items={languageOptions}
                                    label={i18n.t('setup.content.title_language')}
                            />
                        </div>

                        <div class="space-y-3 pt-4 border-t border-border/40">
                            <Label class="text-base font-bold">{i18n.t('setup.content.default_home_section')}</Label>
                            <ResponsiveSelect
                                    bind:value={defaultHomeSection}
                                    items={sectionOptions}
                                    label={i18n.t('setup.content.default_home_section')}
                            />
                        </div>

                        <div class="pt-4 space-y-5 border-t border-border/40">
                            <div class="flex items-center justify-between gap-4">
                                <div class="space-y-0.5">
                                    <Label class="text-base font-bold cursor-pointer" for="showAdultContent">{i18n.t('setup.content.show_nsfw')}</Label>
                                </div>
                                <Switch id="showAdultContent" bind:checked={showAdultContent} class="shrink-0 scale-125" />
                            </div>

                            <div class="flex items-center justify-between gap-4 transition-opacity {!showAdultContent ? 'opacity-50' : ''}">
                                <div class="space-y-0.5">
                                    <Label class="text-base font-bold {showAdultContent ? 'cursor-pointer' : 'cursor-not-allowed'}" for="blurAdultContent">{i18n.t('setup.content.blur_nsfw')}</Label>
                                </div>
                                <Switch id="blurAdultContent" bind:checked={blurAdultContent} disabled={!showAdultContent} class="shrink-0 scale-125" />
                            </div>
                        </div>
                    </div>
            {/if}
            </div>
        {/key}

        </main>

    <footer
            class="shrink-0 pt-5 mt-auto flex items-center justify-between border-t border-border/30 bg-background"
            style="padding-bottom: max(0.5rem, env(safe-area-inset-bottom))"
    >
            <div>
                {#if currentIndex > 0}
                    <Button variant="ghost" onclick={prevStep} class="rounded-sm font-bold h-11 px-5">
                        <ChevronLeft class="mr-1.5 h-4 w-4" /> {i18n.t('setup.navigation.back')}
                    </Button>
                {/if}
            </div>

            <div class="flex items-center gap-3">
                {#if currentStepId !== 'profile'}
                    <Button variant="ghost" onclick={skipStep} class="rounded-sm font-bold h-11 px-5 text-muted-foreground hover:text-foreground transition-colors">
                        {i18n.t('setup.navigation.skip')}
                    </Button>
                {/if}

                {#if currentIndex < availableSteps.length - 1}
                    <Button onclick={nextStep} class="rounded-sm font-bold h-11 px-7 shadow-sm">
                        {i18n.t('setup.navigation.next')} <ChevronRight class="ml-1.5 h-4 w-4" />
                    </Button>
                {:else}
                    <Button onclick={finishSetup} disabled={isSaving} class="rounded-sm font-bold h-11 px-7 shadow-sm bg-primary text-primary-foreground hover:bg-primary/90">
                        {#if isSaving}
                            <Spinner class="mr-2 h-4 w-4 animate-spin" /> {i18n.t('setup.navigation.saving')}
                        {:else}
                            {i18n.t('setup.navigation.finish')} <Check class="ml-1.5 h-4 w-4" />
                        {/if}
                    </Button>
                {/if}
            </div>
        </footer>
    </div>
</div>