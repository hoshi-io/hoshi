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
    import SettingsSection from "@/components/settings/parts/SettingsSection.svelte";
    import SettingRow from "@/components/settings/parts/SettingRow.svelte";

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
    let previewAvatarUrl = $state<string | null>("");

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

    // Only flag a mismatch once the person has started typing the confirmation.
    let passwordMismatch = $derived(confirmPassword.length > 0 && confirmPassword !== newPassword);

    let showDeleteAlert = $state(false);
    let deletingAccount = $state(false);
    let deletePassword = $state("");

    const fieldClass = "rounded-md h-11 w-full sm:w-72";

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

<div class="space-y-14 w-full">

    <SettingsSection
            title={i18n.t('settings.account_section.profile')}
            description={i18n.t('settings.account_section.profile_desc')}
    >
        <form onsubmit={handleUpdateProfile}>
            <div class="flex items-start gap-5 sm:gap-8 py-6 border-b border-border/40">
                <div class="shrink-0 flex flex-col items-center gap-2.5 sm:w-32">
                    <button
                            type="button"
                            class="relative group rounded-full focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-primary transition-transform active:scale-95"
                            onclick={() => fileInput?.click()}
                            aria-label={i18n.t('settings.account_section.change_avatar')}
                    >
                        <Avatar.Root class="size-20 sm:size-24 border border-border/50 shadow-sm transition-opacity group-hover:opacity-90">
                            {#if previewAvatarUrl}
                                <Avatar.Image src={previewAvatarUrl} alt={username} class="object-cover" />
                            {/if}
                            <Avatar.Fallback class="bg-primary/10 text-primary text-3xl font-bold uppercase">
                                {username.charAt(0)}
                            </Avatar.Fallback>
                        </Avatar.Root>

                        <div class="absolute inset-0 flex items-center justify-center rounded-full bg-black/40 opacity-0 group-hover:opacity-100 group-focus-visible:opacity-100 transition-opacity text-white backdrop-blur-[2px]">
                            <Camera class="size-6" />
                        </div>
                    </button>
                    <input bind:this={fileInput} type="file" accept="image/png, image/jpeg, image/webp" class="hidden" onchange={handleFileSelect} />

                    <div class="flex items-center gap-2.5 text-xs sm:text-sm">
                        <button type="button" class="font-semibold text-muted-foreground hover:text-foreground transition-colors" onclick={() => fileInput?.click()}>
                            {i18n.t('settings.account_section.change_avatar').split(' ')[0]}
                        </button>
                        {#if previewAvatarUrl}
                            <span class="text-border">•</span>
                            <button type="button" class="font-semibold text-destructive hover:underline" onclick={handleRemoveAvatar}>
                                {i18n.t('settings.account_section.remove')}
                            </button>
                        {/if}
                    </div>
                </div>

                <div class="flex-1 min-w-0 space-y-2 pt-1 sm:pt-3">
                    <Label for="username" class="text-[15px] font-semibold">{i18n.t('settings.account_section.username')}</Label>
                    <Input id="username" bind:value={username} class="rounded-md h-11 w-full sm:max-w-md" required />
                    <p class="text-sm text-muted-foreground leading-snug">{i18n.t('settings.account_section.username_desc')}</p>
                </div>
            </div>

            <div class="flex justify-end pt-6">
                <Button type="submit" disabled={!hasChanges || savingProfile} class="rounded-md px-8 h-11 font-semibold transition-all">
                    {#if savingProfile}<Spinner class="mr-2 h-4 w-4" />{/if}
                    {hasChanges ? i18n.t('settings.account_section.save') : i18n.t('settings.account_section.saved')}
                </Button>
            </div>
        </form>
    </SettingsSection>

    <SettingsSection
            title={i18n.t('settings.account_section.security')}
            description={i18n.t('settings.account_section.security_desc')}
    >
        <form onsubmit={handleChangePassword}>
            {#if user.hasPassword}
                <SettingRow
                        for="currentPassword"
                        label={i18n.t('settings.account_section.current_password')}
                        description={i18n.t('settings.account_section.current_password_desc')}
                >
                    <Input id="currentPassword" type="password" autocomplete="current-password" bind:value={currentPassword} class={fieldClass} required />
                </SettingRow>
            {/if}

            <SettingRow
                    for="newPassword"
                    label={i18n.t('settings.account_section.new_password')}
                    description={i18n.t('settings.account_section.new_password_desc')}
            >
                <Input id="newPassword" type="password" autocomplete="new-password" bind:value={newPassword} class={fieldClass} required />
            </SettingRow>

            <SettingRow
                    for="confirmPassword"
                    label={i18n.t('settings.account_section.confirm_password')}
                    description={i18n.t('settings.account_section.confirm_new_password')}
            >
                <Input
                        id="confirmPassword"
                        type="password"
                        autocomplete="new-password"
                        aria-invalid={passwordMismatch}
                        bind:value={confirmPassword}
                        class="{fieldClass} {passwordMismatch ? 'border-destructive focus-visible:ring-destructive/40' : ''}"
                        required
                />
            </SettingRow>

            <div class="flex justify-end pt-6">
                <Button type="submit" disabled={!canSavePassword || savingPassword} variant="secondary" class="rounded-md px-8 h-11 font-semibold transition-all">
                    {#if savingPassword}<Spinner class="mr-2 h-4 w-4" />{/if}
                    {i18n.t('settings.account_section.update_password')}
                </Button>
            </div>
        </form>
    </SettingsSection>

    <SettingsSection
            variant="danger"
            title={i18n.t('settings.account_section.danger_zone')}
            description={i18n.t('settings.account_section.danger_zone_desc')}
    >
        <div class="flex flex-col sm:flex-row sm:items-center justify-between gap-4 rounded-lg border border-destructive/30 bg-destructive/5 p-5">
            <div class="space-y-0.5 min-w-0">
                <p class="text-[15px] font-semibold leading-snug">{i18n.t('settings.account_section.delete_account')}</p>
                <p class="text-sm text-muted-foreground leading-snug">{i18n.t('settings.account_section.delete_account_desc')}</p>
            </div>
            <Button type="button" variant="destructive" class="rounded-md h-11 font-semibold w-full sm:w-auto shrink-0" onclick={() => showDeleteAlert = true}>
                <Trash2 class="mr-2 h-4 w-4" /> {i18n.t('settings.account_section.delete_account')}
            </Button>
        </div>
    </SettingsSection>
</div>

<AlertDialog.Root bind:open={showDeleteAlert}>
    <AlertDialog.Content class="border-destructive/20 sm:rounded-lg">
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
                                autocomplete="current-password"
                                bind:value={deletePassword}
                                placeholder="••••••••"
                                class="border-destructive/30 focus-visible:ring-destructive/50"
                        />
                    </div>
                {/if}
            </AlertDialog.Description>
        </AlertDialog.Header>
        <AlertDialog.Footer class="flex-col sm:flex-row gap-3 sm:gap-2 mt-6">
            <AlertDialog.Cancel class="rounded-md font-semibold">{i18n.t('content.cancel')}</AlertDialog.Cancel>
            <AlertDialog.Action
                    class="bg-destructive text-destructive-foreground hover:bg-destructive/90 rounded-md font-semibold"
                    onclick={handleDeleteAccount}
                    disabled={user.hasPassword && !deletePassword}
            >
                {#if deletingAccount}<Spinner class="mr-2 h-4 w-4" />{/if}
                {i18n.t('settings.account_section.confirm_delete')}
            </AlertDialog.Action>
        </AlertDialog.Footer>
    </AlertDialog.Content>
</AlertDialog.Root>