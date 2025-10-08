# Clipboard Module

Comprehensive clipboard system with multi-format support, history, and backend synchronization.

## Architecture

```
clipboard/
├── core/               # Pure types and state management
│   ├── types.ts        # TypeScript definitions
│   └── manager.ts      # State manager with subscriptions
├── hooks/              # React hooks
│   └── useClipboard.ts # Main clipboard hook
├── components/         # UI components
│   ├── ClipboardViewer.tsx   # History viewer
│   └── ClipboardViewer.css   # Styles
└── index.ts            # Public exports
```

## Features

### Multi-Format Support
- **Text**: Plain text content
- **HTML**: Rich text with formatting
- **Bytes**: Binary data
- **Files**: File paths and metadata
- **Images**: Image data with MIME types

### Clipboard History
- Maintains up to 100 recent entries
- Timestamp tracking
- Optional labels for entries
- Fast lookup by entry ID

### Backend Synchronization
- Automatic sync with backend service
- Kernel-level clipboard management
- Per-process and global clipboards
- Permission-based access control

### Browser Fallback
- Works without backend connection
- Uses browser Clipboard API
- Graceful degradation

## Usage

### Basic Hook

```typescript
import { useClipboard } from '@/features/clipboard';

function MyComponent() {
  const { copy, paste, history, current } = useClipboard({
    service: myServiceClient,
    autoLoad: true,
  });

  const handleCopy = async () => {
    const entryId = await copy('Hello, World!');
    console.log('Copied with ID:', entryId);
  };

  const handlePaste = async () => {
    const entry = await paste();
    console.log('Pasted:', entry?.data);
  };

  return (
    <div>
      <button onClick={handleCopy}>Copy</button>
      <button onClick={handlePaste}>Paste</button>
      <div>History: {history.length} entries</div>
    </div>
  );
}
```

### With Options

```typescript
// Copy to global clipboard
await copy('Shared text', { global: true });

// Paste with format specification
await paste({ format: 'html', global: false });

// Get limited history
const recentEntries = await getHistory({ limit: 5, global: false });
```

### Clipboard Viewer

```typescript
import { ClipboardViewer } from '@/features/clipboard';

function ClipboardHistoryPanel() {
  const { history, current, copy } = useClipboard();

  const handleSelect = async (entry: ClipboardEntry) => {
    // Re-copy selected entry
    await copy(entry.data);
  };

  return (
    <ClipboardViewer
      entries={history}
      current={current}
      onSelect={handleSelect}
      onCopy={handleSelect}
      onDelete={(id) => console.log('Delete', id)}
    />
  );
}
```

### Advanced: Subscriptions

```typescript
const { subscribe, unsubscribe, subscribed } = useClipboard();

// Subscribe to all clipboard changes
await subscribe();

// Subscribe to specific formats
await subscribe(['text', 'html']);

// Unsubscribe
await unsubscribe();
```

### Statistics

```typescript
const { getStats } = useClipboard();

const stats = await getStats();
console.log('Total entries:', stats.total_entries);
console.log('Total size:', stats.total_size);
console.log('Process count:', stats.process_count);
```

## API Reference

### useClipboard(options?)

Main hook for clipboard operations.

**Options:**
- `service?: any` - Service client for backend communication
- `autoLoad?: boolean` - Auto-load history on mount (default: true)

**Returns:**
```typescript
{
  // State
  current: ClipboardEntry | null;
  history: ClipboardEntry[];
  stats: ClipboardStats | null;
  subscribed: boolean;
  loading: boolean;
  error: string | null;

  // Actions
  copy: (text: string, options?) => Promise<number>;
  paste: (options?) => Promise<ClipboardEntry | null>;
  getHistory: (options?) => Promise<ClipboardEntry[]>;
  getEntry: (entryId: number) => Promise<ClipboardEntry | null>;
  clear: (global?: boolean) => Promise<void>;
  subscribe: (formats?: string[]) => Promise<void>;
  unsubscribe: () => Promise<void>;
  getStats: () => Promise<ClipboardStats>;
}
```

### ClipboardViewer

Component for displaying clipboard history.

**Props:**
```typescript
{
  entries: ClipboardEntry[];
  current: ClipboardEntry | null;
  onSelect?: (entry: ClipboardEntry) => void;
  onCopy?: (entry: ClipboardEntry) => void;
  onDelete?: (entryId: number) => void;
}
```

## Performance

- **Optimized State**: Uses `useSyncExternalStore` for efficient re-renders
- **Caching**: Local state cache reduces backend calls
- **Lazy Loading**: History loaded on demand
- **Debouncing**: Copy operations debounced for rapid calls

## Security

- **Permission Checks**: Kernel-level permission verification
- **Process Isolation**: Per-process clipboards with sandboxing
- **Size Limits**: 10MB max entry size
- **Format Validation**: Type-safe format handling

## Browser Compatibility

- Modern browsers with Clipboard API support
- Fallback for browsers without backend connection
- Graceful degradation for unsupported formats

## Examples

See `examples/` directory for:
- Basic clipboard operations
- History management
- Format conversion
- Real-time subscriptions
- Integration with dynamic UI

## Testing

```bash
# Unit tests
npm test -- clipboard

# Integration tests
npm run test:integration -- clipboard

# E2E tests
npm run test:e2e -- clipboard
```

## Future Enhancements

- [ ] Clipboard sync across windows
- [ ] Rich text editing
- [ ] Image preview
- [ ] File drag-and-drop
- [ ] Clipboard search
- [ ] Pinned entries
- [ ] Clipboard sharing

