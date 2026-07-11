let enabled = false;

function getFocusables(): HTMLElement[] {
    return Array.from(document.querySelectorAll<HTMLElement>('[data-sn]'))
        .filter(el => {
            const rect = el.getBoundingClientRect();
            const style = getComputedStyle(el);
            return rect.width > 0 && rect.height > 0
                && style.visibility !== 'hidden'
                && style.display !== 'none'
                && !el.hasAttribute('disabled');
        });
}

function center(el: HTMLElement) {
    const r = el.getBoundingClientRect();
    return { x: r.left + r.width / 2, y: r.top + r.height / 2, rect: r };
}

function findNext(direction: 'up' | 'down' | 'left' | 'right'): HTMLElement | null {
    const active = document.activeElement as HTMLElement | null;
    const candidates = getFocusables();

    const current = active?.closest<HTMLElement>('[data-sn]') ?? null;

    if (!current) {
        return candidates[0] ?? null;
    }
    if (!candidates.includes(current)) {
        return candidates[0] ?? null;
    }

    const from = center(current);
    let best: HTMLElement | null = null;
    let bestScore = Infinity;

    for (const el of candidates) {
        if (el === current) continue;
        const to = center(el);
        const dx = to.x - from.x;
        const dy = to.y - from.y;

        const inDirection =
            direction === 'up' ? dy < -1 :
                direction === 'down' ? dy > 1 :
                    direction === 'left' ? dx < -1 :
                        dx > 1;
        if (!inDirection) continue;

        const primary = Math.abs(direction === 'up' || direction === 'down' ? dy : dx);
        const cross = Math.abs(direction === 'up' || direction === 'down' ? dx : dy);

        if (cross > primary) continue;

        const score = primary + cross * 2;
        if (score < bestScore) {
            bestScore = score;
            best = el;
        }
    }

    return best;
}

function onKeydown(e: KeyboardEvent) {
    const map: Record<string, 'up' | 'down' | 'left' | 'right'> = {
        ArrowUp: 'up', ArrowDown: 'down', ArrowLeft: 'left', ArrowRight: 'right',
    };
    const dir = map[e.key];
    if (!dir) return;

    // don't hijack arrow keys inside text inputs / selects / sliders
    const active = document.activeElement as HTMLElement | null;
    if (active && ['INPUT', 'TEXTAREA', 'SELECT'].includes(active.tagName)) {
        if (active.tagName === 'INPUT' && (active as HTMLInputElement).type === 'color') {
            // fine to navigate away from a color swatch
        } else {
            return;
        }
    }

    const next = findNext(dir);
    if (next) {
        e.preventDefault();
        next.focus();
        next.scrollIntoView({ block: 'nearest', inline: 'nearest' });
    }
}

export function enableSpatialNav() {
    if (enabled) return;
    enabled = true;
    window.addEventListener('keydown', onKeydown, true);
}

export function disableSpatialNav() {
    enabled = false;
    window.removeEventListener('keydown', onKeydown, true);
}

export function focusFirstIn(container: HTMLElement | Document = document) {
    const el = container.querySelector<HTMLElement>('[data-sn]');
    el?.focus();
}