import { useEffect, useState } from "react";
import useStorage from "../../hooks/storage"
import { MiniDat, MiniDatInfo } from "../../utils/types";
import { Box, Center, Collapsible, Flex, Input, Spinner, Text, Highlight } from "@chakra-ui/react";
import { invoke } from "@tauri-apps/api/core";
import { filterIsPresent } from "../../utils/utils";

type MiniDatKey = {
    id: string,
    name: string,
    desc: string
}

export default function CSHunterMiniDatPage() {
    const [, get, ,] = useStorage();
    const [isLoading, setIsLoading] = useState<boolean>(true);
    const [currentFilter, setCurrentFilter] = useState<string>("");
    const [sortedMiniDat, setSortedMiniDat] = useState<Map<MiniDatKey, string[]>>(new Map());
    const [finalMiniDat, setFinalMiniDat] = useState<Map<MiniDatKey, string[]>>(new Map());

    useEffect(() => {
        const filtered: Map<MiniDatKey, string[]> = new Map<MiniDatKey, string[]>();
        const keysArray = Array.from(sortedMiniDat.keys());
        for (let i = 0; i < keysArray.length; i++) {
            const key = keysArray[i];
            const values = sortedMiniDat.get(key);
            if (values) {
                let v = values.filter((value) => filterIsPresent(currentFilter, value));
                filtered.set(key, v);
            }
        }
        setFinalMiniDat(filtered);
    }, [currentFilter, sortedMiniDat]);

    useEffect(() => {
        async function setup() {
            setIsLoading(true);
            const mini_dat = await get<MiniDat[]>("mini_dat");
            let map = new Map<MiniDatKey, string[]>();
            for (let i = 0; i < mini_dat.length; i++) {
                const dat = mini_dat[i];
                const _finded = Array.from(map.keys()).filter((k) => k.id === dat.id);
                if (_finded.length != 0) {
                    let val = map.get(_finded[0]);
                    if (val) {
                        val.push(dat.value);
                    }
                } else {
                    const val: string[] = [dat.value];
                    const meta: MiniDatInfo = await invoke("get_mini_dat_info", { id: dat.id });
                    map.set({ id: meta.id, name: meta.name, desc: meta.description }, val);
                }
            }
            setSortedMiniDat(map);
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
                                    Array.from(finalMiniDat.keys()).map((key, index) => (
                                        <Collapsible.Root key={index}>
                                            <Collapsible.Trigger width="full">
                                                <Flex gap={5} justify="space-between" cursor="pointer" borderWidth="1px" borderRadius="20px" padding={5} alignItems="center">
                                                    <Flex direction="column" alignItems="start">
                                                        <Text fontSize="14px">
                                                            {key.name}
                                                        </Text>
                                                        <Text
                                                            textAlign="left"
                                                            whiteSpace="normal"
                                                            fontSize="12px"
                                                            wordBreak="break-word" color="gray">
                                                            {key.desc}
                                                        </Text>
                                                    </Flex>

                                                    <Text>
                                                        {finalMiniDat.get(key)?.length}
                                                    </Text>
                                                </Flex>
                                            </Collapsible.Trigger>
                                            <Collapsible.Content paddingTop={3}>
                                                <Flex direction="column" padding={5} borderWidth="1px" borderRadius="20px">
                                                    {
                                                        finalMiniDat.get(key)?.map((item, i) => (<Text key={i} textAlign="left"
                                                            whiteSpace="normal"
                                                            fontSize="12px"
                                                            wordBreak="break-word" color="gray">
                                                            <Highlight styles={{ background: "white", height: "fit", color: "black" }} query={currentFilter.split("||").map((item) => item.trim())}>
                                                                {item}
                                                            </Highlight>
                                                        </Text>))
                                                    }
                                                </Flex>
                                            </Collapsible.Content>
                                        </Collapsible.Root>
                                    ))
                                }
                            </Box>
                        </Box>
                    </Flex>)
            }
        </>
    )
}