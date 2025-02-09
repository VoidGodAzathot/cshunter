import { useEffect, useState } from "react";
import useStorage from "../../hooks/storage";
import { DriverInfo } from "../../utils/types";
import {
  Box,
  Center,
  Flex,
  HStack,
  Input,
  Spinner,
  Text,
  Highlight,
  Badge,
  Tooltip,
  Button,
} from "@chakra-ui/react";
import {
  PaginationItems,
  PaginationNextTrigger,
  PaginationPrevTrigger,
  PaginationRoot,
} from "../../components/ui/pagination";
import { asyncFilter, filterIsPresent } from "../../utils/utils";
import { Icon } from "@iconify/react/dist/iconify.js";

export default function CSHunterDriversPage() {
  const [, get, ,] = useStorage();
  const pageSize = 50;
  const [isLoading, setIsLoading] = useState<boolean>(true);
  const [currentFilter, setCurrentFilter] = useState<string>("");
  const [drivers, setDrivers] = useState<DriverInfo[]>([]);
  const [currentDrivers, setCurrentDrivers] = useState<DriverInfo[]>([]);
  const [currentPage, setCurrentPage] = useState(1);
  const totalPages = Math.ceil(currentDrivers.length / pageSize);

  const paginatedData = currentDrivers.slice(
    (currentPage - 1) * pageSize,
    currentPage * pageSize
  );

  useEffect(() => {
    setCurrentPage(1);
  }, [totalPages]);

  useEffect(() => {
    setCurrentDrivers(drivers);
  }, [drivers]);

  useEffect(() => {
    async function applyFilter() {
      const filtered = await asyncFilter(
        drivers,
        async (driver) => await filterIsPresent(currentFilter, driver)
      );
      setCurrentDrivers(filtered);
    }

    applyFilter();
  }, [currentFilter]);

  useEffect(() => {
    async function setup() {
      setIsLoading(true);
      let driversInfo = await get<DriverInfo[]>("drivers_info");
      driversInfo = driversInfo.sort(
        (a, b) =>
          (a.trust && a.path.length != 0 ? 1 : 0) -
          (b.trust && b.path.length != 0 ? 1 : 0)
      );
      setDrivers(driversInfo);
      setIsLoading(false);
    }

    setup();
  }, []);

  return (
    <>
      {isLoading ? (
        <Center height="full">
          <Spinner size="xl" />
        </Center>
      ) : (
        <Flex height="full" gap={5} direction="column">
          <Input
            height="50px"
            value={currentFilter}
            onChange={(e) => setCurrentFilter(e.target.value)}
            _placeholder={{ color: "gray" }}
            variant="outline"
            borderColor="border"
            placeholder="Фильтр"
            borderRadius="20px"
          />

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
              {paginatedData.map((driver, i) => (
                <Flex
                  key={i}
                  direction="row"
                  justify="space-between"
                  paddingX={5}
                  paddingY={5}
                  borderRadius={20}
                  borderWidth="1px"
                >
                  <Flex direction="column" gap={1}>
                    <Box>
                      <Text
                        minWidth="min-content"
                        whiteSpace="normal"
                        fontSize="14px"
                        wordBreak="break-word"
                      >
                        Имя драйвера
                      </Text>
                      <Text
                        minWidth="min-content"
                        whiteSpace="normal"
                        wordBreak="break-word"
                        color="gray"
                        fontSize="14px"
                      >
                        <Highlight
                          styles={{
                            background: "white",
                            height: "fit",
                            color: "black",
                          }}
                          query={currentFilter
                            .split("||")
                            .map((item) => item.trim())}
                        >
                          {driver.name}
                        </Highlight>
                      </Text>
                    </Box>

                    <Box>
                      <Text
                        minWidth="min-content"
                        whiteSpace="normal"
                        fontSize="14px"
                        wordBreak="break-word"
                      >
                        Путь установки
                      </Text>
                      <Text
                        minWidth="min-content"
                        whiteSpace="normal"
                        wordBreak="break-word"
                        color="gray"
                        fontSize="14px"
                      >
                        {driver.path.length == 0 ? (
                          <Badge borderRadius="20px" colorPalette="red">
                            не удалось получить путь (возможно драйвер встроен в
                            систему)
                          </Badge>
                        ) : (
                          driver.path
                        )}
                      </Text>
                    </Box>
                  </Flex>

                  <Box>
                    <Badge
                      borderRadius="20px"
                      colorPalette={
                        driver.trust && driver.path.length != 0
                          ? "green"
                          : "red"
                      }
                    >
                      {driver.trust && driver.path.length != 0
                        ? "легитимный"
                        : "нелегитимный"}
                    </Badge>
                  </Box>
                </Flex>
              ))}
            </Box>
          </Box>

          <PaginationRoot
            count={currentDrivers.length}
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
        </Flex>
      )}
    </>
  );
}
