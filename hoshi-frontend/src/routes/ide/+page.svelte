<script lang="ts">
    import * as Resizable from '$lib/components/ui/resizable';
    import * as Tabs from '$lib/components/ui/tabs';
    import * as Select from '$lib/components/ui/select';
    import * as Avatar from '$lib/components/ui/avatar';
    import * as AlertDialog from '$lib/components/ui/alert-dialog';
    import { Button } from '$lib/components/ui/button';
    import { ScrollArea } from '$lib/components/ui/scroll-area';
    import { Plus, Play, Trash2, BookOpen, Terminal, FolderCode, SquareFunction } from 'lucide-svelte';
    import { openUrl } from '@tauri-apps/plugin-opener';
    import CodeEditor from '@/components/ide/CodeEditor.svelte';
    import ParamInput from '@/components/ide/ParamInput.svelte';
    import NewExtensionDialog from '@/components/ide/NewExtensionDialog.svelte';
    import { sortedFunctions } from '@/components/ide/function-signatures';
    import Card from '$lib/components/settings/extensions/Card.svelte';
    import {ideState} from "@/app/idle-state.svelte.js";

    let newExtOpen = $state(false);
    let deleteTarget = $state<string | null>(null);
    let activeTab = $state('code');

    $effect(() => {
        ideState.loadDevExtensions();
    });

    const DOCS = {
        root: 'https://hoshi-io.github.io/docs/extensions/getting-started',
        manifest: 'https://hoshi-io.github.io/docs/extensions/extension-manifest',
    };

    function initials(name: string) {
        return name.trim().slice(0, 1).toUpperCase() || '?';
    }
</script>

<Resizable.PaneGroup direction="horizontal" class="h-full">

    <!-- Left: extension list -->
    <Resizable.Pane defaultSize={17} minSize={14} maxSize={28}>
        <div class="flex h-full flex-col border-r border-border/60">
            <div class="flex items-center justify-between px-3.5 py-3">
                <div class="flex items-center gap-2">
                    <FolderCode class="h-3.5 w-3.5 text-muted-foreground/70" />
                    <span class="text-xs font-semibold uppercase tracking-wide text-muted-foreground">
            Dev Extensions
          </span>
                </div>
                <Button
                        size="icon"
                        variant="ghost"
                        class="h-7 w-7 rounded-sm transition-transform active:scale-90"
                        onclick={() => (newExtOpen = true)}
                >
                    <Plus class="h-4 w-4" />
                </Button>
            </div>

            <ScrollArea class="flex-1 px-2">
                {#if ideState.isLoading}
                    <p class="px-2 py-4 text-sm text-muted-foreground">Loading…</p>
                {:else if !ideState.devExtensions.length}
                    <div class="flex flex-col items-center gap-3 px-4 py-14 text-center">
                        <div class="flex h-10 w-10 items-center justify-center rounded-sm bg-muted/60">
                            <FolderCode class="h-5 w-5 text-muted-foreground/50" />
                        </div>
                        <div class="space-y-1">
                            <p class="text-sm font-medium text-foreground/80">No dev extensions</p>
                            <p class="text-xs text-muted-foreground">Create one to start testing</p>
                        </div>
                        <Button size="sm" variant="secondary" class="rounded-sm mt-1" onclick={() => (newExtOpen = true)}>
                            <Plus class="h-3.5 w-3.5" />
                            New extension
                        </Button>
                    </div>
                {/if}

                {#each ideState.devExtensions as ext (ext.id)}
                    {@const active = ideState.activeExtensionId === ext.id}
                    <div
                            class="group flex items-center gap-2 rounded-sm px-2 py-1.5 cursor-pointer transition-colors
      {active ? 'bg-accent text-accent-foreground' : 'hover:bg-muted/50 text-foreground/80'}"
                            onclick={() => ideState.openExtension(ext.id)}
                            role="button"
                            tabindex="0"
                    >
                        <Avatar.Root class="h-5 w-5 shrink-0 rounded-sm text-[10px]">
                            {#if ext.icon}
                                <Avatar.Image src={ext.icon} alt="" />
                            {/if}
                            <Avatar.Fallback class="rounded-sm {active ? 'bg-primary/20 text-primary' : 'bg-muted text-muted-foreground'}">
                                {initials(ext.name)}
                            </Avatar.Fallback>
                        </Avatar.Root>

                        <span class="flex-1 truncate text-sm">{ext.name}</span>

                        <button
                                class="opacity-0 group-hover:opacity-100 transition-opacity shrink-0 rounded-sm p-1 hover:bg-destructive/10 hover:text-destructive text-muted-foreground/50"
                                onclick={(e) => { e.stopPropagation(); deleteTarget = ext.id; }}
                        >
                            <Trash2 class="h-3 w-3" />
                        </button>
                    </div>
                {/each}
            </ScrollArea>

            <div class="border-t border-border/60 p-2">
                <button
                        onclick={() => openUrl(DOCS.root)}
                        class="flex w-full items-center gap-2 rounded-sm px-2.5 py-2 text-sm text-muted-foreground hover:bg-muted/60 hover:text-foreground transition-colors"
                >
                    <BookOpen class="h-4 w-4" />
                    Extension docs
                </button>
            </div>
        </div>
    </Resizable.Pane>

    <Resizable.Handle />

    <!-- Center: editor + console -->
    <Resizable.Pane defaultSize={56}>
        <Resizable.PaneGroup direction="vertical">
            <Resizable.Pane defaultSize={68}>
                <Tabs.Root bind:value={activeTab} class="flex h-full flex-col gap-0">
                    <div class="flex items-center justify-between border-b border-border/60 px-1.5">
                        <Tabs.List class="h-10 bg-transparent gap-1">
                            <Tabs.Trigger value="code" class="text-sm rounded-sm transition-colors data-[state=active]:shadow-none">Code</Tabs.Trigger>
                            <Tabs.Trigger value="manifest" class="text-sm rounded-sm transition-colors data-[state=active]:shadow-none">Manifest</Tabs.Trigger>
                        </Tabs.List>

                        <div class="flex items-center gap-3 pr-2">
                            {#if activeTab === 'manifest' && ideState.activeExtensionId}
                                <Button
                                        size="sm"
                                        variant="secondary"
                                        class="h-7 rounded-sm text-xs transition-transform active:scale-95"
                                        onclick={() => ideState.saveManifest()}
                                >
                                    Save manifest
                                </Button>
                            {/if}

                            {#if ideState.activeExtensionId}
                                <button
                                        onclick={() => openUrl(DOCS.manifest)}
                                        class="text-xs text-muted-foreground/60 hover:text-foreground transition-colors"
                                >
                                    reference ↗
                                </button>
                            {/if}
                        </div>
                    </div>

                    <Tabs.Content value="code" class="m-0 flex-1 overflow-hidden">
                        {#if ideState.activeExtensionId}
                            <CodeEditor bind:value={ideState.source} lang="javascript" />
                        {:else}
                            <div class="flex h-full flex-col items-center justify-center gap-2 text-muted-foreground">
                                <SquareFunction class="h-8 w-8 opacity-30" />
                                <p class="text-sm">Select or create an extension to begin</p>
                            </div>
                        {/if}
                    </Tabs.Content>

                    <Tabs.Content value="manifest" class="m-0 flex-1 overflow-hidden">
                        {#if ideState.activeExtensionId && ideState.activeExtension}
                            <div class="flex h-full flex-col">
                                <div class="border-b border-border/60 p-3 bg-muted/10">
                                    <Card
                                            ext={ideState.activeExtension}
                                            mode="installed"
                                            onSave={async () => true}
                                            onAction={() => {}}
                                    />
                                </div>
                                <div class="flex-1 overflow-hidden">
                                    <CodeEditor bind:value={ideState.manifestRaw} lang="yaml" />
                                </div>
                            </div>
                        {:else}
                            <div class="flex h-full flex-col items-center justify-center gap-2 text-muted-foreground">
                                <SquareFunction class="h-8 w-8 opacity-30" />
                                <p class="text-sm">Select or create an extension to begin</p>
                            </div>
                        {/if}
                    </Tabs.Content>
                </Tabs.Root>
            </Resizable.Pane>

            <Resizable.Handle />

            <Resizable.Pane defaultSize={32} minSize={12}>
                <div class="flex h-full flex-col border-t border-border/60">
                    <div class="flex items-center gap-2 px-3.5 py-2">
                        <Terminal class="h-3.5 w-3.5 text-muted-foreground/70" />
                        <span class="text-xs font-medium uppercase tracking-wide text-muted-foreground">
              Console
            </span>
                    </div>
                    <div class="flex-1 overflow-auto px-3.5 pb-3">
                        {#each ideState.consoleLines as line}
                            <div class="font-mono text-sm leading-relaxed whitespace-pre {line.startsWith('[error]') ? 'text-destructive' : 'text-foreground/80'}">
                                {line}
                            </div>
                        {:else}
                            <p class="text-sm text-muted-foreground/50 pt-1">Run a function to see output here.</p>
                        {/each}
                    </div>
                </div>
            </Resizable.Pane>
        </Resizable.PaneGroup>
    </Resizable.Pane>

    <Resizable.Handle />

    <!-- Right: run panel -->
    <Resizable.Pane defaultSize={27} minSize={22} maxSize={38}>
        <div class="flex h-full flex-col">
            <div class="flex items-center gap-2 border-b border-border/60 px-4 py-3">
                <SquareFunction class="h-3.5 w-3.5 text-muted-foreground/70" />
                <span class="text-xs font-semibold uppercase tracking-wide text-muted-foreground">
          Run
        </span>
            </div>

            <div class="flex flex-1 flex-col gap-4 p-4 min-h-0">
                <div class="space-y-3">
                    <div>
                        <label class="text-xs font-medium uppercase tracking-wide text-muted-foreground">Function</label>
                        <Select.Root
                                type="single"
                                value={ideState.selectedFunction ?? ''}
                                onValueChange={(v) => ideState.selectFunction(v)}
                        >
                            <Select.Trigger class="mt-1.5 w-full rounded-sm text-sm transition-colors">
                                {ideState.selectedFunction ?? 'Select function'}
                            </Select.Trigger>
                            <Select.Content class="rounded-sm">
                                {#each sortedFunctions(ideState.availableFunctions) as fn (fn.name)}
                                    <Select.Item value={fn.name}>{fn.name}</Select.Item>
                                {/each}
                            </Select.Content>
                        </Select.Root>
                    </div>

                    {#if ideState.currentFunctionDef?.params.length}
                        <div class="space-y-3 rounded-sm border border-border/50 bg-muted/10 p-3">
                            {#each ideState.currentFunctionDef.params as def (def.name)}
                                <ParamInput {def} bind:value={ideState.paramValues[def.name]} />
                            {/each}
                        </div>
                    {:else if ideState.currentFunctionDef}
                        <p class="text-sm text-muted-foreground/70 italic">This function takes no parameters.</p>
                    {:else}
                        <p class="text-sm text-muted-foreground/70">Select a function to see params.</p>
                    {/if}

                    <Button
                            class="w-full gap-2 rounded-sm transition-all active:scale-[0.98]"
                            disabled={!ideState.selectedFunction}
                            onclick={() => ideState.runFunction(ideState.selectedFunction!, ideState.buildArgs())}
                    >
                        <Play class="h-4 w-4" />
                        Run
                    </Button>
                </div>

                <div class="h-px bg-border/60"></div>

                <div class="flex flex-1 flex-col min-h-0">
                    <label class="text-xs font-medium uppercase tracking-wide text-muted-foreground mb-1.5">
                        Result
                    </label>
                    <div class="flex-1 overflow-auto rounded-sm border border-border/60 bg-muted/20 p-3">
                        <pre class="text-sm w-max min-w-full">{JSON.stringify(ideState.lastResult, null, 2)}</pre>
                    </div>
                </div>
            </div>
        </div>
    </Resizable.Pane>

</Resizable.PaneGroup>

<NewExtensionDialog bind:open={newExtOpen} />

<AlertDialog.Root open={!!deleteTarget} onOpenChange={(v) => !v && (deleteTarget = null)}>
    <AlertDialog.Content class="rounded-sm">
        <AlertDialog.Header>
            <AlertDialog.Title>Delete extension?</AlertDialog.Title>
            <AlertDialog.Description>
                This permanently deletes the extension folder and its files. This can't be undone.
            </AlertDialog.Description>
        </AlertDialog.Header>
        <AlertDialog.Footer>
            <AlertDialog.Cancel class="rounded-sm">Cancel</AlertDialog.Cancel>
            <AlertDialog.Action
                    class="rounded-sm bg-destructive text-destructive-foreground hover:bg-destructive/90"
                    onclick={() => { if (deleteTarget) ideState.deleteExtension(deleteTarget); deleteTarget = null; }}
            >
                Delete
            </AlertDialog.Action>
        </AlertDialog.Footer>
    </AlertDialog.Content>
</AlertDialog.Root>