import useStorage from "../hooks/storage";
import { GITHUB_PACKAGE_URL } from "./consts";
import {
  AnalyzeContext,
  Browser,
  CacheDat,
  DownloadDat,
  FileRecord,
  MiniDat,
  ShellBagDat,
  SteamAccount,
  VisitDat,
  Volume,
} from "./types";
import { invoke } from "@tauri-apps/api/core";

export type Task = {
  name: string;
  id: string;
  worker: () => Promise<void>;
};

export const Tasks: Task[] = [
  {
    name: "Подготовка к запуску",
    id: "prepare",
    worker: async () => {
      const [set, ,] = useStorage();
      const volumes: Volume[] = await invoke("get_all_volumes");
      await set<Volume[]>("volumes", volumes);
      let files: string[] = [];
      for (var i = 0; i < volumes.length; i++) {
        const volume = volumes[i];
        const volume_files: string[] = await invoke("get_parallel_files", {
          startPath: volume.path,
        });
        files = [...files, ...volume_files];
      }
      await set<string[]>("all_files", files);
      const vmd_verdict: boolean = await invoke("is_vm");
      await set<boolean>("vmd_verdict", vmd_verdict);
      const github_version: string = await invoke("get_github_version", { url: GITHUB_PACKAGE_URL });
      await set<string>("github_version", github_version);
    },
  },
  {
    name: "Получение журнала файлов",
    id: "get_usn_journal_records",
    worker: async () => {
      const [set, get] = useStorage();
      const volumes = await get<Volume[]>("volumes");
      let response: FileRecord[] = [];
      for (var i = 0; i < volumes.length; i++) {
        const volume = volumes[i];
        let records: FileRecord[] = await invoke("get_usn_journal_records", {
          volume: volume,
          reason: -1,
        });
        response = [...response, ...records];
      }
      set<FileRecord[]>("journal_all", response);
    },
  },
  {
    name: "Получение журнала удаленных файлов",
    id: "get_usn_journal_deleted_records",
    worker: async () => {
      const [set, get] = useStorage();
      const volumes = await get<Volume[]>("volumes");
      let response: FileRecord[] = [];
      for (var i = 0; i < volumes.length; i++) {
        const volume = volumes[i];
        let records: FileRecord[] = await invoke("get_usn_journal_records", {
          volume: volume,
          reason: 512,
        });
        response = [...response, ...records];
      }
      await set<FileRecord[]>("journal_removes", response);
    },
  },
  {
    name: "Получение истории steam аккаунтов",
    id: "get_steam_accounts",
    worker: async () => {
      const [set, ,] = useStorage();
      const accounts: SteamAccount[] = await invoke(
        "get_steam_accounts_history"
      );
      const avatarCache: string[] = await invoke("get_steam_avatar_cache");
      await set<SteamAccount[]>("steam_accounts", accounts);
      await set<string[]>("steam_avatar_cache", avatarCache);
    },
  },
  {
    name: "Получение данных браузеров",
    id: "get_browsers_dat",
    worker: async () => {
      const [set, ,] = useStorage();
      const browsers: Browser[] = await invoke("get_supported_browsers");
      let visit_dat: VisitDat[] = [];
      let cache_dat: CacheDat[] = [];
      let download_dat: DownloadDat[] = [];
      for (var i = 0; i < browsers.length; i++) {
        const browser: Browser = browsers[i];
        visit_dat = [
          ...visit_dat,
          ...((await invoke("get_browser_visit_data", {
            browserId: browser.id,
          })) as VisitDat[]),
        ];
        cache_dat = [
          ...cache_dat,
          ...((await invoke("get_browser_cache_data", {
            browserId: browser.id,
          })) as CacheDat[]),
        ];
        download_dat = [
          ...download_dat,
          ...((await invoke("get_browser_download_data", {
            browserId: browser.id,
          })) as DownloadDat[]),
        ];
      }
      await set<DownloadDat[]>("browsers_download_dat", download_dat);
      await set<CacheDat[]>("browsers_visit_dat", visit_dat);
      await set<CacheDat[]>("browsers_cache_dat", cache_dat);
    },
  },
  {
    name: "Снапшот файлов системы",
    id: "snapshot_files_system",
    worker: async () => {
      const [set, get] = useStorage();
      const all_files = await get<string[]>("all_files");
      const context: AnalyzeContext = await invoke("generate_context", {
        files: all_files,
      });
      await set<AnalyzeContext>("analyzer_context", context);
    },
  },
  {
    name: "Получение данных о использовании",
    id: "collect_mini_dat",
    worker: async () => {
      const [set, ,] = useStorage();
      const mini_dat: MiniDat[] = await invoke("collect_mini_dat");
      await set<MiniDat[]>("mini_dat", mini_dat);
      const shellbag: ShellBagDat[] = await invoke("read_shellbag");
      await set<ShellBagDat[]>("shellbag", shellbag);
    },
  },
  {
    name: "Идентификация устройства",
    id: "get_device_id",
    worker: async () => {
      const [set, ,] = useStorage();
      const device_id: string = await invoke("get_device_id");
      await set<string>("device_id", device_id);
    },
  },
];
