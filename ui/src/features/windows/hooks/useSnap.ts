/**
 * Snap Hook
 * Handles snap-to-edge positioning with visual feedback
 */

import { useState, useCallback, useEffect } from "react";
import { detectZone, getZoneBounds, isValidZone } from "../core/snap";
import type { Bounds, Zone } from "../core/types";

export interface SnapPreview {
  zone: Zone;
  bounds: Bounds;
}

export function useSnap() {
  const [preview, setPreview] = useState<SnapPreview | null>(null);

  const handleDrag = useCallback((x: number, y: number) => {
    const zone = detectZone({ x, y });

    if (isValidZone(zone)) {
      const bounds = getZoneBounds(zone);
      setPreview({ zone, bounds });
    } else {
      setPreview(null);
    }
  }, []);

  const handleDragEnd = useCallback((x: number, y: number): Bounds | null => {
    const zone = detectZone({ x, y });
    setPreview(null);

    if (isValidZone(zone)) {
      return getZoneBounds(zone);
    }

    return null;
  }, []);

  const clearPreview = useCallback(() => {
    setPreview(null);
  }, []);

  useEffect(() => {
    return () => setPreview(null);
  }, []);

  return {
    preview,
    handleDrag,
    handleDragEnd,
    clearPreview,
  };
}
