# Native Apps Module

Support for native TypeScript/React applications in the OS.

## Overview

The native apps system allows developers to write **full React applications** with custom components, unlike Blueprint apps which use prebuilt components and JSON definitions.

### Key Distinctions

| Feature | Blueprint Apps | Native Apps |
|---------|---------------|-------------|
| **Definition** | JSON | TypeScript/React |
| **Components** | Prebuilt (`<Button>`, `<Input>`) | Custom JSX |
| **Development** | AI-generated | Hand-coded |
| **npm Packages** | ❌ | ✅ |
| **Hot Reload** | N/A | ✅ |

## Architecture

```
native/
├── core/
│   ├── types.ts       # TypeScript definitions
│   └── loader.ts      # Dynamic module loader
├── components/
│   ├── renderer.tsx   # App renderer with context
│   ├── content.tsx    # Window content router
│   └── *.css          # Component styles
├── index.ts           # Public exports
└── README.md          # Documentation
```

## Module Loader

### Features

- **Dynamic Imports**: ES module loading with Vite
- **LRU Cache**: Memory-efficient caching with reference counting
- **Error Handling**: Comprehensive error types and recovery
- **Dev/Prod Modes**: HMR in development, optimized bundles in production
- **Timeout Protection**: Prevents hanging loads
- **Memory Management**: Automatic cleanup of unused modules

### Usage

```typescript
import { loader } from '@/features/native';

// Load app
const app = await loader.load('file-explorer', '/apps/file-explorer.js');

// Use app component
<app.component context={context} />

// Release when done
loader.release('file-explorer');
```

### Cache Strategy

1. **Reference Counting**: Track active instances
2. **LRU Eviction**: Remove least recently used when full
3. **TTL Cleanup**: Auto-cleanup after 5 minutes of inactivity
4. **Max Size**: 20 apps cached simultaneously

## Renderer Component

### Features

- **Context Injection**: Provides app with state, executor, window APIs
- **Error Boundaries**: Catches runtime errors
- **Loading States**: Progressive loading UI
- **Lifecycle Management**: Proper mount/unmount handling
- **Memory Cleanup**: Releases resources on unmount

### Usage

```tsx
import { Renderer } from '@/features/native';

<Renderer
  appId="app-123"
  packageId="file-explorer"
  bundlePath="/apps/file-explorer.js"
  windowId="window-456"
/>
```

## Content Router

Routes window content to the appropriate renderer based on app type.

### App Type Detection

```typescript
const appType = window.metadata?.appType || AppType.BLUEPRINT;

if (appType === AppType.NATIVE) {
  // Use native renderer
} else {
  // Use blueprint renderer
}
```

## Type System

### Core Types

```typescript
// App types
enum AppType {
  BLUEPRINT = 'blueprint',
  NATIVE = 'native_web',
  PROCESS = 'native_proc',
}

// Loaded app
interface LoadedApp {
  id: string;
  component: React.ComponentType<NativeAppProps>;
  cleanup?: () => void;
  loadedAt: number;
}

// Window metadata
interface NativeWindowMeta {
  appType: AppType;
  packageId: string;
  bundlePath: string;
  services?: string[];
  permissions?: string[];
}
```

### Error Types

```typescript
class NativeAppError extends Error {
  code: ErrorCode;
  appId?: string;
}

enum ErrorCode {
  LOAD_FAILED = 'LOAD_FAILED',
  NO_DEFAULT_EXPORT = 'NO_DEFAULT_EXPORT',
  INVALID_COMPONENT = 'INVALID_COMPONENT',
  BUNDLE_NOT_FOUND = 'BUNDLE_NOT_FOUND',
  TIMEOUT = 'TIMEOUT',
}
```

## Integration with WindowManager

The native apps system integrates seamlessly with the existing window management:

```tsx
import { Content } from '@/features/native';

// In WindowManager
<Window window={window}>
  <Content
    window={window}
    state={getComponentState(window.id)}
    executor={getToolExecutor(window.id, window.appId)}
  />
</Window>
```

## Performance Optimizations

1. **Lazy Loading**: Apps load only when opened
2. **Code Splitting**: Vite automatically splits bundles
3. **LRU Caching**: Intelligent memory management
4. **Reference Counting**: Prevents premature cleanup
5. **Memoization**: Stable context references

## Development Workflow

### Creating a Native App

```bash
# Create from template
./scripts/create-native-app.sh "My App"

# Develop
cd apps/native/my-app
npm install
npm run dev

# Build
npm run build
```

### App Structure

```
my-app/
├── manifest.json      # Metadata
├── package.json       # Dependencies
├── src/
│   ├── index.tsx      # Entry (default export)
│   ├── App.tsx        # Main component
│   ├── components/    # Custom components
│   └── styles/        # CSS
└── vite.config.ts     # Build config
```

### App Entry Point

```tsx
// src/index.tsx
import type { NativeAppProps } from '@os/sdk';
import App from './App';

export default function MyApp(props: NativeAppProps) {
  return <App {...props} />;
}
```

## Future Enhancements

- [ ] Hot module replacement for faster development
- [ ] Shared dependencies between apps
- [ ] Service worker caching for offline support
- [ ] Progressive loading with code splitting
- [ ] App sandboxing and permissions
- [ ] Performance monitoring and profiling
