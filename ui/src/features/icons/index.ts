/**
 * Icons Feature
 * Desktop icon management system
 */

// Components
export { Icon } from "./components/Icon";
export { Grid } from "./components/Grid";
export { IconContextMenu } from "./components/Context";

// Hooks
export { useGrid } from "./hooks/useGrid";
export { useSelect } from "./hooks/useSelect";
export { useDrag } from "./hooks/useDrag";
export { useDefaults } from "./hooks/useDefaults";

// Store
export {
  useStore as useIconStore,
  useIcons,
  useActions as useIconActions,
  useSelectedIcons,
  useIcon,
  useDraggedIds,
  useViewport,
} from "./store/store";

// Types
export type {
  Icon as IconType,
  IconMetadata,
  GridPosition,
  PixelPosition,
  GridConfig,
  IconSize,
  ArrangeStrategy,
} from "./core/types";

// Utilities
export { arrange, compactLayout } from "./utils/arrange";
export { getDefaultIcons, shouldInitializeDefaults } from "./utils/defaults";
export * from "./core/grid";
export * from "./core/collision";

