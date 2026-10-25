import { invoke } from '@tauri-apps/api/core';
import type {Extension} from "@/api/extensions/types.js";
import {type FunctionDef, FUNCTIONS_BY_TYPE} from "../components/ide/function-signatures.ts";
import {STARTER_TEMPLATES} from "@/components/ide/starter-templates.js";

function errMessage(e: unknown): string {
    if (e && typeof e === 'object' && 'message' in e) {
        return String((e as { message: unknown }).message);
    }
    return String(e);
}

export class IdeState {
    devExtensions = $state<Extension[]>([]);
    activeExtensionId = $state<string | null>(null);
    source = $state<string>('');
    manifestRaw = $state<string>('');
    isLoading = $state(false);
    error = $state<string | null>(null);

    consoleLines = $state<string[]>([]);
    lastResult = $state<unknown>(null);

    selectedFunction = $state<string | null>(null);
    paramValues = $state<Record<string, unknown>>({});
    settingsValues = $state<Record<string, unknown>>({});


    get availableFunctions(): FunctionDef[] {
        const ext = this.devExtensions.find((e) => e.id === this.activeExtensionId);
        if (!ext) return [];
        return FUNCTIONS_BY_TYPE[ext.ext_type] ?? [];
    }

    get currentFunctionDef(): FunctionDef | null {
        return this.availableFunctions.find((f) => f.name === this.selectedFunction) ?? null;
    }

    get activeExtension(): Extension | null {
        return this.devExtensions.find((e) => e.id === this.activeExtensionId) ?? null;
    }

    selectFunction(name: string) {
        this.selectedFunction = name;
        const def = this.availableFunctions.find((f) => f.name === name);
        this.paramValues = {};
        def?.params.forEach((p) => {
            if (p.name === 'category') this.paramValues[p.name] = 'sub'; // documented default
            else if (p.type === 'json') this.paramValues[p.name] = '{}';
            else if (p.type === 'boolean') this.paramValues[p.name] = false;
            else if (p.type === 'number') this.paramValues[p.name] = 0;
            else this.paramValues[p.name] = '';
        });
    }

    buildArgs(): unknown[] {
        const def = this.currentFunctionDef;
        if (!def) return [];
        return def.params.map((p) => {
            const raw = this.paramValues[p.name];
            if (p.type === 'json') {
                try { return JSON.parse(raw as string); } catch { return {}; }
            }
            if (p.type === 'number') return Number(raw);
            return raw;
        });
    }

    async loadDevExtensions() {
        this.isLoading = true;
        this.error = null;
        try {
            const res = await invoke<{ extensions: Extension[] }>('list_dev_extensions');
            this.devExtensions = res.extensions;
            if (!this.activeExtensionId && this.devExtensions.length) {
                await this.openExtension(this.devExtensions[0].id);
            }
        } catch (e) {
            this.error = String(e);
        } finally {
            this.isLoading = false;
        }
    }

    async openExtension(id: string) {
        this.activeExtensionId = id;
        this.error = null;
        try {
            const [source, manifest] = await Promise.all([
                invoke<string>('read_extension_source', { id }),
                invoke<string>('read_manifest_raw', { id }),
            ]);
            this.source = source;
            this.manifestRaw = manifest;
            this.initSettingsValues();
        } catch (e) {
            this.error = String(e);
        }
    }

    async saveSource() {
        if (!this.activeExtensionId) return;
        await invoke('write_extension_source', {
            id: this.activeExtensionId,
            code: this.source,
        });
    }

    async saveManifest() {
        if (!this.activeExtensionId) return;
        try {
            const updated = await invoke<Extension>('write_manifest_raw', {
                id: this.activeExtensionId,
                yaml: this.manifestRaw,
            });
            this.devExtensions = this.devExtensions.map((e) => (e.id === updated.id ? updated : e));
            this.initSettingsValues();
        } catch (e) {
            this.consoleLines = [`[error] manifest: ${errMessage(e)}`];
        }
    }

    async createExtension(name: string, extType: 'anime' | 'manga' | 'novel' | 'torrent') {
        const ext = await invoke<Extension>('create_dev_extension', {
            name,
            extType,
            starterCode: STARTER_TEMPLATES[extType],
        });
        this.devExtensions.push(ext);
        return ext;
    }

    async runFunction(functionName: string, args: unknown[]) {
        if (!this.activeExtensionId) return;
        this.consoleLines = [];
        this.lastResult = null;
        this.error = null;
        await this.saveSource();
        try {
            const res = await invoke<{ result: unknown; console: string[]; error: string | null }>(
                'run_extension_function',
                { extensionId: this.activeExtensionId, functionName, args },
            );
            if (res.error) {
                this.consoleLines = [...res.console, `[error] ${res.error}`];
                this.lastResult = null;
            } else {
                this.consoleLines = res.console;
                this.lastResult = res.result;
            }
        } catch (e) {
            this.consoleLines = [`[error] ${String(e)}`];
        }
    }

    async deleteExtension(id: string) {
        await invoke('delete_dev_extension', { id });
        this.devExtensions = this.devExtensions.filter((e) => e.id !== id);
        if (this.activeExtensionId === id) {
            this.activeExtensionId = null;
            this.source = '';
            this.manifestRaw = '';
            if (this.devExtensions.length) await this.openExtension(this.devExtensions[0].id);
        }
    }

    initSettingsValues() {
        const ext = this.activeExtension;
        if (!ext) { this.settingsValues = {}; return; }
        const values: Record<string, unknown> = {};
        for (const def of ext.setting_definitions) {
            values[def.key] = ext.settings[def.key] ?? def.default;
        }
        this.settingsValues = values;
    }
}

export const ideState = new IdeState();