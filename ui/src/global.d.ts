/**
 * TypeScript declarations for Electron APIs
 */

interface Window {
  electron?: {
    minimize: () => Promise<void>;
    maximize: () => Promise<void>;
    close: () => Promise<void>;
  };
}

