/**
 * Clipboard Core Types
 * Type-safe clipboard data structures
 */

export type ClipboardFormat =
  | "text"
  | "html"
  | "bytes"
  | "files"
  | "image";

export interface ClipboardEntry {
  id: number;
  data: ClipboardData;
  source_pid: number;
  timestamp: number;
  label?: string;
}

export interface ClipboardData {
  type: "Text" | "Html" | "Bytes" | "Files" | "Image";
  data?: string | number[] | string[];
  mime_type?: string;
}

export interface ClipboardStats {
  total_entries: number;
  total_size: number;
  process_count: number;
  global_entries: number;
  subscriptions: number;
}

export interface ClipboardOptions {
  format?: string;
  global?: boolean;
  limit?: number;
}

export interface ClipboardState {
  current: ClipboardEntry | null;
  history: ClipboardEntry[];
  stats: ClipboardStats | null;
  subscribed: boolean;
}

export interface ClipboardActions {
  copy: (data: string, options?: ClipboardOptions) => Promise<number>;
  paste: (options?: ClipboardOptions) => Promise<ClipboardEntry | null>;
  getHistory: (options?: ClipboardOptions) => Promise<ClipboardEntry[]>;
  getEntry: (entryId: number) => Promise<ClipboardEntry | null>;
  clear: (global?: boolean) => Promise<void>;
  subscribe: (formats?: string[]) => Promise<void>;
  unsubscribe: () => Promise<void>;
  getStats: () => Promise<ClipboardStats>;
}

