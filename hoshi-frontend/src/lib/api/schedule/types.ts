import type {FullContent} from "@/api/content/types";

export interface AiringEntry {
    trackerId:    string;
    episode:      number;
    airingAt:     number;
    fullContent:  FullContent;
    userStatus?:   string | null;
    userProgress?: number | null;
    userScore?:    number | null;
}

export interface ScheduleQuery {
    daysBack?: number;
    daysAhead?: number;
}

export interface ScheduleResponse {
    success: boolean;
    data: AiringEntry[];
    total: number;
}