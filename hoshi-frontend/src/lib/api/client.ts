type HttpMethod = "GET" | "POST" | "PUT" | "PATCH" | "DELETE";

export interface CoreError {
    key: string;
    message: string;
}

function buildUrl(path: string, params?: Record<string, unknown>): string {
    if (!params) return `/api/${path}`;

    const query = Object.entries(params)
        .filter(([, v]) => v !== undefined && v !== null)
        .map(([k, v]) => `${encodeURIComponent(k)}=${encodeURIComponent(String(v))}`)
        .join("&");

    return query ? `/api/${path}?${query}` : `/api/${path}`;
}

async function httpRequest<T>(
    path: string,
    method: HttpMethod,
    options?: {
        body?: unknown;
        params?: Record<string, unknown>;
        headers?: Record<string, string>;
    }
): Promise<T> {
    const url = buildUrl(path, options?.params);
    const isRaw = options?.body instanceof Blob || options?.body instanceof ArrayBuffer;

    const res = await fetch(url, {
        method,
        credentials: "include",
        headers: isRaw
            ? { ...options?.headers }
            : { "Content-Type": "application/json", ...options?.headers },
        body: method === "GET"
            ? undefined
            : isRaw
                ? (options?.body as BodyInit)
                : JSON.stringify(options?.body),
    });

    if (!res.ok) throw new Error((await res.text()) || "API error");
    return res.json();
}

async function tauriInvoke<T>(cmd: string, args?: Record<string, unknown>): Promise<T> {
    const { invoke } = await import("@tauri-apps/api/core");

    try {
        return await invoke<T>(cmd, args ?? {});
    } catch (err: any) {
        if (err && typeof err === 'object' && 'key' in err) {
            throw err as CoreError;
        }

        throw {
            key: "error.system.unknown",
            message: typeof err === 'string' ? err : JSON.stringify(err)
        } as CoreError;
    }
}

export interface DualEndpoint<TResult, TBody = unknown, TParams = unknown, TArgs = unknown> {
    http?: {
        path: string;
        method: HttpMethod;
        body?: TBody;
        params?: TParams;
        headers?: Record<string, string>;
    };
    tauri: {
        cmd: string;
        args?: TArgs;
    };
}

export async function call<TResult, TBody = unknown, TParams = unknown, TArgs = unknown>(
    endpoint: DualEndpoint<TResult, TBody, TParams, TArgs>
): Promise<TResult> {
    return tauriInvoke<TResult>(
        endpoint.tauri.cmd,
        endpoint.tauri.args as Record<string, unknown>
    );
}