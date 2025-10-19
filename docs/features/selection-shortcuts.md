# Selection Shortcuts

Implementation of selection keyboard shortcuts (Cmd+A, Cmd+I, Escape) using factory pattern for maintainability and extensibility.

## Overview

Selection shortcuts are defined centrally and wired to context-specific implementations via a factory pattern. This ensures single source of truth while allowing each component to handle selection differently.

## Architecture

### Factory Pattern Approach

```
createSelectionCommands() Factory
  (Central definition)
         
         ├─ Desktop (Icons)
         ├─ File Explorer
         └─ Other Contexts
         
         ▼ ▼ ▼
      Component Actions
```

## Problem Statement

Selection shortcuts need to:
1. Work in input fields with native browser behavior (Cmd+A selects text)
2. Work in desktop icon grid selecting icons
3. Work in file explorers selecting files
4. Work in other contexts (lists, etc.)
5. Be centrally defined (DRY principle)
6. Be discoverable through ShortcutRegistry
7. Allow context-specific implementations

## Solution: Factory Pattern

### Implementation

#### 1. Define Interface & Factory

**File**: `ui/src/features/input/shortcuts/commands/selection.ts`

```typescript
// Interface that components must implement
export interface SelectionActions {
  selectAll: () => void;
  clearSelection: () => void;
  invertSelection?: () => void; // Optional - not all contexts support this
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
      label: "Select All",
      handler: (event) => {
        // In input fields, allow native browser behavior
        const target = event.target as HTMLElement;
        if (target && (target.tagName === "INPUT" || target.tagName === "TEXTAREA" || target.isContentEditable)) {
          return false; // Browser handles it
        }
        actions.selectAll(); // Context handles it
      },
    },
    {
      id: `selection.clear.${scope}`,
      sequence: "Escape",
      label: "Clear Selection",
      handler: () => {
        actions.clearSelection();
      },
    },
    // Invert command only if action is provided
    ...(actions.invertSelection ? [{
      id: `selection.invert.${scope}`,
      sequence: "$mod+i",
      label: "Invert Selection",
      handler: () => {
        actions.invertSelection!();
      },
    }] : []),
  ];
}
```

#### 2. Use in Components

**Desktop Component:**
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

**File Explorer Component:**
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

- One definition of what selection shortcuts are (`createSelectionCommands`)
- One place to update if shortcuts change
- One interface (`SelectionActions`) that all contexts must implement

### DRY (Don't Repeat Yourself)

Instead of repeating shortcut definitions:
```typescript
// Bad - repeated in every component
useShortcuts([{
  id: "my-select-all",
  sequence: "$mod+a",
  handler: ...
}]);
```

Use the factory:
```typescript
// Good - centralized definition
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

### Discoverability

All selection shortcuts are registered in `ShortcutRegistry`:
- Query: `registry.findBySequence("$mod+a")`
- Shows in shortcuts panel/help
- Searchable and inspectable

### Context-Aware

Shortcuts intelligently handle context:
- In input fields: Native browser behavior
- On desktop: Icon grid selection
- In file explorer: File selection
- Automatic detection via event.target

### Extensibility

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

## Shortcuts Registered

| Shortcut | Scope | Action | Notes |
|----------|-------|--------|-------|
| Cmd/Ctrl+A | All | Select All | Native in input fields |
| Escape | All | Clear Selection | Clears current selection |
| Cmd/Ctrl+I | Desktop/Optional | Invert Selection | Only when action provided |

## Migration Path

To add more selection-related shortcuts:

1. Add to factory:
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

3. All components automatically get the new shortcut

## Comparison with Alternatives

| Solution | DRY | Type Safe | Discoverable | Context-Aware |
|----------|-----|-----------|--------------|---------------|
| **Factory Pattern** | Yes | Yes | Yes | Yes |
| Placeholder handlers | Yes | No | Yes | Yes |
| Component-specific | No | Maybe | No | Yes |
| Global context provider | Yes | Maybe | Yes | Maybe |

## Integration with Shortcut System

The factory integrates with the centralized shortcut system:

```typescript
// Shortcuts are automatically registered
useShortcuts(createSelectionCommands(actions));

// Can be queried later
const selectionShortcuts = registry.getByCategory("selection");
const cmdA = registry.findBySequence("$mod+a");
```

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
- Integration with existing architecture

This approach scales to any number of contexts while maintaining simplicity and clarity.

---

**Related Documentation**:
- [Code Standards 2025](CODE_STANDARDS_2025.md)
- [Architecture](ARCHITECTURE.md)
- [Shortcuts README](../ui/src/features/input/shortcuts/README.md)

