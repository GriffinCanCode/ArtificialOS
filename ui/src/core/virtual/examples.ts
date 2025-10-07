/**
 * Virtual Scrolling Examples
 * Usage examples and patterns
 */

// ============================================================================
// Basic List Example
// ============================================================================

/*
import { SimpleList } from "@/core/virtual";

function MyList() {
  const items = Array.from({ length: 10000 }, (_, i) => ({
    id: i,
    name: `Item ${i}`,
  }));

  return (
    <SimpleList
      items={items}
      itemSize={60}
      height={600}
      className="my-list"
    >
      {({ item, style }) => (
        <div style={style} className="list-item">
          {item.name}
        </div>
      )}
    </SimpleList>
  );
}
*/

// ============================================================================
// Variable Size List Example
// ============================================================================

/*
import { VirtualList } from "@/core/virtual";

function VariableList() {
  const items = Array.from({ length: 1000 }, (_, i) => ({
    id: i,
    content: `Item ${i}`.repeat(Math.random() * 10),
  }));

  return (
    <VirtualList
      items={items}
      estimateSize={(index) => {
        // Estimate based on content length
        return Math.max(60, items[index].content.length * 0.5);
      }}
      height={600}
    >
      {({ item, style }) => (
        <div style={style} className="variable-item">
          {item.content}
        </div>
      )}
    </VirtualList>
  );
}
*/

// ============================================================================
// Grid Example
// ============================================================================

/*
import { VirtualGrid } from "@/core/virtual";

function MyGrid() {
  const items = Array.from({ length: 10000 }, (_, i) => ({
    id: i,
    title: `Card ${i}`,
    image: `/images/${i}.jpg`,
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
        <div style={style} className="grid-card">
          <img src={item.image} alt={item.title} />
          <h3>{item.title}</h3>
        </div>
      )}
    </VirtualGrid>
  );
}
*/

// ============================================================================
// Masonry Grid Example
// ============================================================================

/*
import { MasonryGrid } from "@/core/virtual";

function MasonryLayout() {
  const items = Array.from({ length: 1000 }, (_, i) => ({
    id: i,
    title: `Item ${i}`,
    height: Math.floor(Math.random() * 200) + 100,
  }));

  return (
    <MasonryGrid
      items={items}
      columns={3}
      getItemHeight={(item) => item.height}
      gap={16}
      height={600}
    >
      {({ item, style }) => (
        <div
          style={{ ...style, height: item.height }}
          className="masonry-item"
        >
          {item.title}
        </div>
      )}
    </MasonryGrid>
  );
}
*/

// ============================================================================
// Table Example
// ============================================================================

/*
import { VirtualTable } from "@/core/virtual";

function MyTable() {
  const data = Array.from({ length: 10000 }, (_, i) => ({
    id: i,
    name: `User ${i}`,
    email: `user${i}@example.com`,
    age: Math.floor(Math.random() * 50) + 20,
  }));

  const columns = [
    { id: "id", header: "ID", accessor: "id", width: 80 },
    { id: "name", header: "Name", accessor: "name", width: 200 },
    { id: "email", header: "Email", accessor: "email", width: 250 },
    {
      id: "age",
      header: "Age",
      accessor: "age",
      width: 100,
      cell: (value) => `${value} years`,
    },
  ];

  return (
    <VirtualTable
      items={data}
      columns={columns}
      height={600}
      rowHeight={48}
      enableRowSelection
      onRowClick={(row) => console.log("Clicked:", row)}
    />
  );
}
*/

// ============================================================================
// Horizontal List Example
// ============================================================================

/*
import { VirtualList } from "@/core/virtual";

function HorizontalList() {
  const items = Array.from({ length: 1000 }, (_, i) => ({
    id: i,
    label: `Item ${i}`,
  }));

  return (
    <VirtualList
      items={items}
      estimateSize={() => 150}
      horizontal
      height="100%"
    >
      {({ item, style }) => (
        <div style={style} className="horizontal-item">
          {item.label}
        </div>
      )}
    </VirtualList>
  );
}
*/

// ============================================================================
// With Infinite Scroll Example
// ============================================================================

/*
import { useState } from "react";
import { VirtualList, useInfiniteScroll, useVirtualList } from "@/core/virtual";

function InfiniteList() {
  const [items, setItems] = useState(
    Array.from({ length: 50 }, (_, i) => ({ id: i, name: `Item ${i}` }))
  );
  const [hasMore, setHasMore] = useState(true);

  const loadMore = async () => {
    // Simulate API call
    await new Promise(resolve => setTimeout(resolve, 1000));

    const newItems = Array.from(
      { length: 50 },
      (_, i) => ({ id: items.length + i, name: `Item ${items.length + i}` })
    );

    setItems(prev => [...prev, ...newItems]);

    if (items.length > 500) {
      setHasMore(false);
    }
  };

  const { parentRef, virtualizer, virtualItems } = useVirtualList(items);
  const { isLoading } = useInfiniteScroll(virtualizer, {
    hasMore,
    loadMore,
    threshold: 10,
  });

  return (
    <div ref={parentRef} style={{ height: 600, overflow: "auto" }}>
      <div style={{ height: virtualizer.getTotalSize(), position: "relative" }}>
        {virtualItems.map(virtualItem => {
          const item = items[virtualItem.index];
          return (
            <div
              key={item.id}
              style={{
                position: "absolute",
                top: 0,
                left: 0,
                width: "100%",
                height: virtualItem.size,
                transform: `translateY(${virtualItem.start}px)`,
              }}
            >
              {item.name}
            </div>
          );
        })}
      </div>
      {isLoading && <div>Loading more...</div>}
    </div>
  );
}
*/

// ============================================================================
// With Dynamic Component Rendering Example
// ============================================================================

/*
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
*/

export {};
