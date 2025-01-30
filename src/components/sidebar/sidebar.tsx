import { useState } from "react";
import { Page } from "../../utils/types";
import { Box, Button, Flex } from "@chakra-ui/react";

export default function Sidebar({ pages, provider }: { pages: Page[], provider: (page: Page) => JSX.Element }) {
    const [currentPage, setCurrentPage] = useState<Page>(pages[0]);

    return (
        <Box width="full" height="full">
            <Flex paddingRight={5} paddingLeft={5} alignItems="start" height="full" spaceX={5}>
                <Flex spaceY={2} className={"flex transition-all duration-200 ease-in-out flex-col"}>
                    {
                        pages.map((page) =>
                            <Button onClick={() => { if (currentPage.name != page.name) { setCurrentPage(page) } }} cursor={currentPage.name == page.name ? "default" : ""} disabled={currentPage.name == page.name} width="fit" backgroundColor="transparent" variant="ghost" borderRadius={50}>
                                {page.icon}
                            </Button>)
                    }
                </Flex>

                { provider(currentPage) }
            </Flex>
        </Box>
    )
}