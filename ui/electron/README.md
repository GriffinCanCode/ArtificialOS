# Electron Main Process

This directory contains the Electron main process code written in TypeScript.

## Files

- **`main.ts`**: Main Electron process entry point
  - Window management and lifecycle
  - IPC handlers for renderer communication
  - Application menu creation
  - Security settings and crash recovery

- **`preload.ts`**: Preload script for secure IPC bridge
  - Exposes safe APIs to renderer via contextBridge
  - Channel whitelisting for security
  - Type-safe IPC communication

- **`preload.d.ts`**: TypeScript type definitions
  - Global type declarations for `window.electron` API
  - Type-safe API for renderer process

- **`tsconfig.json`**: TypeScript configuration for Electron
  - Node.js module resolution
  - ESNext target with CommonJS output
  - Compiles to `../dist-electron/`

## Build Process

The TypeScript files are compiled to JavaScript before running:

```bash
# Development (compiles once, then runs)
npm run dev:electron

# Production build (compiles Electron + React)
npm run build

# Watch mode (auto-recompile on changes)
npm run build:electron:watch

# Manual compile
npm run build:electron
```

## Output

Compiled JavaScript files are output to `dist-electron/`:
- `dist-electron/main.js` (entry point)
- `dist-electron/preload.js`

## Security Features

- **Context Isolation**: Renderer and main processes are isolated
- **Sandbox Mode**: Renderer runs in Chromium sandbox
- **IPC Whitelisting**: Only approved channels can be invoked
- **No Node Integration**: Renderer cannot access Node.js APIs directly
- **CSP Enforcement**: Web security enabled, no insecure content

## Architecture

```
┌─────────────────┐
│  Renderer (UI)  │
│   React App     │
└────────┬────────┘
         │
    window.electron API
    (via contextBridge)
         │
┌────────▼────────┐
│  Preload Script │
│  IPC Validation │
└────────┬────────┘
         │
    ipcRenderer.invoke
         │
┌────────▼────────┐
│  Main Process   │
│  Window Manager │
│  IPC Handlers   │
└─────────────────┘
```

## Development Notes

- Uses **TypeScript** for type safety and better IDE support
- Follows **Electron security best practices**
- **Window state persistence** for better UX
- **Single instance lock** prevents multiple app instances
- **Graceful error handling** with user-friendly dialogs
- **Performance monitoring** and logging via electron-log

## Type Definitions

The `preload.d.ts` file provides type definitions for renderer code:

```typescript
// Available in renderer process
window.electron.minimize();
window.electron.maximize();
window.electron.close();

const info = await window.electron.system.getInfo();
const theme = await window.electron.system.getTheme();

window.electronLog.info('Message');
window.electronLog.error('Error');
```

## Migration from CJS

This directory was migrated from CommonJS (`.cjs`) to TypeScript (`.ts`) for:
- Better type safety
- Modern ES modules syntax
- Improved IDE support
- Easier maintenance and refactoring

