# Native Apps Developer Guide

Complete guide to developing native TypeScript/React applications for the OS.

## Table of Contents

- [Overview](#overview)
- [Quick Start](#quick-start)
- [Development Workflow](#development-workflow)
- [Build System](#build-system)
- [Tooling](#tooling)
- [Best Practices](#best-practices)
- [SDK Reference](#sdk-reference)
- [Troubleshooting](#troubleshooting)

---

## Overview

Native apps are **full React applications** with complete freedom to:
- ✅ Use any npm packages
- ✅ Write custom React components
- ✅ Use any CSS/styling solution
- ✅ Implement any architecture (MVC, Flux, etc.)

**Unlike Blueprint apps**, native apps:
- ❌ Do NOT use prebuilt components
- ❌ Do NOT use JSON definitions
- ✅ Are hand-coded TypeScript/React

---

## Quick Start

### Create a New App

```bash
# Using Makefile (recommended)
make create-native-app name="My Awesome App"

# Or directly
./scripts/create-native-app.sh "My Awesome App"
```

This generates:
```
apps/native/my-awesome-app/
 manifest.json       # App metadata
 package.json        # Dependencies
 tsconfig.json       # TypeScript config
 vite.config.ts      # Build config (extends shared base)
 .eslintrc.json      # Linting rules
 .prettierrc         # Code formatting
 .gitignore          # Git ignore rules
 README.md           # App documentation
 src/
     index.tsx       # Entry point (default export)
     App.tsx         # Main component
     components/     # Custom components
     hooks/          # Custom hooks
     styles/         # CSS files
```

### Install Dependencies

```bash
cd apps/native/my-awesome-app
npm install
```

### Start Development

```bash
# In app directory
npm run dev

# Or from project root
make watch-native-app name=my-awesome-app
```

### Build for Production

```bash
# In app directory
npm run build

# Or from project root
make build-native-apps
```

---

## Development Workflow

### 1. **Hot Module Replacement (HMR)**

The development server supports HMR for instant feedback:

```bash
# Watch single app with HMR
make watch-native-app name=my-awesome-app

# Watch all apps
make watch-native-apps
```

Changes to source files trigger automatic rebuilds without full page reloads.

### 2. **Type Checking**

TypeScript type checking ensures code quality:

```bash
# In app directory
npm run type-check

# From project root (all apps)
make lint-native-apps
```

### 3. **Linting & Formatting**

Automated code quality tools:

```bash
# Lint single app
npm run lint
npm run lint:fix    # Auto-fix issues

# Format code
npm run format

# From project root
make lint-native-app name=my-awesome-app
make fix-native-apps    # Auto-fix all apps
```

### 4. **Validation**

Validate app structure and manifests:

```bash
make validate-native-apps
```

Checks:
- ✅ Valid manifest.json format
- ✅ Required files present
- ✅ Correct TypeScript configuration
- ✅ Dependencies properly declared

---

## Build System

### Shared Vite Configuration

All apps extend a shared base configuration (`apps/native/vite.config.base.ts`):

```typescript
// apps/native/my-app/vite.config.ts
import { defineNativeAppConfig } from '../vite.config.base';

export default defineNativeAppConfig('MyApp', {
  // App-specific overrides
});
```

#### What's Included

- **Fast Refresh**: React HMR enabled
- **Automatic JSX**: No need for `React` imports
- **Library Mode**: Builds as ES module
- **External Dependencies**: React, React-DOM externalized (shared with host)
- **Optimized Minification**: esbuild for fast builds
- **Source Maps**: Dev only (disabled in production)
- **Code Splitting**: Intelligent chunk strategy
- **Asset Optimization**: Inline assets < 4KB

### Build Output

Apps build to `apps/dist/<app-id>/`:

```
apps/dist/my-awesome-app/
 index.js           # Main bundle (ES module)
 assets/
     *.css          # Extracted CSS
```

### Production Optimization

Production builds are optimized for:
- **Bundle Size**: Tree-shaking, minification
- **Load Performance**: Code splitting, lazy loading
- **Runtime Performance**: Dead code elimination

---

## Tooling

### Makefile Commands

#### Creation & Development

```bash
make create-native-app name="App Name"    # Create new app
make watch-native-apps                    # Watch all apps
make watch-native-app name=app-id         # Watch specific app
```

#### Building

```bash
make build-native-apps                    # Build all apps
make clean-native-apps                    # Clean build artifacts
```

#### Quality Assurance

```bash
make validate-native-apps                 # Validate structure
make lint-native-apps                     # Lint all apps
make lint-native-app name=app-id          # Lint specific app
make fix-native-apps                      # Auto-fix issues
```

### Scripts

#### `create-native-app.sh`

Creates a new app from template with:
- Proper directory structure
- Pre-configured tooling (ESLint, Prettier, TypeScript)
- Shared Vite configuration
- README with examples

#### `build-native-apps.sh`

Builds all native apps in parallel:
- Installs dependencies if needed
- Runs TypeScript compiler
- Builds with Vite
- Reports success/failure

#### `watch-native-apps.sh`

Watches source files and rebuilds on changes:
- Supports `fswatch` for efficient file watching
- Falls back to polling if `fswatch` not available
- Can watch all apps or specific app
- Supports dev server mode with HMR

```bash
# Watch all apps
./scripts/watch-native-apps.sh

# Watch specific app
./scripts/watch-native-apps.sh -a my-app

# Start dev server (HMR)
./scripts/watch-native-apps.sh -a my-app -m serve
```

#### `validate-native-apps.sh`

Validates app structure and configuration:
- Checks manifest.json format
- Validates required fields
- Checks file structure
- Validates TypeScript config
- Verifies dependencies

#### `lint-native-apps.sh`

Lints and type-checks apps:
- Runs TypeScript compiler (--noEmit)
- Runs ESLint (if configured)
- Runs Prettier (if configured)
- Supports auto-fix mode

```bash
# Lint all
./scripts/lint-native-apps.sh

# Lint specific app
./scripts/lint-native-apps.sh -a my-app

# Auto-fix
./scripts/lint-native-apps.sh --fix
```

---

## Best Practices

### 1. **TypeScript Strict Mode**

Enable strict type checking:

```json
{
  "compilerOptions": {
    "strict": true,
    "noUnusedLocals": true,
    "noUnusedParameters": true
  }
}
```

### 2. **Component Structure**

Keep components small and focused:

```typescript
// Good: Small, single-responsibility component
export function UserCard({ user }: { user: User }) {
  return <div>{user.name}</div>;
}

// Bad: Large, multi-responsibility component
export function Dashboard() {
  // 500 lines of code...
}
```

### 3. **State Management**

Use OS state management for app-level state:

```typescript
export default function App({ context }: NativeAppProps) {
  const { state } = context;
  
  // Use OS state for persistence
  useEffect(() => {
    const value = state.get('mydata');
    // ...
  }, []);
  
  // Use React state for UI-only state
  const [isOpen, setIsOpen] = useState(false);
}
```

### 4. **Service Calls**

Use executor for backend services:

```typescript
// Good: Use executor
const result = await context.executor.execute('filesystem.read', { path });

// Bad: Direct HTTP calls
const result = await fetch('/api/filesystem/read');
```

### 5. **Error Handling**

Handle errors gracefully:

```typescript
try {
  const result = await context.executor.execute('service', params);
} catch (error) {
  console.error('Service call failed:', error);
  // Show user-friendly message
}
```

### 6. **Performance**

Optimize for performance:

```typescript
// Use React.memo for expensive components
export const ExpensiveComponent = React.memo(({ data }) => {
  // ...
});

// Use useMemo for expensive calculations
const sortedData = useMemo(() => 
  data.sort((a, b) => a.value - b.value),
  [data]
);

// Use useCallback for event handlers
const handleClick = useCallback(() => {
  // ...
}, [deps]);
```

### 7. **Styling**

Use CSS modules or styled components:

```typescript
// CSS Modules
import styles from './App.module.css';

export function App() {
  return <div className={styles.container}>...</div>;
}

// Or use clsx for conditional classes
import clsx from 'clsx';

<div className={clsx(styles.button, isActive && styles.active)} />
```

---

## SDK Reference

### Context API

Every native app receives a `context` prop with:

#### `context.appId`
- Type: `string`
- Unique identifier for this app instance

#### `context.state`
- Type: `ComponentState`
- Reactive state management
- Methods: `get()`, `set()`, `subscribe()`, `batch()`

#### `context.executor`
- Type: `ToolExecutor`
- Execute backend services
- Method: `execute(toolId, params)`

#### `context.window`
- Type: `AppWindow`
- Window controls
- Methods: `setTitle()`, `setIcon()`, `close()`, `minimize()`, `maximize()`, `focus()`

### Available Services

#### Filesystem

```typescript
// Read file
await executor.execute('filesystem.read', { path: '/path/to/file' });

// Write file
await executor.execute('filesystem.write', { 
  path: '/path/to/file', 
  content: 'data' 
});

// List directory
await executor.execute('filesystem.list', { path: '/path' });

// Create directory
await executor.execute('filesystem.mkdir', { path: '/path/newdir' });

// Delete file/directory
await executor.execute('filesystem.delete', { path: '/path' });
```

#### Storage (Persistent Key-Value)

```typescript
// Set value
await executor.execute('storage.set', { key: 'mykey', value: data });

// Get value
const result = await executor.execute('storage.get', { key: 'mykey' });

// Remove value
await executor.execute('storage.remove', { key: 'mykey' });

// List keys
const result = await executor.execute('storage.list', {});
```

#### HTTP

```typescript
// GET request
await executor.execute('http.get', { 
  url: 'https://api.example.com',
  headers: { 'Authorization': 'Bearer token' }
});

// POST request
await executor.execute('http.post', { 
  url: 'https://api.example.com',
  body: { data: 'value' },
  headers: { 'Content-Type': 'application/json' }
});
```

#### System

```typescript
// Get system info
await executor.execute('system.info', {});

// Get current time
await executor.execute('system.time', {});

// Log message
await executor.execute('system.log', { 
  level: 'info', 
  message: 'Hello' 
});
```

### State Management

```typescript
// Set value
context.state.set('key', 'value');

// Get value
const value = context.state.get('key');

// Subscribe to changes
const unsubscribe = context.state.subscribe('key', (newValue) => {
  console.log('Value changed:', newValue);
});

// Batch updates (single notification)
context.state.batch(() => {
  context.state.set('key1', 'value1');
  context.state.set('key2', 'value2');
});
```

### Window Controls

```typescript
// Set window title
context.window.setTitle('New Title');

// Set window icon
context.window.setIcon('');

// Close window
context.window.close();

// Minimize window
context.window.minimize();

// Maximize window
context.window.maximize();

// Focus window
context.window.focus();
```

---

## Troubleshooting

### Build Errors

**Problem**: TypeScript errors during build

**Solution**:
```bash
# Type check without emitting
npm run type-check

# Check for any TSConfig issues
cat tsconfig.json | jq .
```

**Problem**: Module not found

**Solution**:
```bash
# Reinstall dependencies
rm -rf node_modules package-lock.json
npm install
```

### Development Server Issues

**Problem**: Port already in use

**Solution**:
```bash
# Kill process on port 5174
lsof -ti :5174 | xargs kill -9
```

**Problem**: HMR not working

**Solution**:
```bash
# Restart dev server
npm run dev -- --force

# Or clear Vite cache
rm -rf node_modules/.vite
```

### Runtime Errors

**Problem**: Service calls failing

**Solution**:
```typescript
// Check backend is running
// Check service name is correct
// Check parameters are valid

// Add error handling
try {
  await executor.execute('service', params);
} catch (error) {
  console.error('Service failed:', error);
}
```

**Problem**: State not updating

**Solution**:
```typescript
// Ensure you're subscribing to the right key
context.state.subscribe('mykey', (value) => {
  console.log('Value changed:', value);
});

// Check state is being set
context.state.set('mykey', 'newvalue');
```

---

## Advanced Topics

### Custom Vite Plugins

Add custom plugins to your app's Vite config:

```typescript
import { defineNativeAppConfig } from '../vite.config.base';
import myPlugin from 'vite-plugin-example';

export default defineNativeAppConfig('MyApp', {
  plugins: [myPlugin()],
});
```

### Code Splitting

Split large components for lazy loading:

```typescript
import { lazy, Suspense } from 'react';

const HeavyComponent = lazy(() => import('./components/HeavyComponent'));

export function App({ context }: NativeAppProps) {
  return (
    <Suspense fallback={<div>Loading...</div>}>
      <HeavyComponent />
    </Suspense>
  );
}
```

### Testing

Add tests to your app:

```bash
npm install --save-dev vitest @testing-library/react @testing-library/jest-dom
```

```typescript
// src/App.test.tsx
import { describe, it, expect } from 'vitest';
import { render, screen } from '@testing-library/react';
import App from './App';

describe('App', () => {
  it('renders title', () => {
    render(<App context={mockContext} />);
    expect(screen.getByText('My App')).toBeInTheDocument();
  });
});
```

---

## Resources

- **SDK Reference**: `ui/src/core/sdk/index.ts`
- **Example Apps**: `apps/native/`
- **Shared Vite Config**: `apps/native/vite.config.base.ts`
- **Native Apps Plan**: `docs/NATIVE_APPS_PLAN.md`
- **Architecture**: `docs/ARCHITECTURE.md`

---

## Getting Help

1. Check this guide and the SDK reference
2. Look at example apps in `apps/native/`
3. Run validation: `make validate-native-apps`
4. Check logs: `make logs`

---

**Happy Building!**
