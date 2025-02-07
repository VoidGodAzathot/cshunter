import { invoke } from "@tauri-apps/api/core";
import { useEffect, useState } from "react";
import { Card, Container, Flex, ProgressRoot, Text } from "@chakra-ui/react";
import { ProgressBar, ProgressLabel } from "../../components/ui/progress";
import { Task, Tasks } from "../../utils/tasks";
import PreloadBoxes from "../../components/preload/preload-boxes";
import { getVersion } from "@tauri-apps/api/app";
import { listen } from "@tauri-apps/api/event";

function PreloadMainPage() {
  const [isLoaded, setIsLoaded] = useState<boolean>(false);
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

      setIsLoaded(true);
    }

    setup();
    fetchVersion();
    runPreload();
  }, []);

  useEffect(() => {
    async function runCSHunter() {
      await invoke("run_main_window_and_close_preload");
    }

    if (isLoaded) {
      runCSHunter();
    }
  }, [isLoaded]);

  return (
    <Container
      width="100vw"
      height="calc(100vh - 30px)"
      padding={5}
      paddingTop={0}
      className="font-inter select-none flex justify-center items-center"
    >
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
            <PreloadBoxes />

            <div>
              <Text fontWeight="normal" fontSize={18}>
                cshunter
              </Text>

              <Text color="gray" fontWeight="normal" fontSize={14}>
                {version ? version : "..."}
              </Text>
            </div>
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
    </Container>
  );
}

export default PreloadMainPage;
