<script lang="ts">
    interface Props {
        currentTime: number;
        duration: number;
        compact?: boolean;
        mode?: 'all' | 'current' | 'total';
    }

    let { currentTime, duration, compact = false, mode = 'all' }: Props = $props();

    function format(s: number): string {
        if (!isFinite(s) || s < 0) s = 0;
        const h = Math.floor(s / 3600);
        const m = Math.floor((s % 3600) / 60);
        const sec = Math.floor(s % 60);
        if (h > 0) {
            return `${h}:${String(m).padStart(2, '0')}:${String(sec).padStart(2, '0')}`;
        }
        return `${m}:${String(sec).padStart(2, '0')}`;
    }

    const remaining = $derived(duration - currentTime);
</script>

<div class="time-display">
    {#if mode === 'all' || mode === 'current'}
        <span class="current">{format(currentTime)}</span>
    {/if}

    {#if mode === 'all'}
        <span class="separator">/</span>
    {/if}

    {#if mode === 'all' || mode === 'total'}
        {#if compact}
            <span class="total">-{format(remaining)}</span>
        {:else}
            <span class="total">{format(duration)}</span>
        {/if}
    {/if}
</div>

<style>
    .time-display {
        display: flex;
        align-items: center;
        gap: 3px;
        font-variant-numeric: tabular-nums;
        font-size: 13px;
        font-weight: 500;
        letter-spacing: 0.02em;
        color: rgba(255, 255, 255, 0.8);
    }
</style>