/**
 * File System Hook
 * Manages file system operations and navigation
 */

import { useState, useCallback, useRef } from 'react';
import type { NativeAppContext } from '../sdk';
import type { FileEntry, UseFileSystemReturn, HistoryEntry } from '../types';
import { getParentPath, joinPath } from '../utils';

export function useFileSystem(context: NativeAppContext): UseFileSystemReturn {
  const { executor } = context;

  const [entries, setEntries] = useState<FileEntry[]>([]);
  // Use VFS path instead of absolute path - kernel mounts at /storage
  const [currentPath, setCurrentPath] = useState('/storage');
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  // Navigation history
  const history = useRef<HistoryEntry[]>([]);
  const historyIndex = useRef(-1);

  /**
   * Load directory contents
   */
  const loadDirectory = useCallback(async (path: string) => {
    setLoading(true);
    setError(null);

    try {
      const result = await executor.execute('filesystem.list', { path });

      if (!result || !result.entries) {
        throw new Error('Failed to read directory');
      }

      setEntries(result.entries);
      setCurrentPath(path);

      return result.entries;
    } catch (err) {
      const message = (err as Error).message;
      setError(message);
      throw err;
    } finally {
      setLoading(false);
    }
  }, [executor]);

  /**
   * Navigate to path
   */
  const navigate = useCallback(async (path: string) => {
    await loadDirectory(path);

    // Add to history
    history.current = history.current.slice(0, historyIndex.current + 1);
    history.current.push({ path, timestamp: Date.now() });
    historyIndex.current = history.current.length - 1;
  }, [loadDirectory]);

  /**
   * Refresh current directory
   */
  const refresh = useCallback(async () => {
    await loadDirectory(currentPath);
  }, [currentPath, loadDirectory]);

  /**
   * Navigate to parent directory
   */
  const goUp = useCallback(async () => {
    const parent = getParentPath(currentPath);
    if (parent !== currentPath) {
      await navigate(parent);
    }
  }, [currentPath, navigate]);

  /**
   * Navigate back in history
   */
  const goBack = useCallback(async () => {
    if (historyIndex.current > 0) {
      historyIndex.current--;
      const entry = history.current[historyIndex.current];
      await loadDirectory(entry.path);
    }
  }, [loadDirectory]);

  /**
   * Navigate forward in history
   */
  const goForward = useCallback(async () => {
    if (historyIndex.current < history.current.length - 1) {
      historyIndex.current++;
      const entry = history.current[historyIndex.current];
      await loadDirectory(entry.path);
    }
  }, [loadDirectory]);

  /**
   * Create new folder
   */
  const createFolder = useCallback(async (name: string) => {
    const newPath = joinPath(currentPath, name);

    console.log('[FileSystem] Creating folder:', { name, currentPath, newPath });

    try {
      const result = await executor.execute('filesystem.mkdir', { path: newPath });
      console.log('[FileSystem] Folder created successfully:', result);
      await refresh();
    } catch (err) {
      const errorMsg = `Failed to create folder "${name}": ${(err as Error).message}`;
      console.error('[FileSystem] Create folder failed:', { name, newPath, error: err });
      throw new Error(errorMsg);
    }
  }, [currentPath, executor, refresh]);

  /**
   * Delete entry
   */
  const deleteEntry = useCallback(async (path: string) => {
    try {
      await executor.execute('filesystem.delete', { path });
      await refresh();
    } catch (err) {
      throw new Error(`Failed to delete: ${(err as Error).message}`);
    }
  }, [executor, refresh]);

  /**
   * Rename entry
   */
  const renameEntry = useCallback(async (oldPath: string, newName: string) => {
    const parent = getParentPath(oldPath);
    const newPath = joinPath(parent, newName);

    try {
      await executor.execute('filesystem.move', { source: oldPath, destination: newPath });
      await refresh();
    } catch (err) {
      throw new Error(`Failed to rename: ${(err as Error).message}`);
    }
  }, [executor, refresh]);

  /**
   * Copy entry
   */
  const copyEntry = useCallback(async (source: string, dest: string) => {
    try {
      await executor.execute('filesystem.copy', { source, destination: dest });
      await refresh();
    } catch (err) {
      throw new Error(`Failed to copy: ${(err as Error).message}`);
    }
  }, [executor, refresh]);

  /**
   * Move entry
   */
  const moveEntry = useCallback(async (source: string, dest: string) => {
    try {
      await executor.execute('filesystem.move', { source, destination: dest });
      await refresh();
    } catch (err) {
      throw new Error(`Failed to move: ${(err as Error).message}`);
    }
  }, [executor, refresh]);

  const canGoBack = historyIndex.current > 0;
  const canGoForward = historyIndex.current < history.current.length - 1;

  return {
    entries,
    currentPath,
    loading,
    error,
    navigate,
    refresh,
    goUp,
    goBack,
    goForward,
    canGoBack,
    canGoForward,
    createFolder,
    deleteEntry,
    renameEntry,
    copyEntry,
    moveEntry,
  };
}
