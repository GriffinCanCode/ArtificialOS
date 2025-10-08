/**
 * Icons Feature
 * Desktop icon management system
 */

// Components
export { Icon } from "./components/Icon";
export { Grid } from "./components/Grid";
export { IconContextMenu } from "./components/Context";
export { Search } from "./components/Search";
export { Badge } from "./components/Badge";

// Hooks
export { useGrid } from "./hooks/useGrid";
export { useSelect } from "./hooks/useSelect";
export { useDrag } from "./hooks/useDrag";
export { useDefaults } from "./hooks/useDefaults";
export { useSelectionBox } from "./hooks/useSelectionBox";
export { useKeyboard } from "./hooks/useKeyboard";

// Store
export {
  useStore as useIconStore,
  useIcons,
  useActions as useIconActions,
  useSelectedIcons,
  useIcon,
  useDraggedIds,
  useViewport,
  useSelectionBox as useSelectionBoxState,
  useSearchState,
  useSearchResults,
  useAnchorId,
} from "./store/store";

// Types
export type {
  Icon as IconType,
  IconMetadata,
  IconBadge,
  BadgeType,
  BadgePosition,
  GridPosition,
  PixelPosition,
  GridConfig,
  IconSize,
  ArrangeStrategy,
  SelectionBox,
  SearchState,
  SearchOptions,
} from "./core/types";

// Utilities
export { arrange, compactLayout } from "./utils/arrange";
export { getDefaultIcons, shouldInitializeDefaults } from "./utils/defaults";
export * from "./core/grid";
export * from "./core/collision";
export * from "./utils/badges";
export * from "./utils/search";
export * from "./utils/selection";

