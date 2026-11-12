import type { Action } from 'svelte/action';

export type SortableOptions = {
    axis?: 'x' | 'y';
    /** Called every time the dragged item crosses a neighbour. */
    onmove: (from: number, to: number) => void;
    onend?: () => void;
};

/**
 * Pointer-based reordering for the direct children of `node`; works with mouse and touch.
 * Dragging starts from an element marked `data-drag-handle`; the dragged child gets `data-dragging`.
 */
export const sortable: Action<HTMLElement, SortableOptions> = (node, initial) => {
    let opts = initial;
    let dragged: HTMLElement | null = null;
    let index = -1;
    let pointerId = -1;
    let started = false;
    let startX = 0;
    let startY = 0;
    const THRESHOLD = 5;

    const items = () => Array.from(node.children) as HTMLElement[];

    function onDown(e: PointerEvent) {
        const handle = (e.target as HTMLElement).closest('[data-drag-handle]');
        if (!handle || !node.contains(handle) || e.button !== 0) return;
        // On touch, only handles that opt out of scrolling may drag; anything else is a swipe.
        if (e.pointerType === 'touch' && getComputedStyle(handle).touchAction !== 'none') return;
        const item = items().find(el => el.contains(handle));
        if (!item) return;

        // Nothing is captured until the pointer actually moves, so a plain click still reaches its target.
        dragged = item;
        index = items().indexOf(item);
        pointerId = e.pointerId;
        started = false;
        startX = e.clientX;
        startY = e.clientY;
    }

    function onMoveEvt(e: PointerEvent) {
        if (!dragged || e.pointerId !== pointerId) return;
        if (!started) {
            if (Math.hypot(e.clientX - startX, e.clientY - startY) < THRESHOLD) return;
            started = true;
            dragged.dataset.dragging = '';
            node.setPointerCapture(pointerId);
        }
        e.preventDefault();
        // Layout offsets, not client rects: reorder animations transform items mid-flight.
        const horizontal = opts.axis === 'x';
        const box = node.getBoundingClientRect();
        const pos = horizontal ? e.clientX - box.left + node.scrollLeft : e.clientY - box.top + node.scrollTop;
        const list = items();

        for (let i = 0; i < list.length; i++) {
            if (i === index) continue;
            const el = list[i];
            const mid = horizontal ? el.offsetLeft + el.offsetWidth / 2 : el.offsetTop + el.offsetHeight / 2;
            if ((i < index && pos < mid) || (i > index && pos > mid)) {
                opts.onmove(index, i);
                index = i;
                break;
            }
        }
    }

    function onUp(e: PointerEvent) {
        if (!dragged || e.pointerId !== pointerId) return;
        delete dragged.dataset.dragging;
        if (node.hasPointerCapture(pointerId)) node.releasePointerCapture(pointerId);
        if (started) {
            // Swallow the click that follows a drag so it doesn't also select or toggle.
            const swallow = (ev: Event) => { ev.stopPropagation(); ev.preventDefault(); };
            node.addEventListener('click', swallow, { capture: true, once: true });
            setTimeout(() => node.removeEventListener('click', swallow, { capture: true }), 0);
            opts.onend?.();
        }
        dragged = null;
        index = -1;
        started = false;
    }

    if (getComputedStyle(node).position === 'static') node.style.position = 'relative';

    node.addEventListener('pointerdown', onDown);
    node.addEventListener('pointermove', onMoveEvt);
    node.addEventListener('pointerup', onUp);
    node.addEventListener('pointercancel', onUp);

    return {
        update(next) { opts = next; },
        destroy() {
            node.removeEventListener('pointerdown', onDown);
            node.removeEventListener('pointermove', onMoveEvt);
            node.removeEventListener('pointerup', onUp);
            node.removeEventListener('pointercancel', onUp);
        },
    };
};
