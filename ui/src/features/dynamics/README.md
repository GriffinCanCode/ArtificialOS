# Dynamics Module

Modern, extensible component rendering system for AI-generated applications.

## Architecture

```
dynamics/
├── core/               # Pure types, registry, validation
│   ├── types.ts        # TypeScript definitions
│   ├── registry.ts     # Component registry
│   ├── constants.ts    # Shared constants
│   ├── validation.ts   # Zod validation utilities
│   └── DynamicRenderer.tsx  # Main orchestrator
├── hooks/              # React hooks
│   ├── useComponent.ts    # State + events
│   └── useRegistry.ts     # Registry access
├── components/         # Individual renderers (23 total)
│   ├── primitives/     # Button, Input, Text, etc.
│   ├── layout/         # Container, Grid, List
│   ├── forms/          # Select, Textarea
│   ├── media/          # Image, Video, Audio, Canvas
│   ├── ui/             # Badge, Card, Divider, Modal, Tabs
│   └── special/        # AppShortcut, Iframe, Progress
├── schemas/            # Zod validation schemas
│   ├── primitives.ts
│   ├── layout.ts
│   ├── forms.ts
│   ├── media.ts
│   ├── ui.ts
│   └── special.ts
├── rendering/          # Rendering system
│   ├── renderer.tsx    # Lightweight factory (84 lines)
│   ├── register.ts     # Auto-registration
│   ├── Builder.tsx     # Build progress UI
│   └── virtual.tsx     # Virtual scrolling
├── execution/          # Tool executor
│   └── executor.ts     # Backend integration
├── state/              # State management
│   └── state.ts        # Observable state
└── styles/             # CSS modules
```

## Key Improvements

### Before (Old Architecture)
- ❌ **793-line monolithic switch statement**
- ❌ Hard to maintain
- ❌ Hard to test individual components
- ❌ Hard to extend with new components
- ❌ No type safety for props
- ❌ No validation

### After (New Architecture)
- ✅ **84-line lightweight factory**
- ✅ Each component in its own file (~30-40 lines)
- ✅ Extensible registry pattern
- ✅ Full Zod validation for all props
- ✅ Strong TypeScript types
- ✅ Easily testable
- ✅ ~90% reduction in renderer.tsx size

## Usage

### Basic Rendering

```tsx
import { ComponentRenderer } from "@/dynamics";

<ComponentRenderer 
  component={blueprintComponent}
  state={componentState}
  executor={toolExecutor}
/>
```

### Adding New Components

```tsx
// 1. Create component renderer
// components/custom/MyComponent.tsx
import React from "react";
import type { BaseComponentProps } from "../../core/types";
import { useComponent } from "../../hooks/useComponent";

export const MyComponent: React.FC<BaseComponentProps> = ({ 
  component, 
  state, 
  executor 
}) => {
  const { handleEvent } = useComponent(component, state, executor);
  
  return (
    <div onClick={() => handleEvent("click")}>
      {component.props?.text}
    </div>
  );
};

// 2. Create validation schema (optional)
// schemas/custom.ts
import { z } from "zod";

export const myComponentSchema = z.object({
  text: z.string(),
  variant: z.enum(["default", "special"]).optional(),
});

// 3. Register component
// rendering/register.ts
import { MyComponent } from "../components/custom/MyComponent";
import { myComponentSchema } from "../schemas/custom";

const componentRenderers = [
  // ... existing components
  { 
    type: "my-component", 
    render: MyComponent, 
    schema: myComponentSchema, 
    category: "custom" 
  },
];
```

### Using Hooks

```tsx
import { useComponent, useRegistry } from "@/dynamics";

// In a component renderer
const { localState, handleEvent, handleDebouncedEvent } = useComponent(
  component,
  state,
  executor
);

// Access registry
const { getRenderer, hasRenderer, getTypes } = useRegistry();
```

### Validation

```tsx
import { validateComponentProps, safeParseProps } from "@/dynamics";
import { buttonSchema } from "@/dynamics/schemas";

// Strict validation (throws on error)
const result = validateComponentProps(
  props, 
  buttonSchema, 
  "button", 
  true
);

// Safe validation (returns original props on error)
const validatedProps = safeParseProps(props, buttonSchema);
```

## Design Principles

1. **Separation of Concerns**: Each component in its own file
2. **Type Safety**: Full TypeScript + Zod validation
3. **Testability**: Pure functions and isolated components
4. **Extensibility**: Registry pattern for easy additions
5. **Performance**: Memoization, virtual scrolling, minimal re-renders
6. **Maintainability**: Short files with single responsibility

## Component Categories

- **Primitives** (6): Button, Input, Text, Checkbox, Radio, Slider
- **Layout** (3): Container, Grid, List
- **Forms** (2): Select, Textarea
- **Media** (4): Image, Video, Audio, Canvas
- **UI** (5): Badge, Card, Divider, Modal, Tabs
- **Special** (3): AppShortcut, Iframe, Progress

**Total**: 23 registered components

## Testing

```bash
# Run all dynamics tests
npm test -- dynamics

# Run specific test files
npm test -- renderer.test.ts
npm test -- validation.test.ts

# Coverage
npm run test:coverage -- dynamics
```

## Performance Optimizations

- **Memoization**: React.memo on all renderers
- **Virtual Scrolling**: Automatic for lists > 50 items
- **Validation Caching**: Safe parsing with fallback
- **State Batching**: Batch state updates
- **Smart Re-renders**: Custom comparison functions

## Migration from Old System

The old `renderer.tsx` (793 lines) has been completely replaced with:
- New `renderer.tsx` (84 lines) - factory only
- 23 individual component files (~30-40 lines each)
- Automatic registration system
- Full validation layer

**No breaking changes** - the public API remains the same!
