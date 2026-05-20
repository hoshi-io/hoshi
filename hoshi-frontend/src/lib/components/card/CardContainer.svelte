<script lang="ts">
    import { AspectRatio } from '@/components/ui/aspect-ratio';
    import { Star } from 'lucide-svelte';
    import {proxyApi} from "@/api/proxy";

    let {
        title,
        cover,
        year,
        score,
        shouldBlur = false,
        contentTypeLabel,
        overlay,
    } = $props();

    let objectUrl = $state<string | null>(null);

    $effect(() => {
        if (!cover) return;
        let revoked = false;

        const tryDirect = () => new Promise<string>((resolve, reject) => {
            const img = new Image();
            img.onload = () => resolve(cover);
            img.onerror = () => reject();
            img.src = cover;
        });

        tryDirect()
            .then(url => { if (!revoked) objectUrl = url; })
            .catch(() => {
                proxyApi.fetch({ url: cover })
                    .then(blob => { if (!revoked) objectUrl = URL.createObjectURL(blob); })
                    .catch(() => { if (!revoked) objectUrl = cover; });
            });

        return () => {
            revoked = true;
            if (objectUrl?.startsWith("blob:")) URL.revokeObjectURL(objectUrl);
        };
    });
</script>

<div class="card">
    <div class="cover-wrap">
        <AspectRatio ratio={2/3}>
            <img
                    src={objectUrl ?? ''}
                    alt={title}
                    loading="lazy"
                    class="cover-img {shouldBlur ? 'blur-xl scale-110' : ''}"
            />
            {#if overlay}
                <div class="overlay-slot">
                    {@render overlay()}
                </div>
            {/if}
        </AspectRatio>

        {#if score && !overlay}
            <div class="score-badge">
                <Star class="w-2.5 h-2.5 fill-yellow-400 text-yellow-400" />
                {score}%
            </div>
        {/if}
    </div>

    <div class="card-footer">
        <div class="card-meta">{contentTypeLabel ?? ''}{year ? ` · ${year}` : ''}</div>
        <h3 class="card-title">{title}</h3>
    </div>
</div>

<style>
    .card { width: 100%; height: 100%; display: flex; flex-direction: column; gap: 0.45rem; }
    .cover-wrap { position: relative; width: 100%; border-radius: 0.125rem; overflow: hidden; background: var(--color-background-secondary); }
    .cover-img { width: 100%; height: 100%; object-fit: cover; display: block; transition: transform 0.4s ease; }
    .card:hover .cover-img { transform: scale(1.05); }

    .overlay-slot {
        position: absolute;
        inset: 0;
        display: flex;
        flex-direction: column;
        justify-content: flex-end;
        pointer-events: none;
    }

    .score-badge {
        position: absolute;
        top: 0.45rem; right: 0.45rem;
        display: flex; align-items: center; gap: 0.2rem;
        background: hsla(0, 0%, 0%, 0.55);
        backdrop-filter: blur(4px);
        color: white;
        font-size: 0.65rem; font-weight: 700;
        padding: 0.2rem 0.45rem;
        border-radius: 0.25rem;
    }
    .card-footer { padding: 0 0.15rem; }
    .card-meta { font-size: 0.6rem; font-weight: 600; text-transform: uppercase; letter-spacing: 0.05em; color: var(--color-text-tertiary); margin-bottom: 0.15rem; }
    .card-title { font-size: 0.8rem; font-weight: 700; color: var(--color-text-primary); line-height: 1.25; display: -webkit-box; -webkit-line-clamp: 2; -webkit-box-orient: vertical; overflow: hidden; margin: 0; }
</style>