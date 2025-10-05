/**
 * Core Input Types
 * Comprehensive type definitions for all input handling
 */

// ============================================================================
// Keyboard Types
// ============================================================================

export type ModifierKey = "Alt" | "Control" | "Meta" | "Shift";

export interface KeyboardEventData {
  key: string;
  code: string;
  modifiers: ModifierKey[];
  target: EventTarget | null;
}

export interface ShortcutConfig {
  key: string;
  modifiers?: ModifierKey[];
  handler: (event: KeyboardEvent) => void;
  description?: string;
  allowInInput?: boolean;
}

export interface KeyboardOptions {
  allowedKeys?: string[];
  includeContentEditable?: boolean;
  preventDefault?: boolean;
  stopPropagation?: boolean;
}

// ============================================================================
// Mouse Types
// ============================================================================

export type MouseButton = "left" | "right" | "middle";

export interface MouseEventData {
  x: number;
  y: number;
  clientX: number;
  clientY: number;
  button: MouseButton;
  target: EventTarget | null;
}

export interface DragState {
  isDragging: boolean;
  startX: number;
  startY: number;
  currentX: number;
  currentY: number;
  deltaX: number;
  deltaY: number;
}

export interface MouseOptions {
  button?: MouseButton;
  preventDefault?: boolean;
  threshold?: number;
}

// ============================================================================
// Touch Types
// ============================================================================

export interface TouchEventData {
  touches: TouchPoint[];
  changedTouches: TouchPoint[];
  target: EventTarget | null;
}

export interface TouchPoint {
  id: number;
  x: number;
  y: number;
  clientX: number;
  clientY: number;
}

export interface TouchOptions {
  multiTouch?: boolean;
  preventDefault?: boolean;
  threshold?: number;
}

// ============================================================================
// Gesture Types
// ============================================================================

export interface GestureState {
  offset: [number, number];
  velocity: [number, number];
  direction: [number, number];
  distance: number;
  down: boolean;
}

export interface PinchState {
  scale: number;
  origin: [number, number];
}

export interface SwipeDirection {
  x: "left" | "right" | "none";
  y: "up" | "down" | "none";
}

// ============================================================================
// Form Types
// ============================================================================

export interface FormField<T = any> {
  name: string;
  value: T;
  error?: string;
  touched: boolean;
  dirty: boolean;
}

export interface ValidationResult {
  isValid: boolean;
  errors: Record<string, string>;
}

// ============================================================================
// Event Handler Types
// ============================================================================

export type KeyboardHandler = (event: KeyboardEvent) => void;
export type MouseHandler = (event: MouseEvent) => void;
export type TouchHandler = (event: TouchEvent) => void;
export type FocusHandler = (event: FocusEvent) => void;

export interface EventHandlers {
  onKeyDown?: KeyboardHandler;
  onKeyUp?: KeyboardHandler;
  onClick?: MouseHandler;
  onMouseDown?: MouseHandler;
  onMouseUp?: MouseHandler;
  onMouseMove?: MouseHandler;
  onTouchStart?: TouchHandler;
  onTouchMove?: TouchHandler;
  onTouchEnd?: TouchHandler;
  onFocus?: FocusHandler;
  onBlur?: FocusHandler;
}
