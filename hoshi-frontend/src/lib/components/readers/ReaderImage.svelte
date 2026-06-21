<script lang="ts">
    import { i18n } from "@/stores/i18n.svelte.js";
    import type { MangaReaderState } from "@/app/manga.svelte";
    import {Spinner} from "@/components/ui/spinner";

    interface Props {
        imgEntry: { id: string; url: string };
        readerState: MangaReaderState;
    }

    let { imgEntry, readerState }: Props = $props();

    const status = $derived(readerState.imageStatus[imgEntry.id] || "loading");

    const isPaged = $derived(readerState.layout === "paged");
    const wrapperStyle = $derived.by(() => {
        if (isPaged) {
            switch (readerState.fitMode) {
                case 'width':  return 'flex: 1 1 0; min-width: 0; height: 100%; max-height: 100%;';
                case 'height': return 'height: 100%; flex-shrink: 0; max-width: 100%;';
                case 'fit':    return 'flex: 1 1 0; min-width: 0; height: 100%; max-height: 100%;';
                default:       return 'flex: 1 1 0; min-width: 0; height: 100%; max-height: 100%;';
            }
        } else {
            // scroll mode
            switch (readerState.fitMode) {
                case 'width':  return 'width: 100%; flex-shrink: 1; flex-grow: 1; min-width: 0;';
                case 'height': return 'height: 100%; flex-shrink: 0; max-width: 100%;';
                case 'fit':    return 'flex: 1 1 0; min-width: 0;';
                default:       return 'width: 100%; flex-shrink: 1; flex-grow: 1; min-width: 0;';
            }
        }
    });

    const imgStyle = $derived.by(() => {
        if (isPaged) {
            switch (readerState.fitMode) {
                case 'width':
                    return 'width: 100%; height: auto; max-height: 100%; object-fit: contain; display: block;';
                case 'height':
                    return 'height: 100%; width: auto; max-width: 100%; object-fit: contain; display: block;';
                case 'fit':
                    return 'max-width: 100%; max-height: 100%; width: auto; height: auto; object-fit: contain; display: block;';
                default:
                    return 'width: 100%; height: auto; max-height: 100%; object-fit: contain; display: block;';
            }
        } else {
            // scroll mode
            switch (readerState.fitMode) {
                case 'width':
                    return 'width: 100%; height: auto; display: block;';
                case 'height':
                    return 'height: 100%; width: auto; max-width: 100%; display: block;';
                case 'fit':
                    return 'max-width: 100%; max-height: 90vh; width: auto; height: auto; object-fit: contain; display: block;';
                default:
                    return 'width: 100%; height: auto; display: block;';
            }
        }
    });

    const MIN_SCALE = 1;
    const MAX_SCALE = 4;
    const DOUBLE_TAP_SCALE = 2.2;
    const DOUBLE_TAP_WINDOW = 280; // ms

    let scale = $state(1);
    let panX = $state(0);
    let panY = $state(0);
    let isGesturing = $state(false);

    let wrapperEl: HTMLDivElement;

    // plain (non-reactive) gesture bookkeeping
    let pinchStartDist = 0;
    let pinchStartScale = 1;
    let panOriginX = 0;
    let panOriginY = 0;
    let panStartTouchX = 0;
    let panStartTouchY = 0;
    let lastTapTime = 0;

    function dist(t: TouchList) {
        const dx = t[0].clientX - t[1].clientX;
        const dy = t[0].clientY - t[1].clientY;
        return Math.hypot(dx, dy);
    }

    function clamp(v: number, lo: number, hi: number) {
        return Math.min(hi, Math.max(lo, v));
    }

    function clampPan() {
        if (!wrapperEl) return;
        const rect = wrapperEl.getBoundingClientRect();
        const maxX = Math.max(0, (rect.width * (scale - 1)) / 2);
        const maxY = Math.max(0, (rect.height * (scale - 1)) / 2);
        panX = clamp(panX, -maxX, maxX);
        panY = clamp(panY, -maxY, maxY);
    }

    function resetZoom() {
        scale = 1;
        panX = 0;
        panY = 0;
    }

    function handleTouchStart(e: TouchEvent) {
        if (e.touches.length === 2) {
            e.stopPropagation();
            isGesturing = true;
            pinchStartDist = dist(e.touches);
            pinchStartScale = scale;
        } else if (e.touches.length === 1 && scale > 1) {
            e.stopPropagation();
            isGesturing = true;
            panOriginX = panX;
            panOriginY = panY;
            panStartTouchX = e.touches[0].clientX;
            panStartTouchY = e.touches[0].clientY;
        }
    }

    function handleTouchMove(e: TouchEvent) {
        if (e.touches.length === 2) {
            e.preventDefault();
            e.stopPropagation();
            const newDist = dist(e.touches);
            scale = clamp(pinchStartScale * (newDist / pinchStartDist), MIN_SCALE, MAX_SCALE);
            clampPan();
        } else if (e.touches.length === 1 && isGesturing && scale > 1) {
            e.preventDefault();
            e.stopPropagation();
            panX = panOriginX + (e.touches[0].clientX - panStartTouchX);
            panY = panOriginY + (e.touches[0].clientY - panStartTouchY);
            clampPan();
        }
    }

    function handleTouchEnd(e: TouchEvent) {
        if (e.touches.length === 0) {
            const wasGesturing = isGesturing;
            isGesturing = false;

            if (scale <= 1.02) {
                resetZoom();
            }

            if (!wasGesturing || scale === 1) {
                const now = Date.now();
                if (now - lastTapTime < DOUBLE_TAP_WINDOW) {
                    e.preventDefault();
                    e.stopPropagation();
                    if (scale > 1) {
                        resetZoom();
                    } else {
                        scale = DOUBLE_TAP_SCALE;
                    }
                    lastTapTime = 0;
                } else {
                    lastTapTime = now;
                }
            }
        }
    }

    function handleClickCapture(e: MouseEvent) {
        // while zoomed, swallow clicks so the reader doesn't treat
        // a pan-release as a "tap zone" page turn
        if (scale > 1) {
            e.stopPropagation();
        }
    }
</script>

<div
        bind:this={wrapperEl}
        class="relative {isPaged ? 'flex items-center justify-center' : ''}"
        style={wrapperStyle}
        style:touch-action={isPaged ? 'none' : 'pan-y'}
        style:overflow="hidden"
        ontouchstart={handleTouchStart}
        ontouchmove={handleTouchMove}
        ontouchend={handleTouchEnd}
        ontouchcancel={handleTouchEnd}
        onclickcapture={handleClickCapture}
>
    {#if status === "loading" || status === "error"}
        <div class="absolute inset-0 flex items-center justify-center">
            {#if status === "loading"}
                <Spinner class="size-6 text-primary/50" />
            {:else}
                <Spinner class="size-6 text-destructive/40" />
            {/if}
        </div>
    {/if}

    <img
            src={imgEntry.url}
            alt={i18n.t("reader.page_alt")}
            draggable="false"
            loading="lazy"
            class="transition-opacity duration-500 {status === 'loaded' ? 'opacity-100' : 'opacity-0'}"
            style={imgStyle}
            style:transform="scale({scale}) translate({panX / scale}px, {panY / scale}px)"
            style:transform-origin="center center"
            style:transition={isGesturing ? 'none' : 'transform 0.2s ease'}
            onload={() => readerState.setImgStatus(imgEntry.id, "loaded")}
            onerror={() => readerState.setImgStatus(imgEntry.id, "error")}
            use:readerState.resolveBlobSrc={imgEntry}
            use:readerState.handleImgMount={imgEntry.id}
    />
</div>