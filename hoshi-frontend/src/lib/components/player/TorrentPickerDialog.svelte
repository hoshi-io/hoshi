<script lang="ts">
    import { untrack } from "svelte";
    import { invoke } from "@tauri-apps/api/core";
    import * as Dialog from "$lib/components/ui/dialog";
    import { i18n } from "@/stores/i18n.svelte";
    import type { WatchState } from "@/app/watch.svelte.js";

    let { open = $bindable(false), pageState }: { open: boolean; pageState: WatchState } = $props();

    let query = $state("");
    let results = $state<any[]>([]);
    let loading = $state(false);
    let error = $state<string | null>(null);
    let searched = $state(false);
    let seq = 0; // drops out-of-order responses

    // Prefill with the auto query and search as soon as the dialog opens.
    // untrack: typing in the input must not re-run this and reset the query.
    $effect(() => {
        if (!open) return;
        untrack(() => {
            query = pageState.torrentSearchQuery();
            void search();
        });
    });

    async function search() {
        const q = query.trim();
        if (!q || !pageState.selectedExtension) return;

        const mine = ++seq;
        loading = true;
        error = null;
        try {
            const res = await invoke<{ results: any[] }>("search_torrents", {
                id: pageState.selectedExtension,
                query: q,
                filters: {},
                page: 1,
            });
            if (mine !== seq) return;
            results = [...res.results].sort((a, b) => (b.seeders ?? 0) - (a.seeders ?? 0));
        } catch (e: any) {
            if (mine !== seq) return;
            error = e?.key ? i18n.t(e.key) : i18n.t("errors.unknown_error");
            results = [];
        } finally {
            if (mine === seq) {
                loading = false;
                searched = true;
            }
        }
    }

    function pick(t: any) {
        open = false;
        void pageState.chooseTorrent(t);
    }
</script>

<Dialog.Root bind:open>
    <Dialog.Content class="z-[100] max-w-2xl">
        <Dialog.Header>
            <Dialog.Title>{i18n.t("watch.torrent_pick_title")}</Dialog.Title>
        </Dialog.Header>

        <div class="flex gap-2">
            <input
                    class="min-w-0 flex-1 rounded-md border border-border bg-background px-3 py-2 text-sm outline-none focus:ring-2 focus:ring-ring"
                    bind:value={query}
                    onkeydown={(e) => e.key === "Enter" && search()}
                    placeholder={i18n.t("watch.torrent_search_placeholder")}
            />
            <button
                    class="rounded-md bg-primary px-4 py-2 text-sm font-medium text-primary-foreground disabled:opacity-50"
                    onclick={search}
                    disabled={loading}
            >
                {i18n.t("watch.torrent_search")}
            </button>
        </div>

        <div class="max-h-[60vh] space-y-1 overflow-y-auto">
            {#if loading}
                <p class="py-6 text-center text-sm text-muted-foreground">…</p>
            {:else if error}
                <p class="py-6 text-center text-sm text-destructive">{error}</p>
            {:else if searched && results.length === 0}
                <p class="py-6 text-center text-sm text-muted-foreground">{i18n.t("watch.torrent_no_results")}</p>
            {:else}
                {#each results as t (t.id)}
                    <button
                            class="w-full rounded-md border border-transparent p-2 text-left hover:border-border hover:bg-accent"
                            onclick={() => pick(t)}
                    >
                        <div class="line-clamp-2 text-sm font-medium">{t.title}</div>
                        <div class="mt-1 flex flex-wrap gap-x-3 text-xs text-muted-foreground">
                            {#if t.releaseGroup}<span>{t.releaseGroup}</span>{/if}
                            {#if t.resolution}<span>{t.resolution}</span>{/if}
                            {#if t.size}<span>{t.size}</span>{/if}
                            <span>S {t.seeders ?? 0}</span>
                            <span>L {t.leechers ?? 0}</span>
                        </div>
                    </button>
                {/each}
            {/if}
        </div>
    </Dialog.Content>
</Dialog.Root>