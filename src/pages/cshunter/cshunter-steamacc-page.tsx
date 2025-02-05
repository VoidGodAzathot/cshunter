import { Badge, Box, Center, Flex, Link, Spinner, Text } from "@chakra-ui/react";
import { useEffect, useState } from "react";
import { SteamAccount } from "../../utils/types";
import useStorage from "../../hooks/storage";
import { invoke } from "@tauri-apps/api/core";
import { dateFromUnix } from "../../utils/utils";

type WrappedSteamAccount = {
    source: SteamAccount,
    vac: boolean
}

export default function CSHunterSteamAccPage() {
    const [, get,] = useStorage();
    const [isLoading, setIsLoading] = useState<boolean>(true);
    const [accounts, setAccounts] = useState<WrappedSteamAccount[]>([]);

    useEffect(() => {
        setIsLoading(true);

        async function setup() {
            const accs = await get<SteamAccount[]>("steam_accounts");
            let response: WrappedSteamAccount[] = [];

            for (let i = 0; i < accs.length; i++) {
                const acc = accs[i];
                const vac: boolean = await invoke("is_vac_present", { account: acc });

                response.push({
                    source: acc,
                    vac: vac
                });
            }

            setAccounts(response);
            setIsLoading(false);
        };

        setup();
    }, []);

    return (
        <>
            {
                isLoading ? <Center height="full">
                    <Spinner size="xl" />
                </Center> :
                    (<Box padding={5} borderRadius={20} borderWidth="1px" width="full" height="full" overflow="hidden" scrollbar="hidden">
                        <Box spaceY={3} paddingRight={5} direction="column" scrollbar="visible" width="full" height="full" scrollBehavior="smooth" _scrollbarThumb={{ padding: "5", borderRadius: "20px", width: "1px", background: "gray" }} overflowY="auto">
                            {
                                accounts.map((account, i) => <Flex key={i} direction="row" justify="space-between" gap={5} paddingX={5} paddingY={5} borderRadius={20} borderWidth="1px">
                                    <Flex direction="column" gap={1}>
                                        <Box>
                                            <Text minWidth="min-content"
                                                whiteSpace="normal"
                                                fontSize="14px"
                                                wordBreak="break-word">Логин</Text>
                                            <Link minWidth="min-content"
                                                onClick={async () => {
                                                    await invoke("open_url", { url: `https://steamcommunity.com/profiles/${account.source.id}` });
                                                }}
                                                whiteSpace="normal"
                                                wordBreak="break-word" color="gray" fontSize="14px">{account.source.account_name}</Link>
                                        </Box>

                                        <Box>
                                            <Text minWidth="min-content"
                                                whiteSpace="normal"
                                                fontSize="14px"
                                                wordBreak="break-word">Дата входа</Text>
                                            <Text minWidth="min-content"
                                                whiteSpace="normal"
                                                wordBreak="break-word" color="gray" fontSize="14px">{dateFromUnix(Number(account.source.timestamp))}</Text>
                                        </Box>
                                    </Flex>

                                    <Flex gap={1}>
                                        {
                                            account.vac ? <Badge borderRadius="20px" width="fit" height="fit" colorPalette="red">
                                                vac-бан
                                            </Badge> : <></>
                                        }

                                        {
                                            account.source.most_recent == "1" ? <Badge borderRadius="20px" width="fit" height="fit" colorPalette="green">
                                                текущий
                                            </Badge> : <></>
                                        }
                                    </Flex>
                                </Flex>)
                            }
                        </Box>
                    </Box>)
            }
        </>
    )
}