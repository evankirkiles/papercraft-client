import React from "react";
import ReactDOM from "react-dom/client";
import { Engine } from "@/components/Engine/Engine";

import "./main.scss";

ReactDOM.createRoot(document.getElementById("root")!).render(
  /* <React.StrictMode> */
  <Engine />
  /* </React.StrictMode> */
);
