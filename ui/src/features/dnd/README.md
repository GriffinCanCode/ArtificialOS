# Drag & Drop Module

Advanced drag and drop functionality using `@dnd-kit` for sortable lists and file uploads.

## Architecture

```
dnd/
├── core/              # Pure functions and types
│   ├── types.ts       # TypeScript definitions
│   └── utils.ts       # Utility functions
├── hooks/             # React hooks
│   ├── useSortable.ts # Sortable list management
│   └── useDropzone.ts # File drop handling
├── store/             # State management
│   └── store.ts       # Zustand dock store
├── components/        # Reusable components
│   ├── Sortable.tsx   # Generic sortable wrapper
│   ├── SortableItem.tsx # Individual sortable item
│   ├── Dropzone.tsx   # File dropzone
│   └── Dropzone.css   # Dropzone styles
└── index.ts           # Public exports
```

## Features

### ✅ Sortable Lists
- Drag and drop reordering
- Horizontal, vertical, and grid layouts
- Keyboard accessibility
- Touch support
- Customizable animations

### ✅ File Drops
- Drag and drop file uploads
- File type validation
- File size limits
- Multiple file support
- Preview generation for images
- Custom validators

### ✅ Dock Management
- Persistent dock item storage
- Reorderable app launcher
- Pinned items
- LocalStorage sync

## Usage

### Sortable List

```typescript
import { Sortable, useSortable } from "@/dnd";

function MyList() {
  const items = [
    { id: "1", name: "Item 1" },
    { id: "2", name: "Item 2" },
  ];

  const handleSort = (result) => {
    console.log("Reordered:", result);
  };

  return (
    <Sortable
      items={items}
      onSort={handleSort}
      strategy="vertical"
      renderItem={(item) => <div>{item.name}</div>}
    />
  );
}
```

### File Dropzone

```typescript
import { Dropzone } from "@/dnd";

function FileUpload() {
  const handleDrop = (result) => {
    console.log("Files:", result.files);
    console.log("Rejected:", result.rejectedFiles);
  };

  return (
    <Dropzone
      onDrop={handleDrop}
      accept={["image/*", ".pdf"]}
      maxSize={5 * 1024 * 1024} // 5MB
      maxFiles={10}
      multiple
      showPreview
    />
  );
}
```

### Custom Hook Usage

```typescript
import { useSortable, useDropzone } from "@/dnd";

function CustomComponent() {
  // Sortable
  const {
    items,
    activeId,
    handleDragStart,
    handleDragEnd,
  } = useSortable({
    items: myItems,
    onSort: handleReorder,
  });

  // Dropzone
  const {
    isDragging,
    files,
    getRootProps,
    getInputProps,
  } = useDropzone({
    onDrop: handleFiles,
    accept: ["image/*"],
  });

  return (
    <div {...getRootProps()}>
      <input {...getInputProps()} />
      {/* Your UI */}
    </div>
  );
}
```

### Dock Store

```typescript
import { useDockItems, useDockActions } from "@/dnd";

function Dock() {
  const items = useDockItems();
  const { reorder, add, remove } = useDockActions();

  return (
    <div>
      {items.map((item) => (
        <button key={item.id} onClick={() => launch(item.action)}>
          {item.icon} {item.label}
        </button>
      ))}
    </div>
  );
}
```

## API Reference

### Types

```typescript
// Sortable item (must have unique id)
interface SortableItem {
  id: UniqueIdentifier;
}

// Sort result passed to onSort callback
interface SortResult {
  activeId: UniqueIdentifier;
  overId: UniqueIdentifier;
  oldIndex: number;
  newIndex: number;
}

// File drop configuration
interface FileDropConfig {
  accept?: string[];        // MIME types or extensions
  maxSize?: number;         // Bytes
  maxFiles?: number;        // Number of files
  multiple?: boolean;       // Allow multiple files
  disabled?: boolean;       // Disable drop zone
}

// Dock item
interface DockItem extends SortableItem {
  id: string;
  label: string;
  icon: string;
  action: string;
  order: number;
  pinned?: boolean;
}
```

### Utilities

```typescript
// File validation
validateFileType(file: File, accept?: string[]): boolean
validateFileSize(file: File, maxSize?: number): boolean
validateFile(file: File, config: FileDropConfig, validator?: FileValidator): string | null

// Array manipulation
arrayMove<T>(array: T[], from: number, to: number): T[]
arrayInsert<T>(array: T[], index: number, item: T): T[]
arrayRemove<T>(array: T[], index: number): T[]

// File helpers
formatFileSize(bytes: number): string
getFileExtension(filename: string): string
isImageFile(file: File): boolean
createFilePreview(file: File): Promise<string>
```

## Integration

### Desktop Dock

The desktop dock is now sortable:
- Drag items to reorder
- Pinned items are protected
- Order persists in localStorage
- Smooth animations

### Future Enhancements

1. **Window Tabs**: Reorderable tabs (ready to integrate)
2. **Desktop Icons**: Grid-based icon arrangement
3. **File Manager**: Drag files between folders
4. **Custom Drag Previews**: Rich drag overlays

## Performance

- Optimized re-renders with `useShallow`
- Minimal DOM updates during drag
- Efficient collision detection
- Virtual scrolling compatible

## Accessibility

- Full keyboard navigation (arrows, enter, escape)
- Screen reader announcements
- Focus management
- ARIA attributes

## Testing

Comprehensive test coverage:
- Core utilities (100%)
- Hooks (useSortable, useDropzone)
- Store (Zustand dock store)
- Edge cases and error handling

Run tests:
```bash
npm test -- dnd
```

## Browser Support

- Chrome/Edge 90+
- Firefox 88+
- Safari 14+
- Mobile browsers (iOS 13+, Android 5+)

## Dependencies

- `@dnd-kit/core` - Core drag and drop primitives
- `@dnd-kit/sortable` - Sortable presets and utilities
- `@dnd-kit/utilities` - CSS utilities
- `zustand` - State management

## License

MIT
