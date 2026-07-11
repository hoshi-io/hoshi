import { listen } from "@tauri-apps/api/event";
import {listStore} from "@/app/list.svelte.js";

type ImportState =
    | { status: "importing"; imported: number; total: number | null }
    | { status: "done"; imported: number }
    | { status: "error"; message: string };

export const importStatuses = $state<Record<string, ImportState>>({});

export async function setupImportListener() {
    await listen<any>("tracker:import", ({ payload }) => {
        const name = payload.tracker_name;
        switch (payload.type) {
            case "started":
                importStatuses[name] = { status: "importing", imported: 0, total: null };
                break;
            case "progress":
                importStatuses[name] = { status: "importing", imported: payload.imported, total: payload.total };
                break;
            case "done":
                importStatuses[name] = { status: "done", imported: payload.imported };
                setTimeout(() => delete importStatuses[name], 5000);
                listStore.refresh();
                break;
            case "error":
                importStatuses[name] = { status: "error", message: payload.message };
                break;
        }
    });
}