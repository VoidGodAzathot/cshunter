import { invoke } from "@tauri-apps/api/core";
import { useEffect, useState } from "react";
import useStorage from "../../hooks/storage";
import { Card, ProgressRoot, Text } from "@chakra-ui/react";
import { ProgressBar, ProgressLabel } from "../../components/ui/progress";
import { Task } from "../../utils/task";
import { AnalyzeContext, Browser, CacheDat, DownloadDat, FileRecord, SteamAccount, VisitDat, Volume } from "../../utils/types";
import PreloadBoxes from "../../components/preload/preload-boxes";
import { getVersion } from '@tauri-apps/api/app';

const tasks: Task[] = [
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
                const volume_files: string[] = await invoke("get_parallel_files", { startPath: volume.path })
                files = [...files, ...volume_files];
            }
            await set<string[]>("all_files", files);
        }
    },
    {
        name: "Получение журнала файлов",
        id: "get_usn_journal_records",
        worker: async () => {
            const [set, get,] = useStorage();
            const volumes = await get<Volume[]>("volumes");
            let response: FileRecord[] = [];
            for (var i = 0; i < volumes.length; i++) {
                const volume = volumes[i];
                let records: FileRecord[] = await invoke("get_usn_journal_records", { volume: volume, reason: -1 });
                response = [...response, ...records];
            }
            set<FileRecord[]>("journal_all", response);
        }
    },
    {
        name: "Получение журнала удаленных файлов",
        id: "get_usn_journal_records",
        worker: async () => {
            const [set, get,] = useStorage();
            const volumes = await get<Volume[]>("volumes");
            let response: FileRecord[] = [];
            for (var i = 0; i < volumes.length; i++) {
                const volume = volumes[i];
                let records: FileRecord[] = await invoke("get_usn_journal_records", { volume: volume, reason: 512 });
                response = [...response, ...records];
            }
            await set<FileRecord[]>("journal_removes", response);
        }
    },
    {
        name: "Получение истории steam аккаунтов",
        id: "get_steam_accounts",
        worker: async () => {
            const [set, ,] = useStorage();
            const accounts: SteamAccount[] = await invoke("get_steam_accounts_history");
            await set<SteamAccount[]>("steam_accounts", accounts);
        }
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
                visit_dat = [...visit_dat, ...(await invoke("get_browser_visit_data", { browserId: browser.id }) as VisitDat[])]
                cache_dat = [...cache_dat, ...(await invoke("get_browser_cache_data", { browserId: browser.id }) as CacheDat[])]
                download_dat = [...download_dat, ...(await invoke("get_browser_download_data", { browserId: browser.id }) as DownloadDat[])]
            }
            await set<DownloadDat[]>("browsers_download_dat", download_dat);
            await set<CacheDat[]>("browsers_visit_dat", visit_dat);
            await set<CacheDat[]>("browsers_cache_dat", cache_dat);
        }
    },
    {
        name: "Снапшот файлов системы",
        id: "snapshot_files_system",
        worker: async () => {
            const [set, get,] = useStorage();
            const all_files = await get<string[]>("all_files");
            const context: AnalyzeContext = await invoke("generate_context", { files: all_files });
            await set<AnalyzeContext>("analyzer_context", context);
        }
    },
    {
        name: "Идентификация устройства",
        id: "get_device_id",
        worker: async () => {
            const [set, ,] = useStorage();
            const device_id: string = await invoke("get_device_id");
            await set<string>("device_id", device_id);
        }
    }
];

function PreloadMainPage() {
    const [isLoaded, setIsLoaded] = useState<boolean>(false);
    const [completedTasks, setCompletedTasks] = useState<number>(0);
    const [currentTask, setCurrentTask] = useState<Task | undefined>(undefined);
    const [version, setVersion] = useState<string | undefined>(undefined);

    useEffect(() => {
        async function run_preload() {
            const version = await getVersion();
            setVersion(version);
            for (var i = 1; i < tasks.length + 1; i++) {
                const task = tasks[i - 1];
                setCurrentTask(task)
                await task.worker();
                setCompletedTasks(i);
            }
            await new Promise((resolve) => setTimeout(resolve, 2000));
            setIsLoaded(true);
        }

        run_preload();
    }, []);

    useEffect(() => {
        async function run_cshunter() {
            await invoke("run_main_window_and_close_preload");
        }

        if (isLoaded) {
            run_cshunter();
        }
    }, [isLoaded]);

    return (
        <div data-tauri-drag-region={true} className="font-inter w-[100vw] h-[100vh] select-none flex justify-center items-center">
            <Card.Root variant="subtle" width="320px">
                <Card.Body>
                    <Card.Title spaceX={5} paddingBottom={5} className="items-center flex">
                        <PreloadBoxes />

                        <div>
                            <Text fontWeight="normal" fontSize={18}>
                                cshunter
                            </Text>

                            <Text color="gray" fontWeight="normal" fontSize={14}>
                                { version ? version : "..." }
                            </Text>
                        </div>
                    </Card.Title>
                </Card.Body>
                <Card.Footer>
                    <ProgressRoot size="lg" striped animated spaceY={1} className="flex-rows" value={completedTasks} max={tasks.length} variant="subtle" w="full" maxW="full">
                        <ProgressLabel color={"gray"} fontSize={12}>
                            { currentTask ? currentTask.name : "Ожидание" }
                        </ProgressLabel>
                        <ProgressBar borderRadius={10} />
                    </ProgressRoot>
                </Card.Footer>
            </Card.Root>
        </div>
    )
}

export default PreloadMainPage;