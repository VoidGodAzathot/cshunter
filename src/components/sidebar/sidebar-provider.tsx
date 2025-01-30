import { Box } from "@chakra-ui/react";
import { Page } from "../../utils/types";

export default function SidebarProvider(page: Page) {
    return (
        <Box backgroundColor="#18181B" borderRadius={20} borderWidth="1.5px" width="full" height="calc(100vh - 50px)">
            <Box className="select-none" padding={5} height="full">
                { page.source }
            </Box>
        </Box>
    )
}