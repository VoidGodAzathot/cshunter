import { invoke } from "@tauri-apps/api/core";

export default function useStorage(): [<T>(name: string, value: T) => Promise<void>, <T>(name: string) => Promise<T>, () => Promise<Map<string, any>>] {
    async function set<T>(name: string, value: T) {
        await invoke('set_storage', { name, value: JSON.stringify(value) });
    }

    async function get<T>(name: string): Promise<T> {
        const result = await invoke('get_storage', { name });
        return result ? JSON.parse(result.toString()) : null;
    }

    async function getAll(): Promise<Map<string, any>> {
        const result: Map<string, any> = await invoke('get_all_storage');
        const map = new Map<string, any>();
        Object.entries(result).forEach(([key, value]) => {
            map.set(key, JSON.parse(value.toString()));
        });
        return map;
    }

    return [set, get, getAll];
}