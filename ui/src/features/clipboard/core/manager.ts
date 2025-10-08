/**
 * Clipboard Manager
 * High-level clipboard management with caching and state
 */

import type {
  ClipboardEntry,
  ClipboardStats,
  ClipboardState
} from "./types";

export class ClipboardManager {
  private state: ClipboardState = {
    current: null,
    history: [],
    stats: null,
    subscribed: false,
  };

  private listeners: Set<(state: ClipboardState) => void> = new Set();

  /**
   * Subscribe to state changes
   */
  subscribe(listener: (state: ClipboardState) => void): () => void {
    this.listeners.add(listener);
    return () => this.listeners.delete(listener);
  }

  /**
   * Get current state
   */
  getState(): ClipboardState {
    return { ...this.state };
  }

  /**
   * Update state and notify listeners
   */
  private setState(updater: (state: ClipboardState) => ClipboardState) {
    this.state = updater(this.state);
    this.listeners.forEach(listener => listener(this.state));
  }

  /**
   * Update current entry
   */
  setCurrent(entry: ClipboardEntry | null) {
    this.setState(state => ({ ...state, current: entry }));
  }

  /**
   * Update history
   */
  setHistory(history: ClipboardEntry[]) {
    this.setState(state => ({ ...state, history }));
  }

  /**
   * Update stats
   */
  setStats(stats: ClipboardStats) {
    this.setState(state => ({ ...state, stats }));
  }

  /**
   * Update subscription status
   */
  setSubscribed(subscribed: boolean) {
    this.setState(state => ({ ...state, subscribed }));
  }

  /**
   * Add entry to history (local cache)
   */
  addToHistory(entry: ClipboardEntry) {
    this.setState(state => ({
      ...state,
      current: entry,
      history: [entry, ...state.history.slice(0, 99)], // Keep last 100
    }));
  }

  /**
   * Clear all state
   */
  clearState() {
    this.setState(() => ({
      current: null,
      history: [],
      stats: null,
      subscribed: false,
    }));
  }
}

export const clipboardManager = new ClipboardManager();

