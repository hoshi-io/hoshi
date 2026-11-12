<script lang="ts">
    import { listStore } from "@/app/list.svelte";
    import { Button } from "@/components/ui/button";
    import { Spinner } from "@/components/ui/spinner";
    import { Check, BookmarkPlus } from "lucide-svelte";
    import {cn} from "@/utils";
    import { layoutState, openListEditor } from "@/stores/layout.svelte";
    import { contentApi } from "@/api/content/content";
    import type { CardSource } from "@/utils/normalize";
    import { toast } from "svelte-sonner";
    import { i18n } from "@/stores/i18n.svelte";

    let {
        cid,
        source = null,
        title,
        contentType,
        coverImage,
        size = "icon",
        variant = "secondary",
        headers,
        class: className = "",
        label
    } = $props<{
        cid: string | null;
        source?: CardSource | null;
        title: string;
        contentType: string;
        coverImage?: string | null;
        size?: "default" | "sm" | "lg" | "icon" | null;
        variant?: "default" | "destructive" | "outline" | "secondary" | "ghost" | "link" | null;
        class?: string;
        headers?: any
        label?: string;
    }>();

    let resolvedCid = $state<string | null>(null);
    let resolving = $state(false);

    const effectiveCid = $derived(cid ?? resolvedCid);
    const isInList = $derived(!!effectiveCid && listStore.hasCid(effectiveCid));
    const busy = $derived(listStore.isLoading || resolving);

    async function handleClick() {
        if (resolving) return;
        let target = effectiveCid;
        if (!target) {
            if (!source) return;
            resolving = true;
            try {
                target = (await contentApi.get(source.name, source.id)).content.cid;
                resolvedCid = target;
            } catch (e: any) {
                toast.error(i18n.t(e?.key || 'error.system.unknown'));
                return;
            } finally {
                resolving = false;
            }
        }
        openListEditor({
            cid: target,
            title,
            contentType,
            coverImage,
            headers
        });
    }

</script>

{#if layoutState.isTV}
    <button
            type="button"
            data-sn
            onclick={handleClick}
            class={cn(
                "flex h-14 scroll-my-24 items-center gap-2 rounded-lg border-2 px-6 text-base font-bold outline-none",
                "transition-transform duration-150 focus:scale-105 focus:ring-4 focus:ring-primary focus:ring-offset-2 focus:ring-offset-background",
                isInList ? "border-primary bg-primary/15 text-primary" : "border-border/60 bg-card text-foreground",
                className
            )}
    >
        {#if busy}
            <Spinner class="size-5" />
        {:else if isInList}
            <Check class="size-5" />
        {:else}
            <BookmarkPlus class="size-5" />
        {/if}
        {#if label}{label}{/if}
    </button>
{:else}
    <Button
            {size}
            {variant}
            class={cn("rounded-sm", className)}
            onclick={handleClick}
    >
        {#if busy}
            <Spinner class="w-4 h-4" />
        {:else if isInList}
            <Check class="w-4 h-4 text-green-500" />
        {:else}
            <BookmarkPlus class="w-4 h-4" />
        {/if}
    </Button>
{/if}