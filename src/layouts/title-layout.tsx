import { useEffect, useState } from "react";
import {
  getCurrentWebviewWindow,
  WebviewWindow,
} from "@tauri-apps/api/webviewWindow";
import { Badge, Button, Flex, Text } from "@chakra-ui/react";
import { Icon } from "@iconify/react";
import useStorage from "../hooks/storage";
import { Tooltip } from "../components/ui/tooltip";
import {
  MenuContent,
  MenuItem,
  MenuRoot,
  MenuTrigger,
} from "../components/ui/menu";
import { getVersion } from "@tauri-apps/api/app";
import { invoke } from "@tauri-apps/api/core";
import { GITHUB_URL } from "../utils/consts";
import { Tag, tryApplyTag } from "../utils/tags";

function TitleLayout({ children }: { children: JSX.Element }) {
  const [, get, , setupListen] = useStorage();
  const [window, setWindow] = useState<WebviewWindow>();
  const [minimizable, setMinimizable] = useState<boolean>(false);
  const [maximizable, setMaximizable] = useState<boolean>(false);
  const [closable, setClosable] = useState<boolean>(false);
  const [version, setVersion] = useState<string | undefined>(undefined);
  const [tags, setTags] = useState<Tag[]>([]);

  async function maximize() {
    if (window) {
      if (await window.isMaximized()) {
        await window.unmaximize();
      } else {
        await window.maximize();
      }
    }
  }

  useEffect(() => {
    async function setup() {
      const localVersion: string = await getVersion();
      const githubVersion: string = await get<string>("github_version") ?? "undefined";
      const is_vm = await get<boolean>("vmd_verdict");

      if (is_vm) {
        setTags((bef) => tryApplyTag(bef, "vmd_verdict"));
      }

      if (githubVersion != null && githubVersion.length != 0 && localVersion !== githubVersion) {
        setTags((bef) => tryApplyTag(bef, "no_last_version"));
      }

      setVersion(await getVersion());
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
      if (n === "vmd_verdict" || n === "github_version") await setup();
    });
  }, []);

  return (
    <>
      {window ? (
        <div>
          <header
            data-tauri-drag-region={true}
            className="h-[30px] w-full select-none items-center justify-between flex"
          >
            <h3 />

            <Flex>
              <Flex
                alignItems="center"
                justify="center"
                height={30}
                paddingRight={1}
                gap={1}
              >
                {tags.map((tag) => (
                  <Tooltip
                    contentProps={{
                      padding: "10px",
                      width: "200px",
                      fontFamily: "inter",
                      background: "#18181B",
                      color: "white",
                      borderRadius: "20px",
                      shadow: "none",
                      borderWidth: "1px",
                    }}
                    openDelay={100}
                    content={tag.desc}
                  >
                    <Badge
                      fontFamily="inter"
                      colorPalette={tag.color}
                      borderRadius="20px"
                    >
                      {tag.msg}
                    </Badge>
                  </Tooltip>
                ))}
              </Flex>

              <MenuRoot positioning={{ placement: "bottom" }} variant="subtle">
                <MenuTrigger>
                  <Button
                    disabled={window.label != "cshunter"}
                    borderRadius="0px"
                    variant="ghost"
                    width={30}
                    height={30}
                  >
                    <Icon color="gray" icon="material-symbols:menu"></Icon>
                  </Button>
                </MenuTrigger>
                <MenuContent
                  spaceY="5px"
                  shadow="none"
                  backgroundColor="#18181B"
                  borderRadius="20px"
                  borderWidth="1px"
                >
                  <MenuItem
                    onClick={async () =>
                      await invoke("open_url", { url: GITHUB_URL })
                    }
                    fontSize="13px"
                    borderRadius="20px"
                    value="github-source"
                  >
                    <Icon icon="mdi:github"></Icon> Исходный код
                  </MenuItem>
                  <Flex
                    className="select-none"
                    fontSize="12px"
                    color="gray"
                    gap={5}
                    width="full"
                    height="full"
                    alignItems="center"
                    justify="center"
                  >
                    <Text fontSize="12px" color="gray">
                      {version}
                    </Text>
                  </Flex>
                </MenuContent>
              </MenuRoot>

              <Button
                backgroundColor="bg"
                disabled={!minimizable}
                onClick={async () => await window.minimize()}
                size="sm"
                width={30}
                height={30}
                variant="subtle"
                className="hover:opacity-80 items-center justify-center"
                borderRadius="0"
              >
                <Icon icon="qlementine-icons:windows-minimize-16"></Icon>
              </Button>
              <Button
                backgroundColor="bg"
                disabled={!maximizable}
                onClick={() => maximize()}
                width={30}
                height={30}
                size="sm"
                variant="subtle"
                className="hover:opacity-80 items-center justify-center"
                borderRadius="0"
              >
                <Icon icon={"qlementine-icons:windows-maximize-16"}></Icon>
              </Button>
              <Button
                backgroundColor="bg"
                disabled={!closable}
                onClick={async () => await window.close()}
                size="sm"
                width={30}
                height={30}
                variant="subtle"
                className="hover:opacity-80 items-center justify-center"
                borderRadius="0"
              >
                <Icon icon="qlementine-icons:windows-close-16"></Icon>
              </Button>
            </Flex>
          </header>

          {children}
        </div>
      ) : (
        <></>
      )}
    </>
  );
}

export default TitleLayout;
