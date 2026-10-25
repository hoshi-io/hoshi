<script lang="ts">
    import { listStore } from "@/app/list.svelte";
    import { Button } from "@/components/ui/button";
    import { Spinner } from "@/components/ui/spinner";
    import { Check, BookmarkPlus } from "lucide-svelte";
    import {cn} from "@/utils";
    import { layoutState, openListEditor } from "@/stores/layout.svelte";

    let {
        cid,
        title,
        contentType,
        coverImage,
        size = "icon",
        variant = "secondary",
        headers,
        class: className = "",
        label
    } = $props<{
        cid: string;
        title: string;
        contentType: string;
        coverImage?: string | null;
        size?: "default" | "sm" | "lg" | "icon" | null;
        variant?: "default" | "destructive" | "outline" | "secondary" | "ghost" | "link" | null;
        class?: string;
        headers?: any
        label?: string;
    }>();

    const isInList = $derived(listStore.hasCid(cid));

    function handleClick() {
        openListEditor({
            cid,
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
        {#if listStore.isLoading}
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
        {#if listStore.isLoading}
            <Spinner class="w-4 h-4" />
        {:else if isInList}
            <Check class="w-4 h-4 text-green-500" />
        {:else}
            <BookmarkPlus class="w-4 h-4" />
        {/if}
    </Button>
{/if}