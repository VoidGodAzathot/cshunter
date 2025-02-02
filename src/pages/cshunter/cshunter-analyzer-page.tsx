import { useEffect, useState } from "react";
import useStorage from "../../hooks/storage";
import { Badge, Box, Button, Center, Flex, HStack, Link, Spinner, Stat, Text } from "@chakra-ui/react";
import { AnalyzeContext } from "../../utils/types";
import { Icon } from "@iconify/react/dist/iconify.js";
import { open } from '@tauri-apps/plugin-dialog';
import { invoke } from "@tauri-apps/api/core";
import { Toaster, toaster } from "../../components/ui/toaster"
import { readText } from '@tauri-apps/plugin-clipboard-manager';
import { PaginationItems, PaginationNextTrigger, PaginationPrevTrigger, PaginationRoot } from "../../components/ui/pagination";

type Match = {
    name: string,
    path: string
}

export default function CSHunterAnalyzerPage() {
    const [, get,] = useStorage();
    const [isLoading, setIsLoading] = useState<boolean>(true);
    const [isMatching, setIsMatching] = useState<boolean>(false);
    const [sysContext, setSysContext] = useState<AnalyzeContext | undefined>(undefined);
    const [currentContext, setCurrentContext] = useState<AnalyzeContext | undefined>(undefined);
    const [currentMatches, setCurrentMatches] = useState<Match[]>([]);
    const pageSize = 100;
    const [currentPage, setCurrentPage] = useState(1);
    const totalPages = Math.ceil(currentMatches.length / pageSize);

    const paginatedData = currentMatches.slice(
        (currentPage - 1) * pageSize,
        currentPage * pageSize
    );

    useEffect(() => {
        setCurrentPage(1);
    }, [totalPages]);

    useEffect(() => {
        async function setup() {
            const context: AnalyzeContext = await get<AnalyzeContext>("analyzer_context");
            setSysContext(context);
            setIsLoading(false);
        };

        setup();
    }, []);

    useEffect(() => {
        async function applyFilter() {
            setIsMatching(true);

            if (currentContext && sysContext) {
                let matches: Match[] = [];

                for (let i = 0; i < currentContext.items.length; i++) {
                    const item = currentContext.items[i];
                    const fromSys = sysContext.items.filter((item_2) => item.crc32 == item_2.crc32);
                    const mapped = fromSys.map((item) => { return { name: item.name, path: item.path } });
                    matches = [...matches, ...mapped];
                }

                setCurrentMatches(matches);
            }

            setIsMatching(false);
        };

        applyFilter();
    }, [currentContext, sysContext]);

    return (
        <>
            <Toaster />

            {
                isLoading ?
                    <Center height="full">
                        <Spinner size="xl" />
                    </Center>
                    : (
                        <Flex height="full" justify="space-between" direction="column" gap={5}>
                            <Flex gap={5} width="full">
                                <Flex justify="space-between" direction="column" width="full" borderWidth="1px" padding={5} borderRadius="20px">
                                    <Flex direction="column">
                                        <Text>Текущий контекст</Text>
                                        <Text fontSize="14px" color="gray">Выберите .json файл</Text>
                                    </Flex>

                                    <Flex alignItems="center" justify="center" width="full" height="full">
                                        {
                                            currentContext ? <Stat.Root justifyContent="center" borderWidth="1px" paddingY={2} paddingX={5} marginY={2} borderRadius={20} gap={0}>
                                                <Stat.Label fontSize={12}>
                                                    Всего файлов
                                                </Stat.Label>
                                                <Stat.ValueText fontSize={16}>
                                                    {currentContext.items.length}
                                                </Stat.ValueText>
                                            </Stat.Root> : <Badge width="fit" borderRadius="20px" height="fit" colorPalette="red">не загружено</Badge>
                                        }
                                    </Flex>

                                    <Button onClick={async () => {
                                        const clipboardBuf = await readText();
                                        if (clipboardBuf.startsWith("http")) {
                                            const context: AnalyzeContext | undefined = await invoke("create_analyzer_context_from_url", { url: clipboardBuf })
                                            if (context) {
                                                setCurrentContext(context);
                                                toaster.create({
                                                    title: "Успешно загружено",
                                                    description: "Загрузка выполнена с ссылки из буффера обмена.",
                                                    type: "success"
                                                });
                                                return;
                                            }
                                        }
                                        const file = await open({ multiple: false, filters: [{ name: "", extensions: ["json"] }], directory: false })
                                        if (file) {
                                            const context: AnalyzeContext | undefined = await invoke("create_analyzer_context", { path: file })
                                            if (context) {
                                                setCurrentContext(context);
                                                toaster.create({
                                                    title: "Успешно загружено",
                                                    type: "success"
                                                });
                                                return;
                                            }
                                            toaster.create({
                                                title: "Ошибка загрузки",
                                                type: "error"
                                            });
                                        }
                                    }} variant="subtle" marginTop="auto" borderRadius={50} height="50px">
                                        <Icon icon="ic:outline-plus"></Icon>
                                        Загрузить
                                    </Button>
                                </Flex>

                                <Flex justify="start" gap={1} direction="column" width="full" borderWidth="1px" padding={5} borderRadius="20px">
                                    <Text>Системный снапшот файлов</Text>

                                    {
                                        sysContext ?
                                            <>
                                                <Stat.Root justifyContent="center" borderWidth="1px" paddingY={2} paddingX={5} marginY={2} borderRadius={20} gap={0}>
                                                    <Stat.Label fontSize={12}>
                                                        Всего файлов
                                                    </Stat.Label>
                                                    <Stat.ValueText fontSize={16}>
                                                        {sysContext.items.length}
                                                    </Stat.ValueText>
                                                </Stat.Root>
                                            </>
                                            : <></>
                                    }

                                    <Flex gap={2}>
                                        {sysContext ? sysContext.items.length > 0 ? <Badge borderRadius="20px" width="fit" colorPalette="green">активен</Badge> : <Badge width="fit" borderRadius="20px" colorPalette="red">не активен</Badge> : <Badge borderRadius="20px" width="fit" colorPalette="red">ошибка получения</Badge>}
                                        {sysContext ? <Badge borderRadius="20px" colorPalette="blue">только .dll и .exe</Badge> : <></>}
                                    </Flex>
                                </Flex>

                                <Flex height="full" direction="column" gap={1} align="top" justify="space-between" borderWidth="1px" padding={5} borderRadius="20px">
                                    <Flex direction="column" gap={1}>
                                        <Text>Генерация контекста</Text>
                                        <Text fontSize="14px" color="gray">Выберите папку</Text>
                                    </Flex>

                                    <Flex height="full" marginTop="auto" alignItems="end">
                                        <Button onClick={async () => {
                                            const dir = await open({ multiple: false, canCreateDirectories: true, directory: true })
                                            if (dir) {
                                                const files: string[] = await invoke("get_parallel_files", { startPath: dir })
                                                const context: AnalyzeContext | undefined = await invoke("generate_context", { files: files });
                                                if (context) {
                                                    await invoke("save_context", { dir: dir, context: context });
                                                    toaster.create({
                                                        title: "Успешно создано",
                                                        description: "Файл сохранен в той же папке с названием \"context.json\".",
                                                        type: "success"
                                                    });
                                                    return;
                                                }

                                                toaster.create({
                                                    title: "Ошибка создания",
                                                    type: "error"
                                                });
                                            }
                                        }} variant="subtle" borderRadius={50} height="50px">
                                            <Icon icon="ic:outline-plus"></Icon>
                                            Выбрать
                                        </Button>
                                    </Flex>
                                </Flex>

                            </Flex>

                            <Box padding={5} borderRadius={20} borderWidth="1px" width="full" height="full" overflow="hidden" scrollbar="hidden">
                                <Box spaceY={3} paddingRight={5} direction="column" scrollbar="visible" width="full" height="full" scrollBehavior="smooth" _scrollbarThumb={{ padding: "5", borderRadius: "20px", width: "1px", background: "gray" }} overflowY="auto">
                                    {
                                        isMatching || currentMatches.length == 0 ? <Center gap={5} className="flex flex-col" height="full">
                                            <Spinner size="xl" />
                                            {
                                                isMatching ? <Text color="gray">Применение контекста</Text> : <Text color="gray">Ожидание</Text>
                                            }
                                        </Center> :
                                            <>
                                                {
                                                    paginatedData.map((match, i) =>
                                                        <Box key={i} paddingX={5} paddingY={5} borderRadius={20} borderWidth="1px">
                                                            <Text minWidth="min-content"
                                                                whiteSpace="normal"
                                                                wordBreak="break-word">{match.name}</Text>
                                                            <Link onClick={async () => {
                                                                await invoke("open_explorer", { path: match.path });
                                                            }} minWidth="min-content"
                                                                whiteSpace="normal"
                                                                wordBreak="break-word" color="gray" fontSize="12px">{match.path}</Link>
                                                        </Box>
                                                    )
                                                }
                                            </>
                                    }
                                </Box>
                            </Box>

                            {
                                paginatedData.length > 0 ? (<PaginationRoot
                                    count={currentMatches.length}
                                    pageSize={pageSize}
                                    page={currentPage}
                                    onPageChange={(s) => setCurrentPage(s.page)}
                                >
                                    <HStack wrap="wrap">
                                        <PaginationPrevTrigger
                                            disabled={currentPage === 1}
                                            onClick={() => setCurrentPage((p) => Math.max(1, p - 1))}
                                        />
                                        <PaginationItems />
                                        <PaginationNextTrigger
                                            disabled={currentPage === totalPages}
                                            onClick={() => setCurrentPage((p) => Math.min(totalPages, p + 1))}
                                        />
                                    </HStack>
                                </PaginationRoot>) : <></>
                            }
                        </Flex>
                    )
            }
        </>
    )
}