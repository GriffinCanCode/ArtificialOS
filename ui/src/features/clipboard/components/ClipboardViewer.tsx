/**
 * Clipboard Viewer Component
 * Displays clipboard history with entry details
 */

import type { ClipboardEntry } from "../core/types";
import "./ClipboardViewer.css";

interface ClipboardViewerProps {
  entries: ClipboardEntry[];
  current: ClipboardEntry | null;
  onSelect?: (entry: ClipboardEntry) => void;
  onCopy?: (entry: ClipboardEntry) => void;
  onDelete?: (entryId: number) => void;
}

export function ClipboardViewer({
  entries,
  current,
  onSelect,
  onCopy,
  onDelete,
}: ClipboardViewerProps) {
  const formatTimestamp = (timestamp: number) => {
    const date = new Date(timestamp / 1000); // Convert from microseconds
    return date.toLocaleString();
  };

  const getDataPreview = (entry: ClipboardEntry): string => {
    const { data } = entry;

    if (data.type === "Text") {
      const text = typeof data.data === "string" ? data.data : String(data.data);
      return text.length > 100 ? text.substring(0, 100) + "..." : text;
    }

    if (data.type === "Html") {
      return "[HTML Content]";
    }

    if (data.type === "Image") {
      return `[Image: ${data.mime_type || "unknown"}]`;
    }

    if (data.type === "Files") {
      const files = Array.isArray(data.data) ? data.data : [];
      return `[Files: ${files.length}]`;
    }

    return "[Binary Data]";
  };

  if (entries.length === 0) {
    return (
      <div className="clipboard-viewer-empty">
        <p>No clipboard history</p>
      </div>
    );
  }

  return (
    <div className="clipboard-viewer">
      {entries.map((entry) => (
        <div
          key={entry.id}
          className={`clipboard-entry ${current?.id === entry.id ? "current" : ""}`}
          onClick={() => onSelect?.(entry)}
        >
          <div className="entry-header">
            <span className="entry-type">{entry.data.type}</span>
            <span className="entry-time">{formatTimestamp(entry.timestamp)}</span>
          </div>

          <div className="entry-preview">
            {getDataPreview(entry)}
          </div>

          {entry.label && (
            <div className="entry-label">{entry.label}</div>
          )}

          <div className="entry-actions">
            <button onClick={(e) => { e.stopPropagation(); onCopy?.(entry); }}>
              Copy
            </button>
            <button onClick={(e) => { e.stopPropagation(); onDelete?.(entry.id); }}>
              Delete
            </button>
          </div>
        </div>
      ))}
    </div>
  );
}

