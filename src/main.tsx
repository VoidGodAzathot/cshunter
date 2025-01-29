import React from "react";
import ReactDOM from "react-dom/client";
import { Provider } from "./components/ui/provider";
import { defaultSystem } from "@chakra-ui/react";
import "./index.css"
import PreloadMainPage from "./pages/preload/preload-main-page";

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <React.StrictMode>
    <Provider {...defaultSystem}>
      <PreloadMainPage />
    </Provider>
  </React.StrictMode>,
);
