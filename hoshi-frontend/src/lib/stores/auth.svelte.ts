import type {RegisterRequest, UserInfo} from "@/api/auth/types";
import {authApi} from "@/api/auth/auth";
import {usersApi} from "@/api/users/users";
import {appConfig} from "@/stores/config.svelte.js";
import type {CoreError} from "@/api/client";
import {call} from "@/api/client";

class AuthStore {
    user = $state<UserInfo | null>(null);
    loading = $state(false);
    initialized = $state(false);

    isAuthenticated = $derived(this.user !== null);

    async login(userId: number, password?: string) {
        this.loading = true;

        try {
            const res = await authApi.login({ userId, password });
            this.user = res.user;
            await appConfig.load();
        } catch (err) {
            throw err as CoreError;
        } finally {
            this.loading = false;
        }
    }

    async register(data: RegisterRequest, avatarFile?: File | null) {
        this.loading = true;

        try {
            const res = await authApi.register(data);
            this.user = res.user;

            if (avatarFile) {
                await usersApi.uploadAvatar(avatarFile);
                this.user = await usersApi.getMe();
            }

            await appConfig.load();
            return res;
        } catch (err) {
            throw err as CoreError;
        } finally {
            this.loading = false;
        }
    }

    async logout() {
        try {
            await authApi.logout();
        } catch (err) {
            console.error("Backend logout failed:", err);
        } finally {
            this.user = null;
            appConfig.clear();
        }
    }

    async restore(force = false) {
        if (this.initialized && !force) return;

        this.loading = true;

        try {
            const res = await call<{ user: UserInfo }>({
                tauri: { cmd: "get_current_profile" }
            });

            this.user = res.user;
            await appConfig.load();

        } catch (err) {
            console.log("No active profile:", err);
            this.user = null;
            appConfig.clear();
        } finally {
            this.loading = false;
            this.initialized = true;
        }
    }
}

export const auth = new AuthStore();