# Dynamic Component Hooks

High-performance React hooks for dynamic component rendering with external state management.

## Overview

This directory contains specialized hooks optimized for dynamic UI rendering with AI-generated blueprints. All hooks are designed to work with the `ComponentState` external store and `ToolExecutor` backend integration.

## Hooks

### `useSyncState`

Efficient external state subscription using React 18's `useSyncExternalStore`.

**Benefits:**
- Zero unnecessary re-renders (only updates when subscribed value changes)
- Eliminates `forceUpdate` anti-pattern
- Optimal React 18 integration
- Concurrent mode safe

**Usage:**
```tsx
import { useSyncState } from "@/features/dynamics";

const MyComponent = ({ component, state }) => {
  const value = useSyncState(state, component.id, "default");
  
  return <div>{value}</div>;
};
```

**When to use:**
- Any component that needs to read from ComponentState
- Non-input components (buttons, text, badges, etc.)
- Read-only state subscriptions

### `useInputState`

High-performance input handling with RAF batching and smart debouncing.

**Features:**
- **Instant visual feedback** - typing appears immediately with synchronous state updates
- **Controlled input compatible** - proper synchronous updates for all keystrokes (including space)
- **Smart debouncing** - only backend calls are debounced, not UI updates
- **Auto-flush on blur** - ensures pending backend calls execute immediately
- **Configurable** - adjust debounce timing for backend events

**Usage:**
```tsx
import { useInputState } from "@/features/dynamics";

const MyInput = ({ component, state, executor }) => {
  const { value, onChange, onBlur } = useInputState(
    component, 
    state, 
    executor,
    {
      eventDebounce: 300, // Backend event debounce (ms)
    }
  );
  
  return (
    <input 
      value={value} 
      onChange={(e) => onChange(e.target.value)}
      onBlur={onBlur}
    />
  );
};
```

**Options:**
- `eventDebounce` (default: 300ms) - Debounce time for backend event calls

**When to use:**
- Text inputs
- Textareas
- Any component with rapid user input

### `useComponent`

Legacy hook for general component event handling and state management.

**Usage:**
```tsx
import { useComponent } from "@/features/dynamics";

const MyButton = ({ component, state, executor }) => {
  const { localState, handleEvent } = useComponent(component, state, executor);
  
  return (
    <button onClick={() => handleEvent("click")}>
      {component.props?.text}
    </button>
  );
};
```

**When to use:**
- Non-input interactive components (buttons, links, etc.)
- Components that need event handlers without state
- Legacy components not yet migrated to new hooks

**Note:** For input components, prefer `useInputState` for better performance.

### `useRegistry`

Access to the component registry for type checking and rendering.

**Usage:**
```tsx
import { useRegistry } from "@/features/dynamics";

const MyRenderer = () => {
  const { getRenderer, hasRenderer, getTypes } = useRegistry();
  
  if (!hasRenderer("button")) {
    console.warn("Button renderer not registered");
  }
  
  return null;
};
```

## Performance Comparison

### Before (Old System)
```tsx
// ❌ Problems:
// - Double re-renders (setLocalState + forceUpdate)
// - 500ms debounce on typing (laggy)
// - No RAF batching
// - Spacebar events sometimes lost

const { localState, handleDebouncedEvent } = useComponent(component, state, executor);

<input
  value={localState ?? ""}
  onChange={(e) => {
    state.set(component.id, e.target.value);
    handleDebouncedEvent("change", { value: e.target.value }, 500); // Laggy!
  }}
/>
```

### After (New System)
```tsx
// ✅ Improvements:
// - Single efficient re-render via useSyncExternalStore
// - 300ms debounce only for backend, instant UI
// - Synchronous state updates for controlled inputs
// - All keyboard events work perfectly (including space!)

const { value, onChange, onBlur } = useInputState(component, state, executor, {
  eventDebounce: 300,
});

<input
  value={value}
  onChange={(e) => onChange(e.target.value)} // Instant & synchronous!
  onBlur={onBlur}
/>
```

## Architecture

```
User Types → onChange
  ↓
  ├─→ Visual Update (instant & synchronous)
  │   └─→ state.set() → useSyncExternalStore → re-render
  │
  └─→ Backend Event (debounced 300ms)
      └─→ executor.execute() → AI backend
```

**Why synchronous updates?**
React controlled inputs require synchronous state updates in the onChange handler. Async updates (via RAF, setTimeout, etc.) cause React to reject keystrokes, especially space and special characters.

## Migration Guide

### Migrating from `useComponent` to `useInputState`

**Before:**
```tsx
export const Input = ({ component, state, executor }) => {
  const { localState, handleDebouncedEvent } = useComponent(component, state, executor);
  
  return (
    <input
      value={localState ?? ""}
      onChange={(e) => {
        state.set(component.id, e.target.value);
        if (component.on_event?.change) {
          handleDebouncedEvent("change", { value: e.target.value }, 500);
        }
      }}
    />
  );
};
```

**After:**
```tsx
export const Input = ({ component, state, executor }) => {
  const { value, onChange, onBlur } = useInputState(component, state, executor);
  
  return (
    <input
      value={value}
      onChange={(e) => onChange(e.target.value)}
      onBlur={onBlur}
    />
  );
};
```

### Migrating non-input components to `useSyncState`

**Before:**
```tsx
export const Badge = ({ component, state, executor }) => {
  const { localState } = useComponent(component, state, executor);
  
  return <span>{localState}</span>;
};
```

**After:**
```tsx
export const Badge = ({ component, state, executor }) => {
  const value = useSyncState(state, component.id);
  
  return <span>{value}</span>;
};
```

## Best Practices

1. **Use the right hook for the job:**
   - `useInputState` → Text inputs, textareas
   - `useSyncState` → Read-only state access
   - `useComponent` → Event handlers, buttons, interactive elements

2. **Configure debounce appropriately:**
   - 100-200ms for fast interactions
   - 300-500ms for expensive backend operations
   - 0ms for instant backend sync (use with caution)

3. **Always include `onBlur`:**
   - Ensures pending backend calls execute immediately
   - Critical for form submission flows

## Troubleshooting

### Input still feels laggy
- Check `eventDebounce` setting (should be 300ms or less)
- Ensure you're using `useInputState`, not `useComponent`
- Verify state updates are synchronous (no async wrappers)

### Changes not saving
- Add `onBlur={onBlur}` to your input
- Check that `component.on_event?.change` exists
- Verify backend executor is properly configured

### TypeScript errors
- Import types from `@/features/dynamics`
- Use `InputStateReturn` type for hook returns
- Ensure ComponentState and ToolExecutor types are correct

## Testing

```tsx
import { renderHook } from "@testing-library/react";
import { useInputState } from "./useInputState";

describe("useInputState", () => {
  it("provides instant value updates", () => {
    const { result } = renderHook(() =>
      useInputState(mockComponent, mockState, mockExecutor)
    );
    
    act(() => {
      result.current.onChange("new value");
    });
    
    expect(result.current.value).toBe("new value");
  });
});
```

## See Also

- [Dynamic Components README](../README.md)
- [State Management](../state/state.ts)
- [Component Renderer](../rendering/renderer.tsx)
