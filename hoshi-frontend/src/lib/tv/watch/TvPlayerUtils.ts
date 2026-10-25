import type { PlaybackTrack } from '@/app/watch.svelte.js';

export type TvPanelId = 'episodes' | 'audio' | 'subtitles' | 'quality' | 'source' | 'server' | 'torrent';

export function formatTime(seconds: number): string {
    const s = Math.max(0, Math.floor(seconds || 0));
    const h = Math.floor(s / 3600);
    const m = Math.floor((s % 3600) / 60);
    const sec = s % 60;
    if (h > 0) return `${h}:${String(m).padStart(2, '0')}:${String(sec).padStart(2, '0')}`;
    return `${m}:${String(sec).padStart(2, '0')}`;
}

let names: Intl.DisplayNames | null = null;
try {
    names = new Intl.DisplayNames(['en'], { type: 'language' });
} catch {
    names = null;
}

function languageName(code: string): string | null {
    const normalized = code.trim().toLowerCase();
    if (!normalized || normalized === 'und' || !names) return null;
    try {
        const name = names.of(normalized);
        if (!name || name.toLowerCase() === normalized) return null;
        return name;
    } catch {
        return null;
    }
}

const isFilenameLike = (title: string) => /\.(srt|vtt|ass|ssa|sub|ttml)$/i.test(title.trim());

export function baseTrackLabel(track: PlaybackTrack): string {
    if (track.title && !isFilenameLike(track.title)) return track.title;
    if (track.lang) return languageName(track.lang) || track.lang;
    return track.title || `Track ${track.id}`;
}

/** Labels for a list of tracks; duplicates get a "(2)", "(3)" suffix. */
export function buildTrackLabels(tracks: PlaybackTrack[]): Map<number, string> {
    const bases = tracks.map((t) => ({ id: t.id, base: baseTrackLabel(t) }));
    const totals = new Map<string, number>();
    for (const { base } of bases) totals.set(base, (totals.get(base) ?? 0) + 1);

    const seen = new Map<string, number>();
    const labels = new Map<number, string>();
    for (const { id, base } of bases) {
        if ((totals.get(base) ?? 1) <= 1) {
            labels.set(id, base);
            continue;
        }
        const index = (seen.get(base) ?? 0) + 1;
        seen.set(base, index);
        labels.set(id, index === 1 ? base : `${base} (${index})`);
    }
    return labels;
}

export function qualityLabel(track: PlaybackTrack): string {
    if (track.title) return track.title;
    const parts: string[] = [];
    if (track.h) parts.push(`${track.h}p`);
    if (track.bitrate) {
        const mbps = track.bitrate / 1_000_000;
        parts.push(mbps >= 1 ? `${mbps.toFixed(1)} Mbps` : `${Math.round(track.bitrate / 1000)} kbps`);
    }
    return parts.length ? parts.join(' · ') : `Track ${track.id}`;
}