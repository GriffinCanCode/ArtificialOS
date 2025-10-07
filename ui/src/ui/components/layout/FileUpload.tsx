/**
 * FileUpload Component
 * Complete file upload solution with drag-and-drop, progress tracking, and status management
 */

import React, { useState, useCallback, useRef } from "react";
import { randomId } from "../../../core/utils/math";
import { Dropzone } from "../../../features/dnd/components/Dropzone";
import type { FileDropConfig, DropResult, FileValidator } from "../../../features/dnd/core/types";
import { formatFileSize } from "../../../features/dnd/core/utils";
import "./FileUpload.css";

// ============================================================================
// Types
// ============================================================================

export type UploadStatus = "pending" | "uploading" | "completed" | "error" | "cancelled";

export interface UploadFile {
  id: string;
  file: File;
  preview?: string;
  status: UploadStatus;
  progress: number;
  error?: string;
  uploadedUrl?: string;
}

export interface FileUploadConfig extends FileDropConfig {
  /**
   * Function to handle file upload
   * Should return a promise that resolves to the uploaded file URL
   */
  uploadFn: (file: File, onProgress: (progress: number) => void) => Promise<string>;

  /**
   * Auto-upload files immediately after selection
   */
  autoUpload?: boolean;

  /**
   * Allow removing uploaded files
   */
  allowRemove?: boolean;

  /**
   * Show upload button when autoUpload is false
   */
  showUploadButton?: boolean;
}

interface FileUploadProps extends FileUploadConfig {
  validator?: FileValidator;
  onUploadComplete?: (files: UploadFile[]) => void;
  onUploadError?: (file: UploadFile, error: string) => void;
  className?: string;
}

// ============================================================================
// Component
// ============================================================================

export const FileUpload: React.FC<FileUploadProps> = ({
  uploadFn,
  autoUpload = false,
  allowRemove = true,
  showUploadButton = true,
  onUploadComplete,
  onUploadError,
  validator,
  className,
  ...dropzoneConfig
}) => {
  const [uploadFiles, setUploadFiles] = useState<UploadFile[]>([]);
  const abortControllersRef = useRef<Map<string, AbortController>>(new Map());

  // Generate unique ID for each file
  const generateFileId = useCallback(() => {
    return randomId();
  }, []);

  // Upload a single file
  const handleUploadFile = useCallback(
    async (fileId: string) => {
      const uploadFile = uploadFiles.find((f) => f.id === fileId);
      if (!uploadFile) return;

      // Create abort controller for this upload
      const abortController = new AbortController();
      abortControllersRef.current.set(fileId, abortController);

      // Update status to uploading
      setUploadFiles((prev) =>
        prev.map((f) => (f.id === fileId ? { ...f, status: "uploading" as UploadStatus } : f))
      );

      try {
        // Progress callback
        const onProgress = (progress: number) => {
          setUploadFiles((prev) =>
            prev.map((f) => (f.id === fileId ? { ...f, progress: Math.min(progress, 100) } : f))
          );
        };

        // Upload the file
        const uploadedUrl = await uploadFn(uploadFile.file, onProgress);

        // Update to completed
        setUploadFiles((prev) => {
          const updated = prev.map((f) =>
            f.id === fileId
              ? { ...f, status: "completed" as UploadStatus, progress: 100, uploadedUrl }
              : f
          );

          // Notify completion
          const completedFiles = updated.filter((f) => f.status === "completed");
          if (onUploadComplete) {
            onUploadComplete(completedFiles);
          }

          return updated;
        });
      } catch (error) {
        const errorMessage = error instanceof Error ? error.message : "Upload failed";

        setUploadFiles((prev) =>
          prev.map((f) =>
            f.id === fileId ? { ...f, status: "error" as UploadStatus, error: errorMessage } : f
          )
        );

        if (onUploadError) {
          onUploadError(uploadFile, errorMessage);
        }
      } finally {
        abortControllersRef.current.delete(fileId);
      }
    },
    [uploadFiles, uploadFn, onUploadComplete, onUploadError]
  );

  // Handle file drop/selection
  const handleDrop = useCallback(
    async (result: DropResult) => {
      const newFiles: UploadFile[] = result.files.map((droppedFile) => ({
        id: generateFileId(),
        file: droppedFile.file,
        preview: droppedFile.preview,
        status: "pending" as UploadStatus,
        progress: 0,
      }));

      setUploadFiles((prev) => [...prev, ...newFiles]);

      if (autoUpload) {
        // Upload files immediately
        newFiles.forEach((file) => {
          handleUploadFile(file.id);
        });
      }
    },
    [autoUpload, generateFileId, handleUploadFile]
  );

  // Upload all pending files
  const uploadAllFiles = useCallback(() => {
    const pendingFiles = uploadFiles.filter((f) => f.status === "pending");
    pendingFiles.forEach((file) => handleUploadFile(file.id));
  }, [uploadFiles, handleUploadFile]);

  // Cancel upload
  const cancelUpload = useCallback((fileId: string) => {
    const controller = abortControllersRef.current.get(fileId);
    if (controller) {
      controller.abort();
      abortControllersRef.current.delete(fileId);
    }

    setUploadFiles((prev) =>
      prev.map((f) => (f.id === fileId ? { ...f, status: "cancelled" as UploadStatus } : f))
    );
  }, []);

  // Retry upload
  const retryUpload = useCallback(
    (fileId: string) => {
      setUploadFiles((prev) =>
        prev.map((f) =>
          f.id === fileId
            ? { ...f, status: "pending" as UploadStatus, progress: 0, error: undefined }
            : f
        )
      );

      handleUploadFile(fileId);
    },
    [handleUploadFile]
  );

  // Remove file
  const removeFile = useCallback((fileId: string) => {
    setUploadFiles((prev) => prev.filter((f) => f.id !== fileId));
  }, []);

  // Clear all files
  const clearAllFiles = useCallback(() => {
    // Cancel all ongoing uploads
    abortControllersRef.current.forEach((controller) => controller.abort());
    abortControllersRef.current.clear();

    setUploadFiles([]);
  }, []);

  // Get status icon
  const getStatusIcon = (status: UploadStatus): string => {
    switch (status) {
      case "pending":
        return "⏳";
      case "uploading":
        return "⬆️";
      case "completed":
        return "✅";
      case "error":
        return "❌";
      case "cancelled":
        return "⛔";
      default:
        return "📄";
    }
  };

  // Get status label
  const getStatusLabel = (status: UploadStatus): string => {
    switch (status) {
      case "pending":
        return "Pending";
      case "uploading":
        return "Uploading...";
      case "completed":
        return "Completed";
      case "error":
        return "Failed";
      case "cancelled":
        return "Cancelled";
      default:
        return "Unknown";
    }
  };

  const hasPendingFiles = uploadFiles.some((f) => f.status === "pending");
  const hasUploadingFiles = uploadFiles.some((f) => f.status === "uploading");

  return (
    <div className={`file-upload-wrapper ${className || ""}`}>
      <Dropzone {...dropzoneConfig} onDrop={handleDrop} validator={validator} showPreview={false} />

      {uploadFiles.length > 0 && (
        <div className="upload-list-container">
          <div className="upload-list-header">
            <span className="upload-list-title">Files ({uploadFiles.length})</span>
            <div className="upload-list-actions">
              {!autoUpload && showUploadButton && hasPendingFiles && (
                <button
                  onClick={uploadAllFiles}
                  className="upload-action-btn upload-all-btn"
                  disabled={hasUploadingFiles}
                >
                  Upload All
                </button>
              )}
              <button onClick={clearAllFiles} className="upload-action-btn clear-all-btn">
                Clear All
              </button>
            </div>
          </div>

          <div className="upload-list">
            {uploadFiles.map((uploadFile) => (
              <div key={uploadFile.id} className={`upload-item status-${uploadFile.status}`}>
                <div className="upload-item-icon">{getStatusIcon(uploadFile.status)}</div>

                <div className="upload-item-content">
                  <div className="upload-item-info">
                    <div className="upload-item-name">{uploadFile.file.name}</div>
                    <div className="upload-item-meta">
                      <span className="upload-item-size">
                        {formatFileSize(uploadFile.file.size)}
                      </span>
                      <span className="upload-item-status">
                        {getStatusLabel(uploadFile.status)}
                      </span>
                    </div>
                  </div>

                  {uploadFile.status === "uploading" && (
                    <div className="upload-progress">
                      <div className="upload-progress-bar">
                        <div
                          className="upload-progress-fill"
                          style={{ width: `${uploadFile.progress}%` }}
                        />
                      </div>
                      <span className="upload-progress-text">{uploadFile.progress}%</span>
                    </div>
                  )}

                  {uploadFile.error && (
                    <div className="upload-error-message">{uploadFile.error}</div>
                  )}
                </div>

                <div className="upload-item-actions">
                  {uploadFile.status === "pending" && !autoUpload && (
                    <button
                      onClick={() => handleUploadFile(uploadFile.id)}
                      className="upload-btn"
                      title="Upload"
                    >
                      ⬆️
                    </button>
                  )}

                  {uploadFile.status === "uploading" && (
                    <button
                      onClick={() => cancelUpload(uploadFile.id)}
                      className="cancel-btn"
                      title="Cancel"
                    >
                      ⏸️
                    </button>
                  )}

                  {uploadFile.status === "error" && (
                    <button
                      onClick={() => retryUpload(uploadFile.id)}
                      className="retry-btn"
                      title="Retry"
                    >
                      🔄
                    </button>
                  )}

                  {allowRemove && uploadFile.status !== "uploading" && (
                    <button
                      onClick={() => removeFile(uploadFile.id)}
                      className="remove-btn"
                      title="Remove"
                    >
                      ×
                    </button>
                  )}
                </div>
              </div>
            ))}
          </div>
        </div>
      )}
    </div>
  );
};
