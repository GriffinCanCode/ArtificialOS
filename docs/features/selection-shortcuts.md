# Selection Shortcuts Architecture

**Status**: Implemented  
**Pattern**: Factory Pattern  
**Last Updated**: 2025-01-17

## Overview

This document describes the implementation of selection keyboard shortcuts (Cmd+A, Cmd+I, Escape) in AgentOS. The factory pattern approach maintains a single definition of shortcuts while allowing context-specific behavior across different components.

## Problem

Selection shortcuts need to:
1. Work in input fields with native browser behavior
2. Work in desktop icon grid selecting icons
3. Work in file explorers selecting files
4. Work in other contexts (text editors, lists, etc.)
5. Be centrally defined (DRY principle)
6. Be discoverable through ShortcutRegistry
7. Allow context-specific implementations

### Failed Approaches

Hardcoding shortcuts in each component violates DRY principles and prevents discoverability.

Using placeholder handlers creates confusion with non-functional implementations.

## Solution: Factory Pattern

### Architecture

```
         createSelectionCommands() Factory            
  (Central definition of shortcuts + interface)      
                         
           ├─┬─┬
           ▼ ▼ ▼
    Desktop      File        Other   
    (Icons)     Explorer    Context  
           
         ▼       ▼       ▼
    Icon Store   File Store  Custom Logic
```

### Implementation

#### 1. Define Interface & Factory

**File**: `ui/src/features/input/shortcuts/commands/selection.ts`

```typescript
// Interface that components must implement
export interface SelectionActions {
  selectAll: () => void;
  clearSelection: () => void;
  invertSelection?: () => void; // Optional
}

// Factory creates shortcuts wired to actions
export function createSelectionCommands(
  actions: SelectionActions,
  scope: ShortcutScope = "desktop"
): ShortcutConfig[] {
  return [
    {
      id: `selection.all.${scope}`,
      sequence: "$mod+a",
      handler: (event) => {
        // Input field detection - allow native behavior
        const target = event.target as HTMLElement;
        if (isInputField(target)) {
          return false; // Browser handles it
        }
        actions.selectAll(); // Context handles it
      },
    },
    // ... other commands
  ];
}
```

#### 2. Use in Components

**File**: `ui/src/ui/components/layout/Desktop.tsx`

```typescript
import { createSelectionCommands } from "../../../features/input";

export const Desktop = () => {
  const iconActions = useIconActions();
  const icons = useIcons();
  const selectedIcons = useSelectedIcons();

  // Register selection shortcuts for desktop icon grid
  useShortcuts(
    createSelectionCommands(
      {
        selectAll: iconActions.selectAll,
        clearSelection: iconActions.clearSelection,
        invertSelection: () => {
          // Custom invert logic for icons
          const allIds = icons.map(i => i.id);
          const selectedSet = new Set(selectedIcons.map(i => i.id));
          const invertedIds = allIds.filter(id => !selectedSet.has(id));
          iconActions.clearSelection();
          invertedIds.forEach(id => iconActions.select(id, true));
        },
      },
      "desktop" // Scope
    )
  );
};
```

#### 3. File Explorer Example

```typescript
export const FileExplorer = () => {
  const fileActions = useFileActions();
  
  useShortcuts(
    createSelectionCommands(
      {
        selectAll: fileActions.selectAll,
        clearSelection: fileActions.clearSelection,
        // No invert for this context
      },
      "window" // Different scope
    )
  );
};
```

## Benefits

### Single Source of Truth

One definition of what selection shortcuts are (`createSelectionCommands`)
One place to update if shortcuts change
One interface (`SelectionActions`) that all contexts must implement

### DRY

Instead of repeating in every component:
```typescript
useShortcuts([{
  id: "my-select-all",
  sequence: "$mod+a",
  handler: ...
}]);
```

Just call the factory:
```typescript
useShortcuts(createSelectionCommands(actions));
```

### Type Safety

TypeScript enforces the interface:
```typescript
createSelectionCommands({
  selectAll: myActions.selectAll,     // Required
  clearSelection: myActions.clear,     // Required
  invertSelection: myActions.invert,   // Optional
  randomMethod: () => {},              // Error: not in interface
});
```

### Discoverable

All selection shortcuts are registered in `ShortcutRegistry`
Can query: `registry.findBySequence("$mod+a")`
Shows in shortcuts panel/help
Searchable and inspectable

### Context-Aware

In input fields: Native browser behavior (text selection)
On desktop: Icon grid selection
In file explorer: File selection
Automatic detection of context via event.target

### Extensible

Adding a new context is trivial:
```typescript
// New component just implements the interface
useShortcuts(
  createSelectionCommands({
    selectAll: myCustomSelectAll,
    clearSelection: myCustomClear,
  }, "my-scope")
);
```

### Testable

```typescript
// Easy to test the factory
const commands = createSelectionCommands(mockActions);
expect(commands).toHaveLength(3);
expect(commands[0].sequence).toBe("$mod+a");

// Easy to test components
const mockSelectAll = jest.fn();
render(<Desktop actions={{ selectAll: mockSelectAll, ... }} />);
userEvent.keyboard("{Meta>}a{/Meta}");
expect(mockSelectAll).toHaveBeenCalled();
```

### Self-Documenting

```typescript
// Clear what's needed:
interface SelectionActions {
  selectAll: () => void;        // Required
  clearSelection: () => void;    // Required
  invertSelection?: () => void;  // Optional - not all contexts support this
}
```

## Migration Path

If adding more selection-related shortcuts:

1. Add to factory (single change):
```typescript
export function createSelectionCommands(actions: SelectionActions) {
  return [
    // ... existing
    {
      id: `selection.selectSimilar.${scope}`,
      sequence: "$mod+shift+a",
      handler: actions.selectSimilar, // New action
    },
  ];
}
```

2. Update interface:
```typescript
interface SelectionActions {
  selectSimilar?: () => void; // Optional
}
```

3. All components automatically get the new shortcut definition
4. Components opt-in by providing the action

## Comparison with Alternatives

| Solution | DRY | Type Safe | Discoverable | Context-Aware |
|----------|-----|-----------|--------------|---------------|
| Factory Pattern | Yes | Yes | Yes | Yes |
| Placeholder handlers | Yes | No | Yes | Yes |
| Component-specific | No | Maybe | No | Yes |
| Global context provider | Yes | Maybe | Yes | Maybe |

## Benefits in Practice

1. **Developer onboarding**: New developers see `SelectionActions` interface and know exactly what to implement

2. **Consistency**: Impossible to have different shortcuts for selection across the app

3. **Refactoring**: Change `$mod+a` to `$mod+shift+a` in ONE place

4. **Debugging**: Can inspect all registered selection shortcuts via registry

5. **Documentation**: The factory IS the documentation

## Future Enhancements

Possible improvements while maintaining architecture:

### Selection Hints

```typescript
createSelectionCommands(actions, "desktop", {
  hints: {
    selectAll: "Select all icons",
    invertSelection: "Invert icon selection",
  }
});
```

### Custom Sequences

```typescript
createSelectionCommands(actions, "desktop", {
  sequences: {
    selectAll: "$mod+shift+a", // Override default
  }
});
```

### Conditional Actions

```typescript
createSelectionCommands({
  selectAll: () => items.length > 0 ? doSelect() : showToast("Nothing to select"),
  clearSelection: () => selected.length > 0 ? doClear() : undefined,
});
```

## Summary

The factory pattern with `createSelectionCommands()` provides:

- Single source of truth for shortcuts
- Eliminated code duplication
- Type safety through TypeScript
- Easy extension for new contexts
- Self-documenting interface
- Integration with existing architecture (ShortcutRegistry)

This approach scales to any number of contexts while maintaining simplicity.

---

**Related Documentation**:
- [Code Standards 2025](CODE_STANDARDS_2025.md)
- [Architecture](ARCHITECTURE.md)
- [Shortcuts README](../ui/src/features/input/shortcuts/README.md)

