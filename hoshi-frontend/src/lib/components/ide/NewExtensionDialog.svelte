<script lang="ts">
    import * as Dialog from '$lib/components/ui/dialog';
    import * as Select from '$lib/components/ui/select';
    import { Button } from '$lib/components/ui/button';
    import { Input } from '$lib/components/ui/input';
    import { Label } from '$lib/components/ui/label';
    import {ideState} from "@/app/idle-state.svelte.js";

    let { open = $bindable(false) }: { open: boolean } = $props();

    let name = $state('');
    let extType = $state<'anime' | 'manga' | 'novel'>('anime');
    let isCreating = $state(false);
    let error = $state<string | null>(null);

    async function handleCreate() {
        if (!name.trim()) {
            error = 'Name is required';
            return;
        }
        isCreating = true;
        error = null;
        try {
            const ext = await ideState.createExtension(name.trim(), extType);
            await ideState.openExtension(ext.id);
            reset();
            open = false;
        } catch (e) {
            error = String(e);
        } finally {
            isCreating = false;
        }
    }

    function reset() {
        name = '';
        extType = 'anime';
        error = null;
    }

    $effect(() => {
        if (!open) reset();
    });
</script>

<Dialog.Root bind:open>
    <Dialog.Content class="sm:max-w-md rounded-sm">
        <Dialog.Header>
            <Dialog.Title class="text-base">New Dev Extension</Dialog.Title>
            <Dialog.Description class="text-sm">
                Scaffolds a starter manifest and script. The id gets a
                <code class="rounded-sm bg-muted px-1 py-0.5 text-xs">-dev</code> suffix automatically.
            </Dialog.Description>
        </Dialog.Header>

        <div class="space-y-4 py-2">
            <div class="space-y-1.5">
                <Label for="ext-name" class="text-sm">Name</Label>
                <Input
                        id="ext-name"
                        class="rounded-sm"
                        bind:value={name}
                        placeholder="My Test Extension"
                        onkeydown={(e) => e.key === 'Enter' && handleCreate()}
                />
            </div>

            <div class="space-y-1.5">
                <Label class="text-sm">Type</Label>
                <Select.Root type="single" bind:value={extType}>
                    <Select.Trigger class="w-full rounded-sm">
                        {extType}
                    </Select.Trigger>
                    <Select.Content class="rounded-sm">
                        <Select.Item value="anime">Anime</Select.Item>
                        <Select.Item value="manga">Manga</Select.Item>
                        <Select.Item value="novel">Novel</Select.Item>
                    </Select.Content>
                </Select.Root>
            </div>

            {#if error}
                <p class="rounded-sm bg-destructive/10 px-2.5 py-1.5 text-xs text-destructive">{error}</p>
            {/if}
        </div>

        <Dialog.Footer>
            <Button variant="outline" class="rounded-sm" onclick={() => (open = false)}>Cancel</Button>
            <Button class="rounded-sm" onclick={handleCreate} disabled={isCreating}>
                {isCreating ? 'Creating…' : 'Create'}
            </Button>
        </Dialog.Footer>
    </Dialog.Content>
</Dialog.Root>