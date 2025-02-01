import React from "react";
import ReactDOM from "react-dom/client";
import { Provider } from "./components/ui/provider";
import { defaultSystem } from "@chakra-ui/react";
import "./index.css"
import TitleLayout from "./layouts/title-layout";
import CSHunterWrapperPages from "./pages/cshunter/cshunter-wrapper-pages";

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <React.StrictMode>
    <Provider {...defaultSystem}>
      <TitleLayout>
        <CSHunterWrapperPages />
      </TitleLayout>
    </Provider>
  </React.StrictMode>,
);