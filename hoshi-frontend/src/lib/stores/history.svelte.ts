export interface HistoryEntry {
    id: number | string;
    title: string;
    coverImage: string | null;
}

const MAX_HISTORY = 20;

class HistoryStore {
    entries = $state<HistoryEntry[]>([]);

    add(entry: HistoryEntry) {
        this.entries = [entry, ...this.entries.filter((e) => e.id !== entry.id)].slice(
            0,
            MAX_HISTORY
        );
    }

    remove(id: HistoryEntry['id']) {
        this.entries = this.entries.filter((e) => e.id !== id);
    }

    clear() {
        this.entries = [];
    }
}

export const historyStore = new HistoryStore();