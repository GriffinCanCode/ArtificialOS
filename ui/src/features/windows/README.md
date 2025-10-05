# Windows Module

Centralized, modular window management architecture for the OS desktop environment.

## Architecture

```
windows/
├── core/              # Pure functions and types
│   ├── types.ts       # TypeScript definitions
│   ├── viewport.ts    # Viewport calculations
│   ├── bounds.ts      # Position/size utilities
│   ├── snap.ts        # Snap-to-edge logic
│   └── constraints.ts # Window constraints
├── store/             # State management
│   └── store.ts       # Zustand store
├── hooks/             # React hooks
│   ├── useSnap.ts     # Snap functionality
│   ├── useKeyboard.ts # Keyboard shortcuts
│   ├── useDrag.ts     # Drag handling
│   └── useManager.ts  # High-level interface
├── utils/             # Helper utilities
│   ├── animations.ts  # Animation helpers
│   └── sync.ts        # Backend sync
└── index.ts           # Public exports
```

## Usage

### Window Manager

```typescript
import { useManager } from "@/windows";

function Desktop() {
  const manager = useManager();
  
  const handleLaunch = () => {
    manager.open("app-id", "App Title", uiSpec, "🚀");
  };
  
  return <div>Windows: {manager.windows.length}</div>;
}
```

### Direct Store Access

```typescript
import { useStore, useActions } from "@/windows";

function MyComponent() {
  const windows = useStore((state) => state.windows);
  const { open, close, focus } = useActions();
  
  // Use actions...
}
```

### Snap-to-Edge

```typescript
import { useSnap } from "@/windows";

function WindowComponent() {
  const { preview, handleDrag, handleDragEnd } = useSnap();
  
  // Use in drag handlers...
}
```

### Keyboard Shortcuts

```typescript
import { useKeyboard } from "@/windows";

function App() {
  const windows = useStore((state) => state.windows);
  const actions = useActions();
  
  // Enables Alt+Tab, Cmd+W, Cmd+M automatically
  useKeyboard(windows, {
    onFocus: actions.focus,
    onMinimize: actions.minimize,
    onClose: actions.close,
  });
}
```

## Core Functions

### Viewport

```typescript
import { getViewport, getAvailable, getCenterPosition } from "@/windows";

const viewport = getViewport(); // { width, height }
const available = getAvailable(); // Available space minus menubar/taskbar
const center = getCenterPosition({ width: 800, height: 600 });
```

### Bounds

```typescript
import { 
  constrainBounds, 
  getMaximizedBounds, 
  getCascadePosition 
} from "@/windows";

const constrained = constrainBounds(bounds); // Keep in viewport
const maximized = getMaximizedBounds(); // Full available space
const position = getCascadePosition(3); // Cascade 3rd window
```

### Snap

```typescript
import { detectZone, getZoneBounds, isValidZone } from "@/windows";

const zone = detectZone({ x: 10, y: 10 }); // Zone.LEFT
const bounds = getZoneBounds(zone); // Bounds for left half
const valid = isValidZone(zone); // true if not Zone.NONE
```

### Constraints

```typescript
import { constrainSize, isValidSize } from "@/windows";

const size = constrainSize({ width: 200, height: 100 }); // Applies min/max
const valid = isValidSize({ width: 500, height: 400 }); // Validates
```

## Design Principles

1. **Modularity**: Each module handles a specific concern
2. **Type Safety**: Full TypeScript with strict types
3. **Testability**: Pure functions for easy unit testing
4. **Reusability**: Composable utilities and hooks
5. **Performance**: Optimized calculations and state updates
6. **Extensibility**: Easy to add new features and behaviors

## Best Practices

- Use `useManager()` for high-level operations
- Use direct store access for performance-critical paths
- Pure functions (core/) are unit-testable without React
- Hooks encapsulate React-specific logic
- Store handles all state mutations
- Utils provide specialized functionality

## Migration Guide

Old pattern:
```typescript
import { useWindowStore, useWindowActions } from "@/store/windowStore";
import { useWindowSnap } from "@/hooks/useWindowSnap";
```

New pattern:
```typescript
import { useStore, useActions, useSnap } from "@/windows";
```

All functionality has been preserved and enhanced with better organization.
