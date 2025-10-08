/**
 * Terminal Resize Hook
 * Handles terminal resize events
 */

import { useEffect, useCallback } from 'react';
import type { Terminal } from '@xterm/xterm';
import type { FitAddon } from '@xterm/addon-fit';
import type { NativeAppContext } from '../sdk';

interface UseTerminalResizeProps {
  terminal: Terminal | null;
  fitAddon: FitAddon | null;
  sessionId: string | null;
  context: NativeAppContext;
  containerRef: React.RefObject<HTMLDivElement>;
}

export function useTerminalResize({
  terminal,
  fitAddon,
  sessionId,
  context,
  containerRef,
}: UseTerminalResizeProps) {
  /**
   * Resize terminal to fit container
   */
  const resize = useCallback(async () => {
    if (!terminal || !fitAddon || !sessionId) return;

    try {
      fitAddon.fit();

      // Notify backend of new dimensions
      await context.executor.execute('terminal.resize', {
        session_id: sessionId,
        cols: terminal.cols,
        rows: terminal.rows,
      });
    } catch (err) {
      console.error('[Terminal] Resize failed:', err);
    }
  }, [terminal, fitAddon, sessionId, context.executor]);

  /**
   * Handle window resize
   */
  useEffect(() => {
    if (!terminal || !fitAddon) return;

    const handleResize = () => {
      resize();
    };

    window.addEventListener('resize', handleResize);

    // Initial resize
    resize();

    return () => {
      window.removeEventListener('resize', handleResize);
    };
  }, [terminal, fitAddon, resize]);

  /**
   * Handle container resize (e.g., window resize)
   */
  useEffect(() => {
    if (!containerRef.current || !terminal || !fitAddon) return;

    const resizeObserver = new ResizeObserver(() => {
      resize();
    });

    resizeObserver.observe(containerRef.current);

    return () => {
      resizeObserver.disconnect();
    };
  }, [containerRef, terminal, fitAddon, resize]);

  return {
    resize,
  };
}

