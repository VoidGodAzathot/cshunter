import { useEffect, useState } from "react";
import { getCurrentWebviewWindow, WebviewWindow } from "@tauri-apps/api/webviewWindow";
import { Button } from "@chakra-ui/react";
import { Icon } from '@iconify/react';

function TitleLayout({ children }: { children: JSX.Element }) {
    const [window, setWindow] = useState<WebviewWindow>();
    const [maximized, setMaximized] = useState<boolean | undefined>();

    async function maximize() {
        if (maximized) {
            await window?.unmaximize();
        } else {
            await window?.maximize();
        }

        setMaximized(!maximized);
    }

    useEffect(() => {
        async function fetchWindow() {
            const _window = await getCurrentWebviewWindow();
            _window.onResized(async (_) => setMaximized(await window?.isMaximized()));
            setMaximized(await _window.isMaximized());
            setWindow(_window);
        }

        fetchWindow();
    }, []);

    return (
        <div>
            <header data-tauri-drag-region={true} className=" h-[30px] w-full select-none items-center justify-between bg-[#18181B] flex">
                <h3 />

                <div>
                    <Button onClick={async () => await window?.minimize()} size="sm" width={30} height={30} variant="subtle" className="items-center justify-center" borderRadius="0">
                        <Icon icon="qlementine-icons:windows-minimize-16"></Icon>
                    </Button>
                    <Button onClick={() => maximize()} width={30} height={30} size="sm" variant="subtle" className="items-center justify-center" borderRadius="0">
                        <Icon icon={
                            maximized ? "qlementine-icons:windows-unmaximize-16" : "qlementine-icons:windows-maximize-16"
                        }></Icon>
                    </Button>
                    <Button onClick={async () => await window?.close()} size="sm" width={30} height={30} colorPalette="red" variant="subtle" className="items-center justify-center" borderRadius="0">
                        <Icon icon="qlementine-icons:windows-close-16"></Icon>
                    </Button>
                </div>
            </header>

            {children}
        </div>
    )
}

export default TitleLayout;