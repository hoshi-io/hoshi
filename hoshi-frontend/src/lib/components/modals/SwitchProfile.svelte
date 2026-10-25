<script lang="ts">
    import { tick } from 'svelte';
    import { goto } from '$app/navigation';
    import { usersApi } from '$lib/api/users/users';
    import { auth } from '@/stores/auth.svelte.js';
    import { i18n } from '@/stores/i18n.svelte.js';
    import { layoutState } from '@/stores/layout.svelte.js';
    import type { UserResponse } from '$lib/api/users/types';
    import * as Dialog from '$lib/components/ui/dialog';
    import { Button } from '$lib/components/ui/button';
    import { Input } from '$lib/components/ui/input';
    import * as Avatar from '$lib/components/ui/avatar';
    import { Skeleton } from '$lib/components/ui/skeleton';
    import { Plus, ArrowLeft, Lock, Check } from 'lucide-svelte';
    import { Spinner } from "$lib/components/ui/spinner";
    import { fly } from 'svelte/transition';
    import { cubicOut } from 'svelte/easing';
    import type { CoreError } from "@/api/client";
    import TvDialog from '@/tv/TvDialog.svelte';

    let { open = $bindable(false) } = $props();

    let users = $state<UserResponse[]>([]);
    let loading = $state(false);

    let error = $state<CoreError | null>(null);

    let selectedUser = $state<UserResponse | null>(null);
    let passwordInput = $state("");
    let loginLoading = $state(false);
    // Which profile is being signed in right now (shows a spinner on that row only).
    let pendingId = $state<number | null>(null);

    const currentId = $derived(auth.user?.id);

    const prefersReducedMotion =
        typeof matchMedia !== 'undefined' && matchMedia('(prefers-reduced-motion: reduce)').matches;
    const slideIn = (x: number) => ({ x: prefersReducedMotion ? 0 : x, duration: prefersReducedMotion ? 0 : 200, easing: cubicOut });

    $effect(() => {
        if (open) {
            loadUsers();
            selectedUser = null;
            passwordInput = "";
            error = null;
        }
    });

    async function loadUsers() {
        loading = true;
        try {
            const res = await usersApi.getAll();
            users = res.users;
        } catch (err) {
            console.error("Failed to load users", err);
            error = err as CoreError;
        } finally {
            loading = false;
        }
        if (layoutState.isTV) focusProfiles();
    }

    // TV: the list loads after the dialog mounted, so focus the current profile once it exists.
    async function focusProfiles() {
        await tick();
        const root = document.querySelector('[data-sn-trap]');
        const target =
            root?.querySelector<HTMLElement>('[data-profile-row][aria-current="true"]') ??
            root?.querySelector<HTMLElement>('[data-sn]');
        target?.focus();
    }

    async function focusPassword() {
        await tick();
        document.querySelector<HTMLElement>('[data-sn-trap] input')?.focus();
    }

    async function handleUserClick(user: UserResponse) {
        error = null;
        // Already signed in as this profile: nothing to switch, so don't log in again and reload the app.
        if (user.id === currentId) {
            open = false;
            return;
        }
        if (user.hasPassword) {
            selectedUser = user;
            passwordInput = "";
            if (layoutState.isTV) focusPassword();
        } else {
            await attemptLogin(user.id);
        }
    }

    async function attemptLogin(userId: number, password?: string) {
        loginLoading = true;
        pendingId = userId;
        error = null;
        try {
            await auth.login(userId, password);
            open = false;
            location.reload();
        } catch (err) {
            error = err as CoreError;
        } finally {
            loginLoading = false;
            pendingId = null;
        }
    }

    async function goToSetup() {
        open = false;
        await auth.logout();
        goto(layoutState.isTV ? '/tv/setup' : '/setup');
    }

    // TV: Back on the password step returns to the list; on the list it closes the dialog.
    function tvBack() {
        if (selectedUser) {
            selectedUser = null;
            error = null;
            focusProfiles();
        } else {
            open = false;
        }
    }

    const rowClass =
        "group flex items-center gap-4 p-3 rounded-sm w-full text-left outline-none cursor-default " +
        "transition-[background-color,transform] duration-150 hover:bg-muted/60 active:scale-[0.99] " +
        "focus-visible:ring-2 focus-visible:ring-primary focus-visible:ring-offset-2 focus-visible:ring-offset-background " +
        "disabled:pointer-events-none disabled:opacity-60";

    const tvRowClass =
        "flex w-full items-center gap-5 rounded-lg p-4 text-left outline-none " +
        "transition-transform duration-150 focus:scale-[1.03] focus:bg-muted/60 focus:ring-4 focus:ring-primary/70 " +
        "disabled:opacity-60";

    const tvButtonClass =
        "flex h-14 items-center justify-center gap-2 rounded-lg px-8 text-lg font-bold outline-none " +
        "transition-transform duration-150 focus:scale-105 focus:ring-4 focus:ring-primary/70 disabled:opacity-50";
</script>

{#if layoutState.isTV}
    {#if open}
        <TvDialog
                title={selectedUser ? selectedUser.username : i18n.t('layout.switch_profile')}
                description={selectedUser ? i18n.t('layout.enter_password') : i18n.t('layout.who_is_watching')}
                onclose={tvBack}
        >
            {#if !selectedUser}
                {#if loading}
                    <div class="flex h-40 items-center justify-center">
                        <Spinner class="size-10 animate-spin text-primary" />
                    </div>
                {:else}
                    <div class="flex flex-col gap-2">
                        {#each users as user (user.id)}
                            {@const isCurrent = user.id === currentId}
                            <button
                                    type="button"
                                    data-sn
                                    data-profile-row
                                    class={tvRowClass}
                                    disabled={loginLoading}
                                    aria-current={isCurrent ? 'true' : undefined}
                                    onclick={() => handleUserClick(user)}
                            >
                                <Avatar.Root class="size-14 shrink-0 border border-border {isCurrent ? 'ring-2 ring-primary ring-offset-2 ring-offset-card' : ''}">
                                    <Avatar.Image src={user.avatar} alt={user.username} class="object-cover" />
                                    <Avatar.Fallback class="bg-primary/10 text-lg font-bold text-primary">
                                        {user.username[0].toUpperCase()}
                                    </Avatar.Fallback>
                                </Avatar.Root>

                                <span class="flex-1 truncate text-xl font-bold">{user.username}</span>

                                {#if pendingId === user.id}
                                    <Spinner class="size-6 shrink-0 animate-spin text-primary" />
                                {:else if isCurrent}
                                    <Check class="size-6 shrink-0 text-primary" />
                                {:else if user.hasPassword}
                                    <Lock class="size-6 shrink-0 text-muted-foreground/60" />
                                {/if}
                            </button>
                        {/each}

                        <div class="my-1 h-px w-full bg-border/40"></div>

                        <button type="button" data-sn class={tvRowClass} disabled={loginLoading} onclick={goToSetup}>
                            <div class="flex size-14 shrink-0 items-center justify-center rounded-full border border-dashed border-muted-foreground/50">
                                <Plus class="size-6 text-muted-foreground" />
                            </div>
                            <span class="flex-1 text-xl font-bold text-muted-foreground">{i18n.t('layout.add_profile')}</span>
                        </button>

                        {#if error}
                            <p role="alert" class="px-2 pt-2 text-base font-semibold text-destructive">{i18n.t(error.key)}</p>
                        {/if}
                    </div>
                {/if}
            {:else}
                <!-- Enter in the field submits; Up/Down leave it (spatial nav) to reach the buttons. -->
                <form
                        class="space-y-5"
                        onsubmit={(e) => { e.preventDefault(); if (selectedUser) attemptLogin(selectedUser.id, passwordInput); }}
                >
                    <div class="space-y-2">
                        <input
                                type="password"
                                data-sn
                                autocomplete="current-password"
                                placeholder={i18n.t('layout.enter_password')}
                                bind:value={passwordInput}
                                aria-invalid={error ? true : undefined}
                                class="h-14 w-full rounded-lg border-2 bg-secondary/50 px-4 text-xl outline-none focus:border-primary
                                       {error ? 'border-destructive' : 'border-border'}"
                        />
                        {#if error}
                            <p role="alert" class="text-base font-semibold text-destructive">{i18n.t(error.key)}</p>
                        {/if}
                    </div>

                    <div class="flex items-center justify-between gap-3">
                        <button type="button" data-sn class="{tvButtonClass} bg-muted/30 text-foreground" onclick={tvBack}>
                            <ArrowLeft class="size-5" />
                        </button>
                        <button
                                type="submit"
                                data-sn
                                disabled={loginLoading || !passwordInput}
                                class="{tvButtonClass} bg-primary text-primary-foreground"
                        >
                            {#if loginLoading}
                                <Spinner class="size-5 animate-spin" />
                            {/if}
                            {i18n.t('layout.login')}
                        </button>
                    </div>
                </form>
            {/if}
        </TvDialog>
    {/if}
{:else}
    <Dialog.Root bind:open>
        <Dialog.Content class="sm:max-w-sm rounded-sm bg-card/95 backdrop-blur-xl border-border/50 p-6">
            {#if !selectedUser}
                <div in:fly={slideIn(-16)}>
                    <Dialog.Header class="mb-4">
                        <Dialog.Title class="text-xl font-black text-left">{i18n.t('layout.switch_profile')}</Dialog.Title>
                        <Dialog.Description class="text-left">{i18n.t('layout.who_is_watching')}</Dialog.Description>
                    </Dialog.Header>

                    <div class="flex flex-col gap-1">
                        {#if loading}
                            <!-- Same row shape as the real profiles, so the dialog doesn't jump when they arrive. -->
                            {#each Array(2) as _}
                                <div class="flex items-center gap-4 p-3">
                                    <Skeleton class="size-11 rounded-full shrink-0" />
                                    <Skeleton class="h-4 w-32 rounded-sm" />
                                </div>
                            {/each}
                        {:else}
                            {#each users as user (user.id)}
                                {@const isCurrent = user.id === currentId}
                                <button
                                        class={rowClass}
                                        disabled={loginLoading}
                                        aria-current={isCurrent ? 'true' : undefined}
                                        onclick={() => handleUserClick(user)}
                                >
                                    <Avatar.Root class="size-11 shrink-0 border border-border transition-shadow {isCurrent ? 'ring-2 ring-primary ring-offset-2 ring-offset-card' : 'group-hover:border-primary/50'}">
                                        <Avatar.Image src={user.avatar} alt={user.username} class="object-cover" />
                                        <Avatar.Fallback class="bg-primary/10 text-primary font-bold text-sm">
                                            {user.username[0].toUpperCase()}
                                        </Avatar.Fallback>
                                    </Avatar.Root>

                                    <span class="flex-1 truncate text-sm font-semibold text-foreground">
                                        {user.username}
                                    </span>

                                    {#if pendingId === user.id}
                                        <Spinner class="size-4 animate-spin text-primary shrink-0" />
                                    {:else if isCurrent}
                                        <Check class="size-4 text-primary shrink-0" />
                                    {:else if user.hasPassword}
                                        <Lock class="size-4 text-muted-foreground/60 shrink-0" />
                                    {/if}
                                </button>
                            {/each}

                            <div class="h-px w-full bg-border/40 my-1"></div>

                            <button class={rowClass} disabled={loginLoading} onclick={goToSetup}>
                                <div class="size-11 rounded-full border border-dashed border-muted-foreground/50 flex items-center justify-center group-hover:border-primary/50 group-hover:bg-primary/5 transition-colors shrink-0">
                                    <Plus class="size-5 text-muted-foreground group-hover:text-primary transition-colors" />
                                </div>
                                <span class="flex-1 text-sm font-semibold text-muted-foreground group-hover:text-foreground transition-colors">
                                    {i18n.t('layout.add_profile')}
                                </span>
                            </button>
                        {/if}

                        <!-- Errors used to show only on the password screen; a failed login for a profile without a password was silent. -->
                        {#if error}
                            <p role="alert" class="px-3 pt-2 text-sm font-semibold text-destructive">{i18n.t(error.key)}</p>
                        {/if}
                    </div>
                </div>
            {:else}
                <div in:fly={slideIn(16)}>
                    <Dialog.Header class="space-y-0">
                        <Button variant="ghost" size="icon" class="-ml-2 h-8 w-8 rounded-sm" onclick={() => { selectedUser = null; error = null; }}>
                            <ArrowLeft class="size-4" />
                        </Button>
                        <Dialog.Description class="sr-only">{i18n.t('layout.enter_password')}</Dialog.Description>
                    </Dialog.Header>

                    <div class="flex flex-col items-center gap-3 pt-1 pb-5">
                        <Avatar.Root class="size-16 border border-border">
                            <Avatar.Image src={selectedUser.avatar} alt={selectedUser.username} class="object-cover" />
                            <Avatar.Fallback class="bg-primary/10 text-primary font-bold text-lg">{selectedUser.username[0].toUpperCase()}</Avatar.Fallback>
                        </Avatar.Root>
                        <Dialog.Title class="text-lg font-black">{selectedUser.username}</Dialog.Title>
                    </div>

                    <form class="space-y-4" onsubmit={(e) => { e.preventDefault(); if (selectedUser) attemptLogin(selectedUser.id, passwordInput); }}>
                        <div class="space-y-2">
                            <Input
                                    type="password"
                                    autocomplete="current-password"
                                    placeholder={i18n.t('layout.enter_password')}
                                    bind:value={passwordInput}
                                    aria-invalid={error ? true : undefined}
                                    class="h-11 rounded-sm {error ? 'border-destructive focus-visible:ring-destructive/40' : ''}"
                                    autofocus
                            />
                            {#if error}
                                <p role="alert" class="text-sm font-semibold text-destructive">{i18n.t(error.key)}</p>
                            {/if}
                        </div>
                        <Button type="submit" class="w-full h-11 rounded-sm font-bold" disabled={loginLoading || !passwordInput}>
                            {#if loginLoading}
                                <Spinner class="size-4 animate-spin mr-2" />
                            {/if}
                            {i18n.t('layout.login')}
                        </Button>
                    </form>
                </div>
            {/if}
        </Dialog.Content>
    </Dialog.Root>
{/if}