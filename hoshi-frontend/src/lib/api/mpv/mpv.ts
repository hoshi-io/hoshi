import { call } from "@/api/client";
import type { OscId } from "./types";

export const mpvApi = {
    downloadOsc(name: OscId) {
        return call<void>({
            tauri: { cmd: "download_osc", args: { name } },
        });
    },

    downloadKnownScript(name: string) {
        return call<void>({
            tauri: { cmd: "download_known_script", args: { name } },
        });
    },
};