/**
 * Preview Panel Component
 * Smart inline previews for files
 */

import { useEffect, useState } from 'react';
import type { FileEntry } from '../types';
import type { NativeAppContext } from '../sdk';
import { getFileIcon, formatFileSize, formatDate } from '../utils';
import './PreviewPanel.css';

interface PreviewPanelProps {
  entry: FileEntry;
  context: NativeAppContext;
  onOpen: () => void;
}

export function PreviewPanel({ entry, context, onOpen }: PreviewPanelProps) {
  const [preview, setPreview] = useState<any>(null);
  const [loading, setLoading] = useState(false);
  const icon = getFileIcon(entry);

  useEffect(() => {
    if (!entry.is_dir) {
      loadPreview();
    }
  }, [entry.path]);

  const loadPreview = async () => {
    const ext = entry.name.split('.').pop()?.toLowerCase() || '';

    // Images - show inline
    if (['jpg', 'jpeg', 'png', 'gif', 'webp', 'bmp', 'svg'].includes(ext)) {
      setPreview({ type: 'image', src: `file://${entry.path}` });
      return;
    }

    // Text/code - show preview
    if (['txt', 'md', 'json', 'js', 'ts', 'tsx', 'jsx', 'py', 'go', 'rs', 'css', 'html', 'yaml', 'yml', 'toml'].includes(ext)) {
      setLoading(true);
      try {
        const result = await context.executor.execute('filesystem.read', { path: entry.path });
        const content = result?.content || '';
        const lines = content.split('\n').slice(0, 50); // First 50 lines
        setPreview({ type: 'text', content: lines.join('\n'), truncated: content.split('\n').length > 50 });
      } catch (err) {
        setPreview({ type: 'error', message: 'Could not load preview' });
      } finally {
        setLoading(false);
      }
      return;
    }

    // PDF
    if (ext === 'pdf') {
      setPreview({ type: 'pdf', path: entry.path });
      return;
    }

    // No preview available
    setPreview({ type: 'none' });
  };

  return (
    <div className="preview-panel">
      {/* Header */}
      <div className="preview-header">
        <div className="preview-icon" style={{ color: icon.color }}>
          {icon.emoji}
        </div>
        <div className="preview-title">
          <div className="preview-name">{entry.name}</div>
          <div className="preview-meta">
            {!entry.is_dir && formatFileSize(entry.size)} · {formatDate(entry.modified)}
          </div>
        </div>
      </div>

      {/* Content */}
      <div className="preview-content">
        {entry.is_dir ? (
          <div className="preview-placeholder">
            <div className="placeholder-icon">📁</div>
            <div className="placeholder-text">Folder</div>
            <div className="placeholder-hint">Double-click to open</div>
          </div>
        ) : loading ? (
          <div className="preview-loading">
            <div className="spinner">↻</div>
            <div className="loading-text">Loading preview...</div>
          </div>
        ) : preview?.type === 'image' ? (
          <div className="preview-image-container">
            <img src={preview.src} alt={entry.name} className="preview-image" />
          </div>
        ) : preview?.type === 'text' ? (
          <div className="preview-text">
            <pre className="text-content">{preview.content}</pre>
            {preview.truncated && (
              <div className="text-truncated">... preview truncated</div>
            )}
          </div>
        ) : preview?.type === 'pdf' ? (
          <div className="preview-placeholder">
            <div className="placeholder-icon">📄</div>
            <div className="placeholder-text">PDF Document</div>
            <div className="placeholder-hint">Click Open to view</div>
          </div>
        ) : preview?.type === 'error' ? (
          <div className="preview-error">
            <div className="error-icon">⚠️</div>
            <div className="error-text">{preview.message}</div>
          </div>
        ) : (
          <div className="preview-placeholder">
            <div className="placeholder-icon">{icon.emoji}</div>
            <div className="placeholder-text">No preview available</div>
            <div className="placeholder-hint">Click Open to view</div>
          </div>
        )}
      </div>

      {/* Actions */}
      <div className="preview-actions">
        <button className="action-button primary" onClick={onOpen}>
          Open
        </button>
        <button
          className="action-button"
          onClick={async () => {
            await context.executor.execute('clipboard.write', { text: entry.path });
          }}
        >
          Copy Path
        </button>
      </div>

      {/* Details */}
      <div className="preview-details">
        <div className="detail-row">
          <div className="detail-label">Path</div>
          <div className="detail-value">{entry.path}</div>
        </div>
        {!entry.is_dir && (
          <>
            <div className="detail-row">
              <div className="detail-label">Size</div>
              <div className="detail-value">{formatFileSize(entry.size)}</div>
            </div>
            {entry.mime_type && (
              <div className="detail-row">
                <div className="detail-label">Type</div>
                <div className="detail-value">{entry.mime_type}</div>
              </div>
            )}
          </>
        )}
        <div className="detail-row">
          <div className="detail-label">Modified</div>
          <div className="detail-value">{new Date(entry.modified).toLocaleString()}</div>
        </div>
      </div>
    </div>
  );
}

