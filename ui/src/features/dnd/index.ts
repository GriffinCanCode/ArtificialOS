/**
 * Drag & Drop Module
 * Exports for drag and drop functionality
 */

// Types
export type {
  SortableItem as SortableItemType,
  SortConfig,
  SortResult,
  FileDropConfig,
  DroppedFile,
  DropResult,
  DropzoneState,
  DockItem,
  DockConfig,
  SortHandler,
  DropHandler,
  FileValidator,
} from "./core/types";

// Utilities
export {
  validateFileType,
  validateFileSize,
  validateFile,
  processFiles,
  arrayMove,
  arrayInsert,
  arrayRemove,
  createFilePreview,
  revokeFilePreview,
  formatFileSize,
  getFileExtension,
  isImageFile,
} from "./core/utils";

// Hooks
export { useSortable } from "./hooks/useSortable";
export { useDropzone } from "./hooks/useDropzone";

// Store
export { useStore as useDockStore, useActions as useDockActions, useDockItems } from "./store/store";

// Components
export { Sortable } from "./components/Sortable";
export { SortableItem } from "./components/SortableItem";
export { Dropzone } from "./components/Dropzone";
