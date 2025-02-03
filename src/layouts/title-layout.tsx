import { useEffect, useState } from "react";
import { getCurrentWebviewWindow, WebviewWindow } from "@tauri-apps/api/webviewWindow";
import { Badge, Button, Flex } from "@chakra-ui/react";
import { Icon } from '@iconify/react';
import useStorage from "../hooks/storage";
import { Tooltip } from "../components/ui/tooltip";
import { Tag } from "../utils/types";

function TitleLayout({ children }: { children: JSX.Element }) {
    const [, get, , setupListen] = useStorage();
    const [window, setWindow] = useState<WebviewWindow>();
    const [minimizable, setMinimizable] = useState<boolean>(false);
    const [maximizable, setMaximizable] = useState<boolean>(false);
    const [closable, setClosable] = useState<boolean>(false);
    const [tags, setTags] = useState<Tag[]>([]);

    async function maximize() {
        if (await window?.isMaximized()) {
            await window?.unmaximize();
        } else {
            await window?.maximize();
        }
    }

    useEffect(() => {
        async function setup() {
            const is_vm = await get<boolean>("vmd_verdict");
    
            if (is_vm && tags.filter((tag) => tag.id === "vmd_verdict").length == 0) {
                const tag: Tag = { msg: "vm env", desc: "Программа запущена в виртуальной среде.", id: "vmd_verdict", color: "red" };
                setTags((bef) => [...bef, ...[tag]]);
            }
        }

        async function fetchWindow() {
            const _window = await getCurrentWebviewWindow();
            setWindow(_window);
            setMinimizable(await _window.isMinimizable());
            setMaximizable(await _window.isMaximizable());
            setClosable(await _window.isClosable());
        }

        fetchWindow();
        setup();
        setupListen(async (n) => {
            if (n === "vmd_verdict")
                await setup();
        });
    }, []);

    return (
        <div>
            <header data-tauri-drag-region={true} className="h-[30px] w-full select-none items-center justify-between flex">
                <h3 />

                <Flex>
                    <Flex alignItems="center" justify="center" height={30} paddingRight={1} gap={1}>
                        {
                            tags.map((tag) =>
                            (
                                <Tooltip contentProps={{ "padding": "10px", "width": "200px", "fontFamily": "inter", "background": "#18181B", "color": "white", "borderRadius": "20px", "shadow": "none", "borderWidth": "1px" }} openDelay={100} content={tag.desc}>
                                    <Badge fontFamily="inter" colorPalette={tag.color} borderRadius="20px">
                                        {tag.msg}
                                    </Badge>
                                </Tooltip>
                            ))
                        }
                    </Flex>

                    <Button backgroundColor="bg" disabled={!minimizable} onClick={async () => await window?.minimize()} size="sm" width={30} height={30} variant="subtle" className="hover:opacity-80 items-center justify-center" borderRadius="0">
                        <Icon icon="qlementine-icons:windows-minimize-16"></Icon>
                    </Button>
                    <Button backgroundColor="bg" disabled={!maximizable} onClick={() => maximize()} width={30} height={30} size="sm" variant="subtle" className="hover:opacity-80 items-center justify-center" borderRadius="0">
                        <Icon icon={
                            "qlementine-icons:windows-maximize-16"
                        }></Icon>
                    </Button>
                    <Button backgroundColor="bg" disabled={!closable} onClick={async () => await window?.close()} size="sm" width={30} height={30} variant="subtle" className="hover:opacity-80 items-center justify-center" borderRadius="0">
                        <Icon icon="qlementine-icons:windows-close-16"></Icon>
                    </Button>
                </Flex>
            </header>

            {children}
        </div>
    )
}

export default TitleLayout;