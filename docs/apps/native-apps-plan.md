# Native Apps Implementation

## Overview

This document describes the implementation of native applications alongside Blueprint apps. The system supports three types of applications:

1. **Blueprint Apps**: JSON-based UI definitions (existing system)
2. **Native Web Apps**: TypeScript/React applications with custom components (implemented)
3. **Native Process Apps**: OS executables and scripts (planned)

---

## Architecture

### Three-Tier App System

```
┌─ BROWSER LAYER ────────────────────────────────────────────┐
│                                                              │
│  Blueprint Apps      Native Web Apps      Native Process    │
│  (JSON UISpec)       (React/TypeScript)    Apps (Terminal)   │
│                                                              │
│  DynamicRenderer     NativeAppRenderer     ProcessUI         │
│  Prebuilt UI         Custom Components     WebSocket/IPC     │
│                                                              │
└─ ────────────────────────────────────────────────────────────┘
                           │
          App Registry (Unified)
          - blueprint
          - native_web
          - native_proc (planned)

┌─ BACKEND LAYER (Go) ────────────────────────────────────────┐
│                                                              │
│  Shared APIs (all apps)                                      │
│  - ToolExecutor                                              │
│  - ServiceExecutor                                           │
│  - All providers (filesystem, storage, auth, http)          │
│                                                              │
│  Process Manager (future)                                    │
│  - Spawn OS processes                                        │
│  - Manage stdio/stderr streams                               │
│  - WebSocket for real-time I/O                               │
│                                                              │
└─ ────────────────────────────────────────────────────────────┘

┌─ KERNEL LAYER (Rust) ──────────────────────────────────────┐
│                                                              │
│  Native OS Processes (future)                                │
│  - Python scripts                                            │
│  - CLI tools                                                 │
│  - Compiled binaries                                         │
│  - Node.js applications                                      │
│  - Shell scripts                                             │
│                                                              │
└─ ────────────────────────────────────────────────────────────┘
```

### Key Distinctions

| Feature | Blueprint | Native Web | Native Process |
|---------|-----------|-----------|------------------|
| **Definition** | JSON | TypeScript/React | Executable |
| **UI** | Prebuilt components | Custom React | Terminal/UI |
| **Development** | AI-generated | Hand-coded | Any language |
| **Components** | Button, Input, etc. | Your own JSX | N/A |
| **Execution** | Browser only | Browser only | OS process |
| **APIs** | ToolExecutor | ToolExecutor | stdio/syscalls |

---

## Implementation Status

### Phase 1: Core Infrastructure ✓ Implemented

#### 1.1 App Type System

**Status**: Complete

Backend types support three app types:

```go
type AppType string

const (
    AppTypeBlueprint  AppType = "blueprint"
    AppTypeNativeWeb  AppType = "native_web"
    AppTypeNativeProc AppType = "native_proc"
)
```

**Location**: `backend/internal/shared/types/registry.go`

#### 1.2 Native App Bundling System

**Status**: Complete

Directory structure in place:

```
apps/
 blueprint/              # Blueprint apps (.bp files)
    productivity/
    system/

 native/                 # Native TypeScript/React apps
    browser/
    file-explorer/
    settings/
    hub/
    terminal/
    vite.config.base.ts  # Shared build config

 dist/                   # Built native apps
     browser/
     file-explorer/
     hub/
     settings/
     terminal/
```

Existing native apps:
- **Browser**: Full-featured web browser
- **File Explorer**: File manager with Miller Columns
- **Settings**: System configuration
- **Hub**: App launcher and discovery
- **Terminal**: Shell integration

**Location**: `apps/native/`

#### 1.3 Frontend App SDK

**Status**: Complete

SDK provides clean API for app developers:

```typescript
export interface NativeAppContext {
  appId: string;
  state: ComponentState;
  executor: ToolExecutor;
  window: AppWindow;
}
```

Features:
- State management helpers
- Service call abstraction
- Filesystem, Storage, HTTP APIs
- System APIs
- Window control methods
- Lifecycle hooks

**Location**: `ui/src/core/sdk/index.ts`

---

### Phase 2: App Loading & Execution ✓ Implemented

#### 2.1 Native App Loader

**Status**: Complete

Sophisticated module loader with:
- Dynamic ES module imports
- LRU caching with reference counting
- Development/production mode support
- Comprehensive error handling
- CSS injection and cleanup
- Load timeouts and validation

**Features**:
- Caches up to 20 apps
- 5-minute TTL for unused apps
- Reference counting for multiple instances
- Automatic CSS loading and cleanup
- Load time tracking and logging

**Location**: `ui/src/features/native/core/loader.ts`

#### 2.2 Native App Renderer

**Status**: Complete

React component for rendering native apps:
- Loads app bundles dynamically
- Creates app context
- Handles loading and error states
- Integrates with window manager

**Location**: `ui/src/features/native/components/renderer.tsx`

#### 2.3 Window Manager Integration

**Status**: Complete

WindowManager routes to appropriate renderer:
- Blueprint apps use DynamicRenderer
- Native apps use NativeAppRenderer
- Process apps use ProcessUI (planned)

**Location**: `ui/src/features/windows/`

---

### Phase 3: Backend Integration ✓ Implemented

#### 3.1 Registry Support

**Status**: Complete

Registry handles all three app types:
- Package type discrimination
- BundlePath for native web apps
- WebManifest for metadata
- ProcManifest for process apps (prepared)

**Location**: `backend/internal/shared/types/registry.go`

#### 3.2 Launch Endpoints

**Status**: Complete (native_web), Planned (native_proc)

Launch endpoint handles app type switching:

```go
func (h *Handlers) LaunchRegistryApp(c *gin.Context) {
    switch pkg.Type {
    case types.AppTypeNativeWeb:
        h.launchNativeWebApp(c, pkg)
    case types.AppTypeNativeProc:
        h.launchNativeProcApp(c, pkg)
    default:
        h.launchBlueprintApp(c, pkg)
    }
}
```

**Location**: `backend/internal/api/http/handlers.go`

#### 3.3 App Bundling

**Status**: Complete

Native app bundles served from `apps/dist/`:
- Each app has its own bundle directory
- Contains `index.js` (entry point)
- Contains `assets/` directory with CSS
- Bundles are built with Vite

---

### Phase 4: Developer Experience ✓ Implemented

#### 4.1 App Generator

**Status**: Complete

Scripts for app scaffolding:

```bash
./scripts/create-native-app.sh "My App"
```

Creates:
- Directory structure
- manifest.json
- package.json with React dependencies
- TypeScript configuration
- Vite configuration
- README and examples

**Location**: `scripts/create-native-app.sh`

#### 4.2 Build Scripts

**Status**: Complete

- `build-native-apps.sh`: Build all apps in parallel
- `watch-native-apps.sh`: Watch and rebuild on changes
- `validate-native-apps.sh`: Validate structure
- `lint-native-apps.sh`: Lint and type-check

**Location**: `scripts/`

---

### Phase 5: Example Native Apps ✓ Implemented

Five complete native applications demonstrate capabilities:

#### Browser
- Tab management
- History tracking
- Full web browsing
- DOM embedding
- Network requests

#### File Explorer
- Miller Columns UI
- Command Palette
- Smart previews
- Intelligent search
- Keyboard shortcuts
- Multiple selection modes

#### Settings
- System configuration
- Tabbed interface
- Storage management
- Developer tools
- Appearance customization

#### Hub
- App launcher and discovery
- Search and fuzzy matching
- Favorites and recent apps
- Icon rendering
- App grid layout

#### Terminal
- Shell integration
- Session management
- Command history
- Terminal resize handling
- Output streaming

---

### Phase 6: Native Process Apps ⏳ Planned

This phase covers running OS processes within the window system.

#### 6.1 Backend Process Provider

**Status**: Planned

Needed components:
- Process spawning and management
- stdin/stdout/stderr stream handling
- Process lifecycle management (spawn, kill, status)
- Resource limits and sandboxing

#### 6.2 WebSocket Process Streaming

**Status**: Planned

Real-time I/O streaming over WebSocket:
- stdout streaming
- stderr streaming
- stdin input handling
- Process exit signals

#### 6.3 Frontend Terminal UI

**Status**: Planned

Terminal component for process apps:
- Terminal output rendering
- Input handling
- ANSI color support
- Command echoing

#### 6.4 Process App Manifest

**Status**: Spec defined

Manifest format for process apps:

```json
{
  "id": "python-runner",
  "type": "native_proc",
  "proc_manifest": {
    "executable": "python3",
    "args": [],
    "working_dir": "/tmp",
    "ui_type": "terminal",
    "env": {}
  }
}
```

---

## Development Guide

### Creating a Native App

#### 1. Setup

```bash
./scripts/create-native-app.sh "My App"
cd apps/native/my-app
npm install
npm run dev
```

#### 2. App Structure

```
src/
 index.tsx           # Entry point (exports default)
 App.tsx             # Main component
 components/         # Custom components
 hooks/              # Custom hooks
 styles/             # CSS files
 types.ts            # TypeScript types
manifest.json        # App metadata
```

#### 3. Entry Point

```typescript
import React from 'react';
import type { NativeAppProps } from '@os/sdk';
import App from './App';

export default function MyApp(props: NativeAppProps) {
  return <App {...props} />;
}
```

#### 4. Main Component

```typescript
import React, { useState, useEffect } from 'react';
import type { NativeAppProps } from '@os/sdk';

export default function App({ context }: NativeAppProps) {
  const { state, executor, window } = context;
  
  useEffect(() => {
    // Initialize app
  }, []);
  
  return <div>Your app here</div>;
}
```

#### 5. Using Services

```typescript
// Call backend service
const result = await executor.execute('filesystem.list', { path: '/' });

// Store persistent data
await executor.execute('storage.set', { key: 'data', value: {...} });

// Make HTTP requests
await executor.execute('http.get', { url: 'https://api.example.com' });

// Update state
state.set('mykey', 'myvalue');

// Control window
window.setTitle('New Title');
```

#### 6. Building

```bash
npm run build
# Output: ../../dist/<app-id>/index.js
```

---

## Technical Details

### Vite Configuration

Native apps use a shared Vite configuration:

**File**: `apps/native/vite.config.base.ts`

Features:
- React Fast Refresh
- Automatic JSX runtime
- ES module output
- React externalized (provided by host)
- Source maps in development
- Minification in production
- CSS code splitting

### Manifest Format

```json
{
  "id": "my-app",
  "name": "My App",
  "type": "native_web",
  "version": "1.0.0",
  "icon": "/apps/native/my-app/assets/icon.svg",
  "category": "system",
  "author": "system",
  "description": "Description of the app",
  "permissions": ["READ_FILE", "WRITE_FILE"],
  "services": ["filesystem", "storage"],
  "exports": {
    "component": "MyApp"
  },
  "tags": ["my", "tags"]
}
```

### SDK API

Available on `context` parameter:

```typescript
// State
context.state.set(key, value)
context.state.get(key)
context.state.subscribe(key, callback)
context.state.batch(() => {...})

// Services
context.executor.execute(toolId, params)

// Window
context.window.setTitle(title)
context.window.setIcon(icon)
context.window.close()
context.window.minimize()
context.window.maximize()
context.window.focus()
```

---

## Future Work

### Native Process Apps (Phase 6)

The infrastructure is partially designed but not yet implemented:

- Process spawning and management
- WebSocket streaming for real-time I/O
- Terminal UI component for interactive shells
- Process app manifest format
- Resource limits and sandboxing

This would enable running Python scripts, CLI tools, and other OS executables within the window system.

### Other Future Enhancements

- Hot reload for native apps
- App versioning and updates
- Dependency management
- Permission UI for user confirmation
- App code signing
- Custom component registration
- Per-app theme overrides
- App marketplace

---

## Summary

Native web apps are fully implemented with:
- 5 complete example applications
- Sophisticated module loader with caching
- Clean SDK for app developers
- Build system with Vite
- Developer tools (create, build, validate, lint)
- Full integration with window manager
- Service execution via ToolExecutor

Native process apps remain as planned future work, with infrastructure partially designed but not yet implemented.

All three app types (Blueprint, Native Web, Native Process) are supported in the type system and registry, enabling future expansion.
