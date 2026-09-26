<script lang="ts">
    import { Spinner } from "$lib/components/ui/spinner";
    import { AlertCircle } from "lucide-svelte";

    let {
        isLoading,
        error
    }: {
        isLoading: boolean;
        error?: { key?: string; message?: string } | null;
    } = $props();
</script>

{#if isLoading}
    <div class="absolute inset-0 z-30 flex items-center justify-center pointer-events-none">
        <Spinner class="size-10" />
    </div>
{/if}

{#if error}
    <div class="absolute inset-x-0 top-20 z-30 flex justify-center px-6 pointer-events-none">
        <div class="pointer-events-auto max-w-lg w-full bg-destructive/10 border border-destructive/30 text-destructive-foreground p-3.5 rounded-xl shadow-2xl backdrop-blur-md flex items-start gap-3">
            <AlertCircle class="w-5 h-5 text-destructive shrink-0 mt-0.5" />
            <div class="flex-1 text-xs space-y-0.5">
                <p class="font-semibold text-destructive">
                    {error.key || 'Playback Error'}
                </p>
                {#if error.message}
                    <p class="text-destructive/80 leading-relaxed">
                        {error.message}
                    </p>
                {/if}
            </div>
        </div>
    </div>
{/if}