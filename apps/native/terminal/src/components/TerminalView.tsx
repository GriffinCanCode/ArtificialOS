/**
 * Terminal View Component
 * Single terminal instance with xterm.js
 */

import { useEffect, useRef, useState } from 'react';
import { Terminal } from '@xterm/xterm';
import { FitAddon } from '@xterm/addon-fit';
import { SearchAddon } from '@xterm/addon-search';
import { WebLinksAddon } from '@xterm/addon-web-links';
import type { NativeAppContext } from '../sdk';
import type { TerminalSettings } from '../types';
import { useTerminalSession } from '../hooks/useTerminalSession';
import { useTerminalResize } from '../hooks/useTerminalResize';
import '@xterm/xterm/css/xterm.css';

interface TerminalViewProps {
  sessionId: string;
  context: NativeAppContext;
  settings: TerminalSettings;
  onTitleChange?: (title: string) => void;
}

export function TerminalView({
  sessionId,
  context,
  settings,
  onTitleChange,
}: TerminalViewProps) {
  const containerRef = useRef<HTMLDivElement>(null);
  const [terminal, setTerminal] = useState<Terminal | null>(null);
  const [fitAddon, setFitAddon] = useState<FitAddon | null>(null);

  // Initialize terminal
  useEffect(() => {
    if (!containerRef.current) return;

    const term = new Terminal({
      fontFamily: settings.fontFamily,
      fontSize: settings.fontSize,
      cursorBlink: settings.cursorBlink,
      cursorStyle: settings.cursorStyle,
      scrollback: settings.scrollback,
      theme: settings.theme === 'dark' ? {
        background: '#0a0a0a',
        foreground: '#ffffff',
        cursor: '#6366f1',
        cursorAccent: '#0a0a0a',
        selectionBackground: 'rgba(99, 102, 241, 0.3)',
        black: '#000000',
        red: '#ef4444',
        green: '#22c55e',
        yellow: '#eab308',
        blue: '#3b82f6',
        magenta: '#a855f7',
        cyan: '#06b6d4',
        white: '#f5f5f5',
        brightBlack: '#737373',
        brightRed: '#f87171',
        brightGreen: '#4ade80',
        brightYellow: '#facc15',
        brightBlue: '#60a5fa',
        brightMagenta: '#c084fc',
        brightCyan: '#22d3ee',
        brightWhite: '#ffffff',
      } : undefined,
    });

    // Add addons
    const fit = new FitAddon();
    const search = new SearchAddon();
    const webLinks = new WebLinksAddon();

    term.loadAddon(fit);
    term.loadAddon(search);
    term.loadAddon(webLinks);

    // Open terminal in container
    term.open(containerRef.current);

    // Welcome message
    term.writeln('\x1b[1;36m╔════════════════════════════════════════╗\x1b[0m');
    term.writeln('\x1b[1;36m║\x1b[0m  \x1b[1;32mAgentOS Terminal\x1b[0m                   \x1b[1;36m║\x1b[0m');
    term.writeln('\x1b[1;36m╚════════════════════════════════════════╝\x1b[0m');
    term.writeln('');

    setTerminal(term);
    setFitAddon(fit);

    return () => {
      term.dispose();
    };
  }, [settings]);

  // Session management (I/O)
  useTerminalSession({
    terminal,
    sessionId,
    context,
  });

  // Resize handling
  useTerminalResize({
    terminal,
    fitAddon,
    sessionId,
    context,
    containerRef,
  });

  // Handle title changes
  useEffect(() => {
    if (!terminal || !onTitleChange) return;

    const disposable = terminal.onTitleChange((title) => {
      onTitleChange(title);
    });

    return () => {
      disposable.dispose();
    };
  }, [terminal, onTitleChange]);

  return (
    <div
      ref={containerRef}
      className="terminal-view"
      style={{
        width: '100%',
        height: '100%',
      }}
    />
  );
}

