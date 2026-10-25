<script lang="ts">
    import { toast } from 'svelte-sonner';
    import { Trash2, AlertTriangle } from 'lucide-svelte';
    import { usersApi } from '@/api/users/users.js';
    import type { UserPrivate } from '@/api/users/types.js';
    import { i18n } from '@/stores/i18n.svelte.js';
    import * as Avatar from '@/components/ui/avatar';
    import { Spinner } from '@/components/ui/spinner';
    import TvDialog from '@/tv/components/TvDialog.svelte';

    let {
        user,
        onUpdate,
        onDeleted
    }: {
        user: UserPrivate;
        onUpdate: () => Promise<void>;
        onDeleted: () => void;
    } = $props();

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

    let showDeleteDialog = $state(false);
    let deletingAccount = $state(false);

    function closeDeleteDialog() {
        if (deletingAccount) return;
        showDeleteDialog = false;
    }

    async function deleteAccount() {
        if (deletingAccount) return;
        deletingAccount = true;
        try {
            // No password on TV: accounts can't set one here.
            await usersApi.deleteMe({ password: '' });
            showDeleteDialog = false;
            onDeleted();
        } catch (error: any) {
            toast.error(error?.message);
            showDeleteDialog = false;
        } finally {
            deletingAccount = false;
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

    <section class="mt-6 space-y-4 border-t border-border/40 pt-8">
        <header class="space-y-1">
            <h2 class="text-3xl font-black tracking-tight text-destructive">
                {i18n.t('settings.account_section.danger_zone')}
            </h2>
            <p class="text-base text-muted-foreground">{i18n.t('settings.account_section.danger_zone_desc')}</p>
        </header>

        <div class="flex items-center justify-between gap-6 rounded-lg border border-destructive/30 bg-destructive/5 p-6">
            <div class="min-w-0 space-y-1">
                <p class="text-lg font-bold leading-snug">{i18n.t('settings.account_section.delete_account')}</p>
                <p class="text-sm leading-snug text-muted-foreground">
                    {i18n.t('settings.account_section.delete_account_desc')}
                </p>
            </div>
            <button
                    type="button"
                    data-sn
                    onclick={() => (showDeleteDialog = true)}
                    class="flex h-12 shrink-0 scroll-my-24 items-center gap-2 rounded-lg bg-destructive px-8 text-base font-bold text-destructive-foreground outline-none
                           focus:ring-4 focus:ring-destructive/50 focus:ring-offset-2 focus:ring-offset-background"
            >
                <Trash2 class="size-5" />
                {i18n.t('settings.account_section.delete_account')}
            </button>
        </div>
    </section>
</div>

{#if showDeleteDialog}
    <TvDialog
            title={i18n.t('settings.account_section.are_you_sure')}
            description={i18n.t('settings.account_section.delete_warning')}
            onclose={closeDeleteDialog}
    >
        <div class="flex items-center gap-3 text-destructive">
            <AlertTriangle class="size-6 shrink-0" />
            <span class="text-base font-semibold">{user.username}</span>
        </div>

        <div class="flex justify-end gap-3 pt-2">
            <button
                    type="button"
                    data-sn
                    onclick={closeDeleteDialog}
                    disabled={deletingAccount}
                    class="h-12 rounded-lg border-2 border-border/60 px-8 text-base font-bold text-foreground/80 outline-none
                           focus:ring-4 focus:ring-primary/60 disabled:opacity-50"
            >
                {i18n.t('content.cancel')}
            </button>
            <button
                    type="button"
                    data-sn
                    onclick={deleteAccount}
                    disabled={deletingAccount}
                    class="flex h-12 items-center gap-2 rounded-lg bg-destructive px-8 text-base font-bold text-destructive-foreground outline-none
                           focus:ring-4 focus:ring-destructive/50 disabled:opacity-50"
            >
                {#if deletingAccount}<Spinner class="size-4" />{/if}
                {i18n.t('settings.account_section.confirm_delete')}
            </button>
        </div>
    </TvDialog>
{/if}