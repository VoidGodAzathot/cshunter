import { Box, Container, Highlight, Flex, HStack, Text, Center, Spinner } from "@chakra-ui/react"
import { PaginationItems, PaginationNextTrigger, PaginationPrevTrigger, PaginationRoot } from "../ui/pagination"
import { useEffect, useState } from "react";
import { dateFromWebkit } from "../../utils/utils";
import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";
import { Icon } from "@iconify/react/dist/iconify.js";

export default function BrowserTableDataWrapper(filter: string, data: object[]): JSX.Element {
    const getAllKeys = <T extends object>(arr: T[]) => {
        return Array.from(new Set(arr.flatMap(Object.keys)));
    };

    return <BrowserTableData filter={filter} data={data} keys={getAllKeys(data).filter((d) => d != "browser")}></BrowserTableData>
}

function BrowserTableData({ filter, data, keys }: { filter: string, data: object[]; keys: string[] }) {
    const [isLoading, setIsLoading] = useState<boolean>(true);
    const [currentPage, setCurrentPage] = useState(1);
    const [pageSize, setPageSize] = useState<number>(2);
    const totalPages = Math.ceil(data.length / pageSize);

    const paginatedData = data.slice(
        (currentPage - 1) * pageSize,
        currentPage * pageSize
    );

    useEffect(() => {
        async function setup() {
            function resize(height: number) {
                if (height > 1000) {
                    setPageSize(3);
                } else {
                    if (height < 800) {
                        setPageSize(1);
                    } else {
                        setPageSize(2);
                    }
                }
            }

            const window = await getCurrentWebviewWindow();
            resize((await window.size()).height);
            window.onResized((e) => {
                resize(e.payload.height);
            });
            setIsLoading(false);
        }

        setup();
    }, []);

    useEffect(() => {
        setCurrentPage(1);
    }, [totalPages]);

    return (
        <>
            {
                isLoading ? <Center minH="100px"><Spinner /></Center> : (<Flex gap="5" direction="column" justify="space-between">
                    <Flex gap={{ base: 2, md: 5 }} direction="column">
                        {paginatedData.length > 0 ? paginatedData.map((item, index) => (
                            <Container
                                margin="0"
                                fontSize={{ base: 12, md: 14 }}
                                borderRadius={{ base: "10px", md: "20px" }}
                                borderWidth="1px"
                                p={{ base: 3, md: 5 }}
                                key={index}
                                minWidth="min-content"
                            >
                                {keys.map((key) => (
                                    <Box
                                        minWidth="min-content"
                                        whiteSpace="normal"
                                        wordBreak="break-word"
                                        mb={2}
                                        key={key}
                                    >
                                        <Text fontWeight="medium">
                                            {key === "title" ? "Заголовок"
                                                : key === "url" ? "Ссылка"
                                                    : key === "timestamp" ? "Дата"
                                                        : key === "file" ? "Путь"
                                                            : ""}
                                        </Text>
                                        <Text
                                            overflow="hidden"
                                            maxH="150px"
                                            color="gray">
                                            <Highlight
                                                query={filter.length == 0 ? [] : filter.split("||").map((item) => item.trim())}
                                                styles={{ background: "white", color: "black" }}
                                            >
                                                {key === "timestamp"
                                                    ? dateFromWebkit((item as Record<string, any>)[key] as number)
                                                    : (item as Record<string, any>)[key] || "-"}
                                            </Highlight>
                                        </Text>
                                    </Box>
                                ))}
                            </Container>
                        )) :
                            <Center gap="1" width="full" height="full" className="flex flex-col">
                                <Icon color="gray" width="60px" height="60px" icon="lets-icons:sad-light"></Icon>
                                <Text fontSize="14px" color="gray">Ничего не найдено по фильтру "{filter}"</Text>
                            </Center>}
                    </Flex>

                    {
                        paginatedData.length > 0 ? (<PaginationRoot
                            count={data.length}
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
                </Flex>)
            }
        </>
    );
}