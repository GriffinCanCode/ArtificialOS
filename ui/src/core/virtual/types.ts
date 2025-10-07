/**
 * Virtual Scrolling Types
 * Strong TypeScript definitions for virtualization system
 */

import type { BlueprintComponent } from "../store/appStore";
import type { ComponentState } from "../../features/dynamics/state/state";
import type { ToolExecutor } from "../../features/dynamics/execution/executor";

// ============================================================================
// Core Types
// ============================================================================

/**
 * Base props for all virtualized components
 */
export interface VirtualBaseProps<T = any> {
  items: T[];
  className?: string;
  overscan?: number;
  scrollMargin?: number;
}

/**
 * Props for component rendering in virtual lists
 */
export interface VirtualComponentProps {
  items: BlueprintComponent[];
  state: ComponentState;
  executor: ToolExecutor;
  renderer: VirtualComponentRenderer;
  className?: string;
  overscan?: number;
}

/**
 * Component renderer function type
 */
export type VirtualComponentRenderer = React.ComponentType<{
  component: BlueprintComponent;
  state: ComponentState;
  executor: ToolExecutor;
}>;

// ============================================================================
// List Types
// ============================================================================

/**
 * Virtual list configuration
 */
export interface VirtualListConfig<T = any> extends VirtualBaseProps<T> {
  height?: number | string;
  estimateSize?: (index: number) => number;
  getItemKey?: (index: number, item: T) => string | number;
  horizontal?: boolean;
}

/**
 * Virtual list item render props
 */
export interface VirtualListItemProps<T = any> {
  item: T;
  index: number;
  style: React.CSSProperties;
}

// ============================================================================
// Grid Types
// ============================================================================

/**
 * Virtual grid configuration
 */
export interface VirtualGridConfig<T = any> extends VirtualBaseProps<T> {
  columns: number;
  rowHeight?: number | ((index: number) => number);
  height?: number | string;
  width?: number | string;
  gap?: number;
}

/**
 * Virtual grid item render props
 */
export interface VirtualGridItemProps<T = any> {
  item: T;
  rowIndex: number;
  columnIndex: number;
  index: number;
  style: React.CSSProperties;
}

// ============================================================================
// Table Types
// ============================================================================

/**
 * Table column definition
 */
export interface VirtualTableColumn<T = any> {
  id: string;
  header: string | React.ReactNode;
  accessor: keyof T | ((row: T) => any);
  width?: number;
  minWidth?: number;
  maxWidth?: number;
  cell?: (value: any, row: T, rowIndex: number) => React.ReactNode;
}

/**
 * Virtual table configuration
 */
export interface VirtualTableConfig<T = any> extends VirtualBaseProps<T> {
  columns: VirtualTableColumn<T>[];
  height?: number | string;
  rowHeight?: number | ((index: number) => number);
  enableColumnVirtualization?: boolean;
  enableRowSelection?: boolean;
  onRowClick?: (row: T, index: number) => void;
}

/**
 * Virtual table row render props
 */
export interface VirtualTableRowProps<T = any> {
  row: T;
  index: number;
  columns: VirtualTableColumn<T>[];
  style: React.CSSProperties;
  selected?: boolean;
  onClick?: () => void;
}

// ============================================================================
// Utility Types
// ============================================================================

/**
 * Size estimation function type
 */
export type SizeEstimator = (index: number) => number;

/**
 * Dynamic size measurement result
 */
export interface MeasuredSize {
  index: number;
  size: number;
}

/**
 * Virtualization metrics for monitoring
 */
export interface VirtualMetrics {
  totalItems: number;
  visibleItems: number;
  scrollOffset: number;
  totalSize: number;
  overscan: number;
}

/**
 * Auto-size configuration
 */
export interface AutoSizeConfig {
  minSize?: number;
  maxSize?: number;
  defaultSize: number;
}
