import {
  Badge,
  Box,
  Center,
  Flex,
  Link,
  Spinner,
  Text,
} from "@chakra-ui/react";
import { useEffect, useState } from "react";
import { SteamAccount } from "../../utils/types";
import useStorage from "../../hooks/storage";
import { invoke } from "@tauri-apps/api/core";
import { dateFromUnix } from "../../utils/utils";
import { Icon } from "@iconify/react/dist/iconify.js";

type WrappedSteamAccount = {
  source: SteamAccount;
  inCache: boolean;
  vac: boolean;
};

export default function CSHunterSteamAccPage() {
  const [, get] = useStorage();
  const [isLoading, setIsLoading] = useState<boolean>(true);
  const [accounts, setAccounts] = useState<WrappedSteamAccount[]>([]);

  useEffect(() => {
    setIsLoading(true);

    async function setup() {
      const accs = await get<SteamAccount[]>("steam_accounts") ?? [];
      const cache = await get<string[]>("steam_avatar_cache") ?? [];
      let response: WrappedSteamAccount[] = [];

      for (const [_, acc] of accs.entries()) {
        const vac: boolean = await invoke("is_vac_present", { account: acc });

        response.push({
          source: acc,
          inCache: cache.filter((val) => val === acc.id.toString()).length > 0,
          vac: vac,
        });
      }

      setAccounts(response);
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
            {accounts.length > 0 ? (
              accounts.map((account, i) => (
                <Flex
                  key={i}
                  direction="row"
                  justify="space-between"
                  gap={5}
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
                        Логин
                      </Text>
                      <Link
                        minWidth="min-content"
                        onClick={async () => {
                          await invoke("open_url", {
                            url: `https://steamcommunity.com/profiles/${account.source.id}`,
                          });
                        }}
                        whiteSpace="normal"
                        wordBreak="break-word"
                        color="gray"
                        fontSize="14px"
                      >
                        {account.source.account_name}
                      </Link>
                    </Box>

                    <Box>
                      <Text
                        minWidth="min-content"
                        whiteSpace="normal"
                        fontSize="14px"
                        wordBreak="break-word"
                      >
                        Дата входа
                      </Text>
                      <Text
                        minWidth="min-content"
                        whiteSpace="normal"
                        wordBreak="break-word"
                        color="gray"
                        fontSize="14px"
                      >
                        {dateFromUnix(Number(account.source.timestamp))}
                      </Text>
                    </Box>
                  </Flex>

                  <Flex gap={1}>
                    {account.vac ? (
                      <Badge
                        borderRadius="20px"
                        width="fit"
                        height="fit"
                        colorPalette="red"
                      >
                        vac-бан
                      </Badge>
                    ) : (
                      <></>
                    )}

                    {account.source.most_recent == "1" ? (
                      <Badge
                        borderRadius="20px"
                        width="fit"
                        height="fit"
                        colorPalette="green"
                      >
                        текущий
                      </Badge>
                    ) : (
                      <></>
                    )}

                    {!account.inCache ? (
                      <Badge
                        borderRadius="20px"
                        width="fit"
                        height="fit"
                        colorPalette="red"
                      >
                        неавторизованный
                      </Badge>
                    ) : (
                      <Badge
                        borderRadius="20px"
                        width="fit"
                        height="fit"
                        colorPalette="green"
                      >
                        авторизованный
                      </Badge>
                    )}
                  </Flex>
                </Flex>
              ))
            ) : (
              <Center
                gap="1"
                width="full"
                height="full"
                className="flex flex-col"
              >
                <Icon
                  color="gray"
                  width="60px"
                  height="60px"
                  icon="lets-icons:sad-light"
                ></Icon>
                <Text fontSize="14px" color="gray">
                  Аккаунты не найдены
                </Text>
              </Center>
            )}
          </Box>
        </Box>
      )}
    </>
  );
}
