/**
 * Drag & Drop Core Types
 * Type definitions for drag and drop functionality
 */

import type { UniqueIdentifier } from "@dnd-kit/core";

// ============================================================================
// Sortable Types
// ============================================================================

export interface SortableItem {
  id: UniqueIdentifier;
}

export interface SortConfig {
  strategy?: "vertical" | "horizontal" | "grid";
  disabled?: boolean;
  animationDuration?: number;
}

export interface SortResult {
  activeId: UniqueIdentifier;
  overId: UniqueIdentifier;
  oldIndex: number;
  newIndex: number;
}

// ============================================================================
// Dropzone Types
// ============================================================================

export interface FileDropConfig {
  accept?: string[];
  maxSize?: number;
  maxFiles?: number;
  multiple?: boolean;
  disabled?: boolean;
}

export interface DroppedFile {
  file: File;
  preview?: string;
  error?: string;
}

export interface DropResult {
  files: DroppedFile[];
  rejectedFiles: DroppedFile[];
}

export interface DropzoneState {
  isDragging: boolean;
  files: DroppedFile[];
  error: string | null;
}

// ============================================================================
// Dock Types
// ============================================================================

export interface DockItem extends SortableItem {
  id: string;
  label: string;
  icon: string;
  action: string;
  order: number;
  pinned?: boolean;
}

export interface DockConfig {
  maxItems?: number;
  allowReorder?: boolean;
  allowRemove?: boolean;
  showLabels?: boolean;
}

// ============================================================================
// Event Handler Types
// ============================================================================

export type SortHandler = (result: SortResult) => void;
export type DropHandler = (result: DropResult) => void;
export type FileValidator = (file: File) => string | null;
