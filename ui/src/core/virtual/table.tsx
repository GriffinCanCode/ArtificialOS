/**
 * Virtual Table Component
 * High-performance table virtualization with column virtualization
 */

import { useRef, useState } from "react";
import { useVirtualizer } from "@tanstack/react-virtual";
import { toRgbaString } from "@/core/utils/color";
import type { VirtualTableConfig, VirtualTableColumn } from "./types";

// ============================================================================
// Virtual Table
// ============================================================================

interface VirtualTableProps<T = any> extends VirtualTableConfig<T> {
  onSelectionChange?: (selectedIndices: number[]) => void;
}

/**
 * Generic virtual table with row and optional column virtualization
 */
export const VirtualTable = <T,>({
  items,
  columns,
  height = 600,
  rowHeight = 48,
  enableColumnVirtualization = false,
  enableRowSelection = false,
  onRowClick,
  onSelectionChange,
  className = "",
  overscan = 5,
}: VirtualTableProps<T>) => {
  const parentRef = useRef<HTMLDivElement>(null);
  const [selectedRows, setSelectedRows] = useState<Set<number>>(new Set());

  // Virtualize rows
  const rowVirtualizer = useVirtualizer({
    count: items.length,
    getScrollElement: () => parentRef.current,
    estimateSize: typeof rowHeight === "function" ? rowHeight : () => rowHeight,
    overscan,
  });

  // Optionally virtualize columns
  const columnVirtualizer = useVirtualizer({
    horizontal: true,
    count: columns.length,
    getScrollElement: () => parentRef.current,
    estimateSize: (index) => columns[index].width || 150,
    overscan: 3,
    enabled: enableColumnVirtualization,
  });

  const virtualRows = rowVirtualizer.getVirtualItems();
  const virtualColumns = enableColumnVirtualization
    ? columnVirtualizer.getVirtualItems()
    : null;

  const handleRowClick = (index: number) => {
    const row = items[index];

    if (enableRowSelection) {
      setSelectedRows((prev) => {
        const newSet = new Set(prev);
        if (newSet.has(index)) {
          newSet.delete(index);
        } else {
          newSet.add(index);
        }
        onSelectionChange?.(Array.from(newSet));
        return newSet;
      });
    }

    onRowClick?.(row, index);
  };

  const getCellValue = (row: T, column: VirtualTableColumn<T>) => {
    if (typeof column.accessor === "function") {
      return column.accessor(row);
    }
    return row[column.accessor as keyof T];
  };

  return (
    <div
      ref={parentRef}
      className={`virtual-table ${className}`}
      style={{
        height: typeof height === "number" ? `${height}px` : height,
        overflow: "auto",
        contain: "strict",
      }}
    >
      {/* Header */}
      <div
        style={{
          display: "flex",
          position: "sticky",
          top: 0,
          zIndex: 1,
          background: "var(--color-surface, #1a1a1a)",
          borderBottom: "1px solid var(--color-border, #2a2a2a)",
        }}
      >
        {enableRowSelection && (
          <div style={{ width: 40, padding: "12px 8px", fontWeight: 600 }}>
            <input
              type="checkbox"
              onChange={(e) => {
                if (e.target.checked) {
                  const allIndices = Array.from({ length: items.length }, (_, i) => i);
                  setSelectedRows(new Set(allIndices));
                  onSelectionChange?.(allIndices);
                } else {
                  setSelectedRows(new Set());
                  onSelectionChange?.([]);
                }
              }}
              checked={selectedRows.size === items.length && items.length > 0}
            />
          </div>
        )}
        {enableColumnVirtualization && virtualColumns
          ? virtualColumns.map((virtualCol) => {
              const column = columns[virtualCol.index];
              return (
                <div
                  key={column.id}
                  style={{
                    width: column.width || 150,
                    minWidth: column.minWidth || 80,
                    maxWidth: column.maxWidth,
                    padding: "12px 16px",
                    fontWeight: 600,
                    textAlign: "left",
                  }}
                >
                  {column.header}
                </div>
              );
            })
          : columns.map((column) => {
              return (
            <div
              key={column.id}
              style={{
                width: column.width || 150,
                minWidth: column.minWidth || 80,
                maxWidth: column.maxWidth,
                padding: "12px 16px",
                fontWeight: 600,
                textAlign: "left",
              }}
            >
              {column.header}
            </div>
          );
        })}
      </div>

      {/* Body */}
      <div
        style={{
          height: `${rowVirtualizer.getTotalSize()}px`,
          width: "100%",
          position: "relative",
        }}
      >
        {virtualRows.map((virtualRow) => {
          const row = items[virtualRow.index];
          const isSelected = selectedRows.has(virtualRow.index);

          return (
            <div
              key={virtualRow.index}
              style={{
                position: "absolute",
                top: 0,
                left: 0,
                width: "100%",
                height: `${virtualRow.size}px`,
                transform: `translateY(${virtualRow.start}px)`,
                display: "flex",
                alignItems: "center",
                background: isSelected
                  ? `var(--color-primary-alpha-10, ${toRgbaString("#667eea", 0.1)})`
                  : virtualRow.index % 2 === 0
                    ? "var(--color-background, #0a0a0a)"
                    : "var(--color-surface, #1a1a1a)",
                cursor: onRowClick ? "pointer" : "default",
                borderBottom: "1px solid var(--color-border, #2a2a2a)",
              }}
              onClick={() => handleRowClick(virtualRow.index)}
            >
              {enableRowSelection && (
                <div style={{ width: 40, padding: "0 8px" }}>
                  <input
                    type="checkbox"
                    checked={isSelected}
                    onChange={() => handleRowClick(virtualRow.index)}
                    onClick={(e) => e.stopPropagation()}
                  />
                </div>
              )}
              {enableColumnVirtualization && virtualColumns
                ? virtualColumns.map((virtualCol) => {
                    const column = columns[virtualCol.index];
                    const value = getCellValue(row, column);
                    const cellContent = column.cell
                      ? column.cell(value, row, virtualRow.index)
                      : value;

                    return (
                      <div
                        key={column.id}
                        style={{
                          width: column.width || 150,
                          minWidth: column.minWidth || 80,
                          maxWidth: column.maxWidth,
                          padding: "12px 16px",
                          overflow: "hidden",
                          textOverflow: "ellipsis",
                          whiteSpace: "nowrap",
                        }}
                      >
                        {cellContent}
                      </div>
                    );
                  })
                : columns.map((column) => {
                    const value = getCellValue(row, column);
                    const cellContent = column.cell
                      ? column.cell(value, row, virtualRow.index)
                      : value;

                    return (
                      <div
                        key={column.id}
                        style={{
                          width: column.width || 150,
                          minWidth: column.minWidth || 80,
                          maxWidth: column.maxWidth,
                          padding: "12px 16px",
                          overflow: "hidden",
                          textOverflow: "ellipsis",
                          whiteSpace: "nowrap",
                        }}
                      >
                        {cellContent}
                      </div>
                    );
                  })}
            </div>
          );
        })}
      </div>
    </div>
  );
};

VirtualTable.displayName = "VirtualTable";
