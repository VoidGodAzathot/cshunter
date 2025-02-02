import { useEffect, useState } from "react";
import { getCurrentWebviewWindow, WebviewWindow } from "@tauri-apps/api/webviewWindow";
import { Button } from "@chakra-ui/react";
import { Icon } from '@iconify/react';

function TitleLayout({ children }: { children: JSX.Element }) {
    const [window, setWindow] = useState<WebviewWindow>();
    const [minimizable, setMinimizable] = useState<boolean>(false);
    const [maximizable, setMaximizable] = useState<boolean>(false);
    const [closable, setClosable] = useState<boolean>(false);

    async function maximize() {
        if (await window?.isMaximized()) {
            await window?.unmaximize();
        } else {
            await window?.maximize();
        }
    }

    useEffect(() => {
        async function fetchWindow() {
            const _window = await getCurrentWebviewWindow();
            setWindow(_window);
            setMinimizable(await _window.isMinimizable());
            setMaximizable(await _window.isMaximizable());
            setClosable(await _window.isClosable());
        }

        fetchWindow();
    }, []);

    return (
        <div>
            <header data-tauri-drag-region={true} className="h-[30px] w-full select-none items-center justify-between flex">
                <h3 />

                <div>
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
                </div>
            </header>

            {children}
        </div>
    )
}

export default TitleLayout;