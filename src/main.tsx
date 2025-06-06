import ReactDOM from "react-dom/client";
import { Provider } from "./components/ui/provider";
import { defaultSystem } from "@chakra-ui/react";
import "./index.css";
import PreloadMainPage from "./pages/preload/preload-main-page";
import TitleLayout from "./layouts/title-layout";

function disableMenuAndRefresh() {
  if (window.location.hostname !== "tauri.localhost") {
    return;
  }

  document.addEventListener("keydown", function (event) {
    if (
      event.key === "F5" ||
      (event.ctrlKey && event.key === "r") ||
      (event.metaKey && event.key === "r")
    ) {
      event.preventDefault();
    }
  });

  document.addEventListener(
    "contextmenu",
    (e) => {
      e.preventDefault();
      return false;
    },
    { capture: true }
  );

  document.addEventListener(
    "selectstart",
    (e) => {
      e.preventDefault();
      return false;
    },
    { capture: true }
  );
}

disableMenuAndRefresh();

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <Provider {...defaultSystem}>
    <TitleLayout>
      <PreloadMainPage />
    </TitleLayout>
  </Provider>
);
