<script lang="ts">
    import { Spinner } from "@/components/ui/spinner";
    import { toast } from "svelte-sonner";
    import type { ExtensionsConfig } from "@/api/config/types";
    import { extensionsApi } from "@/api/extensions/extensions";
    import { extensions } from "@/stores/extensions.svelte.js";
    import { i18n } from "@/stores/i18n.svelte.js";
    import Card from "./Card.svelte";

    let {
        config = $bindable(),
        onSave
    }: {
        config: ExtensionsConfig,
        onSave: () => Promise<void> | void
    } = $props();

    let uninstallingIds = $state<Set<string>>(new Set());
    let savingIds = $state<Set<string>>(new Set());

    async function handleUninstall(id: string) {
        uninstallingIds = new Set(uninstallingIds).add(id);
        try {
            await extensions.uninstall(id);
            toast.success(i18n.t('settings.extension_section.extension_uninstalled'));
        } catch (error: any) {
            const errorMessage = typeof error === 'string' ? error : error?.message || i18n.t('errors.unknown');
            toast.error(errorMessage);
        } finally {
            const newSet = new Set(uninstallingIds);
            newSet.delete(id);
            uninstallingIds = newSet;
        }
    }

    async function handleSaveSettings(id: string, newSettings: Record<string, any>): Promise<boolean> {
        savingIds = new Set(savingIds).add(id);
        try {
            const res = await extensionsApi.updateSettings(id, newSettings);
            if (res.ok) {
                const index = extensions.installed.findIndex(e => e.id === id);
                if (index !== -1) {
                    extensions.installed[index].settings = { ...newSettings };
                }
                toast.success(i18n.t('settings.extension_section.changes_updated'));
                return true;
            }
            return false;
        } catch (error: any) {
            const errorMessage = typeof error === 'string' ? error : error?.message || i18n.t('errors.unknown');
            toast.error(errorMessage);
            return false;
        } finally {
            const newSet = new Set(savingIds);
            newSet.delete(id);
            savingIds = newSet;
        }
    }
</script>

<div class="space-y-6">
    <div class="space-y-4">
        {#if extensions.loading && extensions.installed.length === 0}
            <div class="flex justify-center py-8">
                <Spinner class="h-8 w-8 animate-spin text-muted-foreground" />
            </div>
        {:else if extensions.installed.length === 0}
            <div class="py-12 text-center border border-dashed border-border/40 rounded-2xl bg-muted/5">
                <p class="text-muted-foreground font-medium">{i18n.t('settings.extension_section.no_installed')}</p>
            </div>
        {:else}
            <div class="grid grid-cols-1 lg:grid-cols-2 gap-3">
                {#each extensions.installed as ext (ext.id)}
                    <Card
                            {ext}
                            source={ext.id.startsWith('lnr_') ? 'lnreader' : undefined}
                            isSaving={savingIds.has(ext.id)}
                            onAction={handleUninstall}
                            onSave={handleSaveSettings}
                    />
                {/each}
            </div>
        {/if}
    </div>
</div>