/**
 * Type definitions for Browser app
 */

export interface BrowserTab {
  id: string;
  title: string;
  url: string;
  favicon?: string;
  loading: boolean;
  canGoBack: boolean;
  canGoForward: boolean;
  history: string[];
  historyIndex: number;
  isPinned?: boolean;
  isPrivate?: boolean;
  console?: ConsoleOutput[];
}

export interface Bookmark {
  id: string;
  title: string;
  url: string;
  favicon?: string;
  folder?: string;
  dateAdded: number;
  tags?: string[];
}

export interface HistoryEntry {
  id: string;
  url: string;
  title: string;
  visitTime: number;
  favicon?: string;
}

export interface Download {
  id: string;
  url: string;
  filename: string;
  progress: number;
  completed: boolean;
  error?: string;
  startTime: number;
  endTime?: number;
}

export interface BrowserSettings {
  searchEngine: 'duckduckgo' | 'google' | 'bing';
  homepage: string;
  newTabPage: 'blank' | 'homepage' | 'bookmarks';
  downloadPath: string;
  enableJavaScript: boolean;
  enableImages: boolean;
  enableReaderMode: boolean;
  showConsole: boolean;
}

export interface ConsoleOutput {
  level: string;
  message: string;
  time: string;
}

export type RenderMode = 'iframe' | 'proxy' | 'reader';

export interface TabContent {
  mode: RenderMode;
  html?: string;
  error?: string;
}

