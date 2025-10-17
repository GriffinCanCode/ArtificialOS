/**
 * Dropzone Hook
 * React hook for file drop functionality
 */

import { useState, useCallback, useRef } from "react";
import type {
  FileDropConfig,
  DroppedFile,
  DropHandler,
  FileValidator,
} from "../core/types";
import { processFiles, createFilePreview } from "../core/utils";

interface UseDropzoneConfig extends FileDropConfig {
  onDrop?: DropHandler;
  validator?: FileValidator;
  generatePreviews?: boolean;
}

interface UseDropzoneReturn {
  isDragging: boolean;
  files: DroppedFile[];
  error: string | null;
  getRootProps: () => {
    onDrop: (e: React.DragEvent) => void;
    onDragOver: (e: React.DragEvent) => void;
    onDragEnter: (e: React.DragEvent) => void;
    onDragLeave: (e: React.DragEvent) => void;
    onClick: () => void;
  };
  getInputProps: () => {
    ref: React.RefObject<HTMLInputElement>;
    type: string;
    multiple: boolean;
    accept: string | undefined;
    onChange: (e: React.ChangeEvent<HTMLInputElement>) => void;
    style: { display: string };
  };
  clearFiles: () => void;
  removeFile: (index: number) => void;
}

/**
 * Hook for file drop zone with drag and drop support
 */
export function useDropzone({
  onDrop,
  validator,
  generatePreviews = false,
  accept,
  maxSize,
  maxFiles,
  multiple = true,
  disabled = false,
}: UseDropzoneConfig = {}): UseDropzoneReturn {
  const [isDragging, setIsDragging] = useState(false);
  const [files, setFiles] = useState<DroppedFile[]>([]);
  const [error, setError] = useState<string | null>(null);
  const inputRef = useRef<HTMLInputElement>(null);
  const dragCounter = useRef(0);

  const config: FileDropConfig = { accept, maxSize, maxFiles, multiple, disabled };

  const handleFiles = useCallback(
    async (fileList: FileList | File[]) => {
      if (disabled) return;

      const { valid, rejected } = processFiles(fileList, config, validator);

      // Generate previews for image files if requested
      if (generatePreviews) {
        const withPreviews = await Promise.all(
          valid.map(async (item) => {
            if (item.file.type.startsWith("image/")) {
              try {
                const preview = await createFilePreview(item.file);
                return { ...item, preview };
              } catch {
                return item;
              }
            }
            return item;
          })
        );

        setFiles(withPreviews);
      } else {
        setFiles(valid);
      }

      if (rejected.length > 0) {
        setError(rejected[0].error || "Some files were rejected");
      } else {
        setError(null);
      }

      if (onDrop) {
        onDrop({ files: valid, rejectedFiles: rejected });
      }
    },
    [config, validator, generatePreviews, onDrop, disabled]
  );

  const handleDrop = useCallback(
    (e: React.DragEvent) => {
      e.preventDefault();
      e.stopPropagation();

      setIsDragging(false);
      dragCounter.current = 0;

      if (disabled) return;

      const droppedFiles = e.dataTransfer.files;
      if (droppedFiles.length > 0) {
        handleFiles(droppedFiles);
      }
    },
    [handleFiles, disabled]
  );

  const handleDragOver = useCallback((e: React.DragEvent) => {
    e.preventDefault();
    e.stopPropagation();
  }, []);

  const handleDragEnter = useCallback((e: React.DragEvent) => {
    e.preventDefault();
    e.stopPropagation();

    dragCounter.current++;
    if (e.dataTransfer.items && e.dataTransfer.items.length > 0) {
      setIsDragging(true);
    }
  }, []);

  const handleDragLeave = useCallback((e: React.DragEvent) => {
    e.preventDefault();
    e.stopPropagation();

    dragCounter.current--;
    if (dragCounter.current === 0) {
      setIsDragging(false);
    }
  }, []);

  const handleClick = useCallback(() => {
    if (!disabled && inputRef.current) {
      inputRef.current.click();
    }
  }, [disabled]);

  const handleInputChange = useCallback(
    (e: React.ChangeEvent<HTMLInputElement>) => {
      if (e.target.files && e.target.files.length > 0) {
        handleFiles(e.target.files);
      }
    },
    [handleFiles]
  );

  const clearFiles = useCallback(() => {
    setFiles([]);
    setError(null);
    if (inputRef.current) {
      inputRef.current.value = "";
    }
  }, []);

  const removeFile = useCallback((index: number) => {
    setFiles((prev) => prev.filter((_, i) => i !== index));
  }, []);

  const getRootProps = useCallback(
    () => ({
      onDrop: handleDrop,
      onDragOver: handleDragOver,
      onDragEnter: handleDragEnter,
      onDragLeave: handleDragLeave,
      onClick: handleClick,
    }),
    [handleDrop, handleDragOver, handleDragEnter, handleDragLeave, handleClick]
  );

  const getInputProps = useCallback(
    () => ({
      ref: inputRef,
      type: "file" as const,
      multiple,
      accept: accept?.join(","),
      onChange: handleInputChange,
      style: { display: "none" },
    }),
    [multiple, accept, handleInputChange]
  );

  return {
    isDragging,
    files,
    error,
    getRootProps,
    getInputProps,
    clearFiles,
    removeFile,
  };
}
