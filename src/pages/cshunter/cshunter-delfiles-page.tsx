import { useEffect, useState } from "react"
import { FileRecord } from "../../utils/types"
import useStorage from "../../hooks/storage";
import { Box, Center, Flex, Highlight, HStack, Input, Spinner, Text } from "@chakra-ui/react";
import { PaginationItems, PaginationNextTrigger, PaginationPrevTrigger, PaginationRoot } from "../../components/ui/pagination";
import { asyncFilter, dateFromUsn, filterIsPresent } from "../../utils/utils";

export default function CSHunterDelFilesPage() {
    const [, get,] = useStorage();
    const pageSize = 100;
    const [isLoading, setIsLoading] = useState<boolean>(true);
    const [currentFilter, setCurrentFilter] = useState<string>("");
    const [files, setFiles] = useState<FileRecord[]>([]);
    const [currentFiles, setCurrentFiles] = useState<FileRecord[]>([]);
    const [currentPage, setCurrentPage] = useState(1);
    const totalPages = Math.ceil(currentFiles.length / pageSize);

    const paginatedData = currentFiles.slice(
        (currentPage - 1) * pageSize,
        currentPage * pageSize
    );

    useEffect(() => {
        setCurrentPage(1);
    }, [totalPages]);

    useEffect(() => {
        setCurrentFiles(files);
    }, [files]);

    useEffect(() => {
        async function applyFilter() {
            const filtered = await asyncFilter(files, async (file) => filterIsPresent(currentFilter, file));
            setCurrentFiles(filtered);
        };

        applyFilter();
    }, [currentFilter]);

    useEffect(() => {
        async function setup() {
            setIsLoading(true);
            setFiles(await get<FileRecord[]>("journal_removes"));
            setIsLoading(false);
        };

        setup();
    }, []);

    return (
        <>
            {
                isLoading ? <Center height="full">
                    <Spinner size="xl" />
                </Center> : (
                    <Flex height="full" gap={5} direction="column">
                        <Input height="50px" value={currentFilter} onChange={(e) => setCurrentFilter(e.target.value)} _placeholder={{ color: "gray" }} variant="outline" borderColor="border" placeholder="Фильтр" borderRadius="20px" />

                        <Box padding={5} borderRadius={20} borderWidth="1px" width="full" height="full" overflow="hidden" scrollbar="hidden">
                            <Box spaceY={3} paddingRight={5} direction="column" scrollbar="visible" width="full" height="full" scrollBehavior="smooth" _scrollbarThumb={{ padding: "5", borderRadius: "20px", width: "1px", background: "gray" }} overflowY="auto">
                                {
                                    paginatedData.map((file) =>
                                        <Box direction="row" className="flex" justifyContent="space-between" paddingX={5} paddingY={5} borderRadius={20} borderWidth="1px">
                                            <Box>
                                                <Text minWidth="min-content"
                                                    whiteSpace="normal"
                                                    fontSize="14px"
                                                    wordBreak="break-word">Имя файла</Text>
                                                <Text minWidth="min-content"
                                                    whiteSpace="normal"
                                                    wordBreak="break-word" color="gray" fontSize="14px">
                                                    <Highlight styles={{ background: "white", height: "fit", color: "black" }} query={currentFilter.split("||").map((item) => item.trim())}>
                                                        {file.name}
                                                    </Highlight>
                                                </Text>
                                            </Box>

                                            <Box>
                                                <Text minWidth="min-content"
                                                    whiteSpace="normal"
                                                    fontSize="14px"
                                                    wordBreak="break-word">Дата удаления</Text>
                                                <Text minWidth="min-content"
                                                    whiteSpace="normal"
                                                    wordBreak="break-word" color="gray" fontSize="14px">{dateFromUsn(file.timestamp)}</Text>
                                            </Box>
                                        </Box>)
                                }
                            </Box>
                        </Box>

                        <PaginationRoot
                            count={currentFiles.length}
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
                        </PaginationRoot>
                    </Flex>
                )
            }
        </>
    )
}