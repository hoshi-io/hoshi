<script lang="ts">
    import { goto } from '$app/navigation';
    import { scale as scaleTransition, draw } from 'svelte/transition';
    import { cubicOut } from 'svelte/easing';
    import { Button } from '$lib/components/ui/button';
    import { ZoomIn, ZoomOut, LocateFixed } from 'lucide-svelte';
    import type { RelationGraph } from "@/api/content/types";
    import { computeLayout, NODE_H, NODE_W, relationColor, relationLabel } from "@/relationLayout";

    let { graph, rootCid, onNavigate }: {
        graph: RelationGraph;
        rootCid: string;
        onNavigate?: () => void;
    } = $props();

    const layout = $derived(computeLayout(graph, rootCid));
    const presentTypes = $derived([...new Set(graph.edges.map((e) => e.relationType))]);

    let scale = $state(1);
    let panX = $state(0);
    let panY = $state(0);
    let hoveredCid = $state<string | null>(null);
    let container: HTMLDivElement;
    let fitted = false;

    function fitToScreen() {
        if (!container || layout.width === 0) return;
        const rect = container.getBoundingClientRect();
        const fitScale = Math.min(
            (rect.width - 120) / layout.width,
            (rect.height - 120) / layout.height,
            1
        );
        scale = Math.max(fitScale, 0.25);
        panX = (rect.width - layout.width * scale) / 2;
        panY = (rect.height - layout.height * scale) / 2;
    }

    $effect(() => {
        if (fitted || !container || layout.width === 0) return;
        fitToScreen();
        fitted = true;
    });

    let dragging = false;
    let didDrag = false;
    let lastX = 0;
    let lastY = 0;

    function onPointerDown(e: PointerEvent) {
        if (e.button !== 0 && e.pointerType === 'mouse') return;
        e.preventDefault();
        dragging = true;
        didDrag = false;
        lastX = e.clientX;
        lastY = e.clientY;
        container.setPointerCapture(e.pointerId);
    }

    function onPointerMove(e: PointerEvent) {
        if (!dragging) return;
        const dx = e.clientX - lastX;
        const dy = e.clientY - lastY;
        if (Math.abs(dx) > 2 || Math.abs(dy) > 2) didDrag = true;
        panX += dx;
        panY += dy;
        lastX = e.clientX;
        lastY = e.clientY;
    }

    function onPointerUp(e: PointerEvent) {
        dragging = false;
        if (didDrag) return;

        const el = document.elementFromPoint(e.clientX, e.clientY) as HTMLElement | null;
        const cardEl = el?.closest('[data-cid]') as HTMLElement | null;
        if (cardEl?.dataset.cid) {
            openNode(cardEl.dataset.cid);
        }
    }

    function onWheel(e: WheelEvent) {
        e.preventDefault();
        const delta = e.deltaY > 0 ? -0.1 : 0.1;
        adjustZoom(delta);
    }

    function adjustZoom(delta: number) {
        scale = Math.min(2, Math.max(0.2, scale + delta));
    }

    function openNode(cid: string) {
        if (didDrag) return;
        goto(`/c/${cid}`);
        onNavigate?.();
    }

    function curvePath(x1: number, y1: number, x2: number, y2: number) {
        const midX = (x1 + x2) / 2;
        return `M ${x1},${y1} C ${midX},${y1} ${midX},${y2} ${x2},${y2}`;
    }

    function isEdgeActive(sourceCid: string, targetCid: string) {
        return !hoveredCid || hoveredCid === sourceCid || hoveredCid === targetCid;
    }
</script>

<div class="relative flex h-full w-full flex-col bg-background overflow-hidden">

    <!-- Canvas -->
    <div
            bind:this={container}
            class="absolute inset-0 touch-none select-none"
            style="cursor: {dragging ? 'grabbing' : 'grab'};"
            onpointerdown={onPointerDown}
            onpointermove={onPointerMove}
            onpointerup={onPointerUp}
            onpointerleave={onPointerUp}
            onwheel={onWheel}
    >
        <div
                class="absolute origin-top-left will-change-transform"
                style="transform: translate({panX}px, {panY}px) scale({scale});"
        >
            <svg class="absolute top-0 left-0 pointer-events-none" width={layout.width} height={layout.height}>
                {#each layout.edges as edge, i (edge.sourceCid + edge.targetCid + edge.relationType)}
                    {@const from = layout.nodes.find((n) => n.cid === edge.sourceCid)}
                    {@const to = layout.nodes.find((n) => n.cid === edge.targetCid)}
                    {#if from && to}
                        {@const active = isEdgeActive(edge.sourceCid, edge.targetCid)}
                        <path
                                in:draw={{ duration: 800, delay: i * 50, easing: cubicOut }}
                                d={curvePath(from.x + NODE_W, from.y + NODE_H / 2, to.x, to.y + NODE_H / 2)}
                                fill="none"
                                stroke={relationColor(edge.relationType)}
                                stroke-width={active ? 2.5 : 1.5}
                                opacity={active ? 0.9 : 0.15}
                                class="transition-opacity duration-200"
                        />
                    {/if}
                {/each}
            </svg>

            {#each layout.nodes as node, i (node.cid)}
                <div
                        data-cid={node.cid}
                        in:scaleTransition={{ duration: 400, delay: 300 + (i * 20), start: 0.8, easing: cubicOut }}
                        role="button"
                        tabindex="0"
                        onkeydown={(e) => e.key === 'Enter' && openNode(node.cid)}
                        onmouseenter={() => (hoveredCid = node.cid)}
                        onmouseleave={() => (hoveredCid = null)}
                        class="absolute overflow-hidden rounded-xl border shadow-sm transition-[box-shadow,opacity] duration-200 cursor-pointer hover:shadow-xl hover:z-10
            {node.cid === rootCid ? 'ring-4 ring-primary shadow-primary/20' : ''}"
                        style="left: {node.x}px; top: {node.y}px; width: {NODE_W}px; height: {NODE_H}px;"
                >
                    <img
                            src={node.coverImage ?? undefined}
                            alt=""
                            draggable="false"
                            class="h-full w-full object-cover bg-muted pointer-events-none"
                    />
                    <div class="absolute inset-x-0 bottom-0 bg-gradient-to-t from-black/95 via-black/60 to-transparent p-3 pt-8 pointer-events-none">
                        <span class="line-clamp-2 text-sm font-semibold leading-tight text-white drop-shadow-md select-none">
                            {node.title}
                        </span>
                    </div>
                </div>
            {/each}
        </div>
    </div>

    <div class="absolute bottom-4 right-4 z-20 flex flex-col items-end gap-3 pointer-events-none">
        <div class="flex gap-2 pointer-events-auto">
            <Button variant="outline" size="icon" class="h-8 w-8 bg-background/95 shadow-md" onclick={() => adjustZoom(0.1)} title="Zoom In">
                <ZoomIn class="size-4" />
            </Button>
            <Button variant="outline" size="icon" class="h-8 w-8 bg-background/95 shadow-md" onclick={() => adjustZoom(-0.1)} title="Zoom Out">
                <ZoomOut class="size-4" />
            </Button>
            <Button variant="outline" size="icon" class="h-8 w-8 bg-background/95 shadow-md" onclick={fitToScreen} title="Recenter">
                <LocateFixed class="size-4" />
            </Button>
        </div>

        <div class="flex flex-wrap justify-end gap-x-4 gap-y-1.5 rounded-md border bg-background/95 backdrop-blur px-4 py-2 text-xs shadow-md pointer-events-auto w-full">
            {#each presentTypes as type}
                <div class="flex items-center gap-1.5">
                    <span class="size-2.5 rounded-full flex-shrink-0" style="background: {relationColor(type)}"></span>
                    <span class="text-muted-foreground font-medium">{relationLabel(type)}</span>
                </div>
            {/each}
        </div>
    </div>
</div>