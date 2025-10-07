/**
 * Dropzone Component
 * File drop zone with drag and drop support
 */

import React from "react";
import { useDropzone } from "../hooks/useDropzone";
import type { FileDropConfig, DropHandler, FileValidator } from "../core/types";
import { formatFileSize } from "../core/utils";
import "./Dropzone.css";

// ============================================================================
// Types
// ============================================================================

interface DropzoneProps extends FileDropConfig {
  onDrop?: DropHandler;
  validator?: FileValidator;
  className?: string;
  children?: React.ReactNode;
  showPreview?: boolean;
  generatePreviews?: boolean;
}

// ============================================================================
// Component
// ============================================================================

export const Dropzone: React.FC<DropzoneProps> = ({
  onDrop,
  validator,
  className,
  children,
  showPreview = true,
  generatePreviews = true,
  ...config
}) => {
  const { isDragging, files, error, getRootProps, getInputProps, clearFiles, removeFile } =
    useDropzone({
      onDrop,
      validator,
      generatePreviews,
      ...config,
    });

  return (
    <div className="dropzone-wrapper">
      <div
        {...getRootProps()}
        className={`dropzone ${isDragging ? "dragging" : ""} ${className || ""}`}
      >
        <input {...getInputProps()} />
        {children || (
          <div className="dropzone-content">
            <div className="dropzone-icon">📁</div>
            <p className="dropzone-text">
              {isDragging ? "Drop files here" : "Drag & drop files here, or click to select"}
            </p>
            {config.accept && <p className="dropzone-hint">Accepts: {config.accept.join(", ")}</p>}
            {config.maxSize && (
              <p className="dropzone-hint">Max size: {formatFileSize(config.maxSize)}</p>
            )}
          </div>
        )}
      </div>

      {error && <div className="dropzone-error">{error}</div>}

      {showPreview && files.length > 0 && (
        <div className="dropzone-preview">
          <div className="preview-header">
            <span>Files ({files.length})</span>
            <button onClick={clearFiles} className="preview-clear">
              Clear all
            </button>
          </div>
          <div className="preview-list">
            {files.map((item, index) => (
              <div key={index} className="preview-item">
                {item.preview && (
                  <img src={item.preview} alt={item.file.name} className="preview-image" />
                )}
                <div className="preview-info">
                  <div className="preview-name">{item.file.name}</div>
                  <div className="preview-size">{formatFileSize(item.file.size)}</div>
                </div>
                <button onClick={() => removeFile(index)} className="preview-remove">
                  ×
                </button>
              </div>
            ))}
          </div>
        </div>
      )}
    </div>
  );
};
