/**
 * React Application Entry Point
 */

import React from "react";
import ReactDOM from "react-dom/client";
import App from "./App";
import "./styles/global.css";
import { initializeColorSystem } from "../core/utils/color";

// Initialize color system with OKLCH variables
initializeColorSystem("#667eea", "dark");

ReactDOM.createRoot(document.getElementById("root")!).render(
  <React.StrictMode>
    <App />
  </React.StrictMode>
);
