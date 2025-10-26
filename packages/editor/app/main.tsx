import React from "react";
import ReactDOM from "react-dom/client";
import { Engine } from "@/index";

import "./main.scss";

const root = document.getElementById("root");
if (root) {
  ReactDOM.createRoot(root).render(
    <React.StrictMode>
      <Engine />
    </React.StrictMode>
  );
}
