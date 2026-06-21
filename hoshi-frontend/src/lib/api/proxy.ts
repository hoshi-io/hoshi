import { type as getOsType } from "@tauri-apps/plugin-os";

export interface ProxyParams {
    url: string;
    referer?: string;
    origin?: string;
    userAgent?: string;
}

export function buildTauriProxyUrl(params: ProxyParams): string {
    const osType = getOsType();

    const proxyBaseUrl =
        osType === "linux"
            ? "proxy://localhost"
            : "http://proxy.localhost/proxy";

    const query = new URLSearchParams();

    query.set("url", params.url.trim());

    if (params.referer?.trim()) {
        query.set("referer", params.referer.trim());
    }

    if (params.origin?.trim()) {
        query.set("origin", params.origin.trim());
    }

    if (params.userAgent?.trim()) {
        query.set("userAgent", params.userAgent.trim());
    }

    return `${proxyBaseUrl}?${query.toString()}`;
}

export const proxyApi = {
    async fetch(params: ProxyParams): Promise<Blob> {
        const res = await fetch(buildTauriProxyUrl(params));

        if (!res.ok) {
            throw new Error(`Proxy error: ${res.status}`);
        }

        return res.blob();
    },
};