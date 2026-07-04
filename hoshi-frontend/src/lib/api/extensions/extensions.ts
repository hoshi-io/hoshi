import { call } from "@/api/client";
import type {
    Extension,
    ExtensionFiltersResponse,
    ExtensionSettingsResponse,
    InstallExtensionResponse, LNReaderMarketplaceEntry, NativeMarketplaceEntry,
    SoraMarketplaceEntry, TachiyomiMarketplaceEntry,
    UninstallExtensionResponse, UpdateExtensionResponse,
    UpdateExtensionSettingsResponse,
} from "./types";

export const extensionsApi = {
    getAll(): Promise<Extension[]> {
        return call<{ extensions: Extension[] }>({
            tauri: { cmd: "get_extensions" },
        }).then(res => res.extensions ?? []);
    },

    getSettings(id: string) {
        return call<ExtensionSettingsResponse>({
            tauri: { cmd: "get_extension_settings", args: { id } },
        });
    },

    updateSettings(id: string, settings: Record<string, unknown>) {
        return call<UpdateExtensionSettingsResponse>({
            tauri: { cmd: "update_extension_settings", args: { id, settings } },
        });
    },

    getFilters(name: string) {
        return call<ExtensionFiltersResponse>({
            tauri: { cmd: "get_extension_filters", args: { name } },
        });
    },

    install(manifestUrl: string) {
        return call<InstallExtensionResponse>({
            tauri: { cmd: "install_extension", args: { manifestUrl } },
        });
    },

    installLNReader(entry: LNReaderMarketplaceEntry) {
        return call<InstallExtensionResponse>({
            tauri: { cmd: "install_lnreader_extension", args: { entry } },
        });
    },

    installSora(entry: SoraMarketplaceEntry) {
        return call<InstallExtensionResponse>({
            tauri: { cmd: "install_sora_extension", args: { entry } },
        });
    },

    installTachiyomi(downloadUrl: string, entry: TachiyomiMarketplaceEntry) {
        return call<InstallExtensionResponse>({
            tauri: { cmd: "install_tachiyomi_extension", args: { downloadUrl, entry } },
        });
    },

    update(id: string, manifestUrl: string) {
        return call<UpdateExtensionResponse>({
            tauri: { cmd: "update_extension", args: { id, manifestUrl } },
        });
    },

    uninstall(id: string) {
        return call<UninstallExtensionResponse>({
            tauri: { cmd: "uninstall_extension", args: { id } },
        });
    },

    getImageRequestHeaders: (extensionId: string, imageUrl: string, chapterUrl?: string) => {
        if (!chapterUrl) chapterUrl = "";
        return call<Record<string, string>>({
            tauri: { cmd: "get_image_request_headers", args: { extensionId, imageUrl, chapterUrl } },
        });
    },
};