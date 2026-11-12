import { Book, BookText, Clapperboard, Compass, Flame, Heart, Library, Sparkles, Star, Tv } from 'lucide-svelte';

export const TAB_ICONS = {
    tv: Tv,
    book: Book,
    'book-text': BookText,
    flame: Flame,
    star: Star,
    heart: Heart,
    sparkles: Sparkles,
    compass: Compass,
    library: Library,
    clapperboard: Clapperboard,
} as const;

export type TabIcon = keyof typeof TAB_ICONS;

export function tabIcon(name: string) {
    return TAB_ICONS[name as TabIcon] ?? Compass;
}
