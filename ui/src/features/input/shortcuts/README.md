# Shortcut Management System

Centralized, extensible keyboard shortcut management with conflict detection, scope management, and persistence.

## Architecture

```
shortcuts/
├── core/              # Core shortcut logic (framework-agnostic)
│   ├── types.ts       # TypeScript type definitions
│   ├── platform.ts    # Platform detection and key mapping
│   ├── parser.ts      # Shortcut sequence parsing
│   ├── formatter.ts   # Display formatting
│   ├── conflict.ts    # Conflict detection & resolution
│   └── registry.ts    # Central shortcut registry
├── store/             # Zustand state management
│   └── store.ts       # Persistent shortcut store
├── hooks/             # React hooks
│   ├── useShortcut.ts   # Single shortcut registration
│   ├── useShortcuts.ts  # Multiple shortcuts registration
│   └── useScope.ts      # Scope management
├── commands/          # Command definitions
│   ├── system.ts      # System commands
│   ├── window.ts      # Window management
│   ├── selection.ts   # Selection commands
│   ├── clipboard.ts   # Clipboard operations
│   └── index.ts       # Command registry
├── index.ts           # Public API
└── README.md          # This file
```

## Features

- **🎯 Centralized Registry**: Single source of truth for all shortcuts
- **🔄 Scope Management**: Context-aware shortcuts (global, window, desktop, app)
- **⚡ Priority System**: Automatic conflict resolution
- **💾 Persistence**: Save custom shortcuts to localStorage
- **🔍 Conflict Detection**: Detect and resolve shortcut conflicts
- **🌍 Platform Aware**: Automatic Mac/Windows/Linux adaptation
- **📊 Statistics**: Track shortcut usage and performance
- **♿ Accessibility**: Screen reader friendly formatting
- **🎨 Customizable**: Users can customize any shortcut
- **🧪 Testable**: Pure functions for easy testing

## Quick Start

### Basic Usage

```typescript
import { useShortcut } from "@/shortcuts";

function MyComponent() {
  // Register a simple shortcut
  useShortcut("my.command", {
    sequence: "$mod+k",
    label: "Open Command Palette",
    handler: () => console.log("Command triggered!"),
  });

  return <div>Press Cmd/Ctrl+K</div>;
}
```

### Multiple Shortcuts

```typescript
import { useShortcuts } from "@/shortcuts";

function MyComponent() {
  useShortcuts([
    {
      id: "save",
      sequence: "$mod+s",
      label: "Save",
      handler: () => save(),
    },
    {
      id: "open",
      sequence: "$mod+o",
      label: "Open",
      handler: () => open(),
    },
  ]);
}
```

### Scoped Shortcuts

```typescript
import { useScope, useScopedShortcuts } from "@/shortcuts";

function WindowComponent() {
  // Activate window scope
  useScope("window");

  // Register window-scoped shortcuts
  useScopedShortcuts("window", [
    {
      id: "window.close",
      sequence: "$mod+w",
      label: "Close Window",
      handler: () => closeWindow(),
    },
  ]);
}
```

## Key Concepts

### Shortcut Sequences

Shortcuts use a simple string format with `+` separators:

```typescript
"$mod+k"           // Cmd+K on Mac, Ctrl+K on Windows/Linux
"Control+Shift+p"  // Ctrl+Shift+P on all platforms
"Alt+Tab"          // Alt+Tab
"Escape"           // Single key
```

**Special modifier**: `$mod` automatically resolves to `Meta` (⌘) on Mac and `Control` on Windows/Linux.

### Scopes

Scopes determine where shortcuts are active:

- **global**: Active everywhere (highest priority)
- **window**: Active in focused window
- **desktop**: Active on desktop only
- **creator**: Active in creator mode
- **hub**: Active in hub
- **terminal**: Active in terminal
- **app**: Active in specific app

### Priority

Priority determines which shortcut wins in conflicts:

- **critical**: System-critical shortcuts (e.g., Escape)
- **high**: Important shortcuts (e.g., Cmd+K)
- **normal**: Standard shortcuts (default)
- **low**: Low-priority shortcuts

### Categories

Categories organize shortcuts:

- **system**: System-level shortcuts
- **window**: Window management
- **navigation**: Navigation shortcuts
- **editing**: Text editing
- **selection**: Selection management
- **clipboard**: Clipboard operations
- **app**: App-specific
- **developer**: Developer tools
- **custom**: User-defined

## Advanced Usage

### Conflict Detection

```typescript
import { useConflicts, useStats } from "@/shortcuts";

function ShortcutDebugger() {
  const conflicts = useConflicts();
  const stats = useStats();

  return (
    <div>
      <h2>Conflicts: {conflicts.length}</h2>
      {conflicts.map((c) => (
        <div key={c.sequence}>
          {c.sequence}: {c.shortcuts.map((s) => s.id).join(", ")}
        </div>
      ))}

      <h2>Statistics</h2>
      <pre>{JSON.stringify(stats, null, 2)}</pre>
    </div>
  );
}
```

### Custom Shortcuts

```typescript
import { useActions } from "@/shortcuts";

function SettingsPanel() {
  const { customize, resetCustomization } = useActions();

  const handleCustomize = (id: string, newSequence: string) => {
    customize(id, newSequence);
  };

  const handleReset = (id: string) => {
    resetCustomization(id);
  };
}
```

### Dynamic Shortcuts

```typescript
import { useShortcut } from "@/shortcuts";
import { useState } from "react";

function DynamicComponent() {
  const [enabled, setEnabled] = useState(true);

  useShortcut("dynamic.command", {
    sequence: "$mod+d",
    label: "Dynamic Command",
    handler: () => console.log("Dynamic!"),
    enabled, // Dynamically enable/disable
  });
}
```

### Command Integration

```typescript
import { allCommands, getCommandsByCategory } from "@/shortcuts";

function CommandPalette() {
  const systemCommands = getCommandsByCategory("system");

  return (
    <div>
      {systemCommands.map((cmd) => (
        <div key={cmd.id}>
          {cmd.label} - {cmd.sequence}
        </div>
      ))}
    </div>
  );
}
```

## Formatting and Display

```typescript
import { formatShortcut, formatShortcutHTML } from "@/shortcuts";

// Get formatted display
const formatted = formatShortcut("$mod+k");
console.log(formatted.display);  // "⌘K" on Mac, "Ctrl+K" on Windows
console.log(formatted.symbols);  // "⌘K"
console.log(formatted.verbose);  // "Command K"

// Get HTML for rendering
const html = formatShortcutHTML("$mod+Shift+p");
// <kbd>⌘</kbd><kbd>⇧</kbd><kbd>P</kbd>
```

## Platform Detection

```typescript
import { detectPlatform, isMac, isWindows } from "@/shortcuts";

const platform = detectPlatform(); // "mac" | "windows" | "linux" | "unknown"

if (isMac()) {
  // Mac-specific logic
}
```

## Testing

The system is designed for easy testing with pure functions:

```typescript
import { parseSequence, validateSequence, matchesSequence } from "@/shortcuts";

describe("Shortcuts", () => {
  test("parses sequences", () => {
    const parsed = parseSequence("$mod+k");
    expect(parsed.key).toBe("k");
    expect(parsed.modifiers).toContain("Meta"); // on Mac
  });

  test("validates sequences", () => {
    const result = validateSequence("$mod+k");
    expect(result.valid).toBe(true);
  });

  test("matches events", () => {
    const event = new KeyboardEvent("keydown", {
      key: "k",
      metaKey: true,
    });
    expect(matchesSequence(event, "$mod+k")).toBe(true);
  });
});
```

## Best Practices

### 1. Use Descriptive IDs

```typescript
// ✅ Good: Descriptive, namespaced
useShortcut("editor.save", { ... });
useShortcut("window.close", { ... });

// ❌ Bad: Generic, unclear
useShortcut("save", { ... });
useShortcut("close", { ... });
```

### 2. Use `$mod` for Cross-Platform

```typescript
// ✅ Good: Works on all platforms
useShortcut("save", { sequence: "$mod+s", ... });

// ❌ Bad: Mac-only
useShortcut("save", { sequence: "Meta+s", ... });
```

### 3. Set Appropriate Scopes

```typescript
// ✅ Good: Specific scope
useShortcut("window.close", {
  sequence: "$mod+w",
  scope: "window", // Only when window is focused
  ...
});

// ❌ Bad: Global scope for specific action
useShortcut("window.close", {
  sequence: "$mod+w",
  scope: "global", // Conflicts with other uses
  ...
});
```

### 4. Handle Conflicts Gracefully

```typescript
// ✅ Good: Check for conflicts
useShortcut("my.command", {
  sequence: "$mod+k",
  priority: "normal", // Will defer to higher priority
  ...
});

// ❌ Bad: High priority for non-critical
useShortcut("my.command", {
  sequence: "$mod+k",
  priority: "critical", // Conflicts with system shortcuts
  ...
});
```

### 5. Provide Good Labels and Descriptions

```typescript
// ✅ Good: Clear and descriptive
useShortcut("editor.save", {
  sequence: "$mod+s",
  label: "Save File",
  description: "Save the current file to disk",
  ...
});

// ❌ Bad: No labels
useShortcut("editor.save", {
  sequence: "$mod+s",
  handler: () => save(),
});
```

## API Reference

### Hooks

- `useShortcut(id, options)` - Register a single shortcut
- `useShortcuts(configs)` - Register multiple shortcuts
- `useScope(scope, active)` - Activate a scope
- `useScopes(scopes, active)` - Activate multiple scopes
- `useSimpleShortcut(sequence, handler, enabled)` - Simple shortcut registration
- `useGlobalShortcut(id, sequence, handler, options)` - Register global shortcut

### Store Hooks

- `useShortcuts()` - Subscribe to all shortcuts
- `useActions()` - Get shortcut actions
- `useShortcut(id)` - Subscribe to specific shortcut
- `useShortcutsByScope(scope)` - Get shortcuts by scope
- `useShortcutsByCategory(category)` - Get shortcuts by category
- `useActiveScopes()` - Get active scopes
- `useConflicts()` - Get detected conflicts
- `useStats()` - Get usage statistics

### Core Functions

- `parseSequence(sequence)` - Parse shortcut sequence
- `validateSequence(sequence)` - Validate sequence
- `normalizeSequence(sequence)` - Normalize for comparison
- `formatShortcut(sequence)` - Format for display
- `matchesSequence(event, sequence)` - Check if event matches
- `findConflicts(shortcuts)` - Detect conflicts
- `resolveConflict(conflict, scopes)` - Resolve conflict

### Platform Functions

- `detectPlatform()` - Detect current platform
- `isMac()` - Check if Mac
- `isWindows()` - Check if Windows
- `isLinux()` - Check if Linux
- `getPlatformModifier()` - Get platform modifier key

## Integration with Existing Code

### Migrating from Old System

```typescript
// Old pattern (manual event listeners)
useEffect(() => {
  const handleKeyDown = (e: KeyboardEvent) => {
    if ((e.metaKey || e.ctrlKey) && e.key === "k") {
      e.preventDefault();
      handleCommand();
    }
  };
  window.addEventListener("keydown", handleKeyDown);
  return () => window.removeEventListener("keydown", handleKeyDown);
}, []);

// New pattern (centralized shortcuts)
useShortcut("command", {
  sequence: "$mod+k",
  label: "Execute Command",
  handler: handleCommand,
});
```

## Performance

The shortcut system is optimized for performance:

- **Lazy binding**: Shortcuts are only bound when their scope is active
- **Event delegation**: Single global listener with efficient routing
- **Memoization**: Parsed sequences are cached
- **Minimal re-renders**: Zustand provides fine-grained subscriptions

## Browser Compatibility

Compatible with all modern browsers:
- Chrome/Edge 90+
- Firefox 88+
- Safari 14+

## Future Enhancements

- [ ] Sequence chains (e.g., "g d" = press g then d)
- [ ] Macro recording
- [ ] Import/export shortcut configurations
- [ ] Shortcut hints overlay
- [ ] Learning mode (shows shortcuts as you use features)
- [ ] Analytics dashboard
- [ ] Voice command integration

## Contributing

When adding new shortcuts:

1. Define in appropriate command file (`commands/*.ts`)
2. Use descriptive IDs with dot notation
3. Add clear labels and descriptions
4. Set appropriate scope, priority, and category
5. Update tests if needed

## License

MIT

