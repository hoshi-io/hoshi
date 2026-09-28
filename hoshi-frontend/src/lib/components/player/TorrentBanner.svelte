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

    interface LiveStats {
        state: string;
        progressBytes: number;
        totalBytes: number;
        downloadBps: number;
        uploadBps: number;
        peers: number;
    }

    let stats = $state<LiveStats | null>(null);

    // Accept both camelCase and snake_case so a missing serde rename on the
    // Rust side can't silently zero out every field.
    function normalize(s: any): LiveStats | null {
        if (!s || typeof s !== "object") return null;
        return {
            state: s.state ?? "unknown",
            progressBytes: Number(s.progressBytes ?? s.progress_bytes ?? 0),
            totalBytes: Number(s.totalBytes ?? s.total_bytes ?? 0),
            downloadBps: Number(s.downloadBps ?? s.download_bps ?? 0),
            uploadBps: Number(s.uploadBps ?? s.upload_bps ?? 0),
            peers: Number(s.peers ?? 0),
        };
    }

    $effect(() => {
        const id = sessionId;
        if (!visible) return;

        let cancelled = false;
        let warned = false;
        const poll = async () => {
            try {
                const s = await invoke<any | null>("get_torrent_stats", { sessionId: id });
                if (!cancelled) stats = normalize(s);
            } catch (e) {
                if (!warned) {
                    warned = true;
                    console.error("get_torrent_stats failed", e);
                }
            }
        };
        poll();
        const timer = setInterval(poll, 1000);
        return () => { cancelled = true; clearInterval(timer); };
    });

    function fmtBytes(n: number): string {
        if (!Number.isFinite(n) || n < 0) return "0 B";
        if (n < 1024) return `${Math.round(n)} B`;
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

    // ---- title marquee: only scrolls when the title doesn't fit ----------
    let boxEl = $state<HTMLElement | null>(null);
    let textEl = $state<HTMLElement | null>(null);
    let overflow = $state(0);

    $effect(() => {
        void torrent.title;
        if (!boxEl || !textEl) return;

        const box = boxEl;
        const text = textEl;
        const measure = () => {
            overflow = Math.max(0, Math.ceil(text.scrollWidth - box.clientWidth));
        };
        measure();

        const ro = new ResizeObserver(measure);
        ro.observe(box);
        ro.observe(text);
        return () => ro.disconnect();
    });

    // ~30px/s plus a pause at each end (built into the keyframes).
    const marqueeSeconds = $derived(Math.max(4, overflow / 30 + 3));
</script>

<div
        class="absolute top-16 md:top-20 left-1/2 -translate-x-1/2 z-50 w-[min(92vw,36rem)] transition-opacity duration-300
               {visible ? 'opacity-100 pointer-events-auto' : 'opacity-0 pointer-events-none'}"
        onclick={(e) => e.stopPropagation()}
        role="presentation"
>
    <div class="flex items-center gap-3 rounded-2xl bg-black/90 px-4 py-2 text-white shadow-lg backdrop-blur-md">
        <div class="min-w-0 flex-1">
            <div
                    bind:this={boxEl}
                    class="marquee-box overflow-hidden text-sm font-medium"
                    class:fade-edges={overflow > 0}
                    title={torrent.title}
            >
                <span
                        bind:this={textEl}
                        class="marquee-text inline-block whitespace-nowrap"
                        class:scrolling={overflow > 0 && visible}
                        class:truncated={overflow > 0 && !visible}
                        style="--shift: {overflow}px; --dur: {marqueeSeconds}s;"
                >{torrent.title}</span>
            </div>
            <div class="mt-0.5 flex flex-wrap gap-x-3 text-xs text-white/60">
                {#if torrent.seeders != null}
                    <span>{i18n.t("watch.torrent_seeders")}: {torrent.seeders}</span>
                {/if}
                {#if stats}
                    {#if stats.state !== "live"}
                        <span class="capitalize">{stats.state}</span>
                    {/if}
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

<style>
    .fade-edges {
        mask-image: linear-gradient(to right, transparent 0, black 0.75rem, black calc(100% - 0.75rem), transparent 100%);
    }

    @keyframes torrent-title-marquee {
        0%, 15%   { transform: translateX(0); }
        85%, 100% { transform: translateX(calc(-1 * var(--shift))); }
    }

    @media (prefers-reduced-motion: no-preference) {
        .scrolling {
            animation: torrent-title-marquee var(--dur) ease-in-out infinite alternate;
        }
    }

    /* Reduced motion (or banner hidden): fall back to a plain ellipsis. */
    @media (prefers-reduced-motion: reduce) {
        .scrolling { display: block; overflow: hidden; text-overflow: ellipsis; }
    }
    .truncated { display: block; overflow: hidden; text-overflow: ellipsis; }
</style>