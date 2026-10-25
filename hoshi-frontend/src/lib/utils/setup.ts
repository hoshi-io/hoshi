import { auth } from '@/stores/auth.svelte.js';
import { appConfig } from '@/stores/config.svelte.js';

export type SetupPrefs = {
    showAdultContent: boolean;
    blurAdultContent: boolean;
    preferredMetadataProvider: 'anilist' | 'myanimelist' | 'kitsu' | 'simkl';
    titleLanguage: 'romaji' | 'english' | 'native' | 'chinese';
    defaultHomeSection: 'anime' | 'manga' | 'novel';
};

export const DEFAULT_SETUP_PREFS: SetupPrefs = {
    showAdultContent: false,
    blurAdultContent: true,
    preferredMetadataProvider: 'anilist',
    titleLanguage: 'romaji',
    defaultHomeSection: 'anime'
};

export async function completeSetup(opts: {
    username: string;
    avatarFile?: File | null;
    prefs: SetupPrefs;
}) {
    const { username, avatarFile = null, prefs } = opts;

    await auth.register({ username }, avatarFile);

    await appConfig.update({
        general: {
            showAdultContent: prefs.showAdultContent,
            blurAdultContent: prefs.blurAdultContent,
            needSetup: false
        },
        ui: {
            sidebarCollapsed: appConfig.data?.ui?.sidebarCollapsed ?? false,
            disableCardTrailers: appConfig.data?.ui?.disableCardTrailers ?? false,
            defaultHomeSection: prefs.defaultHomeSection,
            titleLanguage: prefs.titleLanguage
        },
        content: {
            preferredMetadataProvider: prefs.preferredMetadataProvider,
            autoUpdateProgress: appConfig.data?.content?.autoUpdateProgress ?? true,
            scoreFormat: 'point10'
        },
        extensions: { repoUrls: appConfig.data?.extensions?.repoUrls || [] }
    });
}