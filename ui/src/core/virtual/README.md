# Virtual Scrolling System

High-performance virtualization system built on `@tanstack/react-virtual` with advanced features and strong TypeScript support.

## Features

- ✅ **Variable-size items** - Dynamic content sizing
- ✅ **Grid virtualization** - Multi-column layouts with dynamic columns
- ✅ **Table virtualization** - Row and column virtualization with selection
- ✅ **Horizontal scrolling** - Full horizontal virtualization support
- ✅ **Masonry layouts** - Variable height grid items
- ✅ **Infinite scroll** - Built-in infinite loading support
- ✅ **Strong typing** - Full TypeScript support
- ✅ **Performance optimized** - Minimal re-renders, efficient updates
- ✅ **Extensible** - Easy to customize and extend

## Installation

Already installed as part of the project dependencies:

```bash
npm install @tanstack/react-virtual
```

## Basic Usage

### Simple List

```tsx
import { SimpleList } from "@/core/virtual";

function MyList() {
  const items = Array.from({ length: 10000 }, (_, i) => ({
    id: i,
    name: `Item ${i}`,
  }));

  return (
    <SimpleList items={items} itemSize={60} height={600}>
      {({ item, style }) => (
        <div style={style} className="list-item">
          {item.name}
        </div>
      )}
    </SimpleList>
  );
}
```

### Variable Size List

```tsx
import { VirtualList } from "@/core/virtual";

function VariableList() {
  const items = [...]; // your items

  return (
    <VirtualList
      items={items}
      estimateSize={(index) => {
        // Estimate item size based on content
        return items[index].height || 60;
      }}
      height={600}
    >
      {({ item, style }) => (
        <div style={style}>{item.content}</div>
      )}
    </VirtualList>
  );
}
```

### Virtual Grid

```tsx
import { VirtualGrid } from "@/core/virtual";

function MyGrid() {
  const items = Array.from({ length: 10000 }, (_, i) => ({
    id: i,
    title: `Card ${i}`,
  }));

  return (
    <VirtualGrid
      items={items}
      columns={4}
      rowHeight={250}
      gap={16}
      height={600}
    >
      {({ item, style }) => (
        <div style={style} className="card">
          <h3>{item.title}</h3>
        </div>
      )}
    </VirtualGrid>
  );
}
```

### Virtual Table

```tsx
import { VirtualTable } from "@/core/virtual";

function MyTable() {
  const data = [...]; // your data

  const columns = [
    { id: "id", header: "ID", accessor: "id", width: 80 },
    { id: "name", header: "Name", accessor: "name", width: 200 },
    {
      id: "email",
      header: "Email",
      accessor: "email",
      width: 250,
    },
  ];

  return (
    <VirtualTable
      items={data}
      columns={columns}
      height={600}
      rowHeight={48}
      enableRowSelection
      onRowClick={(row) => console.log(row)}
    />
  );
}
```

## Advanced Usage

### Horizontal Virtualization

```tsx
<VirtualList
  items={items}
  estimateSize={() => 150}
  horizontal
  height="100%"
>
  {({ item, style }) => (
    <div style={style}>{item.label}</div>
  )}
</VirtualList>
```

### Masonry Grid

```tsx
import { MasonryGrid } from "@/core/virtual";

<MasonryGrid
  items={items}
  columns={3}
  getItemHeight={(item) => item.height}
  gap={16}
  height={600}
>
  {({ item, style }) => (
    <div style={{ ...style, height: item.height }}>
      {item.content}
    </div>
  )}
</MasonryGrid>
```

### Infinite Scroll

```tsx
import { useVirtualList, useInfiniteScroll } from "@/core/virtual";

function InfiniteList() {
  const [items, setItems] = useState([...initialItems]);
  const [hasMore, setHasMore] = useState(true);

  const loadMore = async () => {
    const newItems = await fetchMoreItems();
    setItems((prev) => [...prev, ...newItems]);
    if (newItems.length === 0) setHasMore(false);
  };

  const { parentRef, virtualizer, virtualItems } = useVirtualList(items);
  const { isLoading } = useInfiniteScroll(virtualizer, {
    hasMore,
    loadMore,
    threshold: 10,
  });

  // Render virtual items...
}
```

### Dynamic Sizing with Measurement

```tsx
import { useMeasure } from "@/core/virtual";

function DynamicList() {
  const { measureRef, getSize } = useMeasure();

  return (
    <VirtualList
      items={items}
      estimateSize={(index) => getSize(index, 60)}
    >
      {({ item, index, style }) => (
        <div
          ref={(el) => measureRef(index, el)}
          style={style}
        >
          {item.content}
        </div>
      )}
    </VirtualList>
  );
}
```

### Scroll Control

```tsx
import { useVirtualList, useScrollToItem } from "@/core/virtual";

function ScrollableList() {
  const { virtualizer, ...rest } = useVirtualList(items);
  const { scrollToItem, scrollToTop, scrollToBottom } = useScrollToItem(virtualizer);

  return (
    <>
      <button onClick={() => scrollToItem(50)}>Jump to item 50</button>
      <button onClick={scrollToTop}>Top</button>
      <button onClick={scrollToBottom}>Bottom</button>
      {/* Render list... */}
    </>
  );
}
```

## API Reference

### Components

#### `VirtualList`
- `items` - Array of items to render
- `height` - Container height (number or string)
- `estimateSize` - Size estimator function
- `getItemKey` - Key generator for items
- `overscan` - Number of items to render outside viewport
- `horizontal` - Enable horizontal scrolling
- `children` - Render function

#### `SimpleList`
- Same as VirtualList but with `itemSize` instead of `estimateSize`

#### `VirtualGrid`
- `items` - Array of items
- `columns` - Number of columns
- `rowHeight` - Fixed or dynamic row height
- `gap` - Gap between items
- `height`, `width` - Container dimensions

#### `VirtualTable`
- `items` - Array of data rows
- `columns` - Column definitions
- `rowHeight` - Row height (fixed or dynamic)
- `enableColumnVirtualization` - Virtualize columns
- `enableRowSelection` - Enable row selection
- `onRowClick` - Row click handler

### Hooks

#### `useVirtualList(items, options)`
Returns virtualizer instance and helpers

#### `useVirtualGrid(items, columns, options)`
Grid-specific virtualization setup

#### `useMeasure()`
Dynamic size measurement

#### `useScrollToItem(virtualizer)`
Programmatic scrolling

#### `useInfiniteScroll(virtualizer, options)`
Infinite scroll loading

#### `useVirtualMetrics(virtualizer)`
Performance metrics

### Utilities

#### Size Estimators
- `fixedSize(size)` - Fixed size for all items
- `variableSize(sizes, default)` - Variable sizes with fallback
- `autoSize(config)` - Auto-sizing with bounds
- `dynamicSize(measureFn, cache)` - Dynamic measurement

#### Grid Utilities
- `calculateGridDimensions(total, columns)`
- `indexToGrid(index, columns)`
- `gridToIndex(row, col, columns)`

#### Key Generators
- `generateKey(index, prefix)`
- `generateItemKey(item, index, prefix)`

## Performance Tips

1. **Use memoization** - Memoize item components
2. **Stable keys** - Use stable keys for items
3. **Estimate accurately** - Better estimates = fewer re-measurements
4. **Limit overscan** - Don't render too many off-screen items
5. **Avoid inline functions** - Define render functions outside

## Integration with Dynamics

```tsx
import { SimpleList } from "@/core/virtual";
import { ComponentRenderer } from "@/features/dynamics/rendering/renderer";

function DynamicComponentList({ components, state, executor }) {
  return (
    <SimpleList
      items={components}
      itemSize={100}
      height={600}
      getItemKey={(index, component) => component.id}
    >
      {({ item: component, style }) => (
        <div style={style}>
          <ComponentRenderer
            component={component}
            state={state}
            executor={executor}
          />
        </div>
      )}
    </SimpleList>
  );
}
```

## TypeScript Support

All components and hooks are fully typed:

```typescript
import type {
  VirtualListConfig,
  VirtualGridConfig,
  VirtualTableColumn,
  SizeEstimator,
} from "@/core/virtual";

// Define your data type
interface MyData {
  id: number;
  name: string;
}

// Type-safe column definitions
const columns: VirtualTableColumn<MyData>[] = [
  { id: "id", header: "ID", accessor: "id" },
  { id: "name", header: "Name", accessor: "name" },
];
```

## Testing

All components are designed to be easily testable:

```tsx
import { render } from "@testing-library/react";
import { SimpleList } from "@/core/virtual";

test("renders list items", () => {
  const items = [{ id: 1 }, { id: 2 }];
  
  const { container } = render(
    <SimpleList items={items} itemSize={60} height={600}>
      {({ item }) => <div>{item.id}</div>}
    </SimpleList>
  );
  
  expect(container).toMatchSnapshot();
});
```

## Browser Support

- Chrome/Edge 90+
- Firefox 88+
- Safari 14+

Requires support for:
- CSS containment
- Intersection Observer (for infinite scroll)
- RequestAnimationFrame

## Migration from react-window

The new system is more powerful and flexible:

```tsx
// Old (react-window)
<FixedSizeList
  height={600}
  itemCount={items.length}
  itemSize={60}
  width="100%"
>
  {({ index, style }) => (
    <div style={style}>{items[index].name}</div>
  )}
</FixedSizeList>

// New (TanStack Virtual)
<SimpleList
  items={items}
  itemSize={60}
  height={600}
>
  {({ item, style }) => (
    <div style={style}>{item.name}</div>
  )}
</SimpleList>
```

## Examples

See `examples.ts` for comprehensive usage examples.
