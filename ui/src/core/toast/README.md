# Toast Notification System

A sophisticated, non-intrusive notification system built on [Sonner](https://sonner.emilkowal.ski/), fully integrated with AgentOS's architecture.

## Features

- ✅ **Multiple Types**: success, error, warning, info, loading, promise
- ✅ **Undo Actions**: Easy undo functionality for reversible operations
- ✅ **Progress Tracking**: Show progress for long-running operations
- ✅ **Promise Integration**: Automatic state updates for async operations
- ✅ **Rich Content**: Support for descriptions, actions, and custom content
- ✅ **Customizable**: Full control over duration, position, and styling
- ✅ **Accessible**: WCAG compliant with keyboard navigation
- ✅ **Type-Safe**: Full TypeScript support with strong typing
- ✅ **Metrics**: Integrated with AgentOS monitoring system

## Usage

### Direct Import (Frontend)

```typescript
import { toast } from "@/core/toast";

// Simple success toast
toast.success("Operation completed!");

// Toast with description
toast.error("Failed to save", {
  description: "Please check your network connection",
  duration: 5000,
});

// Toast with action button
toast.info("New version available", {
  action: {
    label: "Update",
    onClick: () => window.location.reload(),
  },
});

// Undo toast
toast.undo("Item deleted", () => {
  // Restore item logic
  console.log("Item restored");
});

// Progress toast
const toastId = toast.progress("Uploading files", 0);
// Update progress
toast.dismiss(toastId);
toast.progress("Uploading files", 50);

// Promise toast (auto-updates based on promise state)
toast.promise(
  fetchData(),
  {
    loading: "Loading data...",
    success: "Data loaded successfully!",
    error: "Failed to load data",
  }
);

// Promise with dynamic messages
toast.promise(
  uploadFile(file),
  {
    loading: "Uploading...",
    success: (data) => `Uploaded ${data.filename}`,
    error: (err) => `Upload failed: ${err.message}`,
  }
);
```

### Via Executor (Blueprint DSL)

Toast notifications can be triggered from blueprint components using the `toast` executor:

```yaml
# Success toast
on_event:
  click: "toast.success"
  params:
    message: "Item saved successfully!"
    duration: 3000

# Error toast with description
on_event:
  submit: "toast.error"
  params:
    message: "Validation failed"
    description: "Please check the highlighted fields"
    duration: 5000

# Toast with undo action
on_event:
  delete: "toast.undo"
  params:
    message: "Item deleted"
    on_undo: "restore_item"
    duration: 10000

# Progress toast
on_event:
  upload: "toast.progress"
  params:
    key: "upload-progress"
    message: "Uploading files"
    percent: 0

# Update progress
on_event:
  progress: "toast.update"
  params:
    key: "upload-progress"
    message: "Uploading files"
    type: "progress"
    percent: 50

# Loading toast
on_event:
  load: "toast.loading"
  params:
    key: "loading-data"
    message: "Loading data..."

# Dismiss specific toast
on_event:
  complete: "toast.dismiss"
  params:
    key: "loading-data"

# Dismiss all toasts
on_event:
  cancel: "toast.dismiss_all"
```

## Toast Types

### Success
Shows a green checkmark icon with success styling.
```typescript
toast.success("Changes saved!");
```

### Error
Shows a red X icon with error styling.
```typescript
toast.error("Failed to connect");
```

### Warning
Shows a yellow warning icon.
```typescript
toast.warning("Low disk space");
```

### Info
Shows a blue info icon.
```typescript
toast.info("New feature available");
```

### Loading
Shows an animated spinner.
```typescript
const id = toast.loading("Processing...");
// Later dismiss it
toast.dismiss(id);
```

### Promise
Automatically updates based on promise state.
```typescript
toast.promise(asyncOperation(), {
  loading: "Processing...",
  success: "Done!",
  error: "Failed",
});
```

## Options

All toast methods accept an optional `ToastOptions` object:

```typescript
interface ToastOptions {
  // Unique identifier for the toast
  id?: string | number;
  
  // Duration in milliseconds (default: 4000)
  duration?: number;
  
  // Position on screen
  position?: "top-left" | "top-center" | "top-right" 
    | "bottom-left" | "bottom-center" | "bottom-right";
  
  // Allow manual dismissal via close button
  dismissible?: boolean;
  
  // Additional description text
  description?: string;
  
  // Action button
  action?: {
    label: string;
    onClick: () => void;
  };
  
  // Cancel button
  cancel?: {
    label: string;
    onClick: () => void;
  };
  
  // Custom icon
  icon?: React.ReactNode;
  
  // Callback when dismissed
  onDismiss?: (id: string | number) => void;
  
  // Callback when auto-closed
  onAutoClose?: (id: string | number) => void;
  
  // Custom CSS class
  className?: string;
  
  // Mark as important (more prominent styling)
  important?: boolean;
}
```

## Advanced Examples

### Multi-step Process with Progress

```typescript
const processFiles = async (files: File[]) => {
  const total = files.length;
  
  for (let i = 0; i < files.length; i++) {
    const percent = Math.round((i / total) * 100);
    toast.progress(`Processing files`, percent, {
      id: "file-process",
    });
    
    await processFile(files[i]);
  }
  
  toast.dismiss("file-process");
  toast.success(`Processed ${total} files`);
};
```

### Undo with State Management

```typescript
const deleteItem = (itemId: string) => {
  const item = getItem(itemId);
  removeItem(itemId);
  
  toast.undo(`Deleted ${item.name}`, () => {
    restoreItem(item);
    toast.success("Item restored");
  }, {
    duration: 10000, // Give more time to undo
  });
};
```

### Chained Toasts

```typescript
const performOperation = async () => {
  const loadingId = toast.loading("Starting operation...");
  
  try {
    await step1();
    toast.dismiss(loadingId);
    
    const step2Id = toast.loading("Step 2 of 3...");
    await step2();
    toast.dismiss(step2Id);
    
    const step3Id = toast.loading("Finalizing...");
    await step3();
    toast.dismiss(step3Id);
    
    toast.success("Operation completed!");
  } catch (error) {
    toast.dismiss(loadingId);
    toast.error("Operation failed", {
      description: error.message,
    });
  }
};
```

### Custom Positioning

```typescript
// Top-right for notifications
toast.info("New message", {
  position: "top-right",
});

// Bottom-center for actions
toast.success("Changes saved", {
  position: "bottom-center",
});
```

## Integration with Existing Systems

### With Session Manager

```typescript
const saveSession = async (name: string) => {
  const promise = sessionManager.save(name);
  
  toast.promise(promise, {
    loading: "Saving session...",
    success: "Session saved!",
    error: "Failed to save session",
  });
};
```

### With Window Actions

```typescript
import { toast } from "@/core/toast";
import { useActions } from "@/features/windows";

const MyComponent = () => {
  const { close } = useActions();
  
  const handleClose = (windowId: string) => {
    close(windowId);
    toast.success("Window closed", {
      action: {
        label: "Reopen",
        onClick: () => {
          // Reopen logic
        },
      },
    });
  };
};
```

### With Form Submissions

```typescript
const onSubmit = async (data: FormData) => {
  const toastId = toast.loading("Submitting...");
  
  try {
    await submitForm(data);
    toast.dismiss(toastId);
    toast.success("Form submitted successfully!");
  } catch (error) {
    toast.dismiss(toastId);
    toast.error("Submission failed", {
      description: error.message,
      action: {
        label: "Retry",
        onClick: () => onSubmit(data),
      },
    });
  }
};
```

## Styling

Custom styles are defined in `styles.css` and follow AgentOS's design system:

- Dark theme with glass morphism effects
- Purple accent colors matching the OS theme
- Smooth animations and transitions
- Responsive on mobile devices

To override styles, use the `className` option:

```typescript
toast.success("Custom styled toast", {
  className: "my-custom-toast",
});
```

## Best Practices

1. **Keep messages concise** - Use title + description for longer content
2. **Use appropriate types** - Match the toast type to the action result
3. **Provide actions when relevant** - Offer undo/retry options
4. **Set appropriate durations** - Longer for important messages, shorter for confirmations
5. **Don't spam** - Dismiss old toasts before showing new ones for the same operation
6. **Use keys for updates** - Track toasts that need to be updated or dismissed
7. **Test accessibility** - Ensure keyboard navigation works

## Architecture

```
core/toast/
├── types.ts          # TypeScript definitions
├── store.ts          # Zustand state management
├── utils.ts          # Toast helper functions
├── styles.css        # Custom styling
├── index.ts          # Public API exports
└── README.md         # This file

features/dynamics/execution/executors/system/
└── toast-executor.ts # Executor for blueprint integration
```

## Comparison: Toast vs Notification

| Feature | Toast | OS Notification |
|---------|-------|-----------------|
| **Location** | In-app, overlayed | System notification center |
| **Persistence** | Temporary (auto-dismiss) | Persists until user dismisses |
| **Interaction** | Can have action buttons | Limited interaction |
| **Styling** | Fully customizable | System-dependent |
| **Permission** | None required | Requires user permission |
| **Use Case** | App feedback, confirmations | Important system alerts |

**Rule of thumb**: Use toasts for app-level feedback (save confirmations, errors, progress). Use OS notifications for critical alerts that need attention even when app is in background.

## Testing

```typescript
import { toast } from "@/core/toast";
import { render, screen } from "@testing-library/react";

describe("Toast", () => {
  it("shows success toast", () => {
    toast.success("Test message");
    expect(screen.getByText("Test message")).toBeInTheDocument();
  });
  
  it("handles undo action", () => {
    const mockUndo = jest.fn();
    toast.undo("Action performed", mockUndo);
    
    const undoButton = screen.getByText("Undo");
    undoButton.click();
    
    expect(mockUndo).toHaveBeenCalled();
  });
});
```

## Performance

- Toasts are rendered via React Portal for optimal performance
- State updates are batched to prevent unnecessary re-renders
- Animations use GPU-accelerated CSS transforms
- Zustand store provides efficient state updates
- Integrated with metrics system for monitoring

## Future Enhancements

- [ ] Toast stacking/grouping for related messages
- [ ] Custom toast templates for common patterns
- [ ] Toast history/replay functionality
- [ ] Multi-language support
- [ ] Sound effects (optional)
- [ ] Toast queue management for rate limiting
