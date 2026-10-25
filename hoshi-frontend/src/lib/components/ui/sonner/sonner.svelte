<script lang="ts">
	import Loader2Icon from "@lucide/svelte/icons/loader-2";

	import { type ToasterProps as SonnerProps } from "svelte-sonner";
	import { themeManager } from "@/stores/theme.svelte.js";

	let { ...restProps }: SonnerProps = $props();
</script>

<Sonner
		theme={themeManager.theme === 'light' ? 'light' : 'dark'}
	class="toaster group"
	style="--normal-bg: var(--color-popover); --normal-text: var(--color-popover-foreground); --normal-border: var(--color-border);"
	{...restProps}
	>{#snippet loadingIcon()}
		<Loader2Icon class="size-4 animate-spin" />
	{/snippet}
	{#snippet successIcon()}
		<CircleCheckIcon class="size-4" />
	{/snippet}
	{#snippet errorIcon()}
		<OctagonXIcon class="size-4" />
	{/snippet}
	{#snippet infoIcon()}
		<InfoIcon class="size-4" />
	{/snippet}
	{#snippet warningIcon()}
		<TriangleAlertIcon class="size-4" />
	{/snippet}
</Sonner>
