/**
 * Icons Core Types
 * Type definitions for desktop icon management
 */

// ============================================================================
// Icon Types
// ============================================================================

export type IconType = "app" | "file" | "folder" | "shortcut" | "native";

export type IconSize = "small" | "medium" | "large";

export type ArrangeStrategy = "grid" | "name" | "type" | "date" | "size";

// ============================================================================
// Position & Grid
// ============================================================================

export interface GridPosition {
  row: number; // Grid row (0-indexed)
  col: number; // Grid column (0-indexed)
}

export interface PixelPosition {
  x: number; // Pixel x-coordinate
  y: number; // Pixel y-coordinate
}

export interface GridConfig {
  cellWidth: number; // Width of each grid cell
  cellHeight: number; // Height of each grid cell
  padding: number; // Padding between icons
  marginTop: number; // Top margin (for menu bar)
  marginLeft: number; // Left margin
  marginRight: number; // Right margin
  marginBottom: number; // Bottom margin (for dock)
}

// ============================================================================
// Icon Interface
// ============================================================================

export interface Icon {
  id: string; // Unique identifier
  type: IconType; // Icon type
  label: string; // Display label
  icon: string; // Icon emoji or image URL
  position: GridPosition; // Grid position
  metadata: IconMetadata; // Type-specific metadata
  isSelected: boolean; // Selection state
  isDragging: boolean; // Dragging state
  isHovered: boolean; // Hover state
  zIndex: number; // Z-index for layering
  createdAt: number; // Creation timestamp
  updatedAt: number; // Last update timestamp
}

// ============================================================================
// Icon Metadata (Discriminated Union)
// ============================================================================

export type IconMetadata =
  | AppMetadata
  | FileMetadata
  | FolderMetadata
  | ShortcutMetadata
  | NativeMetadata;

export interface AppMetadata {
  type: "app";
  appId: string;
  category?: string;
  launchable: boolean;
}

export interface FileMetadata {
  type: "file";
  path: string;
  extension: string;
  size: number;
  mimeType?: string;
}

export interface FolderMetadata {
  type: "folder";
  path: string;
  itemCount: number;
}

export interface ShortcutMetadata {
  type: "shortcut";
  target: string; // App ID or file path
  targetType: "app" | "file" | "folder" | "url";
}

export interface NativeMetadata {
  type: "native";
  packageId: string;
  bundlePath: string;
}

// ============================================================================
// Selection
// ============================================================================

export interface SelectionBox {
  start: PixelPosition;
  end: PixelPosition;
  isActive: boolean;
}

export interface SelectionState {
  selectedIds: Set<string>;
  lastSelectedId: string | null;
  box: SelectionBox | null;
}

// ============================================================================
// Drag State
// ============================================================================

export interface DragState {
  isDragging: boolean;
  draggedIds: Set<string>; // Multiple icons can be dragged
  startPosition: PixelPosition | null;
  currentPosition: PixelPosition | null;
  offset: PixelPosition | null;
}

// ============================================================================
// Bounds & Collision
// ============================================================================

export interface Bounds {
  x: number;
  y: number;
  width: number;
  height: number;
}

export interface CollisionMap {
  occupied: Map<string, string>; // "row:col" -> icon ID
  available: GridPosition[]; // List of available positions
}

// ============================================================================
// Context Menu
// ============================================================================

export interface ContextAction {
  id: string;
  label: string;
  icon?: string;
  action: (iconId: string) => void;
  disabled?: boolean;
  divider?: boolean;
}

// ============================================================================
// Constants
// ============================================================================

export const DEFAULT_GRID_CONFIG: GridConfig = {
  cellWidth: 80,
  cellHeight: 90,
  padding: 16, // Increased for better visual breathing room
  marginTop: 64, // Menu bar height
  marginLeft: 24, // Increased for better edge spacing
  marginRight: 24, // Increased for better edge spacing
  marginBottom: 96, // Dock height
};

export const ICON_SIZES: Record<IconSize, { width: number; height: number }> = {
  small: { width: 48, height: 48 },
  medium: { width: 64, height: 64 },
  large: { width: 80, height: 80 },
};

export const DRAG_THRESHOLD = 5; // Pixels before drag starts
export const DOUBLE_CLICK_THRESHOLD = 300; // Milliseconds

// ============================================================================
// Helper Type Guards
// ============================================================================

export function isAppIcon(icon: Icon): icon is Icon & { metadata: AppMetadata } {
  return icon.metadata.type === "app";
}

export function isFileIcon(icon: Icon): icon is Icon & { metadata: FileMetadata } {
  return icon.metadata.type === "file";
}

export function isFolderIcon(icon: Icon): icon is Icon & { metadata: FolderMetadata } {
  return icon.metadata.type === "folder";
}

export function isShortcutIcon(icon: Icon): icon is Icon & { metadata: ShortcutMetadata } {
  return icon.metadata.type === "shortcut";
}

export function isNativeIcon(icon: Icon): icon is Icon & { metadata: NativeMetadata } {
  return icon.metadata.type === "native";
}

