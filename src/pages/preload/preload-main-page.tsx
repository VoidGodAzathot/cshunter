import { invoke } from "@tauri-apps/api/core";
import { useEffect, useState } from "react";
import {
  Button,
  Card,
  Container,
  Flex,
  ProgressRoot,
  Text,
} from "@chakra-ui/react";
import { ProgressBar, ProgressLabel } from "../../components/ui/progress";
import { Task, Tasks } from "../../utils/tasks";
import PreloadBoxes from "../../components/preload/preload-boxes";
import { getVersion } from "@tauri-apps/api/app";
import { listen } from "@tauri-apps/api/event";
import useStorage from "../../hooks/storage";
import { Icon } from "@iconify/react/dist/iconify.js";
import { open } from "@tauri-apps/plugin-dialog";
import { toaster } from "../../components/ui/toaster";

function PreloadMainPage() {
  const [set, get, ,] = useStorage();
  const [isLoaded, setIsLoaded] = useState<boolean>(false);
  const [loading, setLoading] = useState<boolean>(false);
  const [completedTasks, setCompletedTasks] = useState<number>(0);
  const [currentTask, setCurrentTask] = useState<Task | undefined>(undefined);
  const [currentTaskStatus, setCurrentTaskStatus] = useState<string>("");
  const [version, setVersion] = useState<string | undefined>(undefined);

  useEffect(() => {
    async function setup() {
      const unlisten = await listen<string>("task_status_update", (e) => {
        setCurrentTaskStatus(`(${e.payload})`);
      });

      return () => unlisten();
    }

    async function fetchVersion() {
      setVersion(await getVersion());
    }

    async function runPreload() {
      if (await get<boolean>("loading")) {
        return;
      }
      await set<boolean>("loading", true);
      for (const [i, task] of Tasks.entries()) {
        setCurrentTask(task);
        try {
          await task.worker();
        } catch (e) {
          console.log(`Error while execution task: ${task.name}`);
        } finally {
          setCompletedTasks(i + 1);
          setCurrentTaskStatus("");
        }
      }
      await set<boolean>("loading", false);
      setIsLoaded(true);
    }

    if (loading) {
      setup();
      fetchVersion();
      runPreload();
    }
  }, [loading]);

  async function runCSHunter() {
    await invoke("run_main_window_and_close_preload");
  }

  useEffect(() => {
    if (isLoaded) {
      runCSHunter();
    }
  }, [isLoaded]);

  return (
    <>
      <Container
        width="100vw"
        height="calc(100vh - 30px)"
        padding={5}
        paddingTop={0}
        className="font-inter select-none flex justify-center items-center"
      >
        {loading ? (
          <>
            <Card.Root
              borderRadius={20}
              borderWidth="1px"
              background="#18181B"
              variant="subtle"
              height="full"
              width="full"
            >
              <Card.Body>
                <Card.Title
                  spaceX={5}
                  paddingBottom={5}
                  className="items-center flex"
                >
                  <Flex
                    width="full"
                    alignItems="center"
                    justify="space-between"
                  >
                    <Flex alignItems="center" gap="20px">
                      <PreloadBoxes />

                      <div>
                        <Text fontWeight="normal" fontSize={18}>
                          cshunter
                        </Text>

                        <Text color="gray" fontWeight="normal" fontSize={14}>
                          {version ? version : "..."}
                        </Text>
                      </div>
                    </Flex>
                  </Flex>
                </Card.Title>
              </Card.Body>
              <Card.Footer>
                <ProgressRoot
                  size="lg"
                  striped
                  animated
                  spaceY={1}
                  className="flex-rows"
                  value={completedTasks}
                  max={Tasks.length}
                  variant="subtle"
                  w="full"
                  maxW="full"
                >
                  <ProgressLabel>
                    <Flex direction="column">
                      <Text color={"white"} fontSize={12}>
                        {currentTask ? currentTask.name : "Ожидание"}
                      </Text>

                      <Text
                        minWidth="min-content"
                        whiteSpace="normal"
                        wordBreak="break-word"
                        color={"gray"}
                        fontSize={12}
                      >
                        {currentTaskStatus.length > 0 ? currentTaskStatus : ""}
                      </Text>
                    </Flex>
                  </ProgressLabel>

                  <ProgressBar borderRadius={10} />
                </ProgressRoot>
              </Card.Footer>
            </Card.Root>
          </>
        ) : (
          <>
            <Flex
              paddingTop="50px"
              height="full"
              justify="space-between"
              width="full"
              alignItems="center"
              direction="column"
              gap="20px"
            >
              <Flex
                width="full"
                alignItems="center"
                direction="column"
                gap="20px"
              >
                <Button
                  onClick={() => {
                    setLoading(true);
                  }}
                  variant="subtle"
                  marginTop="auto"
                  borderRadius={50}
                  height="50px"
                >
                  <Icon icon="qlementine-icons:run-16"></Icon>
                  Запустить
                </Button>

                <Button
                  onClick={async () => {
                    const file = await open({
                      multiple: false,
                      filters: [{ name: "", extensions: ["gz"] }],
                      directory: false,
                    });
                    if (file) {
                      await invoke("import_all_data", { file: file })
                        .then(async () => {
                          await runCSHunter();
                        })
                        .catch((e) => {
                          toaster.create({
                            title: "Ошибка загрузки",
                            description: e,
                            type: "error",
                          });
                        });
                    }
                  }}
                  variant="subtle"
                  marginTop="auto"
                  borderRadius={50}
                  height="50px"
                >
                  <Icon icon="material-symbols:upload-rounded"></Icon>
                  Импортировать
                </Button>
              </Flex>

              <Text
                minWidth="min-content"
                whiteSpace="normal"
                wordBreak="break-word"
                color={"gray"}
                fontSize={12}
              >
                non commercial & open source
              </Text>
            </Flex>
          </>
        )}
      </Container>
    </>
  );
}

export default PreloadMainPage;
