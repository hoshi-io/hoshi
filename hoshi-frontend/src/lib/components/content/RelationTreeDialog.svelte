<script lang="ts">
    import * as Dialog from '$lib/components/ui/dialog/index.js';
    import { Button } from '$lib/components/ui/button';
    import { Network, Loader2, X } from 'lucide-svelte';
    import type { RelationGraph } from "@/api/content/types";
    import { contentApi } from "@/api/content/content";
    import RelationMap from "@/components/content/RelationMap.svelte";

    let { cid }: { cid: string } = $props();

    let open = $state(false);
    let loading = $state(false);
    let graph = $state<RelationGraph | null>(null);
    let error = $state<string | null>(null);

    async function handleOpenChange(next: boolean) {
        open = next;
        if (next && !graph) {
            loading = true;
            error = null;
            try {
                graph = await contentApi.get_relation_tree(cid);
            } catch (e) {
                error = 'Could not load the relation tree.';
                console.error(e);
            } finally {
                loading = false;
            }
        }
    }
</script>

<Dialog.Root {open} onOpenChange={handleOpenChange}>
    <Dialog.Trigger class="inline-flex items-center gap-2 background-muted">
        <Button variant="ghost" class="rounded-sm w-8 h-8 hover:bg-muted/40 text-muted-foreground hover:text-foreground">
            <Network class="size-4" />
        </Button>
    </Dialog.Trigger>

    <Dialog.Content class="!max-w-6xl w-[90vw] h-[85vh] flex flex-col p-0 gap-0 bg-background rounded-xl overflow-hidden">

        <div class="flex-1 relative bg-muted/10 min-h-0">
            {#if loading}
                <div class="flex h-full items-center justify-center text-muted-foreground gap-2">
                    <Loader2 class="size-4 animate-spin" />
                    Building relation tree...
                </div>
            {:else if error}
                <div class="flex h-full items-center justify-center text-destructive text-sm">
                    {error}
                </div>
            {:else if graph}
                <RelationMap {graph} rootCid={cid} onNavigate={() => (open = false)} />
            {/if}
        </div>
    </Dialog.Content>
</Dialog.Root>