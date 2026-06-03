<script lang="ts">
    import { usersApi } from "@/api/users/users";
    import type { UserPrivate } from "@/api/users/types";
    import { toast } from "svelte-sonner";
    import { i18n } from "@/stores/i18n.svelte.js";

    import * as Avatar from "$lib/components/ui/avatar";
    import * as AlertDialog from "$lib/components/ui/alert-dialog";
    import { Button } from "$lib/components/ui/button";
    import { Input } from "$lib/components/ui/input";
    import { Label } from "$lib/components/ui/label";
    import { Spinner } from "$lib/components/ui/spinner";

    import { Trash2, AlertTriangle, Camera } from "lucide-svelte";

    let { user, onUpdate, onDeleted }: {
        user: UserPrivate,
        onUpdate: () => Promise<void>,
        onDeleted: () => void
    } = $props();

    let username = $state("");

    $effect(() => {
        username = user.username;
    });

    let savingProfile = $state(false);
    let previewAvatarUrl = $state<string>("");

    $effect(() => {
        previewAvatarUrl = user.avatar || "";
    })

    let selectedAvatarFile = $state<File | null>(null);
    let avatarRemoved = $state(false);
    let fileInput: HTMLInputElement | undefined = $state();

    let hasChanges = $derived(
        username !== user.username ||
        selectedAvatarFile !== null ||
        avatarRemoved
    );

    let currentPassword = $state("");
    let newPassword = $state("");
    let confirmPassword = $state("");
    let savingPassword = $state(false);

    let canSavePassword = $derived(
        newPassword.length >= 2 &&
        newPassword === confirmPassword &&
        (!user.hasPassword || currentPassword.length > 0)
    );

    let showDeleteAlert = $state(false);
    let deletingAccount = $state(false);
    let deletePassword = $state("");

    function handleFileSelect(e: Event) {
        const target = e.target as HTMLInputElement;
        if (target.files && target.files.length > 0) {
            const file = target.files[0];
            selectedAvatarFile = file;
            previewAvatarUrl = URL.createObjectURL(file);
            avatarRemoved = false;
        }
    }

    function handleRemoveAvatar() {
        selectedAvatarFile = null;
        previewAvatarUrl = null;
        avatarRemoved = true;
        if (fileInput) fileInput.value = "";
    }

    async function handleUpdateProfile(e: Event) {
        e.preventDefault();
        if (!hasChanges) return;

        savingProfile = true;
        try {
            if (username !== user.username) await usersApi.updateMe({ username });

            if (selectedAvatarFile) await usersApi.uploadAvatar(selectedAvatarFile);
            else if (avatarRemoved && user.avatar) await usersApi.deleteAvatar();

            await onUpdate();

            selectedAvatarFile = null;
            avatarRemoved = false;
        } catch (error: any) {
            toast.error(error?.message);
        } finally {
            savingProfile = false;
        }
    }

    async function handleChangePassword(e: Event) {
        e.preventDefault();

        savingPassword = true;
        try {
            await usersApi.changePassword({ currentPassword, newPassword });

            currentPassword = ""; newPassword = ""; confirmPassword = "";
        } catch (error: any) {
            toast.error(error?.message);
        } finally {
            savingPassword = false;
        }
    }

    async function handleDeleteAccount() {
        if (user.hasPassword && !deletePassword) {
            toast.error(i18n.t('settings.account_section.password_required'));
            return;
        }
        deletingAccount = true;

        try {
            await usersApi.deleteMe({ password: deletePassword });
            showDeleteAlert = false;

            onDeleted();
        } catch (error: any) {
            toast.error(error?.message);
            showDeleteAlert = false;
        } finally {
            deletingAccount = false;
        }
    }
</script>

<div class="space-y-16 w-full">

    <section>
        <div class="mb-2">
            <h2 class="text-2xl font-bold tracking-tight">{i18n.t('settings.account_section.profile')}</h2>
            <p class="text-sm text-muted-foreground mt-1">{i18n.t('settings.account_section.profile_desc')}</p>
        </div>

        <form onsubmit={handleUpdateProfile} class="space-y-0">
            <div class="flex flex-col sm:flex-row items-center sm:items-start gap-8 py-8 border-b border-border/40">

                <div class="shrink-0 flex flex-col items-center gap-3 sm:w-40">
                    <button
                            type="button"
                            class="relative group rounded-full focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-primary transition-transform active:scale-95"
                            onclick={() => fileInput.click()}
                            aria-label={i18n.t('settings.account_section.change_avatar')}
                    >
                        <Avatar.Root class="h-24 w-24 sm:h-28 sm:w-28 border border-border/50 shadow-sm transition-opacity group-hover:opacity-90">
                            {#if previewAvatarUrl}
                                <Avatar.Image src={previewAvatarUrl} alt={username} class="object-cover" />
                            {/if}
                            <Avatar.Fallback class="bg-primary/10 text-primary text-4xl font-bold uppercase">
                                {username.charAt(0)}
                            </Avatar.Fallback>
                        </Avatar.Root>

                        <div class="absolute inset-0 flex items-center justify-center rounded-full bg-black/40 opacity-0 group-hover:opacity-100 transition-opacity text-white backdrop-blur-[2px]">
                            <Camera class="size-8" />
                        </div>
                    </button>
                    <input bind:this={fileInput} type="file" accept="image/png, image/jpeg, image/webp" class="hidden" onchange={handleFileSelect} />

                    <div class="flex items-center gap-3 text-sm">
                        <button type="button" class="font-semibold text-muted-foreground hover:text-foreground transition-colors" onclick={() => fileInput.click()}>
                            {i18n.t('settings.account_section.change_avatar').split(' ')[0]}
                        </button>
                        {#if previewAvatarUrl}
                            <span class="text-border/50">•</span>
                            <button type="button" class="font-semibold text-destructive hover:underline transition-all" onclick={handleRemoveAvatar}>
                                {i18n.t('settings.account_section.remove')}
                            </button>
                        {/if}
                    </div>
                </div>

                <div class="flex-1 w-full space-y-2 sm:pt-2">
                    <Label for="username" class="text-base font-bold">{i18n.t('settings.account_section.username')}</Label>
                    <Input id="username" bind:value={username} class="rounded-sm h-11 w-full sm:max-w-md" required />
                    <p class="text-sm text-muted-foreground">{i18n.t('settings.account_section.username_desc')}</p>
                </div>
            </div>

            <div class="flex justify-end pt-8">
                <Button type="submit" disabled={!hasChanges || savingProfile} class="rounded-sm px-8 h-11 font-bold shadow-sm transition-all">
                    {#if savingProfile}<Spinner class="mr-2 h-4 w-4" />{/if}
                    {hasChanges ? i18n.t('settings.account_section.save') : i18n.t('settings.account_section.saved')}
                </Button>
            </div>
        </form>
    </section>

    <section>
        <div class="mb-2">
            <h2 class="text-2xl font-bold tracking-tight">{i18n.t('settings.account_section.security')}</h2>
            <p class="text-sm text-muted-foreground mt-1">{i18n.t('settings.account_section.security_desc')}</p>
        </div>

        <form onsubmit={handleChangePassword} class="space-y-0">
            {#if user.hasPassword}
                <div class="flex flex-col sm:flex-row sm:items-center justify-between gap-4 py-6 border-b border-border/40">
                    <div class="space-y-1 pr-4">
                        <Label for="currentPassword" class="text-base font-bold">{i18n.t('settings.account_section.current_password')}</Label>
                        <p class="text-sm text-muted-foreground">{i18n.t('settings.account_section.current_password_desc')}</p>
                    </div>
                    <Input id="currentPassword" type="password" bind:value={currentPassword} class="rounded-sm h-11 w-full sm:max-w-md" required />
                </div>
            {/if}

            <div class="flex flex-col sm:flex-row sm:items-center justify-between gap-4 py-6 border-b border-border/40">
                <div class="space-y-1 pr-4">
                    <Label for="newPassword" class="text-base font-bold">{i18n.t('settings.account_section.new_password')}</Label>
                    <p class="text-sm text-muted-foreground">{i18n.t('settings.account_section.new_password_desc')}</p>
                </div>
                <Input id="newPassword" type="password" bind:value={newPassword} class="rounded-sm h-11 w-full sm:max-w-md" required />
            </div>

            <div class="flex flex-col sm:flex-row sm:items-center justify-between gap-4 py-6 border-b border-border/40">
                <div class="space-y-1 pr-4">
                    <Label for="confirmPassword" class="text-base font-bold">{i18n.t('settings.account_section.confirm_password')}</Label>
                    <p class="text-sm text-muted-foreground">{i18n.t('settings.account_section.confirm_new_password')}</p>
                </div>
                <Input id="confirmPassword" type="password" bind:value={confirmPassword} class="rounded-sm h-11 w-full sm:max-w-md" required />
            </div>

            <div class="flex justify-end pt-8">
                <Button type="submit" disabled={!canSavePassword || savingPassword} variant="secondary" class="rounded-sm px-8 h-11 font-bold shadow-sm transition-all">
                    {#if savingPassword}<Spinner class="mr-2 h-4 w-4" />{/if}
                    {i18n.t('settings.account_section.update_password')}
                </Button>
            </div>
        </form>
    </section>

    <section>
        <div class="mb-2">
            <h2 class="text-xl font-bold tracking-tight text-destructive">{i18n.t('settings.account_section.danger_zone')}</h2>
            <p class="text-sm text-muted-foreground mt-1">{i18n.t('settings.account_section.danger_zone_desc')}</p>
        </div>

        <div class="flex flex-col sm:flex-row sm:items-center justify-between gap-4 py-6 border-y border-border/40">
            <div class="space-y-1 pr-4">
                <Label class="text-base font-bold text-foreground">{i18n.t('settings.account_section.delete_account')}</Label>
                <p class="text-sm text-muted-foreground">{i18n.t('settings.account_section.delete_account_desc')}</p>
            </div>
            <div class="flex flex-col sm:flex-row gap-3 w-full sm:w-auto shrink-0">
                <Button type="button" variant="destructive" class="rounded-sm h-11 font-bold w-full sm:w-auto shadow-sm" onclick={() => showDeleteAlert = true}>
                    <Trash2 class="mr-2 h-4 w-4" /> {i18n.t('settings.account_section.delete_account')}
                </Button>
            </div>
        </div>
    </section>
</div>

<AlertDialog.Root bind:open={showDeleteAlert}>
    <AlertDialog.Content class="border-destructive/20 sm:rounded-sm">
        <AlertDialog.Header>
            <AlertDialog.Title class="text-destructive flex items-center gap-2 text-xl">
                <AlertTriangle class="h-6 w-6" /> {i18n.t('settings.account_section.are_you_sure')}
            </AlertDialog.Title>
            <AlertDialog.Description class="text-base space-y-4">
                <p>{i18n.t('settings.account_section.delete_warning')}</p>

                {#if user.hasPassword}
                    <div class="space-y-2 pt-2">
                        <Label class="text-foreground">{i18n.t('settings.account_section.verify_password')}</Label>
                        <Input
                                type="password"
                                bind:value={deletePassword}
                                placeholder="••••••••"
                                class="border-destructive/30 focus-visible:ring-destructive/50"
                        />
                    </div>
                {/if}
            </AlertDialog.Description>
        </AlertDialog.Header>
        <AlertDialog.Footer class="flex-col sm:flex-row gap-3 sm:gap-2 mt-6">
            <AlertDialog.Cancel class="rounded-sm font-bold">{i18n.t('content.cancel')}</AlertDialog.Cancel>
            <AlertDialog.Action
                    class="bg-destructive text-destructive-foreground hover:bg-destructive/90 rounded-sm font-bold"
                    onclick={handleDeleteAccount}
                    disabled={user.hasPassword && !deletePassword}
            >
                {#if deletingAccount}<Spinner class="mr-2 h-4 w-4" />{/if}
                {i18n.t('settings.account_section.confirm_delete')}
            </AlertDialog.Action>
        </AlertDialog.Footer>
    </AlertDialog.Content>
</AlertDialog.Root>