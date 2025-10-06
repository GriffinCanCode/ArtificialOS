# Tool Executors

This directory contains all tool executors organized by use case into logical folders.

## Directory Structure

### 📁 **core/**
Foundational executor components that other executors depend on:
- `types.ts` - Shared types and interfaces for all executors
- `service-executor.ts` - Backend service tool execution bridge
- `ui-executor.ts` - Generic UI state manipulation
- `system-executor.ts` - System-level operations (alerts, undo/redo, state management)

### 📁 **app/**
Application lifecycle and navigation management:
- `app-executor.ts` - App spawning, closing, and state persistence
- `hub-executor.ts` - App registry operations (loading and launching apps)
- `navigation-executor.ts` - Navigation operations (tabs, modals)

### 📁 **media/**
Media rendering and playback:
- `browser-executor.ts` - Iframe navigation and browser-like operations
- `canvas-executor.ts` - Canvas drawing operations
- `player-executor.ts` - Media playback operations

### 📁 **data/**
Data operations and persistence:
- `data-executor.ts` - Data manipulation (filter, sort, search)
- `list-executor.ts` - List operations (add, remove, toggle, clear)
- `filesystem-executor.ts` - Filesystem operations with dynamic UI updates
- `form-executor.ts` - Form validation and submission

### 📁 **system/**
System-level integrations and OS features:
- `clipboard-executor.ts` - Copy/paste operations
- `notification-executor.ts` - Browser notifications
- `network-executor.ts` - HTTP requests
- `timer-executor.ts` - setTimeout/setInterval operations

### 📁 **deprecated/**
Legacy executors kept for backward compatibility:
- `calc-executor.ts` - Calculator operations (use `ui-executor` instead)
- `game-executor.ts` - Game state management

## Usage

All executors are re-exported from the main `index.ts` file, so imports remain unchanged:

```typescript
import {
  ServiceExecutor,
  UIExecutor,
  AppExecutor,
  // ... etc
} from './executors';
```

## Architecture

### Executor Types

1. **BaseExecutor** - Synchronous executor interface
   ```typescript
   interface BaseExecutor {
     execute(action: string, params: Record<string, any>): any;
   }
   ```

2. **AsyncExecutor** - Asynchronous executor interface
   ```typescript
   interface AsyncExecutor {
     execute(action: string, params: Record<string, any>): Promise<any>;
   }
   ```

### ExecutorContext

All executors receive an `ExecutorContext` containing:
- `componentState` - Reactive state management
- `appId` - Current app identifier

## Adding New Executors

1. Choose the appropriate category folder (or create a new one if needed)
2. Create your executor file implementing either `BaseExecutor` or `AsyncExecutor`
3. Add the export to `index.ts`
4. Update this README

## Design Principles

- **Separation of Concerns** - Each executor handles a specific domain
- **Composability** - Executors can delegate to other executors
- **Type Safety** - Strong TypeScript typing throughout
- **Logging** - All operations are logged for debugging
- **State Management** - Uses reactive ComponentState for UI updates
