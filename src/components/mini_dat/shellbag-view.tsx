import { useEffect, useState } from "react"
import { ShellBagView } from "../../utils/types"
import { jsonToType } from "../../utils/utils";
import { Text, Flex, Badge } from "@chakra-ui/react";

export default function ShellbagView({ item, }: { item: string, filter: string }) {
    const [type, setType] = useState<ShellBagView | undefined>(undefined);

    useEffect(() => {
        setType(jsonToType<ShellBagView>(item));
    }, []);

    return (
        <>
            {
                type ? <Flex height="full" justify="space-between">
                    <Flex direction="column" gap={1}>
                        <Flex direction="column">
                            <Text minWidth="min-content"
                                whiteSpace="normal"
                                fontSize="12px"
                                wordBreak="break-word">
                                Имя папки
                                {
                                    <Badge marginLeft={5} height="fit" colorPalette={type.action.toString() === "DELETE" ? "red" : "blue"} borderRadius="20px">{type.action.toString().toLowerCase()}</Badge>
                                }
                            </Text>
                            <Text minWidth="min-content"
                                whiteSpace="normal"
                                wordBreak="break-word" color="gray" fontSize="12px">
                                {type.name}
                            </Text>
                        </Flex>

                        <Flex direction="column">
                            <Text minWidth="min-content"
                                whiteSpace="normal"
                                fontSize="12px"
                                wordBreak="break-word">Путь</Text>
                            <Text minWidth="min-content"
                                whiteSpace="normal"
                                wordBreak="break-word" color="gray" fontSize="12px">
                                {type.path}
                            </Text>
                        </Flex>
                    </Flex>
                </Flex> : <></>
            }
        </>
    )
}