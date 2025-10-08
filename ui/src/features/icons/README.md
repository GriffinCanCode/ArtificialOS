# Icons Module

Centralized desktop icon management system with grid-based positioning, drag-and-drop, selection, and auto-arrange algorithms.

## Architecture

```
icons/
├── core/              # Pure functions and types
│   ├── types.ts       # TypeScript definitions
│   ├── grid.ts        # Grid mathematics (bijective mappings)
│   └── collision.ts   # Spatial indexing & collision detection
├── store/             # State management
│   └── store.ts       # Zustand store with persistence
├── hooks/             # React hooks
│   ├── useGrid.ts     # Grid calculations
│   ├── useSelect.ts   # Selection management
│   └── useDrag.ts     # Drag-and-drop
├── components/        # React components
│   ├── Icon.tsx       # Single icon
│   ├── Grid.tsx       # Icon grid container
│   └── Context.tsx    # Context menu
├── utils/             # Helper utilities
│   └── arrange.ts     # Auto-arrange algorithms
└── index.ts           # Public exports
```

## Features

### Default Icons
- **Automatic initialization** - Terminal and File Explorer appear on first load
- **Smart positioning** - Default icons at (0,0) and (0,1)
- **Persistent** - Saved to localStorage after initialization
- **User control** - Can be moved, removed, or rearranged

### Grid System
- **Bijective grid-to-pixel mapping** - Efficient coordinate transformations
- **Spatial hashing** - O(1) collision detection
- **Viewport-aware** - Automatic grid dimension calculation
- **Multiple distance metrics** - Manhattan, Euclidean, Chebyshev

### Icon Management
- **CRUD operations** - Add, remove, update icons
- **Position management** - Grid-based with collision detection
- **State persistence** - Icons saved to localStorage
- **Multiple icon types** - Apps, files, folders, shortcuts, native

### Selection
- **Single selection** - Click to select
- **Multi-selection** - Cmd/Ctrl+Click
- **Range selection** - Shift+Click (TODO)
- **Select all** - Cmd/Ctrl+A
- **Keyboard navigation** - Escape to deselect

### Drag & Drop
- **Drag threshold** - Prevents accidental drags
- **Multi-icon drag** - Drag selected icons together
- **Grid snapping** - Icons snap to grid cells
- **Preview indicator** - Visual feedback during drag
- **Collision avoidance** - Finds nearest available position

### Auto-Arrange
- **Grid arrangement** - Reading order (left-to-right, top-to-bottom)
- **Alphabetical** - Sort by name
- **By type** - Group by icon type
- **By date** - Sort by creation date
- **By size** - Sort by file size (for files)
- **Compact layout** - Fill gaps while preserving order

### Advanced Features
- **K-means clustering** - Group icons by proximity
- **Alignment tools** - Align left, right, top, bottom
- **Distribution** - Evenly distribute icons
- **Quadtree partitioning** - Spatial queries for large grids
- **Context menu** - Right-click actions

## Usage

### Basic Setup

```typescript
import { Grid, useIconActions } from "@/features/icons";

function Desktop() {
  const { add } = useIconActions();

  // Grid automatically initializes with Terminal and File Explorer icons
  // on first load (if desktop is empty)

  // Add additional icons
  const handleAddIcon = () => {
    add({
      type: "app",
      label: "Calculator",
      icon: "🔢",
      position: { row: 0, col: 2 }, // Next to default icons
      metadata: {
        type: "app",
        appId: "calculator",
        launchable: true,
      },
    });
  };

  return (
    <div>
      <Grid
        onIconDoubleClick={(icon) => console.log("Open:", icon)}
        onBackgroundClick={() => console.log("Background clicked")}
      />
      <button onClick={handleAddIcon}>Add Icon</button>
    </div>
  );
}
```

### Default Icons

Default system apps are automatically added to the desktop on first load:

```typescript
import { getDefaultIcons, shouldInitializeDefaults } from "@/features/icons";

// Get list of default icons
const defaults = getDefaultIcons();
// => [
//   { type: "native", label: "Terminal", icon: "💻", ... },
//   { type: "native", label: "Files", icon: "📁", ... }
// ]

// Check if initialization is needed
const needsInit = shouldInitializeDefaults(icons);
// => true if icons array is empty
```

### Icon Actions

```typescript
const {
  add,           // Add new icon
  remove,        // Remove icon
  update,        // Update icon properties
  updatePosition,// Move icon
  select,        // Select icon
  autoArrange,   // Auto-arrange icons
  compact,       // Compact layout
} = useIconActions();

// Add icon
const iconId = add({
  type: "file",
  label: "Document.pdf",
  icon: "📄",
  position: { row: 1, col: 2 },
  metadata: {
    type: "file",
    path: "/documents/file.pdf",
    extension: "pdf",
    size: 1024,
  },
});

// Move icon
updatePosition(iconId, { row: 2, col: 3 });

// Auto-arrange
autoArrange("name"); // Sort by name
```

### Selection Management

```typescript
const selection = useSelect();

// Select icon
selection.select(iconId, { ctrl: true }); // Multi-select

// Select all
selection.selectAll();

// Clear selection
selection.clearSelection();

// Check if selected
const isSelected = selection.isSelected(iconId);
```

### Grid Calculations

```typescript
import { gridToPixel, pixelToGrid, snapToGrid } from "@/features/icons";

// Convert grid to pixels
const pixel = gridToPixel({ row: 2, col: 3 });
// => { x: 364, y: 344 }

// Convert pixels to grid
const grid = pixelToGrid({ x: 400, y: 300 });
// => { row: 2, col: 3 }

// Snap to nearest grid cell
const snapped = snapToGrid({ x: 405, y: 315 });
// => { row: 2, col: 3 }
```

### Collision Detection

```typescript
import { buildCollisionMap, findNearestAvailable } from "@/features/icons";

const icons = useIcons();
const collisionMap = buildCollisionMap(icons);

// Find nearest available position
const nearestPos = findNearestAvailable(
  { row: 2, col: 3 },
  collisionMap,
  maxRows,
  maxCols
);
```

## Mathematical Foundations

### Grid-to-Pixel Mapping (Bijective)

```
F: ℤ² → ℝ²  (Grid space to pixel space)
F(row, col) = (x, y)

x = marginLeft + col × (cellWidth + padding)
y = marginTop + row × (cellHeight + padding)

F⁻¹: ℝ² → ℤ²  (Pixel space to grid space)
F⁻¹(x, y) = (row, col)

col = ⌊(x - marginLeft) / (cellWidth + padding)⌋
row = ⌊(y - marginTop) / (cellHeight + padding)⌋
```

### Distance Metrics

```
Manhattan (L1):  d(p, q) = |p.row - q.row| + |p.col - q.col|
Euclidean (L2):  d(p, q) = √[(p.row - q.row)² + (p.col - q.col)²]
Chebyshev (L∞): d(p, q) = max(|p.row - q.row|, |p.col - q.col|)
```

### Spatial Hashing

```
Key: "row:col"  (O(1) collision detection)

occupied: Map<string, string>  // "row:col" → iconId
```

### BFS for Nearest Available

```
Time Complexity:  O(rows × cols) worst case
Space Complexity: O(rows × cols) for visited set
```

## Testing

```bash
# Run tests
npm test features/icons

# Component tests
npm test Icon.test.tsx

# Hook tests
npm test useGrid.test.ts
```

## Performance

- **Grid calculations**: O(1) - constant time
- **Collision detection**: O(1) - spatial hashing
- **Auto-arrange**: O(n log n) - sorting algorithms
- **Nearest available**: O(n) - BFS traversal
- **Memory**: O(n) - linear in number of icons

## Future Enhancements

- [ ] Range selection with Shift+Click
- [ ] Selection box (drag to select multiple)
- [ ] Icon grouping/folders
- [ ] Custom grid layouts (spiral, circular)
- [ ] Animation springs for smoother movement
- [ ] Icon badges (notifications, status)
- [ ] Thumbnail generation for files
- [ ] Search/filter icons
- [ ] Keyboard navigation (arrow keys)
- [ ] Touch/gesture support

## References

- [Computational Geometry](https://en.wikipedia.org/wiki/Computational_geometry)
- [Spatial Hashing](https://en.wikipedia.org/wiki/Spatial_database#Spatial_index)
- [k-means Clustering](https://en.wikipedia.org/wiki/K-means_clustering)
- [Breadth-First Search](https://en.wikipedia.org/wiki/Breadth-first_search)

