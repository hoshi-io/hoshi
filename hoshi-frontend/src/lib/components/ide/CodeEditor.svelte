<!-- CodeEditor.svelte -->
<script lang="ts">
    import { untrack } from 'svelte';
    import { EditorView, basicSetup } from 'codemirror';
    import { EditorState } from '@codemirror/state';
    import { javascript } from '@codemirror/lang-javascript';
    import { yaml } from '@codemirror/lang-yaml';
    import { oneDark } from '@codemirror/theme-one-dark';

    let {
        value = $bindable(''),
        lang = 'javascript',
    }: { value: string; lang?: 'javascript' | 'yaml' } = $props();

    let container: HTMLDivElement;
    let view: EditorView | null = null;

    function makeExtensions() {
        return [
            basicSetup,
            lang === 'yaml' ? yaml() : javascript(),
            oneDark,
            EditorView.theme({
                '&': { height: '100%' },
                '.cm-scroller': { overflow: 'auto' },
            }),
            EditorView.updateListener.of((u) => {
                if (u.docChanged) value = u.state.doc.toString();
            }),
        ];
    }

    $effect(() => {
        view = new EditorView({
            state: EditorState.create({ doc: untrack(() => value), extensions: makeExtensions() }),
            parent: container,
        });
        return () => view?.destroy();
    });

    $effect(() => {
        if (view && value !== view.state.doc.toString()) {
            view.dispatch({
                changes: { from: 0, to: view.state.doc.length, insert: value },
            });
        }
    });
</script>

<div bind:this={container} class="h-full overflow-hidden text-sm"></div>