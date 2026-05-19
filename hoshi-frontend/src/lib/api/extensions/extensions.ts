import { call } from "@/api/client";
import type {
    Extension,
    ExtensionFiltersResponse,
    ExtensionSettingsResponse,
    InstallExtensionResponse, LNReaderMarketplaceEntry, NativeMarketplaceEntry, TachiyomiMarketplaceEntry,
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

    installAuto(entry: NativeMarketplaceEntry | LNReaderMarketplaceEntry) {
        if ("url" in entry && "iconUrl" in entry) {
            return extensionsApi.installLNReader(entry as LNReaderMarketplaceEntry);
        }
        return extensionsApi.install((entry as NativeMarketplaceEntry).manifestUrl);
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
};