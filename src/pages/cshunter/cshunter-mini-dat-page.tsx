import { useEffect, useState } from "react";
import useStorage from "../../hooks/storage";
import { MiniDat, MiniDatInfo } from "../../utils/types";
import {
  Box,
  Center,
  Collapsible,
  Flex,
  Input,
  Spinner,
  Text,
  Highlight,
  Badge,
} from "@chakra-ui/react";
import { invoke } from "@tauri-apps/api/core";
import { asyncFilter, filterIsPresent } from "../../utils/utils";

type MiniDatKey = {
  id: string;
  name: string;
  desc: string;
  stable: boolean;
  filtering: boolean;
};

export default function CSHunterMiniDatPage() {
  const [, get, ,] = useStorage();
  const [isLoading, setIsLoading] = useState<boolean>(true);
  const [currentFilter, setCurrentFilter] = useState<string>("");
  const [sortedMiniDat, setSortedMiniDat] = useState<Map<MiniDatKey, string[]>>(
    new Map()
  );
  const [finalMiniDat, setFinalMiniDat] = useState<Map<MiniDatKey, string[]>>(
    new Map()
  );

  useEffect(() => {
    async function setup() {
      const filtered: Map<MiniDatKey, string[]> = new Map<
        MiniDatKey,
        string[]
      >();
      const keysArray = Array.from(sortedMiniDat.keys());
      for (let i = 0; i < keysArray.length; i++) {
        const key = keysArray[i];
        const values = sortedMiniDat.get(key);
        if (values) {
          let v = key.filtering
            ? await asyncFilter(
              values,
              async (value) => await filterIsPresent(currentFilter, value)
            )
            : values;
          filtered.set(key, v);
        }
      }
      setFinalMiniDat(filtered);
    }

    setup();
  }, [currentFilter, sortedMiniDat]);

  useEffect(() => {
    async function setup() {
      setIsLoading(true);
      const mini_dat = await get<MiniDat[]>("mini_dat");
      let map = new Map<MiniDatKey, string[]>();
      for (const [_, dat] of mini_dat.entries()) {
        const _finded = Array.from(map.keys()).filter((k) => k.id === dat.id);
        if (_finded.length != 0) {
          let val = map.get(_finded[0]);
          if (val) {
            val.push(dat.value);
          }
        } else {
          const val: string[] = [dat.value];
          const meta: MiniDatInfo = await invoke("get_mini_dat_info", {
            id: dat.id,
          });
          map.set(
            {
              id: meta.id,
              name: meta.name,
              desc: meta.description,
              stable: meta.stable,
              filtering: meta.filtering,
            },
            val
          );
        }
      }
      setSortedMiniDat(map);
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
              {Array.from(finalMiniDat.keys()).map((key, index) => (
                <Collapsible.Root key={index}>
                  <Collapsible.Trigger width="full">
                    <Flex
                      gap={5}
                      justify="space-between"
                      cursor="pointer"
                      borderWidth="1px"
                      borderRadius="20px"
                      padding={5}
                      alignItems="center"
                    >
                      <Flex direction="column" alignItems="start">
                        <Flex align="center" gap={1} spaceX={1}>
                          <Text fontSize="14px">{key.name}</Text>
                          <Flex gap={1}>
                            {key.stable ? (
                              <></>
                            ) : (
                              <Badge colorPalette="red" borderRadius="20px">
                                нестабильный
                              </Badge>
                            )}
                            {key.filtering ? (
                              <></>
                            ) : (
                              <Badge colorPalette="red" borderRadius="20px">
                                фильтры не применяются
                              </Badge>
                            )}
                          </Flex>
                        </Flex>
                        <Text
                          textAlign="left"
                          whiteSpace="normal"
                          fontSize="12px"
                          wordBreak="break-word"
                          color="gray"
                        >
                          {key.desc}
                        </Text>
                      </Flex>

                      <Text>{finalMiniDat.get(key)?.length}</Text>
                    </Flex>
                  </Collapsible.Trigger>
                  <Collapsible.Content paddingTop={3}>
                    <Flex
                      gap={1}
                      direction="column"
                      padding={5}
                      borderWidth="1px"
                      borderRadius="20px"
                    >
                      {finalMiniDat.get(key)?.map((item, i) =>
                      (
                        <Text
                          key={i}
                          textAlign="left"
                          whiteSpace="normal"
                          fontSize="12px"
                          wordBreak="break-word"
                          color="gray"
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
                            {item}
                          </Highlight>
                        </Text>
                      )
                      )}
                    </Flex>
                  </Collapsible.Content>
                </Collapsible.Root>
              ))}
            </Box>
          </Box>
        </Flex>
      )}
    </>
  );
}
