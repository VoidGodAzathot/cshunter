import React from "react";
import ReactDOM from "react-dom/client";
import { Provider } from "./components/ui/provider";
import { defaultSystem } from "@chakra-ui/react";
import "./index.css"
import TitleLayout from "./layouts/title-layout";
import CSHunterWrapperPages from "./pages/cshunter/cshunter-wrapper-pages";

function disableMenuAndRefresh() {
  if (window.location.hostname !== 'tauri.localhost') {
    return
  }

  document.addEventListener('keydown', function (event) {
    if (
      event.key === 'F5' ||
      (event.ctrlKey && event.key === 'r') ||
      (event.metaKey && event.key === 'r')
    ) {
      event.preventDefault();
    }
  });

  document.addEventListener('contextmenu', e => {
    e.preventDefault();
    return false;
  }, { capture: true })

  document.addEventListener('selectstart', e => {
    e.preventDefault();
    return false;
  }, { capture: true })
}

disableMenuAndRefresh();

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <React.StrictMode>
    <Provider {...defaultSystem}>
      <TitleLayout>
        <CSHunterWrapperPages />
      </TitleLayout>
    </Provider>
  </React.StrictMode>,
);