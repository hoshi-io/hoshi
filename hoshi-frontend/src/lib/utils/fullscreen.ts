import { getCurrentWindow } from "@tauri-apps/api/window";
import { type as osType } from "@tauri-apps/plugin-os";

let wasMaximized = false;

/** Desktop window fullscreen. Windows needs unmaximizing first or the window keeps its frame. */
export async function setWindowFullscreen(on: boolean): Promise<boolean> {
    try {
        const win = getCurrentWindow();
        const windows = osType() === "windows";
        if (on === (await win.isFullscreen())) return on;
        if (on) {
            if (windows) {
                wasMaximized = await win.isMaximized();
                if (wasMaximized) await win.unmaximize();
            }
            await win.setFullscreen(true);
        } else {
            await win.setFullscreen(false);
            if (windows && wasMaximized) {
                await win.maximize();
                wasMaximized = false;
            }
        }
        return on;
    } catch (e) {
        console.error("Failed to toggle fullscreen", e);
        return !on;
    }
}

export async function isWindowFullscreen(): Promise<boolean> {
    try {
        return await getCurrentWindow().isFullscreen();
    } catch {
        return false;
    }
}
