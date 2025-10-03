/**
 * TypeScript declarations for Electron APIs
 */

interface Window {
  electron?: {
    minimize: () => Promise<void>;
    maximize: () => Promise<void>;
    close: () => Promise<void>;
  };
  electronLog?: {
    error: (message: string, ...args: any[]) => void;
    warn: (message: string, ...args: any[]) => void;
    info: (message: string, ...args: any[]) => void;
    debug: (message: string, ...args: any[]) => void;
    verbose: (message: string, ...args: any[]) => void;
  };
}

