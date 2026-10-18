import React from "react";
import ReactDOM from "react-dom/client";
import "./index.css";
import App from "./App";
import { TokenSheet } from "./styles/TokenSheet";

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <React.StrictMode>
    {new URLSearchParams(location.search).has("tokens") ? <TokenSheet /> : <App />}
  </React.StrictMode>,
);
