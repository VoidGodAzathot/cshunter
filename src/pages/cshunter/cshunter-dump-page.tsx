import {
  Box,
  Button,
  Center,
  Flex,
  HStack,
  Input,
  Stat,
  Text,
} from "@chakra-ui/react";
import { Icon } from "@iconify/react/dist/iconify.js";
import { useEffect, useState } from "react";
import {
  PaginationItems,
  PaginationNextTrigger,
  PaginationPrevTrigger,
  PaginationRoot,
} from "../../components/ui/pagination";
import useStorage from "../../hooks/storage";
import { invoke } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";
import { toaster, Toaster } from "../../components/ui/toaster";

export default function CSHunterDumpPage() {
  const [, get, ,] = useStorage();
  const [isLoading, setIsLoading] = useState<boolean>(true);
  const [isMatching, setIsMatching] = useState<boolean>(false);
  const [currentFilter, setCurrentFilter] = useState<string>("");
  const [countModulesStrings, setCountModulesStrings] = useState<number>(0);
  const [countStrings, setCountStrings] = useState<number>(0);
  const [matches, setMatches] = useState<string[]>([]);
  const [currentPage, setCurrentPage] = useState(1);
  const pageSize = 1000;
  let paginatedData = [];
  let totalPages = 0;

  totalPages = Math.ceil(matches.length / pageSize);
  paginatedData = matches.slice(
    (currentPage - 1) * pageSize,
    currentPage * pageSize
  );

  useEffect(() => {
    setCurrentPage(1);
  }, [totalPages]);

  useEffect(() => {
    async function setup() {
      setIsLoading(true);
      setCountModulesStrings(await get<number>("cs2_modules_strings_len") ?? 0);
      setCountStrings(await get<number>("cs2_strings_len") ?? 0);
      setIsLoading(false);
    }

    setup();
  }, []);

  return (
    <>
      <Toaster />

      {!countModulesStrings || !countStrings ? (
        <Center gap="1" width="full" height="full" className="flex flex-col">
          <Icon
            color="gray"
            width="60px"
            height="60px"
            icon="lets-icons:sad-light"
          ></Icon>
          <Text fontSize="14px" color="gray">
            Дамп не был сделан
          </Text>
        </Center>
      ) : (
        <Flex height="full" gap={5} direction="column">
          <Flex width="full">
            <Flex width="full" direction="column">
              <Flex
                justify="space-between"
                width="full"
                gap={5}
                direction="row"
              >
                <Flex gap={5}>
                  <Flex alignItems="center" justify="center" direction="column">
                    <Stat.Root borderWidth="1px" p="4" rounded={20}>
                      <Icon
                        color="gray"
                        width="50px"
                        height="50px"
                        icon="oui:token-module"
                      />
                      <Stat.Label width="40" fontSize={14}>
                        Получено строк с модулей процесса
                      </Stat.Label>
                      <Stat.ValueText fontSize={18}>
                        {countModulesStrings}
                      </Stat.ValueText>
                    </Stat.Root>
                  </Flex>
                  <Flex alignItems="center" justify="center" direction="column">
                    <Stat.Root borderWidth="1px" p="4" rounded={20}>
                      <Icon
                        color="gray"
                        width="50px"
                        height="50px"
                        icon="carbon:cics-region"
                      />
                      <Stat.Label width="40" fontSize={14}>
                        Получено строк со всех участков памяти
                      </Stat.Label>
                      <Stat.ValueText fontSize={18}>
                        {countStrings}
                      </Stat.ValueText>
                    </Stat.Root>
                  </Flex>
                </Flex>
                <Flex gap={5} height="full" direction="column" width="fit">
                  <Text>Поиск по фильтру</Text>
                  <Input
                    height="full"
                    value={currentFilter}
                    onChange={(e) => setCurrentFilter(e.target.value)}
                    _placeholder={{ color: "gray" }}
                    variant="outline"
                    borderColor="border"
                    placeholder="Фильтр"
                    borderRadius="20px"
                    textAlign="start"
                  />
                  <Button
                    disabled={isLoading || isMatching}
                    onClick={async () => {
                      if (currentFilter.length == 0) {
                        return;
                      }
                      setIsMatching(true);
                      const matches_: string[] = await invoke("find_strings", {
                        filter: currentFilter,
                      });
                      setMatches(matches_);
                      setIsMatching(false);
                    }}
                    variant="surface"
                    borderRadius={50}
                  >
                    Поиск
                  </Button>
                </Flex>
              </Flex>
            </Flex>
          </Flex>

          <Flex height="calc(100vh - 280px)" direction="column" gap={5}>
            <Box
              padding={5}
              borderRadius={20}
              borderWidth="1px"
              width="full"
              height="full"
              overflow="hidden"
              scrollbar="hidden"
            >
              <Box
                spaceY={3}
                paddingRight={5}
                direction="column"
                scrollbar="visible"
                width="full"
                height="full"
                scrollBehavior="smooth"
                _scrollbarThumb={{
                  padding: "5",
                  borderRadius: "20px",
                  width: "1px",
                  background: "gray",
                }}
                overflowY="auto"
              >
                {paginatedData.map((m) => (
                  <Text
                    minWidth="min-content"
                    whiteSpace="normal"
                    wordBreak="break-word"
                    color="gray"
                    fontSize="12px"
                  >
                    {m}
                  </Text>
                ))}
              </Box>
            </Box>

            <Flex alignItems="center" justify="space-between">
              <PaginationRoot
                count={matches.length}
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
                    onClick={() =>
                      setCurrentPage((p) => Math.min(totalPages, p + 1))
                    }
                  />
                </HStack>
              </PaginationRoot>

              {matches.length == 0 ? (
                <></>
              ) : (
                <Button
                  onClick={async () => {
                    const file = await open({
                      multiple: false,
                      filters: [{ name: "", extensions: ["txt"] }],
                      directory: false,
                    });
                    if (file) {
                      let result: boolean = await invoke(
                        "create_file_and_write",
                        { path: file, data: matches.join("\n") }
                      );
                      toaster.create({
                        title: result
                          ? "Успешно выполнено"
                          : "Ошибка выполнения",
                        type: result ? "success" : "error",
                      });
                    }
                  }}
                  variant="surface"
                  borderRadius="50px"
                >
                  Сохранить совпадения ({matches.length})
                </Button>
              )}
            </Flex>
          </Flex>
        </Flex>
      )}
    </>
  );
}
