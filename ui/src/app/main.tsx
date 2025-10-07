/**
 * React Application Entry Point
 */

import React from "react";
import ReactDOM from "react-dom/client";
import * as ReactJSXRuntime from "react/jsx-runtime";
import App from "./App";
import "./styles/global.css";
import { initializeColorSystem } from "../core/utils/color";

// Expose React globally for native apps to use
// This ensures a single React instance shared across host and native apps
(window as any).React = React;
(window as any).ReactDOM = ReactDOM;
(window as any).ReactJSXRuntime = ReactJSXRuntime;

// Initialize color system with OKLCH variables
initializeColorSystem("#667eea", "dark");

ReactDOM.createRoot(document.getElementById("root")!).render(
  <React.StrictMode>
    <App />
  </React.StrictMode>
);
