/**
 * Terminal Types
 * Type definitions for terminal emulator
 */

// Terminal session from backend
export interface TerminalSession {
  id: string;
  shell: string;
  working_dir: string;
  cols: number;
  rows: number;
  started_at: string;
  active: boolean;
}

// Local tab state
export interface TerminalTab {
  id: string;
  sessionId: string;
  title: string;
  active: boolean;
}

// Terminal settings
export interface TerminalSettings {
  shell: string;
  fontSize: number;
  fontFamily: string;
  theme: 'dark' | 'light';
  cursorStyle: 'block' | 'underline' | 'bar';
  cursorBlink: boolean;
  scrollback: number;
}

// Terminal output from backend
export interface TerminalOutput {
  output: string;
  output_base64: string;
  length: number;
}

