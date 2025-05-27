import { useState } from "react";
import { Page } from "../../utils/types";
import { Box, Button, Flex } from "@chakra-ui/react";
import { Icon } from "@iconify/react/dist/iconify.js";
import { invoke } from "@tauri-apps/api/core";
import { save } from "@tauri-apps/plugin-dialog";
import { toaster } from "../../components/ui/toaster";

export default function Sidebar({
  pages,
  provider,
}: {
  pages: Page[];
  provider: (page: Page) => JSX.Element;
}) {
  const [currentPage, setCurrentPage] = useState<Page>(pages[0]);

  return (
    <Box width="full" height="full">
      <Flex
        paddingRight={5}
        paddingLeft={5}
        alignItems="start"
        height="full"
        spaceX={5}
      >
        <Flex
          justify="space-between"
          height="calc(100vh - 50px)"
          className="flex flex-col transition-colors duration-200 ease-in-out"
        >
          <Flex spaceY={2} className={"flex flex-col"}>
            {pages.map((page, i) => (
              <Button
                key={i}
                color={currentPage.name == page.name ? "black" : "white"}
                background={
                  currentPage.name == page.name ? "white" : "transparent"
                }
                onClick={() => {
                  if (currentPage.name != page.name) {
                    setCurrentPage(page);
                  }
                }}
                cursor={currentPage.name == page.name ? "default" : ""}
                width="50px"
                height="50px"
                variant="ghost"
                borderRadius={50}
              >
                {page.icon}
              </Button>
            ))}
          </Flex>

          <Button
            onClick={async () => {
              const file = await save({
                filters: [{ name: "", extensions: ["gz"] }],
              });
              if (file) {
                await invoke("create_file_and_write", {
                  path: file,
                  data: await invoke("export_all_data"),
                })
                  .then(() => {
                    toaster.create({
                      title: "Успешно выполнено",
                      type: "success",
                    });
                  })
                  .catch((e) => {
                    toaster.create({
                      title: "Ошибка выполнения",
                      description: e,
                      type: "error",
                    });
                  });
              }
            }}
            width="50px"
            height="50px"
            variant="ghost"
            borderRadius={50}
          >
            <Icon icon="tabler:package-export"></Icon>
          </Button>
        </Flex>

        {provider(currentPage)}
      </Flex>
    </Box>
  );
}
