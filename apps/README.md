# Apps Directory

This directory contains all applications for the OS, organized by type.

## Directory Structure

```
apps/
├── blueprint/          # Blueprint apps (.bp files) - AI-generated with prebuilt components
├── native/             # Native TypeScript/React apps - Hand-coded with custom components
├── native-proc/        # Native process apps - Executable programs (Python, CLI, etc.)
└── dist/               # Built native app bundles
```

## Application Types

### 1. Blueprint Apps (`blueprint/`)

**Format**: JSON/Blueprint DSL (`.bp` files)  
**Development**: AI-generated  
**Components**: Prebuilt (Button, Input, Text, etc.)  
**Execution**: Browser only  

**Example**:
```json
{
  "type": "app",
  "title": "Calculator",
  "components": [
    { "type": "button", "text": "Click Me" }
  ]
}
```

**Use Cases**: Quick apps, AI-generated UIs, simple forms

### 2. Native Web Apps (`native/`)

**Format**: TypeScript/React  
**Development**: Hand-coded  
**Components**: Custom (write your own JSX/TSX)  
**Execution**: Browser only  
**Dependencies**: Any npm packages  

**Key Features**:
- ✅ Full React with hooks, custom components
- ✅ Import any npm packages (Monaco Editor, Chart.js, etc.)
- ✅ Write your own JSX/TSX (NO prebuilt components)
- ✅ Access OS APIs via SDK (`executor`, `state`, `window`)
- ✅ Build to single JavaScript bundle

**Use Cases**: Complex UIs, code editors, file explorers, dashboards

### 3. Native Process Apps (`native-proc/`)

**Format**: Executable programs  
**Development**: Any language (Python, Rust, Go, Shell, etc.)  
**Components**: N/A (runs as OS process)  
**Execution**: Actual OS processes  
**UI**: Terminal, headless, or custom WebSocket-based UI  

**Use Cases**: CLI tools, Python scripts, system utilities, data processing

## Creating a New Native Web App

### Quick Start

```bash
# Create new app
./scripts/create-native-app.sh "My Awesome App"

# Navigate to app directory
cd apps/native/my-awesome-app

# Install dependencies
npm install

# Start development server
npm run dev

# Build for production
npm run build
```

### App Structure

```
my-awesome-app/
├── manifest.json          # App metadata (required)
├── package.json           # npm dependencies
├── tsconfig.json          # TypeScript config
├── vite.config.ts         # Build config
└── src/
    ├── index.tsx          # Entry point (exports default component)
    ├── App.tsx            # Main app component
    ├── components/        # Custom components
    ├── hooks/             # Custom hooks
    └── styles/            # CSS files
```

### Example App

**src/App.tsx**:
```typescript
import React, { useState, useEffect } from 'react';
import type { NativeAppProps } from '@os/sdk';

export default function App({ context }: NativeAppProps) {
  const { state, executor, window } = context;
  const [data, setData] = useState([]);

  // Load data on mount
  useEffect(() => {
    async function init() {
      const result = await executor.execute('storage.get', { key: 'mydata' });
      setData(result?.value || []);
    }
    init();
  }, []);

  // Save data
  const save = async () => {
    await executor.execute('storage.set', { key: 'mydata', value: data });
    window.setTitle('My App (saved)');
  };

  return (
    <div>
      <h1>My Awesome App</h1>
      <button onClick={save}>Save</button>
    </div>
  );
}
```

## SDK API Reference

### Context

```typescript
interface NativeAppContext {
  appId: string;              // Unique app instance ID
  state: ComponentState;      // Reactive state management
  executor: ToolExecutor;     // Execute backend services
  window: AppWindow;          // Window controls
}
```

### State Management

```typescript
// Set/get state
context.state.set('key', 'value');
const value = context.state.get('key');

// Subscribe to changes
const unsubscribe = context.state.subscribe('key', (newValue) => {
  console.log('Changed:', newValue);
});
```

### Service Calls

```typescript
// Filesystem
await executor.execute('filesystem.read', { path: '/path/to/file' });
await executor.execute('filesystem.write', { path: '/path', content: 'data' });
await executor.execute('filesystem.list', { path: '/path' });

// Storage
await executor.execute('storage.set', { key: 'mykey', value: {...} });
await executor.execute('storage.get', { key: 'mykey' });

// HTTP
await executor.execute('http.get', { url: 'https://api.example.com' });
```

### Window Controls

```typescript
window.setTitle('New Title');
window.setIcon('🎨');
window.close();
window.minimize();
window.maximize();
```

## Building Apps

### Build All Native Apps

```bash
./scripts/build-native-apps.sh
```

This will:
1. Find all apps in `apps/native/`
2. Install dependencies (if needed)
3. Build each app with Vite
4. Output bundles to `apps/dist/<app-id>/`

### Individual App Build

```bash
cd apps/native/my-app
npm run build
```

## Manifest Format

**manifest.json**:
```json
{
  "id": "my-app",
  "name": "My App",
  "type": "native_web",
  "version": "1.0.0",
  "icon": "🚀",
  "category": "utilities",
  "author": "Your Name",
  "description": "What your app does",
  "permissions": ["READ_FILE", "WRITE_FILE", "NETWORK_ACCESS"],
  "services": ["filesystem", "storage", "http"],
  "exports": {
    "component": "App"
  },
  "tags": ["utility", "productivity"]
}
```

## Permissions

Available permissions:
- `STANDARD` - Basic access
- `READ_FILE` - Read files
- `WRITE_FILE` - Write files
- `CREATE_FILE` - Create files
- `DELETE_FILE` - Delete files
- `LIST_DIRECTORY` - List directories
- `SPAWN_PROCESS` - Spawn OS processes
- `NETWORK_ACCESS` - Make HTTP requests

## Development Tips

### 1. Hot Reload

The development server (`npm run dev`) supports hot module replacement (HMR). Changes are reflected immediately.

### 2. Using npm Packages

Install any package:
```bash
npm install lodash
npm install @types/lodash --save-dev
```

Use in your app:
```typescript
import _ from 'lodash';
```

### 3. Debugging

Use browser DevTools to debug your app. The SDK provides logging:
```typescript
console.log('Debug info:', data);
```

### 4. State Management

For complex state, consider using React Context or state management libraries (Zustand, Jotai, etc.).

### 5. Styling

Use CSS, CSS Modules, or CSS-in-JS libraries (styled-components, emotion, etc.).

## Examples

See `apps/native/` for example apps once Phase 2+ are implemented.

## Next Steps

1. Create your first native app
2. Explore the SDK API
3. Build something awesome!

For more information, see `/docs/NATIVE_APPS_PLAN.md`.
