<script lang="ts">
    import { toast } from 'svelte-sonner';
    import { usersApi } from '@/api/users/users.ts';
    import type { UserPrivate } from '@/api/users/types.ts';
    import { i18n } from '@/stores/i18n.svelte.js';
    import * as Avatar from '@/components/ui/avatar';
    import { Spinner } from '@/components/ui/spinner';

    let { user, onUpdate }: { user: UserPrivate; onUpdate: () => Promise<void> } = $props();

    let username = $state('');
    $effect(() => {
        username = user.username;
    });

    let saving = $state(false);
    const hasChanges = $derived(username.trim() !== '' && username !== user.username);

    async function save(e: Event) {
        e.preventDefault();
        if (!hasChanges || saving) return;
        saving = true;
        try {
            await usersApi.updateMe({ username });
            await onUpdate();
        } catch (error: any) {
            toast.error(error?.message);
        } finally {
            saving = false;
        }
    }
</script>

<div>
    <header class="space-y-1 pb-2">
        <h2 class="text-3xl font-black tracking-tight">{i18n.t('settings.account_section.profile')}</h2>
        <p class="text-base text-muted-foreground">{i18n.t('settings.account_section.profile_desc')}</p>
    </header>

    <form onsubmit={save} class="space-y-6 py-5">
        <div class="flex items-center gap-8">
            <Avatar.Root class="size-28 shrink-0 border border-border/50 shadow-sm">
                {#if user.avatar}
                    <Avatar.Image src={user.avatar} alt={user.username} class="object-cover" />
                {/if}
                <Avatar.Fallback class="bg-primary/10 text-4xl font-bold uppercase text-primary">
                    {user.username.charAt(0)}
                </Avatar.Fallback>
            </Avatar.Root>

            <div class="min-w-0 flex-1 space-y-2">
                <label for="tv-username" class="text-lg font-bold">
                    {i18n.t('settings.account_section.username')}
                </label>
                <input
                        id="tv-username"
                        data-sn
                        type="text"
                        bind:value={username}
                        autocomplete="off"
                        class="h-12 w-full scroll-my-24 rounded-md border-2 border-border/60 bg-card px-4 text-lg outline-none
                               focus:border-primary focus:ring-4 focus:ring-primary/40"
                />
                <p class="text-sm leading-snug text-muted-foreground">
                    {i18n.t('settings.account_section.username_desc')}
                </p>
            </div>
        </div>

        <div class="flex justify-end">
            <button
                    type="submit"
                    data-sn
                    disabled={!hasChanges || saving}
                    class="flex h-12 scroll-my-24 items-center gap-2 rounded-lg bg-primary px-8 text-base font-bold text-primary-foreground outline-none
                           focus:ring-4 focus:ring-primary/50 focus:ring-offset-2 focus:ring-offset-background disabled:opacity-50"
            >
                {#if saving}<Spinner class="size-4" />{/if}
                {hasChanges ? i18n.t('settings.account_section.save') : i18n.t('settings.account_section.saved')}
            </button>
        </div>
    </form>
</div>