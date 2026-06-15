import { call } from "@/api/client";
import type {
    AppConfig,
    GeneralConfig,
    MangaConfig,
    NovelConfig,
    ContentConfig,
    UiConfig, ExtensionsConfig, PlayerConfig
} from "@/api/config/types";

type ConfigSection = {
    general?: Partial<GeneralConfig>;
    ui?: Partial<UiConfig>;
    content?: Partial<ContentConfig>;
    extensions?: Partial<ExtensionsConfig>;
    player?: Partial<PlayerConfig>;
    manga?: Partial<MangaConfig>;
    novel?: Partial<NovelConfig>;
};

export const configApi = {
    getConfig() {
        return call<AppConfig>({
            tauri: { cmd: "get_user_config" },
        });
    },

    patchConfig(patch: ConfigSection) {
        return call<AppConfig>({
            tauri: { cmd: "patch_user_config", args: { patch } },
        });
    },
};