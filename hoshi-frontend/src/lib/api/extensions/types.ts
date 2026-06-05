export type ExtensionType = "anime" | "manga" | "novel" | "unknown";

export type SettingType = "string" | "number" | "boolean" | "select" | "multiselect" | "unknown";

export interface SettingOption {
    value: string;
    label: string;
}

export interface TachiyomiSource {
    name: string;
    lang: string;
    id: string;
    baseUrl: string;
}

export interface TachiyomiMarketplaceEntry {
    name: string;
    pkg: string;
    apk: string;
    lang: string;
    version: string;
    nsfw: number;
    sources: TachiyomiSource[];

    repo_url?: string;
    icon_url?: string;
}

export interface NativeMarketplaceEntry {
    id: string;
    name: string;
    version: string;
    author: string;
    ext_type: ExtensionType;
    language: string;
    main: string;
    icon?: string;
    manifestUrl: string;
}

export type AnyMarketplaceEntry = NativeMarketplaceEntry | LNReaderMarketplaceEntry | TachiyomiMarketplaceEntry;

export interface LNReaderMarketplaceEntry {
    id: string;
    name: string;
    site: string;
    lang: string;
    version: string;
    url: string;
    iconUrl: string;
}

export interface SettingDefinition {
    key: string;
    label: string;
    type: SettingType;
    default: string | number | boolean | string[] | null;
    options?: SettingOption[];
}

export interface Extension {
    id: string;
    name: string;
    version: string;
    author: string;
    icon?: string | null;
    ext_type: ExtensionType;
    language: string;
    nsfw: boolean;
    skip_default_processing: boolean;
    setting_definitions: SettingDefinition[];
    settings: Record<string, unknown>;
    source?: string | null;
}

export interface ExtensionSettingsResponse {
    episodeServers?: string[];
    supportsDub?: boolean;
}

export interface ExtensionFiltersResponse {
    filters: Record<string, unknown>;
}

export interface InstallExtensionResponse {
    ok: boolean;
    extension: Extension;
}

export interface UninstallExtensionResponse {
    ok: boolean;
    id: string;
}

export interface UpdateExtensionSettingsResponse {
    ok: boolean;
    id: string;
}

export interface UpdateExtensionResponse {
    ok: boolean;
    extension: Extension;
}