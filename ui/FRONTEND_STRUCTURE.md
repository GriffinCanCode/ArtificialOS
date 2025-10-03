# Frontend Directory Structure

This document describes the organized structure of the frontend codebase.

## Overview

The frontend has been reorganized to improve maintainability and logical grouping of related files.

## Directory Structure

### `/src/components/`

Components are now organized by their purpose:

#### `/components/chat/` (4 files)
- `ChatInterface.tsx` / `ChatInterface.css` - User chat input and message history
- `ThoughtStream.tsx` / `ThoughtStream.css` - AI thinking process visualization

#### `/components/dialogs/` (6 files)
- `Modal.tsx` / `Modal.css` - Reusable modal component
- `SaveAppDialog.tsx` / `SaveAppDialog.css` - Dialog for saving apps to registry
- `SaveSessionDialog.tsx` / `SaveSessionDialog.css` - Dialog for saving sessions

#### `/components/layout/` (4 files)
- `Launcher.tsx` / `Launcher.css` - App launcher grid for installed apps
- `TitleBar.tsx` / `TitleBar.css` - Custom window title bar with session controls

#### Root Components
- `DynamicRenderer.tsx` / `DynamicRenderer.css` - Main component for rendering AI-generated UIs
- `/forms/` - Form-related components and documentation

### `/src/utils/`

Utilities are now organized by category:

#### `/utils/animation/` (3 files)
- `animationConfig.ts` - Animation configuration constants and timing
- `componentVariants.ts` - CVA (Class Variance Authority) component variants
- `gsapAnimations.ts` - GSAP animation functions and spring configs

#### `/utils/api/` (4 files)
- `apiClient.ts` - General API client utilities
- `registryClient.ts` - Registry service API client
- `sessionClient.ts` - Session service API client
- `websocketClient.ts` - WebSocket client for real-time communication

#### `/utils/monitoring/` (3 files)
- `logger.ts` - Logging utilities and log levels
- `performanceMonitor.ts` - Performance monitoring and metrics
- `useLogger.ts` - React hook for logging with component context

#### Root Utils
- `index.ts` - Barrel export file that re-exports from all subdirectories for convenience

### Other Directories

These remain unchanged:

- `/src/hooks/` - React hooks (5 files)
- `/src/contexts/` - React contexts (WebSocket)
- `/src/store/` - Zustand state management (appStore)
- `/src/types/` - TypeScript type definitions (4 files)
- `/src/lib/` - Third-party library configuration (queryClient)
- `/src/renderer/` - Main app renderer and styles

## Import Patterns

### For components in subdirectories

Components in `chat/`, `dialogs/`, and `layout/` use relative imports:

```typescript
// From components/chat/ or dialogs/ or layout/
import { logger } from "../../utils/monitoring/logger";
import { useAppActions } from "../../store/appStore";
import { useWebSocket } from "../../contexts/WebSocketContext";
```

### For root-level files

Files in hooks, store, contexts, etc. use single-level relative imports:

```typescript
// From hooks/, store/, contexts/, etc.
import { logger } from "../utils/monitoring/logger";
import { WebSocketClient } from "../utils/api/websocketClient";
```

### Using the barrel export

For convenience, you can import common utilities from the main utils index:

```typescript
import { logger, useLogger, WebSocketClient } from "../utils";
```

## Benefits

1. **Better Organization**: Files are grouped by purpose (chat, dialogs, layout, animation, API, monitoring)
2. **Improved Scalability**: Each subdirectory has 3-6 files instead of 12+ files in a flat structure
3. **Clearer Intent**: Directory names clearly indicate the purpose of contained files
4. **Easier Navigation**: Developers can quickly find related files
5. **Maintainability**: Related files are co-located, making changes easier to manage

## Migration Notes

All import paths have been updated throughout the codebase. The barrel export in `utils/index.ts` provides backward compatibility for common imports.

