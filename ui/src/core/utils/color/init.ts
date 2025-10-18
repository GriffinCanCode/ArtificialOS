/**
 * Color System Initialization
 * Apply CSS variables and setup theme system at app startup
 */

import { applyThemeVariables, applyVisualizationVariables } from "./cssVariables";

/**
 * Initialize the color system with default theme
 * Call this once at app startup
 */
export function initializeColorSystem(
  primaryColor: string = "#667eea",
  mode: "light" | "dark" = "dark"
): void {
  // Apply main theme variables
  applyThemeVariables(primaryColor, mode);

  // Apply visualization/chart colors
  applyVisualizationVariables();

  // Add class to html element for theme awareness
  document.documentElement.classList.add(`theme-${mode}`);
  document.documentElement.setAttribute("data-theme", mode);

  // Color system initialization logging moved to startup logging
}

/**
 * Switch theme mode at runtime
 */
export function switchThemeMode(mode: "light" | "dark", primaryColor?: string): void {
  const currentPrimary =
    primaryColor ||
    getComputedStyle(document.documentElement).getPropertyValue("--color-primary-500").trim() ||
    "#667eea";

  // Remove old theme class
  document.documentElement.classList.remove("theme-light", "theme-dark");

  // Apply new theme
  applyThemeVariables(currentPrimary, mode);
  applyVisualizationVariables();

  // Add new theme class
  document.documentElement.classList.add(`theme-${mode}`);
  document.documentElement.setAttribute("data-theme", mode);
}

/**
 * Update primary color while keeping current mode
 */
export function updatePrimaryColor(primaryColor: string): void {
  const currentMode =
    (document.documentElement.getAttribute("data-theme") as "light" | "dark") || "dark";
  applyThemeVariables(primaryColor, currentMode);
}
