import { extensionsApi } from "@/api/extensions/extensions";
import { invoke } from "@tauri-apps/api/core";
import type { Extension } from "@/api/extensions/types";
import type { CoreError } from "@/api/client";

class ExtensionsStore {
    installed = $state<Extension[]>([]);
    loading = $state(false);
    initialized = $state(false);
    error = $state<CoreError | null>(null);
    anime = $derived(this.installed.filter(ext => ext.ext_type === "anime"));
    manga = $derived(this.installed.filter(ext => ext.ext_type === "manga"));
    novel = $derived(this.installed.filter(ext => ext.ext_type === "novel"));
    torrent = $derived(this.installed.filter(ext => ext.ext_type === "torrent"));

    async load(force = false) {
        if (this.initialized && !force) return;

        this.loading = true;
        this.error = null;

        try {
            this.installed = await extensionsApi.getAll();
        } catch (err) {
            this.error = err as CoreError;
            this.installed = [];
            console.error("Failed to load extensions:", err);
        } finally {
            this.loading = false;
            this.initialized = true;
        }
    }

    async install(manifestUrl: string) {
        this.loading = true;

        try {
            const res = await extensionsApi.install(manifestUrl);
            if (res.ok && res.extension) {
                this.installed = [...this.installed, res.extension];
            }
            return res;
        } catch (err) {
            throw err as CoreError;
        } finally {
            this.loading = false;
        }
    }

    async update(id: string, manifestUrl: string) {
        this.loading = true;
        try {
            const res = await extensionsApi.update(id, manifestUrl);
            if (res.ok && res.extension) {
                this.installed = this.installed.map(ext =>
                    ext.id === id ? res.extension! : ext
                );
            }
            return res;
        } catch (err) {
            throw err as CoreError;
        } finally {
            this.loading = false;
        }
    }

    async uninstall(id: string) {
        this.loading = true;

        try {
            const res = await extensionsApi.uninstall(id);
            if (res.ok) {
                this.installed = this.installed.filter(ext => ext.id !== id);
            }
            return res;
        } catch (err) {
            throw err as CoreError;
        } finally {
            this.loading = false;
        }
    }

    isTachiyomi(extId: string): boolean {
        return this.installed.find(e => e.id === extId)?.source === "tachiyomi";
    }
}

export const extensions = new ExtensionsStore();