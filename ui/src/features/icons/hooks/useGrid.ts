/**
 * useGrid Hook
 * Grid calculations and viewport management
 */

import { useEffect, useMemo } from "react";
import { useActions, useViewport } from "../store/store";
import { calculateGridDimensions } from "../core/grid";
import type { GridConfig } from "../core/types";
import { DEFAULT_GRID_CONFIG } from "../core/types";

// ============================================================================
// Grid Hook
// ============================================================================

export interface GridHook {
  rows: number;
  cols: number;
  capacity: number;
  config: GridConfig;
}

/**
 * Grid calculations hook
 * Automatically updates viewport dimensions on window resize
 */
export function useGrid(config: GridConfig = DEFAULT_GRID_CONFIG): GridHook {
  const viewport = useViewport();
  const { updateViewport } = useActions();

  // Calculate grid dimensions from viewport
  const dimensions = useMemo(() => {
    return calculateGridDimensions(viewport.width, viewport.height, config);
  }, [viewport.width, viewport.height, config]);

  // Listen for window resize
  useEffect(() => {
    const handleResize = () => {
      const width = window.innerWidth;
      const height = window.innerHeight;
      const dims = calculateGridDimensions(width, height, config);

      updateViewport(width, height, dims.rows, dims.cols);
    };

    // Initial setup
    handleResize();

    // Listen for resize
    window.addEventListener("resize", handleResize);
    return () => window.removeEventListener("resize", handleResize);
  }, [config, updateViewport]);

  return {
    rows: dimensions.rows,
    cols: dimensions.cols,
    capacity: dimensions.capacity,
    config,
  };
}

