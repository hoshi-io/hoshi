import { extensionsApi } from "@/api/extensions/extensions";
import { contentApi } from "@/api/content/content";
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

    async playWithMpv(
        cid: string,
        epNumber: number,
        extId: string,
        opts: {
            server?: string;
            isDub?: boolean;
            animeTitle: string;
            episodeTitle: string;
            totalEpisodes: number;
            isNsfw: boolean;
            coverImage?: string | null;
            startTime?: number;
            autoUpdateProgress: boolean;
            use_hoshi_config: boolean;
        }
    ) {
        const ext = this.installed.find(e => e.id === extId);
        if (!ext) throw new Error("Extension not found");

        await invoke("launch_mpv", {
            opts: {
                extension: extId,
                server: opts.server ?? "",
                category: opts.isDub ? "dub" : "sub",
                startTime: opts.startTime ?? 0,
                cid,
                epNumber,
                totalEpisodes: opts.totalEpisodes,
                animeTitle: opts.animeTitle,
                episodeTitle: opts.episodeTitle,
                isNsfw: opts.isNsfw,
                coverImage: opts.coverImage ?? null,
                autoUpdateProgress: opts.autoUpdateProgress,
                userId: 0,
                useHoshiMpvConfig: opts.use_hoshi_config,
            },
        });
    }

    async resolveStream(
        cid: string,
        epNumber: number,
        extId: string,
        opts: { server?: string; category?: string }
    ) {
        const playRes = await contentApi.play(cid, extId, epNumber, opts) as any;
        return playRes.data;
    }

    isTachiyomi(extId: string): boolean {
        return this.installed.find(e => e.id === extId)?.source === "tachiyomi";
    }
}

export const extensions = new ExtensionsStore();