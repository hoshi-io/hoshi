export type OscId = 'modernz' | 'mpv-osc-modern' | 'hayase-osc';

export interface DownloadOscRequest {
    name: OscId;
}

export interface DownloadScriptRequest {
    name: string;
}