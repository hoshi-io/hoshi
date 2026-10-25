<script lang="ts">
    let {
        checked = $bindable(false),
        disabled = false,
        label,
        onchange
    }: {
        checked?: boolean;
        disabled?: boolean;
        label?: string;
        onchange?: (checked: boolean) => void;
    } = $props();

    function toggle() {
        if (disabled) return;
        checked = !checked;
        onchange?.(checked);
    }
</script>

<button
        type="button"
        data-sn
        role="switch"
        aria-checked={checked}
        aria-label={label}
        {disabled}
        onclick={toggle}
        class="relative h-9 w-16 shrink-0 scroll-my-24 rounded-full border-2 outline-none transition-colors
               focus:ring-4 focus:ring-primary focus:ring-offset-2 focus:ring-offset-background
               {checked ? 'border-primary bg-primary' : 'border-border bg-muted'}"
>
    <span
            class="absolute top-0.5 size-7 rounded-full bg-white shadow transition-all duration-150
                   {checked ? 'left-[1.875rem]' : 'left-0.5'}"
    ></span>
</button>