<script lang="ts">
    import { invoke } from "@tauri-apps/api/core";
    import { i18n } from "@/stores/i18n.svelte";

    let {
        torrent,
        sessionId,
        visible,
        onChange,
    }: {
        torrent: any;
        sessionId: string;
        visible: boolean;
        onChange?: () => void;
    } = $props();

    let stats = $state<any | null>(null);

    $effect(() => {
        const id = sessionId;
        if (!visible) return;

        let cancelled = false;
        const poll = async () => {
            try {
                const s = await invoke<any | null>("get_torrent_stats", { sessionId: id });
                if (!cancelled) stats = s;
            } catch { /* session already gone */ }
        };
        poll();
        const timer = setInterval(poll, 1000);
        return () => { cancelled = true; clearInterval(timer); };
    });

    function fmtBytes(n: number): string {
        if (n < 1024) return `${n} B`;
        const units = ["KB", "MB", "GB", "TB"];
        let v = n / 1024, i = 0;
        while (v >= 1024 && i < units.length - 1) { v /= 1024; i++; }
        return `${v.toFixed(v >= 100 ? 0 : 1)} ${units[i]}`;
    }

    const pct = $derived(
        stats && stats.totalBytes > 0
            ? Math.min(100, (stats.progressBytes / stats.totalBytes) * 100)
            : 0
    );
</script>

<div
        class="absolute top-16 md:top-20 left-1/2 -translate-x-1/2 z-50 w-[min(92vw,36rem)] transition-opacity duration-300
               {visible ? 'opacity-100 pointer-events-auto' : 'opacity-0 pointer-events-none'}"
        onclick={(e) => e.stopPropagation()}
        role="presentation"
>
    <div class="flex items-center gap-3 rounded-2xl bg-black/90 px-4 py-2 text-white shadow-lg backdrop-blur-md">
        <div class="min-w-0 flex-1">
            <div class="truncate text-sm font-medium" title={torrent.title}>{torrent.title}</div>
            <div class="mt-0.5 flex flex-wrap gap-x-3 text-xs text-white/60">
                {#if torrent.seeders != null}
                    <span>{i18n.t("watch.torrent_seeders")}: {torrent.seeders}</span>
                {/if}
                {#if stats}
                    <span>{i18n.t("watch.torrent_peers")}: {stats.peers}</span>
                    <span>↓ {fmtBytes(stats.downloadBps)}/s</span>
                    <span>↑ {fmtBytes(stats.uploadBps)}/s</span>
                    <span>{pct.toFixed(0)}%</span>
                {/if}
            </div>
        </div>
        <button
                type="button"
                class="shrink-0 rounded-full bg-white/15 px-3 py-1 text-xs font-semibold transition-colors hover:bg-white/25"
                onclick={(e) => { e.stopPropagation(); onChange?.(); }}
        >
            {i18n.t("watch.torrent_change")}
        </button>
    </div>
</div>