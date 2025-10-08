/**
 * Terminal Session Hook
 * Manages terminal session lifecycle and I/O
 */

import { useEffect, useRef, useCallback } from 'react';
import type { Terminal } from '@xterm/xterm';
import type { NativeAppContext } from '../sdk';

interface UseTerminalSessionProps {
  terminal: Terminal | null;
  sessionId: string | null;
  context: NativeAppContext;
}

export function useTerminalSession({
  terminal,
  sessionId,
  context,
}: UseTerminalSessionProps) {
  const pollingIntervalRef = useRef<number | null>(null);
  const isWritingRef = useRef(false);

  /**
   * Send input to terminal session
   */
  const write = useCallback(
    async (data: string) => {
      if (!sessionId) return;

      isWritingRef.current = true;
      try {
        await context.executor.execute('terminal.write', {
          session_id: sessionId,
          input: data,
        });
      } catch (err) {
        console.error('[Terminal] Write failed:', err);
      } finally {
        isWritingRef.current = false;
      }
    },
    [sessionId, context.executor]
  );

  /**
   * Read output from terminal session
   */
  const read = useCallback(async () => {
    if (!sessionId || !terminal) return;

    try {
      const result = await context.executor.execute('terminal.read', {
        session_id: sessionId,
      });

      if (result?.output && result.output.length > 0) {
        terminal.write(result.output);
      }
    } catch (err) {
      console.error('[Terminal] Read failed:', err);
    }
  }, [sessionId, terminal, context.executor]);

  /**
   * Start polling for output
   */
  useEffect(() => {
    if (!sessionId || !terminal) return;

    // Poll every 50ms for output
    pollingIntervalRef.current = window.setInterval(() => {
      if (!isWritingRef.current) {
        read();
      }
    }, 50);

    return () => {
      if (pollingIntervalRef.current) {
        clearInterval(pollingIntervalRef.current);
        pollingIntervalRef.current = null;
      }
    };
  }, [sessionId, terminal, read]);

  /**
   * Handle terminal input
   */
  useEffect(() => {
    if (!terminal) return;

    const disposable = terminal.onData((data) => {
      write(data);
    });

    return () => {
      disposable.dispose();
    };
  }, [terminal, write]);

  return {
    write,
    read,
  };
}

