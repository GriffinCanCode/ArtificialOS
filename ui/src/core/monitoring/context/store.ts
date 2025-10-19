/**
 * Hierarchical Context System for AgentOS
 *
 * Automatically tracks and provides UI hierarchy context:
 * Desktop > Window > App > Component
 *
 * This enables powerful debugging where every log knows exactly where it came from
 * in the UI tree, making complex multi-window, multi-app debugging much easier.
 */

import { create } from "zustand";
import { subscribeWithSelector } from "zustand/middleware";
import type { LogContext } from "../core/logger";

// ============================================================================
// Types
// ============================================================================

export interface HierarchicalContext {
  /** Desktop-level context (session, user, etc.) */
  desktop: {
    sessionId?: string;
    userId?: string;
    appVersion?: string;
    environment?: 'development' | 'production';
    platform?: string;
  };

  /** Current window context */
  window?: {
    windowId: string;
    appId: string;
    title: string;
    appType: 'blueprint' | 'native_web' | 'native_proc';
    zIndex: number;
    isFocused: boolean;
  };

  /** Current app context within the window */
  app?: {
    appId: string;
    instanceId: string;
    type: 'blueprint' | 'native_web' | 'native_proc';
    packageId?: string; // For native apps
    services?: string[];
    permissions?: string[];
  };

  /** Current component context */
  component?: {
    componentId: string;
    componentType: string;
    parentComponentId?: string;
    depth: number; // How deep in component tree
  };
}

export interface ContextBreadcrumb {
  level: 'desktop' | 'window' | 'app' | 'component';
  id: string;
  label: string;
  metadata?: Record<string, unknown>;
}

// ============================================================================
// Store Interface
// ============================================================================

interface HierarchicalContextStore {
  context: HierarchicalContext;
  breadcrumbs: ContextBreadcrumb[];

  // Actions
  setDesktopContext: (context: Partial<HierarchicalContext['desktop']>) => void;
  setWindowContext: (context: HierarchicalContext['window']) => void;
  setAppContext: (context: HierarchicalContext['app']) => void;
  setComponentContext: (context: HierarchicalContext['component']) => void;
  clearWindowContext: () => void;
  clearAppContext: () => void;
  clearComponentContext: () => void;

  // Utilities
  getLogContext: () => LogContext;
  getBreadcrumbPath: () => string;
  getCurrentWindowId: () => string | undefined;
  getCurrentAppId: () => string | undefined;

  // Context tracking
  startTrackingComponent: (componentId: string, componentType: string, parentId?: string) => () => void;
}

// ============================================================================
// Store Implementation
// ============================================================================

export const useHierarchicalContextStore = create<HierarchicalContextStore>()(
  subscribeWithSelector((set, get) => ({
    context: {
      desktop: {
        sessionId: generateSessionId(),
        appVersion: '1.0.0',
        environment: (process.env.NODE_ENV as 'development' | 'production') || 'development',
        platform: getPlatform(),
      },
    },
    breadcrumbs: [],

    setDesktopContext: (context) => {
      set((state) => ({
        context: {
          ...state.context,
          desktop: { ...state.context.desktop, ...context },
        },
      }));
      updateBreadcrumbs();
    },

    setWindowContext: (windowContext) => {
      set((state) => ({
        context: {
          ...state.context,
          window: windowContext,
        },
      }));
      updateBreadcrumbs();
    },

    setAppContext: (appContext) => {
      set((state) => ({
        context: {
          ...state.context,
          app: appContext,
        },
      }));
      updateBreadcrumbs();
    },

    setComponentContext: (componentContext) => {
      set((state) => ({
        context: {
          ...state.context,
          component: componentContext,
        },
      }));
      updateBreadcrumbs();
    },

    clearWindowContext: () => {
      set((state) => ({
        context: {
          desktop: state.context.desktop,
        },
      }));
      updateBreadcrumbs();
    },

    clearAppContext: () => {
      set((state) => ({
        context: {
          desktop: state.context.desktop,
          window: state.context.window,
        },
      }));
      updateBreadcrumbs();
    },

    clearComponentContext: () => {
      set((state) => ({
        context: {
          desktop: state.context.desktop,
          window: state.context.window,
          app: state.context.app,
        },
      }));
      updateBreadcrumbs();
    },

    getLogContext: (): LogContext => {
      const { context } = get();

      return {
        // Flatten hierarchy for logging
        sessionId: context.desktop.sessionId,
        environment: context.desktop.environment,
        platform: context.desktop.platform,

        windowId: context.window?.windowId,
        appId: context.window?.appId || context.app?.appId,
        appType: context.app?.type || context.window?.appType,
        windowTitle: context.window?.title,
        windowFocused: context.window?.isFocused,
        zIndex: context.window?.zIndex,

        appInstanceId: context.app?.instanceId,
        packageId: context.app?.packageId,
        services: context.app?.services,
        permissions: context.app?.permissions,

        componentId: context.component?.componentId,
        componentType: context.component?.componentType,
        componentDepth: context.component?.depth,
        parentComponentId: context.component?.parentComponentId,

        // Add breadcrumb path for easy debugging
        breadcrumbPath: get().getBreadcrumbPath(),
      };
    },

    getBreadcrumbPath: (): string => {
      const { context } = get();
      const parts: string[] = [];

      if (context.window) {
        parts.push(`window:${context.window.windowId.slice(-8)}`);
      }
      if (context.app) {
        parts.push(`app:${context.app.appId}`);
      }
      if (context.component) {
        parts.push(`component:${context.component.componentId}`);
      }

      return parts.join(' > ') || 'desktop';
    },

    getCurrentWindowId: () => {
      return get().context.window?.windowId;
    },

    getCurrentAppId: () => {
      return get().context.app?.appId || get().context.window?.appId;
    },

    startTrackingComponent: (componentId: string, componentType: string, parentId?: string) => {
      const currentComponent = get().context.component;
      const depth = parentId && currentComponent?.componentId === parentId
        ? (currentComponent.depth + 1)
        : 0;

      get().setComponentContext({
        componentId,
        componentType,
        parentComponentId: parentId,
        depth,
      });

      // Return cleanup function
      return () => {
        const current = get().context.component;
        if (current?.componentId === componentId) {
          if (parentId) {
            // Restore parent component context
            get().setComponentContext({
              componentId: parentId,
              componentType: 'unknown', // Would need to track this better
              depth: depth - 1,
            });
          } else {
            get().clearComponentContext();
          }
        }
      };
    },
  }))
);

// ============================================================================
// Internal Helpers
// ============================================================================

function updateBreadcrumbs() {
  const store = useHierarchicalContextStore.getState();
  const { context } = store;
  const breadcrumbs: ContextBreadcrumb[] = [];

  // Desktop breadcrumb
  breadcrumbs.push({
    level: 'desktop',
    id: 'desktop',
    label: `Desktop (${context.desktop.environment})`,
    metadata: context.desktop,
  });

  // Window breadcrumb
  if (context.window) {
    breadcrumbs.push({
      level: 'window',
      id: context.window.windowId,
      label: context.window.title || 'Untitled Window',
      metadata: context.window,
    });
  }

  // App breadcrumb
  if (context.app) {
    breadcrumbs.push({
      level: 'app',
      id: context.app.appId,
      label: `${context.app.appId} (${context.app.type})`,
      metadata: context.app,
    });
  }

  // Component breadcrumb
  if (context.component) {
    breadcrumbs.push({
      level: 'component',
      id: context.component.componentId,
      label: `${context.component.componentType} (depth: ${context.component.depth})`,
      metadata: context.component,
    });
  }

  useHierarchicalContextStore.setState({ breadcrumbs });
}

function generateSessionId(): string {
  return `session_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`;
}

function getPlatform(): string {
  if (typeof window !== 'undefined') {
    return window.navigator?.platform || 'unknown';
  }
  return 'server';
}

// ============================================================================
// Context Provider Integration
// ============================================================================

/**
 * Window context provider - automatically sets window context when rendered
 */
export interface WindowContextProviderProps {
  windowId: string;
  appId: string;
  title: string;
  appType: 'blueprint' | 'native_web' | 'native_proc';
  zIndex: number;
  isFocused: boolean;
  children: React.ReactNode;
}

/**
 * App context provider - automatically sets app context when rendered
 */
export interface AppContextProviderProps {
  appId: string;
  instanceId: string;
  type: 'blueprint' | 'native_web' | 'native_proc';
  packageId?: string;
  services?: string[];
  permissions?: string[];
  children: React.ReactNode;
}

// ============================================================================
// Public API
// ============================================================================

/**
 * Get current hierarchical context for logging
 */
export function getHierarchicalLogContext(): LogContext {
  return useHierarchicalContextStore.getState().getLogContext();
}

/**
 * Get current breadcrumb path for debugging
 */
export function getCurrentBreadcrumbPath(): string {
  return useHierarchicalContextStore.getState().getBreadcrumbPath();
}

/**
 * Set desktop-level context (session, environment, etc.)
 */
export function setDesktopContext(context: Partial<HierarchicalContext['desktop']>): void {
  useHierarchicalContextStore.getState().setDesktopContext(context);
}

/**
 * Manually set window context (usually done automatically by WindowContextProvider)
 */
export function setWindowContext(context: HierarchicalContext['window']): void {
  useHierarchicalContextStore.getState().setWindowContext(context);
}

/**
 * Manually set app context (usually done automatically by AppContextProvider)
 */
export function setAppContext(context: HierarchicalContext['app']): void {
  useHierarchicalContextStore.getState().setAppContext(context);
}

/**
 * Track a component and automatically clean up when it unmounts
 */
export function trackComponent(
  componentId: string,
  componentType: string,
  parentId?: string
): () => void {
  return useHierarchicalContextStore.getState().startTrackingComponent(
    componentId,
    componentType,
    parentId
  );
}

/**
 * Get store for direct access (advanced usage)
 */
export function getHierarchicalContextStore() {
  return useHierarchicalContextStore;
}
