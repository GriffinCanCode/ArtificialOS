/**
 * Drag & Drop Utilities
 * Pure functions for drag and drop operations
 */

import type { DroppedFile, FileDropConfig, FileValidator } from "./types";

// ============================================================================
// File Validation
// ============================================================================

/**
 * Validates file type against allowed types
 */
export function validateFileType(file: File, accept?: string[]): boolean {
  if (!accept || accept.length === 0) return true;

  const fileType = file.type;
  const fileName = file.name;

  return accept.some((type) => {
    if (type.startsWith(".")) {
      return fileName.endsWith(type);
    }
    if (type.endsWith("/*")) {
      return fileType.startsWith(type.replace("/*", ""));
    }
    return fileType === type;
  });
}

/**
 * Validates file size against maximum size
 */
export function validateFileSize(file: File, maxSize?: number): boolean {
  if (!maxSize) return true;
  return file.size <= maxSize;
}

/**
 * Validates a file against all config rules
 */
export function validateFile(
  file: File,
  config: FileDropConfig,
  customValidator?: FileValidator
): string | null {
  if (!validateFileType(file, config.accept)) {
    return `File type not allowed: ${file.type}`;
  }

  if (!validateFileSize(file, config.maxSize)) {
    const maxMB = config.maxSize ? (config.maxSize / 1024 / 1024).toFixed(2) : "unknown";
    return `File too large. Maximum: ${maxMB}MB`;
  }

  if (customValidator) {
    return customValidator(file);
  }

  return null;
}

/**
 * Processes dropped files and validates them
 */
export function processFiles(
  files: FileList | File[],
  config: FileDropConfig,
  customValidator?: FileValidator
): { valid: DroppedFile[]; rejected: DroppedFile[] } {
  const fileArray = Array.from(files);
  const valid: DroppedFile[] = [];
  const rejected: DroppedFile[] = [];

  // Check max files limit
  const maxFiles = config.maxFiles ?? Infinity;
  const filesToProcess = fileArray.slice(0, maxFiles);

  for (const file of filesToProcess) {
    const error = validateFile(file, config, customValidator);

    if (error) {
      rejected.push({ file, error });
    } else {
      valid.push({ file });
    }
  }

  return { valid, rejected };
}

// ============================================================================
// Array Manipulation
// ============================================================================

/**
 * Moves an array item from one position to another
 */
export function arrayMove<T>(array: T[], from: number, to: number): T[] {
  const newArray = [...array];
  const [item] = newArray.splice(from, 1);
  newArray.splice(to, 0, item);
  return newArray;
}

/**
 * Inserts an item at a specific position
 */
export function arrayInsert<T>(array: T[], index: number, item: T): T[] {
  const newArray = [...array];
  newArray.splice(index, 0, item);
  return newArray;
}

/**
 * Removes an item at a specific position
 */
export function arrayRemove<T>(array: T[], index: number): T[] {
  const newArray = [...array];
  newArray.splice(index, 1);
  return newArray;
}

// ============================================================================
// Preview Generation
// ============================================================================

/**
 * Creates a preview URL for an image file
 */
export function createFilePreview(file: File): Promise<string> {
  return new Promise((resolve, reject) => {
    if (!file.type.startsWith("image/")) {
      reject(new Error("Not an image file"));
      return;
    }

    const reader = new FileReader();
    reader.onload = () => resolve(reader.result as string);
    reader.onerror = reject;
    reader.readAsDataURL(file);
  });
}

/**
 * Revokes a preview URL to free memory
 */
export function revokeFilePreview(url: string): void {
  if (url.startsWith("blob:")) {
    URL.revokeObjectURL(url);
  }
}

// ============================================================================
// Format Helpers
// ============================================================================

/**
 * Formats file size in human-readable format
 */
export function formatFileSize(bytes: number): string {
  if (bytes === 0) return "0 Bytes";

  const k = 1024;
  const sizes = ["Bytes", "KB", "MB", "GB"];
  const i = Math.floor(Math.log(bytes) / Math.log(k));

  return `${parseFloat((bytes / Math.pow(k, i)).toFixed(2))} ${sizes[i]}`;
}

/**
 * Gets file extension from filename
 */
export function getFileExtension(filename: string): string {
  const parts = filename.split(".");
  return parts.length > 1 ? parts[parts.length - 1].toLowerCase() : "";
}

/**
 * Checks if file is an image
 */
export function isImageFile(file: File): boolean {
  return file.type.startsWith("image/");
}
