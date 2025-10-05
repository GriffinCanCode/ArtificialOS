/**
 * Snap Preview Component
 * Shows visual feedback when dragging windows near edges
 */

import React from "react";
import type { SnapPreview as SnapPreviewType } from "../../../features/windows";
import "./SnapPreview.css";

interface SnapPreviewProps {
  preview: SnapPreviewType | null;
}

export const SnapPreview: React.FC<SnapPreviewProps> = ({ preview }) => {
  if (!preview) return null;

  const { bounds, zone } = preview;

  return (
    <div
      className="snap-preview"
      style={{
        left: `${bounds.position.x}px`,
        top: `${bounds.position.y}px`,
        width: `${bounds.size.width}px`,
        height: `${bounds.size.height}px`,
      }}
      data-zone={zone}
    />
  );
};
