import "./main.scss";

import React from "react";
import ReactDOM from "react-dom/client";

import { Engine } from "@/index";

const root = document.getElementById("root");
if (root) {
  ReactDOM.createRoot(root).render(
    <React.StrictMode>
      <Engine />
    </React.StrictMode>
  );
}
