export type OAuthFlow = 'implicit' | 'code' | 'pkce' | 'password';

export interface TrackerAuthConfig {
    oauthFlow: OAuthFlow;
    authUrl: string;
    tokenUrl?: string | null;
    clientId?: string | null;
    scopes: string[];
}

export interface TrackerInfo {
    name: string;
    displayName: string;
    iconUrl: string;
    supportedTypes: string[];
    auth: TrackerAuthConfig;
    connected: boolean;
    trackerUserId?: string | null;
    syncEnabled?: boolean | null;
    displayName_user?: string | null;
    avatarUrl?: string | null;
    profileUrl?: string | null;
    totalEntries?: number | null;
    lastSyncedAt?: number | null;
}

export interface TrackerIntegration {
    trackerName: string;
    accessToken?: string;
    username?: string;
    password?: string;
    codeVerifier?: string;
}

export interface AddIntegrationRequest {
    trackerName: string;
    accessToken?: string;
    username?: string;
    password?: string;
    codeVerifier?: string;
}