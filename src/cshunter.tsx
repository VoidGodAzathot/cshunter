import React from "react";
import ReactDOM from "react-dom/client";
import { Provider } from "./components/ui/provider";
import { defaultSystem } from "@chakra-ui/react";
import "./index.css"
import TitleLayout from "./layouts/title-layout";

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <React.StrictMode>
    <Provider {...defaultSystem}>
      <TitleLayout>
        <h3>cshunter is coming</h3>
      </TitleLayout>
    </Provider>
  </React.StrictMode>,
);