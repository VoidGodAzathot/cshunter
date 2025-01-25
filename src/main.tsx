import React from "react";
import ReactDOM from "react-dom/client";
import { Provider } from "./components/ui/provider";
import { defaultSystem } from "@chakra-ui/react";
import "./main.css"

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <React.StrictMode>
    <Provider {...defaultSystem}>
      <h3>cshunter is coming</h3>
    </Provider>
  </React.StrictMode>,
);
