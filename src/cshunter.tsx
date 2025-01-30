import React from "react";
import ReactDOM from "react-dom/client";
import { Provider } from "./components/ui/provider";
import { defaultSystem } from "@chakra-ui/react";
import "./index.css"
import TitleLayout from "./layouts/title-layout";
import CSHunterMainPage from "./pages/cshunter/cshunter-main-page";

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <React.StrictMode>
    <Provider {...defaultSystem}>
      <TitleLayout>
        <CSHunterMainPage />
      </TitleLayout>
    </Provider>
  </React.StrictMode>,
);