import { call } from "@/api/client";
import type { ContentType } from "@/api/content/types";
import type { DiscoverItem, DiscoverQuery, HomeLayout, SequelCard } from "./types";

export const homeApi = {
    getLayout() {
        return call<HomeLayout>({ tauri: { cmd: "get_home_layout" } });
    },

    getDefaultLayout() {
        return call<HomeLayout>({ tauri: { cmd: "get_default_home_layout" } });
    },

    saveLayout(layout: HomeLayout) {
        return call<HomeLayout>({ tauri: { cmd: "save_home_layout", args: { layout } } });
    },

    resetLayout() {
        return call<HomeLayout>({ tauri: { cmd: "reset_home_layout" } });
    },

    discover(query: DiscoverQuery, page = 1) {
        return call<DiscoverItem[]>({ tauri: { cmd: "discover", args: { query, page } } });
    },

    unstartedSequels(contentType: ContentType, limit = 20) {
        return call<SequelCard[]>({
            tauri: { cmd: "get_unstarted_sequels", args: { content_type: contentType, limit } },
        });
    },
};
