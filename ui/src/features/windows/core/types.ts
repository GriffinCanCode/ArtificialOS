/**
 * Window Core Types
 * Comprehensive type definitions for window management
 */

import { Blueprint } from "../../../core/store/appStore";

// ============================================================================
// Basic Types
// ============================================================================

export interface Position {
  x: number;
  y: number;
}

export interface Size {
  width: number;
  height: number;
}

export interface Bounds {
  position: Position;
  size: Size;
}

// ============================================================================
// Window State
// ============================================================================

export enum State {
  NORMAL = "normal",
  MAXIMIZED = "maximized",
  MINIMIZED = "minimized",
  FULLSCREEN = "fullscreen",
}

export enum Zone {
  NONE = "none",
  LEFT = "left",
  RIGHT = "right",
  TOP = "top",
  BOTTOM = "bottom",
  TOP_LEFT = "top-left",
  TOP_RIGHT = "top-right",
  BOTTOM_LEFT = "bottom-left",
  BOTTOM_RIGHT = "bottom-right",
}

// ============================================================================
// Metadata
// ============================================================================

export interface Metadata {
  lastNormalBounds?: Bounds;
  snapZone?: Zone;
  isAnimating: boolean;
  parentWindowId?: string;
  childWindowIds: string[];
  // Native app metadata (optional, extends base metadata)
  appType?: string;
  packageId?: string;
  bundlePath?: string;
  services?: string[];
  permissions?: string[];
  pid?: number;
}

export interface Constraints {
  minWidth: number;
  minHeight: number;
  maxWidth?: number;
  maxHeight?: number;
}

// ============================================================================
// Window Interface
// ============================================================================

export interface Window {
  id: string;
  appId: string;
  title: string;
  icon?: string;
  uiSpec: Blueprint;
  position: Position;
  size: Size;
  isMinimized: boolean;
  isFocused: boolean;
  zIndex: number;
  state: State;
  metadata: Metadata;
}

// ============================================================================
// Constants
// ============================================================================

export const SNAP_THRESHOLD = 20;
export const CASCADE_OFFSET = 30;
export const ANIMATION_DURATION = 200;
export const MIN_WINDOW_WIDTH = 400;
export const MIN_WINDOW_HEIGHT = 300;
export const DEFAULT_WINDOW_WIDTH = 800;
export const DEFAULT_WINDOW_HEIGHT = 600;
export const MENUBAR_HEIGHT = 40;
export const TASKBAR_HEIGHT = 60;

export const DEFAULT_CONSTRAINTS: Constraints = {
  minWidth: MIN_WINDOW_WIDTH,
  minHeight: MIN_WINDOW_HEIGHT,
};
