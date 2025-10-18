/**
 * React Context Providers for Hierarchical Logging Context
 *
 * These providers automatically set and clean up hierarchical context
 * as components mount/unmount, ensuring logs always have proper context.
 */

import React, { useEffect } from 'react';
import {
  useHierarchicalContextStore,
  type WindowContextProviderProps,
  type AppContextProviderProps
} from './hierarchicalContext';

// ============================================================================
// Window Context Provider
// ============================================================================

/**
 * Automatically sets window context for all child components
 * Should be used at the Window component level
 */
export const WindowContextProvider: React.FC<WindowContextProviderProps> = ({
  windowId,
  appId,
  title,
  appType,
  zIndex,
  isFocused,
  children,
}) => {
  const setWindowContext = useHierarchicalContextStore((state) => state.setWindowContext);
  const clearWindowContext = useHierarchicalContextStore((state) => state.clearWindowContext);

  useEffect(() => {
    // Set window context when component mounts
    setWindowContext({
      windowId,
      appId,
      title,
      appType,
      zIndex,
      isFocused,
    });

    // Clear context when component unmounts
    return () => {
      clearWindowContext();
    };
  }, [windowId, appId, title, appType, zIndex, isFocused, setWindowContext, clearWindowContext]);

  // Update context when props change (e.g., focus changes)
  useEffect(() => {
    setWindowContext({
      windowId,
      appId,
      title,
      appType,
      zIndex,
      isFocused,
    });
  }, [windowId, appId, title, appType, zIndex, isFocused, setWindowContext]);

  return <>{children}</>;
};

// ============================================================================
// App Context Provider
// ============================================================================

/**
 * Automatically sets app context for all child components
 * Should be used when rendering app content within a window
 */
export const AppContextProvider: React.FC<AppContextProviderProps> = ({
  appId,
  instanceId,
  type,
  packageId,
  services,
  permissions,
  children,
}) => {
  const setAppContext = useHierarchicalContextStore((state) => state.setAppContext);
  const clearAppContext = useHierarchicalContextStore((state) => state.clearAppContext);

  useEffect(() => {
    // Set app context when component mounts
    setAppContext({
      appId,
      instanceId,
      type,
      packageId,
      services,
      permissions,
    });

    // Clear context when component unmounts
    return () => {
      clearAppContext();
    };
  }, [appId, instanceId, type, packageId, services, permissions, setAppContext, clearAppContext]);

  return <>{children}</>;
};

// ============================================================================
// Component Context Hook
// ============================================================================

/**
 * Hook to automatically track component context
 * Use this in any component that you want to track in logs
 */
export function useComponentTracking(
  componentId: string,
  componentType: string,
  parentComponentId?: string
): void {
  const startTrackingComponent = useHierarchicalContextStore(
    (state) => state.startTrackingComponent
  );

  useEffect(() => {
    const cleanup = startTrackingComponent(componentId, componentType, parentComponentId);
    return cleanup;
  }, [componentId, componentType, parentComponentId, startTrackingComponent]);
}

// ============================================================================
// Debugging Components
// ============================================================================

/**
 * Development component to show current context breadcrumbs
 * Only renders in development mode
 */
export const ContextBreadcrumbs: React.FC = () => {
  const breadcrumbPath = useHierarchicalContextStore((state) => state.getBreadcrumbPath());

  if (process.env.NODE_ENV !== 'development') {
    return null;
  }

  return (
    <div className="fixed top-2 left-2 z-[9999] bg-black/80 text-white text-xs px-2 py-1 rounded font-mono">
      <div className="font-semibold">Context Path:</div>
      <div className="opacity-75">{breadcrumbPath}</div>
    </div>
  );
};

/**
 * Development component to show full context details
 * Toggle with Ctrl/Cmd + Shift + L in development
 */
export const ContextDebugger: React.FC = () => {
  const [isVisible, setIsVisible] = React.useState(false);
  const context = useHierarchicalContextStore((state) => state.context);
  const logContext = useHierarchicalContextStore((state) => state.getLogContext());

  // Toggle visibility with keyboard shortcut
  useEffect(() => {
    const handleKeyDown = (event: KeyboardEvent) => {
      if ((event.ctrlKey || event.metaKey) && event.shiftKey && event.key === 'L') {
        event.preventDefault();
        setIsVisible((prev) => !prev);
      }
    };

    window.addEventListener('keydown', handleKeyDown);
    return () => window.removeEventListener('keydown', handleKeyDown);
  }, []);

  if (process.env.NODE_ENV !== 'development' || !isVisible) {
    return null;
  }

  return (
    <div className="fixed top-4 right-4 z-[9999] bg-black/90 text-white text-xs p-4 rounded-lg max-w-md max-h-96 overflow-auto font-mono">
      <div className="flex items-center justify-between mb-3">
        <h3 className="font-bold text-sm">Hierarchical Context Debug</h3>
        <button
          onClick={() => setIsVisible(false)}
          className="text-white/60 hover:text-white text-lg leading-none"
        >
          ×
        </button>
      </div>

      <div className="space-y-3">
        <div>
          <div className="text-green-400 font-semibold">Desktop Context:</div>
          <pre className="text-xs text-gray-300 ml-2">
            {JSON.stringify(context.desktop, null, 2)}
          </pre>
        </div>

        {context.window && (
          <div>
            <div className="text-blue-400 font-semibold">Window Context:</div>
            <pre className="text-xs text-gray-300 ml-2">
              {JSON.stringify(context.window, null, 2)}
            </pre>
          </div>
        )}

        {context.app && (
          <div>
            <div className="text-purple-400 font-semibold">App Context:</div>
            <pre className="text-xs text-gray-300 ml-2">
              {JSON.stringify(context.app, null, 2)}
            </pre>
          </div>
        )}

        {context.component && (
          <div>
            <div className="text-orange-400 font-semibold">Component Context:</div>
            <pre className="text-xs text-gray-300 ml-2">
              {JSON.stringify(context.component, null, 2)}
            </pre>
          </div>
        )}

        <div className="border-t border-gray-600 pt-3">
          <div className="text-yellow-400 font-semibold">Flattened Log Context:</div>
          <pre className="text-xs text-gray-300 ml-2">
            {JSON.stringify(logContext, null, 2)}
          </pre>
        </div>
      </div>

      <div className="text-xs text-gray-500 mt-3 pt-2 border-t border-gray-700">
        Press Ctrl/Cmd + Shift + L to toggle
      </div>
    </div>
  );
};

// ============================================================================
// Higher-Order Component for Automatic Context
// ============================================================================

/**
 * HOC to automatically add component tracking to any component
 * Simplified version to avoid complex generic type issues
 */
export function withComponentTracking(
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  Component: React.ComponentType<any>,
  componentType: string,
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  getComponentId?: (props: any) => string
// eslint-disable-next-line @typescript-eslint/no-explicit-any
): React.ComponentType<any> {
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  const WrappedComponent: React.FC<any> = (props) => {
    // Generate stable ID using component type and a counter instead of Math.random
    const componentId = React.useMemo(() =>
      getComponentId?.(props) || `${componentType}_${Date.now()}_${++idCounter}`,
      [props]
    );

    useComponentTracking(componentId, componentType);

    return React.createElement(Component, props);
  };

  WrappedComponent.displayName = `withComponentTracking(${Component.displayName || Component.name || componentType})`;

  return WrappedComponent;
}

// Counter for stable component IDs
let idCounter = 0;

// ============================================================================
// Convenience Exports
// ============================================================================

export {
  useComponentTracking as useTracking,
  WindowContextProvider as WindowContext,
  AppContextProvider as AppContext,
  withComponentTracking as withTracking,
};
