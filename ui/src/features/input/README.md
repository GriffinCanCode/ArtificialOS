# Input Handling Module

Centralized, modular input handling architecture for keyboard, mouse, touch, and gesture events.

## Architecture

```
input/
├── core/           # Pure functions for input processing
│   ├── types.ts    # TypeScript type definitions
│   ├── keyboard.ts # Keyboard event utilities
│   ├── mouse.ts    # Mouse event utilities
│   └── gesture.ts  # Gesture detection utilities
├── hooks/          # React hooks for input handling
│   ├── useKeyboard.ts    # Keyboard shortcuts
│   ├── useMouse.ts       # Mouse interactions
│   ├── useGesture.ts     # Gesture handling
│   └── useValidation.ts  # Form validation
├── validation/     # Input validation utilities
│   ├── schemas.ts    # Zod validation schemas
│   └── validators.ts # Pure validation functions
└── formatting/     # Input formatting utilities
    ├── text.ts     # Text transformation
    ├── number.ts   # Number formatting
    └── date.ts     # Date formatting
```

## Usage

### Keyboard Shortcuts

```typescript
import { useKeyboard } from "@/input";

function MyComponent() {
  useKeyboard([
    {
      key: "k",
      modifiers: ["Meta"],
      handler: (e) => console.log("Cmd+K pressed"),
      description: "Open command palette",
    },
  ]);
}
```

### Mouse Interactions

```typescript
import { useDrag } from "@/input";

function DraggableComponent() {
  const { isDragging, onMouseDown } = useDrag(
    (state) => console.log("Drag started", state),
    (state) => console.log("Dragging", state),
    (state) => console.log("Drag ended", state)
  );

  return <div onMouseDown={onMouseDown}>Drag me</div>;
}
```

### Gestures

```typescript
import { useSwipe } from "@/input";

function SwipeableComponent() {
  const bind = useSwipe((direction) => {
    console.log("Swiped", direction);
  });

  return <div {...bind()}>Swipe me</div>;
}
```

### Validation

```typescript
import { useValidation, emailSchema } from "@/input";

function FormComponent() {
  const { validate, errors } = useValidation(emailSchema);

  const handleSubmit = (email: string) => {
    const result = validate(email);
    if (result.isValid) {
      // Submit form
    }
  };
}
```

### Formatting

```typescript
import { formatCurrency, formatRelativeTime, toTitleCase } from "@/input";

const price = formatCurrency(1234.56); // "$1,234.56"
const time = formatRelativeTime(new Date()); // "just now"
const title = toTitleCase("hello world"); // "Hello World"
```

## Design Principles

1. **Modularity**: Each module handles a specific concern
2. **Type Safety**: Full TypeScript support with strict types
3. **Testability**: Pure functions for easy unit testing
4. **Reusability**: Composable utilities and hooks
5. **Performance**: Optimized event handling with proper cleanup
6. **Extensibility**: Easy to add new input types and validators

## Best Practices

- Use hooks for stateful input handling
- Use core utilities for pure functions
- Validate user input with Zod schemas
- Format output with formatting utilities
- Handle edge cases (mobile, accessibility)
- Clean up event listeners properly
- Prevent default behavior when needed
- Debounce expensive operations

## Testing

All modules are designed for easy testing:

```typescript
import { isEmail, formatCurrency } from "@/input";

test("validates email", () => {
  expect(isEmail("test@example.com")).toBe(true);
  expect(isEmail("invalid")).toBe(false);
});

test("formats currency", () => {
  expect(formatCurrency(1234.56)).toBe("$1,234.56");
});
```

## Migration Guide

Old pattern:
```typescript
import { shouldIgnoreKeyboardEvent } from "@/utils/keyboard";
```

New pattern:
```typescript
import { shouldIgnoreKeyboardEvent } from "@/input";
```

All existing utilities have been migrated and enhanced with additional functionality.
