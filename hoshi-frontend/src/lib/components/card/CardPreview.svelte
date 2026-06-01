<script lang="ts">
    import { Star, Play, Plus } from 'lucide-svelte';
    import { fade } from 'svelte/transition';
    import ListEditorButton from "@/components/ListEditorButton.svelte";
    import {i18n} from "@/stores/i18n.svelte";

    const YOUTUBE_REGEXP = /^.*(youtu.be\/|v\/|u\/\w\/|embed\/|watch\?v=|&v=)([^#&?]*).*/;

    let {
        cid, title, cover, bannerImage, trailerUrl,
        score, status, synopsis, episodeCount,
        contentType, contentTypeLabel, shouldBlur,
        href
    }: {
        cid: string;
        title: string;
        cover: string;
        bannerImage?: string | null;
        trailerUrl?: string | null;
        score?: number | null;
        status?: string | null;
        synopsis?: string | null;
        episodeCount?: number | null;
        contentType?: string | null;
        contentTypeLabel?: string | null;
        shouldBlur?: boolean;
        href: string;
    } = $props();

    let isHovered = $state(false);

    const getYoutubeId = (url: string | undefined | null) => {
        if (!url) return null;
        const match = url?.match(YOUTUBE_REGEXP);
        return (match && match[2].length === 11) ? match[2] : null;
    };

    let ytId = $derived(getYoutubeId(trailerUrl));
    let iframeReady = $state(false);
    let thumbnailSrc = $derived(ytId ? `https://i.ytimg.com/vi/${ytId}/maxresdefault.jpg` : bannerImage || cover);

    function onIframeLoad() {
        setTimeout(() => (iframeReady = true), 400);
    }
</script>

<div
        class="preview-card"
        transition:fade={{ duration: 150 }}
        onmouseenter={() => (isHovered = true)}
        onmouseleave={() => {
        isHovered = false;
        iframeReady = false;
    }}
>
    <div class="banner">
        <div class="banner-media">
            <img src={cover} alt="" class="media-img base-cover" />

            {#if isHovered}
                <img
                        src={thumbnailSrc}
                        alt=""
                        class="media-img {shouldBlur ? 'blur-xl scale-110' : ''}"
                        transition:fade={{ duration: 300 }}
                />

                {#if ytId}
                    <iframe
                            src="https://www.youtube.com/embed/{ytId}?autoplay=1&mute=1&controls=0&loop=1&playlist={ytId}&modestbranding=1&rel=0&iv_load_policy=3&disablekb=1"
                            class="trailer-iframe {iframeReady ? 'ready' : ''}"
                            allow="autoplay; encrypted-media"
                            title="Trailer"
                            onload={onIframeLoad}
                    ></iframe>
                {/if}
            {/if}
        </div>
        <div class="banner-overlay"></div>
    </div>

    <div class="content">
        <div class="info-wrapper">
            <h3 class="title">{title}</h3>

            <div class="actions">
                <a {href} class="play-btn">
                    <Play size="0.9rem" fill="currentColor" />
                    {i18n.t(contentType === "anime" ? "content.watch_now" : "content.read_now")}
                </a>

                <ListEditorButton
                        cid={cid}
                        title={title}
                        contentType={contentType || ""}
                        coverImage={cover}
                        size="icon"
                        class="h-9 w-9"
                />
            </div>

            <div class="meta-row">
                {#if score}<span class="score"><Star size="0.7rem" fill="currentColor" /> {score}%</span>{/if}
                <span class="badge">{contentTypeLabel || contentType}</span>
                {#if episodeCount}<span class="badge">{episodeCount} Ep</span>{/if}
                {#if status}<span class="status-text {status.toLowerCase()}">{status}</span>{/if}
            </div>

            {#if synopsis}
                <p class="synopsis">{synopsis.replace(/<[^>]*>/g, '')}</p>
            {/if}
        </div>
    </div>
</div>

<style>
    .preview-card {
        width: 100%;
        height: 100%;
        background: var(--card);
        overflow: hidden;
        display: flex;
        flex-direction: column;
        box-shadow: 0 20px 40px -10px rgba(0, 0, 0, 0.7), 0 10px 20px -10px rgba(0, 0, 0, 0.5);
        pointer-events: auto;
        transform-origin: center center;
        backface-visibility: hidden;
    }

    .banner {
        position: relative;
        aspect-ratio: 16 / 11;
        background: var(--card);
        flex-shrink: 0;
        overflow: hidden;
    }

    .media-img {
        position: absolute;
        inset: 0;
        width: 100%;
        height: 100%;
        object-fit: cover;
    }

    .base-cover {
        z-index: 1;
    }

    .trailer-iframe {
        position: absolute;
        border: none;
        opacity: 0;
        inset: -10% -15%;
        width: 130%;
        height: 120%;
        transform: none;
        transition: opacity 0.6s ease;
        z-index: 2;
        pointer-events: none;
    }

    .trailer-iframe.ready { opacity: 1; }

    .banner-overlay {
        position: absolute;
        inset: 0;
        background: linear-gradient(to top, var(--card) 0%, transparent 40%);
        z-index: 3;
        pointer-events: none;
    }

    .content {
        padding: 1rem;
        background: var(--card);
        margin-top: -0.5rem;
        position: relative;
        z-index: 4;
        flex: 1;
        min-height: 0;
        display: flex;
        flex-direction: column;
    }

    .info-wrapper {
        display: flex;
        flex-direction: column;
        gap: 0.6rem;
        animation: slideUp 0.3s ease-out;
        flex: 1;
        min-height: 0;
    }

    @keyframes slideUp {
        from { opacity: 0; transform: translateY(8px); }
        to   { opacity: 1; transform: translateY(0); }
    }

    .title {
        font-size: 1rem;
        font-weight: 700;
        color: var(--foreground);
        line-height: 1.2;
        margin: 0;
        display: -webkit-box;
        -webkit-line-clamp: 2;
        -webkit-box-orient: vertical;
        overflow: hidden;
    }

    .actions {
        display: flex;
        gap: 0.5rem;
        margin-top: 0.15rem;
    }

    .play-btn {
        flex: 1;
        background: var(--primary);
        color: var(--primary-foreground);
        height: 2.25rem;
        display: flex;
        align-items: center;
        justify-content: center;
        gap: 0.4rem;
        border-radius: 0.4rem;
        text-decoration: none;
        font-weight: 600;
        font-size: 0.8rem;
        transition: filter 0.2s;
    }

    .action-btn-circle {
        width: 2.25rem;
        height: 2.25rem;
        border-radius: 50%;
        border: 1px solid var(--border);
        background: rgba(255, 255, 255, 0.05);
        color: var(--foreground);
        display: flex;
        align-items: center;
        justify-content: center;
        transition: border-color 0.2s, background 0.2s;
        cursor: pointer;
    }

    .action-btn-circle:hover {
        border-color: var(--foreground);
        background: rgba(255, 255, 255, 0.1);
    }

    .meta-row {
        display: flex;
        gap: 0.5rem;
        font-size: 0.7rem;
        font-weight: 600;
        align-items: center;
        color: var(--muted-foreground);
    }

    .score {
        display: flex;
        align-items: center;
        gap: 0.2rem;
        color: #fbbf24;
    }

    .synopsis {
        font-size: 0.75rem;
        color: var(--muted-foreground);
        line-height: 1.5;
        mask-image: linear-gradient(to bottom, black 60%, transparent 100%);
        -webkit-mask-image: linear-gradient(to bottom, black 60%, transparent 100%);
        flex: 1;
        min-height: 0;
        overflow: hidden;
        margin: 0;
    }
</style>