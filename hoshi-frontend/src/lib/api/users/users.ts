import { call, isTauri } from "@/api/client";
import type {
    UserPrivate,
    UpdateUserBody,
    ChangePasswordBody,
    DeleteUserBody,
    UsersResponse,
} from "./types";

export const usersApi = {
    getAll() {
        return call<UsersResponse>({
            tauri: { cmd: "get_all_users" },
        });
    },

    getMe() {
        return call<UserPrivate>({
            tauri: { cmd: "get_me" },
        });
    },

    updateMe(body: UpdateUserBody) {
        return call<void>({
            tauri: { cmd: "update_me", args: { updates: body } },
        });
    },

    deleteMe(body: DeleteUserBody) {
        return call<void>({
            tauri: { cmd: "delete_me", args: { body: body } },
        });
    },

    changePassword(body: ChangePasswordBody) {
        return call<boolean>({
            tauri: { cmd: "change_password", args: {body: body} },
        });
    },

    async uploadAvatar(file: File) {
        if (isTauri()) {
            const { invoke } = await import("@tauri-apps/api/core");
            const data = Array.from(new Uint8Array(await file.arrayBuffer()));
            return invoke<void>("upload_avatar", { data, contentType: file.type });
        }

        return call<void>({
            tauri: { cmd: "upload_avatar" }, // never reached
        });
    },

    deleteAvatar() {
        return call<void>({
            tauri: { cmd: "delete_avatar" },
        });
    },
};