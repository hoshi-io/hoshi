import Book from 'phosphor-svelte/lib/BookIcon';
import BookText from 'phosphor-svelte/lib/NotebookIcon';
import Clapperboard from 'phosphor-svelte/lib/FilmSlateIcon';
import Compass from 'phosphor-svelte/lib/CompassIcon';
import Flame from 'phosphor-svelte/lib/FireIcon';
import Heart from 'phosphor-svelte/lib/HeartIcon';
import Library from 'phosphor-svelte/lib/BooksIcon';
import Sparkles from 'phosphor-svelte/lib/SparkleIcon';
import Star from 'phosphor-svelte/lib/StarIcon';
import Tv from 'phosphor-svelte/lib/TelevisionIcon';

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
