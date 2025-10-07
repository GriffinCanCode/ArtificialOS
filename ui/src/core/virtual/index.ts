/**
 * Virtual Scrolling System
 * High-performance virtualization using @tanstack/react-virtual
 */

// Components
export { VirtualList, SimpleList, DynamicList } from "./list";
export { VirtualGrid, MasonryGrid } from "./grid";
export { VirtualTable } from "./table";

// Hooks
export {
  useVirtualList,
  useVirtualGrid,
  useMeasure,
  useAutoOverscan,
  useScrollToItem,
  useInfiniteScroll,
  useVirtualMetrics,
} from "./hooks";

// Utilities
export {
  fixedSize,
  variableSize,
  autoSize,
  dynamicSize,
  measureElement,
  measureWidth,
  batchMeasure,
  calculateGridDimensions,
  indexToGrid,
  gridToIndex,
  generateKey,
  generateItemKey,
  calculateOverscan,
  throttleScroll,
} from "./utils";

// Types
export type {
  VirtualBaseProps,
  VirtualComponentProps,
  VirtualComponentRenderer,
  VirtualListConfig,
  VirtualListItemProps,
  VirtualGridConfig,
  VirtualGridItemProps,
  VirtualTableConfig,
  VirtualTableColumn,
  VirtualTableRowProps,
  SizeEstimator,
  MeasuredSize,
  VirtualMetrics,
  AutoSizeConfig,
} from "./types";
