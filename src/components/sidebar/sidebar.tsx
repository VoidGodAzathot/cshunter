import { useState } from "react";
import { Page } from "../../utils/types";
import { Box, Button, Flex } from "@chakra-ui/react";

export default function Sidebar({ pages, provider }: { pages: Page[], provider: (page: Page) => JSX.Element }) {
    const [currentPage, setCurrentPage] = useState<Page>(pages[0]);

    return (
        <Box width="full" height="full">
            <Flex paddingRight={5} paddingLeft={5} alignItems="start" height="full" spaceX={5}>
                <Flex justify="space-between" height="calc(100vh - 50px)" className="flex flex-col transition-colors duration-200 ease-in-out">
                    <Flex spaceY={2} className={"flex flex-col"}>
                    {
                        pages.map((page) =>
                            <Button color={currentPage.name == page.name ? "black" : "white"} background={currentPage.name == page.name ? "white" : "transparent"} onClick={() => { if (currentPage.name != page.name) { setCurrentPage(page) } }} cursor={currentPage.name == page.name ? "default" : ""} width="50px" height="50px" variant="ghost" borderRadius={50}>
                                {page.icon}
                            </Button>)
                    }
                    </Flex>
                </Flex>

                { provider(currentPage) }
            </Flex>
        </Box>
    )
}