import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { StorageUpdate } from "../utils/types";

export default function useStorage(): [<T>(name: string, value: T) => Promise<void>, <T>(name: string) => Promise<T>, () => Promise<Map<string, any>>, (callback?: (name: string) => Promise<void>) => Promise<void>] {
    async function set<T>(name: string, value: T) {
        await invoke("set_storage", { name, value: JSON.stringify(value) });
    }

    async function get<T>(name: string): Promise<T> {
        const result = await invoke("get_storage", { name });
        return result ? JSON.parse(result.toString()) : null;
    }

    async function getAll(): Promise<Map<string, any>> {
        const result: Map<string, any> = await invoke("get_all_storage");
        const map = new Map<string, any>();
        Object.entries(result).forEach(([key, value]) => {
            map.set(key, JSON.parse(value.toString()));
        });
        return map;
    }

    async function setupListen(callback?: (name: string) => Promise<void>) {
        await listen("storage_update", async (e) => {
            const payload: StorageUpdate = e.payload as StorageUpdate;
            if (callback) await callback(payload.name);
        })
    }

    return [set, get, getAll, setupListen];
}