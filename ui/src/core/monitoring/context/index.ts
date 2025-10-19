/**
 * Hierarchical Context Subsystem
 * Desktop > Window > App > Component context tracking
 */

export { useHierarchicalContextStore, getHierarchicalLogContext, getCurrentBreadcrumbPath, setDesktopContext, setWindowContext, setAppContext, trackComponent, getHierarchicalContextStore, type HierarchicalContext, type ContextBreadcrumb, type WindowContextProviderProps, type AppContextProviderProps } from './store';
export { WindowContextProvider, AppContextProvider, useComponentTracking, ContextBreadcrumbs, ContextDebugger, withComponentTracking, useComponentTracking as useTracking, WindowContextProvider as WindowContext, AppContextProvider as AppContext, withComponentTracking as withTracking } from './providers';

