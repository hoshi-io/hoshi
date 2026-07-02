<script lang="ts">
    import { proxyApi } from "@/api/proxy";

    let {
        src,
        alt = "",
        class: className = "",
        shouldBlur = false,
        imageHeaders = undefined,
    }: {
        src: string | null | undefined;
        alt?: string;
        class?: string;
        shouldBlur?: boolean;
        imageHeaders?: Record<string, string>;
    } = $props();

    let objectUrl = $state<string | null>(null);
    let isLoaded = $state(false);
    let imgEl = $state<HTMLImageElement | null>(null);
    let lastProcessedSrc: string | null | undefined = undefined;

    $effect(() => {
        if (!src) {
            objectUrl = null;
            isLoaded = false;
            lastProcessedSrc = src;
            return;
        }

        if (src === lastProcessedSrc) {
            return;
        }
        lastProcessedSrc = src;

        let revoked = false;
        isLoaded = false;

        const hasHeaders = imageHeaders && Object.keys(imageHeaders).length > 0;

        const tryDirect = () => new Promise<string>((resolve, reject) => {
            if (hasHeaders) { reject(); return; }

            const img = new Image();
            img.onload = () => resolve(src);
            img.onerror = () => reject();
            img.src = src;
        });

        tryDirect()
            .then(url => { if (!revoked) objectUrl = url; })
            .catch(() => {
                proxyApi.fetch({
                    url: src,
                    referer: hasHeaders ? imageHeaders?.["referer"] : undefined,
                    origin: hasHeaders ? imageHeaders?.["origin"] : undefined,
                    userAgent: hasHeaders ? imageHeaders?.["user-agent"] : undefined,
                })
                    .then(blob => { if (!revoked) objectUrl = URL.createObjectURL(blob); })
                    .catch(() => { if (!revoked) objectUrl = src; });
            });

        return () => {
            revoked = true;
            if (objectUrl?.startsWith("blob:")) URL.revokeObjectURL(objectUrl);
        };
    });

    $effect(() => {
        if (imgEl && !isLoaded && imgEl.complete && imgEl.naturalWidth > 0) {
            isLoaded = true;
        }
    });
</script>

<div class="smart-image-wrapper {className}">
    {#if !isLoaded && src}
        <div class="image-skeleton"></div>
    {/if}

    {#if objectUrl}
        <img
                bind:this={imgEl}
                src={objectUrl}
                {alt}
                loading="lazy"
                onload={() => isLoaded = true}
                class="smart-image {shouldBlur ? 'is-blurred' : ''} {isLoaded ? 'is-loaded' : 'is-loading'}"
        />
    {/if}
</div>

<style>
    img { will-change: transform; }

    .smart-image-wrapper {
        position: relative;
        width: 100%;
        height: 100%;
        overflow: hidden;
        display: flex;
        align-items: center;
        justify-content: center;
        background-color: var(--color-background-secondary, hsl(var(--muted) / 0.3));
    }

    .image-skeleton {
        position: absolute;
        inset: 0;
        background-color: hsl(var(--muted) / 0.5);
        animation: pulse 2s cubic-bezier(0.4, 0, 0.6, 1) infinite;
        z-index: 1;
    }

    @keyframes pulse {
        0%, 100% { opacity: 1; }
        50% { opacity: .4; }
    }

    /* --- Image Pop Animations --- */
    .smart-image {
        width: 100%;
        height: 100%;
        object-fit: cover;
        display: block;
        transition: transform 0.5s cubic-bezier(0.34, 1.56, 0.64, 1), opacity 0.4s ease, filter 0.3s ease;
        will-change: transform, opacity;
    }

    .smart-image.is-loading {
        opacity: 0;
        transform: scale(0.92);
    }

    .smart-image.is-loaded {
        opacity: 1;
        transform: scale(1);
    }

    /* Handle blur explicitly to prevent edge-bleeding */
    .smart-image.is-blurred {
        filter: blur(16px);
        transform: scale(1.15);
    }
</style>