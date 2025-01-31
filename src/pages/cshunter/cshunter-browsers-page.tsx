import { useEffect, useState } from "react"
import { Browser, CacheDat, DownloadDat, VisitDat } from "../../utils/types";
import useStorage from "../../hooks/storage";
import { invoke } from "@tauri-apps/api/core";
import { Badge, Center, createListCollection, Flex, HStack, Input, Spinner, Stat, Tabs } from "@chakra-ui/react";
import BrowserTableDataWrapper from "../../components/browser/browser-table-data";
import { SelectContent, SelectItem, SelectRoot, SelectTrigger, SelectValueText } from "../../components/ui/select";
import { asyncFilter, filterIsPresent } from "../../utils/utils";

type BrowserData = {
    cache: CacheDat[],
    downloads: DownloadDat[],
    visits: VisitDat[]
}

export default function CSHunterBrowsersPage() {
    const [, get,] = useStorage();
    const [selectedBrowser, setSelectedBrowser] = useState<Browser | undefined>();
    const [selectedDataType, setSelectedDataType] = useState<string>("cache");
    const [currentFilter, setCurrentFilter] = useState<string>("");
    const [currentData, setCurrentData] = useState<any[]>([]);
    const [browsers, setBrowsers] = useState<Browser[]>([]);
    const [browsersData, setBrowsersData] = useState<BrowserData | undefined>(undefined);
    const dataTypes = createListCollection({ items: [{ label: "Кэш", value: "cache" }, { label: "Посещения", value: "visits" }, { label: "Загрузки", value: "downloads" }] })

    async function applyFilter() {
        const response = browsersData ? (selectedDataType === "cache" ? await asyncFilter(browsersData.cache, async (item) => item.browser == selectedBrowser?.id && filterIsPresent(currentFilter, item)) : selectedDataType === "downloads" ? await asyncFilter(browsersData.downloads, async (item) => item.browser == selectedBrowser?.id && filterIsPresent(currentFilter, item)) : await asyncFilter(browsersData.visits, async (item) => item.browser == selectedBrowser?.id && filterIsPresent(currentFilter, item))) : [];
        setCurrentData(response);
    }

    useEffect(() => {
        async function fetchBrowserData() {
            const download_dat: DownloadDat[] = await get<DownloadDat[]>("browsers_download_dat");
            const visit_dat: VisitDat[] = await get<VisitDat[]>("browsers_visit_dat");
            const cache_dat: CacheDat[] = await get<CacheDat[]>("browsers_cache_dat");
            const _browsers: Browser[] = await invoke("get_supported_browsers");
            setBrowsersData({ cache: cache_dat, downloads: download_dat, visits: visit_dat });
            setSelectedBrowser(_browsers.find((browser) => browser.support));
            setBrowsers(_browsers);
        }

        fetchBrowserData();
    }, []);

    useEffect(() => {
        applyFilter();
    }, [currentFilter, selectedDataType, browsers, selectedBrowser]);

    return (
        <>
            {
                browsers.length > 0 ?
                    (<Flex height="full" direction={"column"} justify="space-between">
                        <Flex spaceX={5}>
                            <Input value={currentFilter} onChange={(e) => setCurrentFilter(e.target.value)} _placeholder={{ color: "gray" }} variant="outline" borderColor="border" placeholder="Фильтр" borderRadius="20px" />

                            <SelectRoot value={[selectedDataType]} onValueChange={(v) => setSelectedDataType(v.value[0])} positioning={{ placement: "bottom", flip: false }} collection={dataTypes}>
                                <SelectTrigger>
                                    <SelectValueText placeholder="Тип данных" />
                                </SelectTrigger>
                                <SelectContent shadow="none" borderWidth="1px" bg="#18181B" borderRadius={20}>
                                    {
                                        dataTypes.items.map((item) => <SelectItem borderRadius={20} item={item} key={item.value}>{item.label}</SelectItem>)
                                    }
                                </SelectContent>
                            </SelectRoot>
                        </Flex>

                        <Flex height="full" width="full">
                            <Tabs.Root height="full" width="full" paddingRight="40px" variant="plain" value={selectedBrowser ? selectedBrowser.id : ""} onValueChange={(e) => setSelectedBrowser(e.value ? browsers.find((browser) => browser.id == e.value) : undefined)} defaultValue={browsers.find((browser) => browser.support)?.id}>
                                <Flex spaceX={5} paddingY={5}>
                                    <Tabs.List>
                                        {
                                            browsers.map((browser) => (<Tabs.Trigger disabled={!browser.support} borderWidth={selectedBrowser && selectedBrowser.id == browser.id ? "1px" : "0px"} borderRadius={20} key={browser.id} value={browser.id}>{browser.id}</Tabs.Trigger>))
                                        }
                                    </Tabs.List>
                                </Flex>

                                {
                                    browsers.map((browser) => (<Tabs.Content padding={0} value={browser.id}>{BrowserTableDataWrapper(currentFilter, currentData)}</Tabs.Content>))
                                }
                            </Tabs.Root>

                            <Flex gap="5" paddingTop={5} height="fit" direction="column">
                                <Stat.Root borderWidth="1px" p="4" rounded={20}>
                                    <Stat.Label fontSize={14}>
                                        Всего посещений
                                    </Stat.Label>
                                    <HStack>
                                        <Stat.ValueText fontSize={18}>
                                            {browsersData?.visits.length}
                                        </Stat.ValueText>
                                        {
                                            browsersData && browsersData?.visits.length <= 250 ?
                                                (
                                                    <Badge width="fit" height="fit" colorPalette="red">
                                                        очищено
                                                    </Badge>
                                                ) : <></>
                                        }
                                    </HStack>
                                </Stat.Root>
                                <Stat.Root borderWidth="1px" p="4" rounded={20}>
                                    <Stat.Label fontSize={14}>
                                        Всего загрузок
                                    </Stat.Label>
                                    <Stat.ValueText fontSize={18}>
                                        {browsersData?.downloads.length}
                                    </Stat.ValueText>
                                </Stat.Root>
                                <Stat.Root borderWidth="1px" p="4" rounded={20}>
                                    <Stat.Label fontSize={14}>
                                        Размер кэша
                                    </Stat.Label>
                                    <Stat.ValueText fontSize={18}>
                                        {browsersData?.cache.length}
                                    </Stat.ValueText>
                                </Stat.Root>
                            </Flex>
                        </Flex>
                    </Flex>)
                    :
                    <Center height="full">
                        <Spinner size="xl" />
                    </Center>
            }
        </>
    )
}