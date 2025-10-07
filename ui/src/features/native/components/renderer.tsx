/**
 * Native App Renderer
 * Renders loaded native apps with proper context injection
 *
 * Features:
 * - Lazy loading with suspense
 * - Error boundaries
 * - Context provision
 * - Lifecycle management
 * - Memory cleanup
 */

import React, { useEffect, useState, useMemo } from "react";
import { loader } from "../core/loader";
import { createAppContext } from "../../../core/sdk";
import { useActions } from "../../windows";
import { logger } from "../../../core/utils/monitoring/logger";
import type { NativeAppContext, NativeAppProps } from "../../../core/sdk";
import "./renderer.css";

// ============================================================================
// Props
// ============================================================================

interface RendererProps {
  /** Unique app instance ID */
  appId: string;
  /** Package ID in registry */
  packageId: string;
  /** Bundle path for loading */
  bundlePath: string;
  /** Window ID */
  windowId: string;
}

// ============================================================================
// Renderer Component
// ============================================================================

/**
 * Native App Renderer
 * Loads and renders native TypeScript/React applications
 */
export const Renderer: React.FC<RendererProps> = ({ appId, packageId, bundlePath, windowId }) => {
  const [Component, setComponent] = useState<React.ComponentType<NativeAppProps> | null>(null);
  const [error, setError] = useState<Error | null>(null);
  const [loading, setLoading] = useState(true);
  const windowActions = useActions();

  // Create stable app context
  const context = useMemo<NativeAppContext>(
    () => createAppContext(appId, windowId, windowActions),
    [appId, windowId, windowActions]
  );

  // Load the app module
  useEffect(() => {
    let mounted = true;
    let loadStartTime = performance.now();

    const loadApp = async () => {
      setLoading(true);
      setError(null);

      try {
        logger.info("Rendering native app", {
          component: "NativeRenderer",
          appId,
          packageId,
          windowId,
        });

        const loaded = await loader.load(packageId, bundlePath);

        if (!mounted) return;

        const loadTime = performance.now() - loadStartTime;
        logger.info("Native app render ready", {
          component: "NativeRenderer",
          packageId,
          loadTime: `${loadTime.toFixed(2)}ms`,
        });

        setComponent(() => loaded.component);
      } catch (err) {
        if (!mounted) return;

        const loadTime = performance.now() - loadStartTime;
        logger.error("Native app render failed", err as Error, {
          component: "NativeRenderer",
          packageId,
          loadTime: `${loadTime.toFixed(2)}ms`,
        });

        setError(err as Error);
      } finally {
        if (mounted) {
          setLoading(false);
        }
      }
    };

    loadApp();

    return () => {
      mounted = false;
    };
  }, [appId, packageId, bundlePath, windowId]);

  // Release app when unmounting
  useEffect(() => {
    return () => {
      logger.debug("Releasing native app", {
        component: "NativeRenderer",
        packageId,
      });
      loader.release(packageId);
    };
  }, [packageId]);

  // Cleanup executor on unmount
  useEffect(() => {
    return () => {
      context.executor.cleanup();
    };
  }, [context.executor]);

  // Render states
  if (loading) {
    return <LoadingState />;
  }

  if (error) {
    return <ErrorState error={error} packageId={packageId} />;
  }

  if (!Component) {
    return <ErrorState error={new Error("Component not loaded")} packageId={packageId} />;
  }

  // Render the native app
  return (
    <ErrorBoundary packageId={packageId}>
      <Component context={context} />
    </ErrorBoundary>
  );
};

// ============================================================================
// Loading State
// ============================================================================

const LoadingState: React.FC = () => (
  <div className="native-app-loading">
    <div className="loading-spinner" />
    <p className="loading-text">Loading app...</p>
  </div>
);

// ============================================================================
// Error State
// ============================================================================

interface ErrorStateProps {
  error: Error;
  packageId: string;
}

const ErrorState: React.FC<ErrorStateProps> = ({ error, packageId }) => (
  <div className="native-app-error">
    <div className="error-icon">⚠️</div>
    <h3 className="error-title">Failed to Load App</h3>
    <p className="error-message">{error.message}</p>
    <details className="error-details">
      <summary>Technical Details</summary>
      <code className="error-stack">
        <pre>{error.stack || "No stack trace available"}</pre>
      </code>
    </details>
    <p className="error-package">Package: {packageId}</p>
  </div>
);

// ============================================================================
// Error Boundary
// ============================================================================

interface ErrorBoundaryProps {
  children: React.ReactNode;
  packageId: string;
}

interface ErrorBoundaryState {
  hasError: boolean;
  error: Error | null;
}

class ErrorBoundary extends React.Component<ErrorBoundaryProps, ErrorBoundaryState> {
  constructor(props: ErrorBoundaryProps) {
    super(props);
    this.state = { hasError: false, error: null };
  }

  static getDerivedStateFromError(error: Error): ErrorBoundaryState {
    return { hasError: true, error };
  }

  componentDidCatch(error: Error, errorInfo: React.ErrorInfo) {
    logger.error("Native app runtime error", error, {
      component: "NativeRenderer",
      packageId: this.props.packageId,
      componentStack: errorInfo.componentStack,
    });
  }

  render() {
    if (this.state.hasError && this.state.error) {
      return <ErrorState error={this.state.error} packageId={this.props.packageId} />;
    }

    return this.props.children;
  }
}
