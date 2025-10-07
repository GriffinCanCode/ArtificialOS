/**
 * File Explorer Utilities
 * Helper functions for file operations and UI
 */

import type { FileEntry, FileCategory, FileIcon, PathSegment, SortConfig } from './types';

// ============================================================================
// File Type Detection
// ============================================================================

/**
 * Get file category from mime type or extension
 */
export function getFileCategory(entry: FileEntry): FileCategory {
  if (entry.is_dir) return 'folder';

  const mime = entry.mime_type?.toLowerCase() || '';
  const ext = entry.extension?.toLowerCase() || '';

  // Images
  if (mime.startsWith('image/') || ['jpg', 'jpeg', 'png', 'gif', 'svg', 'webp', 'bmp'].includes(ext)) {
    return 'image';
  }

  // Videos
  if (mime.startsWith('video/') || ['mp4', 'avi', 'mkv', 'mov', 'wmv', 'flv', 'webm'].includes(ext)) {
    return 'video';
  }

  // Audio
  if (mime.startsWith('audio/') || ['mp3', 'wav', 'ogg', 'flac', 'm4a', 'aac'].includes(ext)) {
    return 'audio';
  }

  // Archives
  if (['zip', 'tar', 'gz', 'rar', '7z', 'bz2', 'xz'].includes(ext)) {
    return 'archive';
  }

  // Code
  if (['js', 'ts', 'jsx', 'tsx', 'py', 'java', 'c', 'cpp', 'rs', 'go', 'rb', 'php', 'html', 'css', 'json', 'xml', 'yaml', 'yml', 'md'].includes(ext)) {
    return 'code';
  }

  // Documents
  if (['pdf', 'doc', 'docx', 'txt', 'rtf', 'odt', 'xls', 'xlsx', 'ppt', 'pptx'].includes(ext)) {
    return 'document';
  }

  return 'unknown';
}

/**
 * Get file icon (emoji + color) based on type
 */
export function getFileIcon(entry: FileEntry): FileIcon {
  const category = getFileCategory(entry);

  const icons: Record<FileCategory, FileIcon> = {
    folder: { emoji: '📁', color: '#4A90E2' },
    document: { emoji: '📄', color: '#7B68EE' },
    image: { emoji: '🖼️', color: '#F39C12' },
    video: { emoji: '🎬', color: '#E74C3C' },
    audio: { emoji: '🎵', color: '#9B59B6' },
    archive: { emoji: '📦', color: '#95A5A6' },
    code: { emoji: '💻', color: '#2ECC71' },
    unknown: { emoji: '📃', color: '#BDC3C7' },
  };

  return icons[category];
}

// ============================================================================
// File Operations
// ============================================================================

/**
 * Format file size to human-readable string
 */
export function formatFileSize(bytes: number): string {
  if (bytes === 0) return '0 B';

  const units = ['B', 'KB', 'MB', 'GB', 'TB'];
  const k = 1024;
  const i = Math.floor(Math.log(bytes) / Math.log(k));

  return `${(bytes / Math.pow(k, i)).toFixed(1)} ${units[i]}`;
}

/**
 * Format date to relative time
 */
export function formatDate(dateString: string): string {
  try {
    const date = new Date(dateString);
    const now = new Date();
    const diff = now.getTime() - date.getTime();

    const seconds = Math.floor(diff / 1000);
    const minutes = Math.floor(seconds / 60);
    const hours = Math.floor(minutes / 60);
    const days = Math.floor(hours / 24);

    if (days > 7) {
      return date.toLocaleDateString();
    } else if (days > 0) {
      return `${days}d ago`;
    } else if (hours > 0) {
      return `${hours}h ago`;
    } else if (minutes > 0) {
      return `${minutes}m ago`;
    } else {
      return 'Just now';
    }
  } catch {
    return dateString;
  }
}

/**
 * Get file extension
 */
export function getExtension(filename: string): string {
  const parts = filename.split('.');
  return parts.length > 1 ? parts[parts.length - 1].toLowerCase() : '';
}

/**
 * Get filename without extension
 */
export function getBasename(filename: string): string {
  const parts = filename.split('.');
  return parts.length > 1 ? parts.slice(0, -1).join('.') : filename;
}

// ============================================================================
// Path Operations
// ============================================================================

/**
 * Parse path into breadcrumb segments
 */
export function parsePath(path: string): PathSegment[] {
  if (!path || path === '/') {
    return [{ name: 'Root', path: '/' }];
  }

  const parts = path.split('/').filter(Boolean);
  const segments: PathSegment[] = [{ name: 'Root', path: '/' }];

  let currentPath = '';
  for (const part of parts) {
    currentPath += `/${part}`;
    segments.push({
      name: part,
      path: currentPath,
    });
  }

  return segments;
}

/**
 * Get parent directory path
 */
export function getParentPath(path: string): string {
  if (!path || path === '/') return '/';

  const normalized = path.endsWith('/') ? path.slice(0, -1) : path;
  const lastSlash = normalized.lastIndexOf('/');

  return lastSlash <= 0 ? '/' : normalized.substring(0, lastSlash);
}

/**
 * Join path segments
 */
export function joinPath(...segments: string[]): string {
  const joined = segments
    .filter(Boolean)
    .join('/')
    .replace(/\/+/g, '/');

  return joined.startsWith('/') ? joined : '/' + joined;
}

// ============================================================================
// Sorting & Filtering
// ============================================================================

/**
 * Sort file entries
 */
export function sortEntries(entries: FileEntry[], config: SortConfig): FileEntry[] {
  const sorted = [...entries].sort((a, b) => {
    // Always show directories first
    if (a.is_dir && !b.is_dir) return -1;
    if (!a.is_dir && b.is_dir) return 1;

    let comparison = 0;

    switch (config.field) {
      case 'name':
        comparison = a.name.localeCompare(b.name, undefined, { numeric: true, sensitivity: 'base' });
        break;
      case 'size':
        comparison = a.size - b.size;
        break;
      case 'modified':
        comparison = new Date(a.modified).getTime() - new Date(b.modified).getTime();
        break;
      case 'type':
        const extA = getExtension(a.name);
        const extB = getExtension(b.name);
        comparison = extA.localeCompare(extB);
        break;
    }

    return config.direction === 'asc' ? comparison : -comparison;
  });

  return sorted;
}

/**
 * Filter entries by search query
 */
export function filterEntries(entries: FileEntry[], query: string): FileEntry[] {
  if (!query.trim()) return entries;

  const lower = query.toLowerCase();
  return entries.filter(entry =>
    entry.name.toLowerCase().includes(lower)
  );
}

// ============================================================================
// Validation
// ============================================================================

/**
 * Check if filename is valid
 */
export function isValidFilename(name: string): boolean {
  if (!name || name.trim().length === 0) return false;

  // Disallow special characters
  const invalid = /[<>:"/\\|?*\x00-\x1F]/;
  if (invalid.test(name)) return false;

  // Disallow reserved names
  const reserved = ['.', '..'];
  if (reserved.includes(name)) return false;

  return true;
}

/**
 * Sanitize filename
 */
export function sanitizeFilename(name: string): string {
  return name
    .replace(/[<>:"/\\|?*\x00-\x1F]/g, '_')
    .trim();
}
