<script lang="ts">
    import { listStore } from "@/app/list.svelte";
    import { openListEditor } from "@/stores/layout.svelte";
    import { Button } from "@/components/ui/button";
    import { Spinner } from "@/components/ui/spinner";
    import { Check, BookmarkPlus } from "lucide-svelte";
    import {cn} from "@/utils";

    let {
        cid,
        title,
        contentType,
        coverImage,
        size = "icon",
        variant = "secondary",
        headers,
        class: className = ""
    } = $props<{
        cid: string;
        title: string;
        contentType: string;
        coverImage?: string;
        size?: "default" | "sm" | "lg" | "icon" | null;
        variant?: "default" | "destructive" | "outline" | "secondary" | "ghost" | "link" | null;
        class?: string;
        headers?: any
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