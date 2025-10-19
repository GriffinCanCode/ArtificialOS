/**
 * Dynamic Renderer Component
 * Renders AI-generated applications on the fly
 */

import React, { useEffect } from "react";
import { useWebSocket } from "../../../ui/contexts/WebSocketContext";
import { useBlueprint, useAppActions, useAppStore } from "../../../core/store/appStore";
import { useActions, useStore as useWindowStore } from "../../windows";
import { logger } from "../../../core/monitoring/core/logger";

// CRITICAL: Import registration to ensure components are registered
// This triggers the side-effect that registers all component renderers
import "../rendering/register";

// Import split modules
import { PARSE_DEBOUNCE_MS, validateJSONSize, validateJSONDepth } from "./constants";
import { ComponentState } from "../state/state";
import { ToolExecutor } from "../execution/executor";
import { parsePartialBlueprint, filterValidComponents } from "../../../core/utils/blueprintParser";

// ============================================================================
// Main DynamicRenderer Component
// ============================================================================

const DynamicRenderer: React.FC = () => {
  const { client } = useWebSocket();

  // Read current blueprint for lifecycle hooks and UI updates
  const blueprint = useBlueprint();

  // Actions for updating app store
  const {
    setBlueprint,
    setPartialBlueprint,
    setLoading,
    setBuildProgress,
    setError,
    addGenerationThought,
    appendGenerationPreview,
    clearGenerationPreview,
  } = useAppActions();

  const { open: openWindow, update: updateWindow, close: closeWindow } = useActions();

  // Use useMemo to ensure stable references across renders
  const componentState = React.useMemo(() => {
    logger.info("Initializing component state", { component: "DynamicRenderer" });
    const state = new ComponentState();

    // Configure state management
    state.setMaxHistorySize(100); // Keep history for undo/redo

    // Setup global computed values
    state.defineComputed("formValid", ["error.*"], (stateMap) => {
      for (const [key, value] of stateMap.entries()) {
        if (key.startsWith("error.") && value) {
          return false;
        }
      }
      return true;
    });

    // Setup global wildcard listeners for debugging
    if (process.env.NODE_ENV === "development") {
      state.subscribeWildcard("error.*", (event) => {
        logger.error("Error state changed", undefined, {
          component: "ComponentState",
          key: event.key,
          error: event.newValue,
        });
      });
    }

    return state;
  }, []); // Only create once

  const toolExecutor = React.useMemo(() => {
    logger.info("Initializing tool executor", { component: "DynamicRenderer" });
    return new ToolExecutor(componentState);
  }, [componentState]); // Only recreate if componentState changes (which it won't)

  // Track component count for build progress
  const [lastComponentCount, setLastComponentCount] = React.useState(0);
  const renderedComponentIdsRef = React.useRef<Set<string>>(new Set());
  const previousComponentsRef = React.useRef<any[]>([]);

  // Throttle partial JSON parsing
  const parseThrottleTimerRef = React.useRef<number | null>(null);
  const lastParseTimeRef = React.useRef<number>(0);

  // Listen for WebSocket messages
  useEffect(() => {
    if (!client) return;

    logger.debug("Setting up WebSocket message listener", { component: "DynamicRenderer" });

    const unsubscribe = client.onMessage((message) => {
      logger.debugThrottled("Received WebSocket message", {
        component: "DynamicRenderer",
        messageType: message.type,
      });

      switch (message.type) {
        case "generation_start":
          logger.info("UI generation started", {
            component: "DynamicRenderer",
            message: message.message,
          });
          addGenerationThought(message.message);
          setBuildProgress(0);
          setPartialBlueprint({ components: [] });
          clearGenerationPreview(); // Clear accumulated preview from previous generation
          renderedComponentIdsRef.current = new Set(); // Reset for new generation
          setLastComponentCount(0);
          previousComponentsRef.current = []; // Reset previous components for animation tracking
          logger.debug("Generation state initialized (window mode)", {
            component: "DynamicRenderer",
          });
          break;

        case "thought":
          logger.verboseThrottled("Generation thought received", {
            component: "DynamicRenderer",
            content: message.content,
          });
          addGenerationThought(message.content);
          break;

        case "generation_token":
          // Real-time token streaming during UI generation
          console.log("[STREAM] Token received:", message.content?.length || 0, "chars");
          logger.verboseThrottled("Generation token received", {
            component: "DynamicRenderer",
            contentLength: message.content?.length || 0,
          });
          // Accumulate tokens for real-time display
          if (message.content) {
            appendGenerationPreview(message.content);

            // Throttle parsing: parse immediately if enough time has passed, otherwise schedule
            const now = Date.now();
            const timeSinceLastParse = now - lastParseTimeRef.current;

            const doParse = () => {
              console.log("[STREAM] Parsing triggered");
              lastParseTimeRef.current = Date.now();

              // Parse partial Blueprint JSON and update partial UI spec
              const currentPreview = useAppStore.getState().generationPreview;
              const parseResult = parsePartialBlueprint(currentPreview);

              if (parseResult.data && parseResult.data.components.length > 0) {
                const currentCount = parseResult.data.components.length;
                const prevCount = lastComponentCount;

                if (currentCount > prevCount) {
                  console.log(
                    "[STREAM] New components:",
                    currentCount - prevCount,
                    "Total:",
                    currentCount
                  );
                  logger.info("New components parsed from Blueprint", {
                    component: "DynamicRenderer",
                    newComponents: currentCount - prevCount,
                    totalComponents: currentCount,
                  });
                  setLastComponentCount(currentCount);
                }

                // Always update - streaming should be real-time
                console.log("[STREAM] Updating UI with", currentCount, "components");
                setPartialBlueprint(parseResult.data);

                // Update builder window in real-time
                const builderId = (window as any).__builderWindowId;
                if (builderId) {
                  const windowStore = useWindowStore.getState();
                  const builderWindow = windowStore.windows.find((w) => w.appId === builderId);
                  if (builderWindow) {
                    // Update window with latest partial blueprint using proper Zustand update
                    updateWindow(builderWindow.id, {
                      title: `🔨 Building ${parseResult.data.title || "App"}... (${currentCount})`,
                      uiSpec: {
                        type: "app",
                        title: parseResult.data.title ?? "Building...",
                        layout: parseResult.data.layout ?? "vertical",
                        components: parseResult.data.components,
                        style: {},
                        services: [],
                        service_bindings: {},
                        lifecycle_hooks: {},
                      },
                    });
                  }
                }

                // Track component IDs
                parseResult.data.components.forEach((comp) => {
                  renderedComponentIdsRef.current.add(comp.id);
                });

                // Calculate progress
                const estimatedTotal = Math.max(10, currentCount + 5);
                const progress = Math.min(90, (currentCount / estimatedTotal) * 100);
                setBuildProgress(progress);
              }
            };

            // Throttle: parse immediately if enough time passed, otherwise schedule for later
            if (timeSinceLastParse >= PARSE_DEBOUNCE_MS) {
              // Enough time has passed, parse immediately
              doParse();
              // Clear any pending timer
              if (parseThrottleTimerRef.current !== null) {
                window.clearTimeout(parseThrottleTimerRef.current);
                parseThrottleTimerRef.current = null;
              }
            } else {
              // Too soon, schedule for later (but don't keep rescheduling)
              if (parseThrottleTimerRef.current === null) {
                const remainingTime = PARSE_DEBOUNCE_MS - timeSinceLastParse;
                parseThrottleTimerRef.current = window.setTimeout(() => {
                  doParse();
                  parseThrottleTimerRef.current = null;
                }, remainingTime);
              }
            }
          }
          break;

        case "ui_generated":
          try {
            // Reset generation flag to allow new generations
            delete (window as any).__isGenerating;

            // Validate UI spec size and depth
            const blueprintStr = JSON.stringify(message.ui_spec);
            validateJSONSize(blueprintStr);
            validateJSONDepth(message.ui_spec);

            // Filter out invalid components (e.g., nested "props" objects)
            const filteredComponents = filterValidComponents(message.ui_spec.components);
            const filteredSpec = {
              ...message.ui_spec,
              components: filteredComponents,
            };

            logger.info("UI generated successfully", {
              component: "DynamicRenderer",
              title: filteredSpec.title,
              appId: message.app_id,
              componentCount: filteredSpec.components.length,
            });
            setBlueprint(filteredSpec, message.app_id);

            // Transform builder window into final app (same window, just update it)
            const builderId = (window as any).__builderWindowId;
            if (builderId) {
              const windowStore = useWindowStore.getState();
              const builderWindow = windowStore.windows.find((w) => w.appId === builderId);
              if (builderWindow) {
                // Update the SAME window with final app data using proper Zustand update
                updateWindow(builderWindow.id, {
                  appId: message.app_id,
                  title: filteredSpec.title,
                  icon: "✨",
                  uiSpec: filteredSpec,
                });
                // Clear builder reference
                delete (window as any).__builderWindowId;

                logger.info("Transformed builder window to final app", {
                  component: "DynamicRenderer",
                  windowId: builderWindow.id,
                  appId: message.app_id,
                  title: filteredSpec.title,
                });
              } else {
                // Builder window not found, fallback to opening new window
                logger.warn("Builder window not found, opening new window", {
                  component: "DynamicRenderer",
                  builderId,
                  appId: message.app_id,
                });
                delete (window as any).__builderWindowId;
                openWindow(message.app_id, filteredSpec.title, filteredSpec, "✨");
              }
            } else {
              // No builder ID stored, open new window
              logger.warn("No builder window ID found, opening new window", {
                component: "DynamicRenderer",
                appId: message.app_id,
              });
              openWindow(message.app_id, filteredSpec.title, filteredSpec, "✨");
            }

            // Use batch update for initialization
            componentState.batch(() => {
              componentState.clear();
              componentState.set("app_id", message.app_id);
              componentState.set("app_title", filteredSpec.title);
              componentState.set("app_initialized", Date.now());

              // Setup computed values specific to this app
              if (filteredSpec.components) {
                componentState.defineComputed("componentCount", ["app_initialized"], () => {
                  return filteredSpec.components.length;
                });
              }
            });

            toolExecutor.setAppId(message.app_id);

            // Execute lifecycle hooks (on_mount)
            if (filteredSpec.lifecycle_hooks?.on_mount) {
              logger.info("Executing on_mount lifecycle hooks", {
                component: "DynamicRenderer",
                hookCount: filteredSpec.lifecycle_hooks.on_mount.length,
              });
              filteredSpec.lifecycle_hooks.on_mount.forEach(async (toolId: string) => {
                try {
                  await toolExecutor.execute(toolId, {});
                } catch (error) {
                  logger.error(`Failed to execute on_mount hook: ${toolId}`, error as Error, {
                    component: "DynamicRenderer",
                    toolId,
                  });
                }
              });
            }
          } catch (validationError) {
            logger.error("UI spec validation failed", validationError as Error, {
              component: "DynamicRenderer",
            });
            setError(`UI spec validation failed: ${(validationError as Error).message}`);
            setLoading(false);
            // setStreaming(false); // Already false in window mode
            break;
          }
          break;

        case "complete":
          logger.info("UI generation complete", { component: "DynamicRenderer" });
          // Reset generation flag to allow new generations
          delete (window as any).__isGenerating;
          setLoading(false);
          setBuildProgress(100);
          // Clear any pending parse operations
          if (parseThrottleTimerRef.current !== null) {
            window.clearTimeout(parseThrottleTimerRef.current);
            parseThrottleTimerRef.current = null;
          }
          break;

        case "error":
          logger.error("UI generation error", undefined, {
            component: "DynamicRenderer",
            message: message.message,
            previewLength: useAppStore.getState().generationPreview.length,
          });
          // Reset generation flag to allow new generations
          delete (window as any).__isGenerating;
          setError(message.message);

          // Close builder window on error
          const builderId = (window as any).__builderWindowId;
          if (builderId) {
            const windowStore = useWindowStore.getState();
            const builderWindow = windowStore.windows.find((w) => w.appId === builderId);
            if (builderWindow) {
              closeWindow(builderWindow.id);
              delete (window as any).__builderWindowId;
            }
          }

          // Clear any pending parse operations
          if (parseThrottleTimerRef.current !== null) {
            window.clearTimeout(parseThrottleTimerRef.current);
            parseThrottleTimerRef.current = null;
          }
          break;

        default:
          // Ignore other message types
          break;
      }
    });

    return () => {
      logger.debug("Cleaning up WebSocket message listener", { component: "DynamicRenderer" });
      // Clear any pending parse operations
      if (parseThrottleTimerRef.current !== null) {
        window.clearTimeout(parseThrottleTimerRef.current);
        parseThrottleTimerRef.current = null;
      }
      unsubscribe();
    };
  }, [
    client,
    componentState,
    toolExecutor,
    setBlueprint,
    setLoading,
    setError,
    addGenerationThought,
    appendGenerationPreview,
    clearGenerationPreview,
    setBuildProgress,
    setPartialBlueprint,
    openWindow,
    updateWindow,
    closeWindow,
  ]);

  // Track which app has had on_mount executed to prevent re-execution
  const mountedAppIdRef = React.useRef<string | null>(null);
  const appId = useAppStore((state) => state.appId);

  // Execute on_mount hooks when app changes - but only once per app
  useEffect(() => {
    if (blueprint?.lifecycle_hooks?.on_mount && appId && appId !== mountedAppIdRef.current) {
      logger.info("Executing on_mount lifecycle hooks", {
        component: "DynamicRenderer",
        hookCount: blueprint.lifecycle_hooks.on_mount.length,
        appId: appId,
      });

      mountedAppIdRef.current = appId;

      blueprint.lifecycle_hooks.on_mount.forEach(async (toolId: string) => {
        try {
          await toolExecutor.execute(toolId, {});
        } catch (error) {
          logger.error(`Failed to execute on_mount hook: ${toolId}`, error as Error, {
            component: "DynamicRenderer",
            toolId,
          });
        }
      });
    }
  }, [appId, blueprint?.lifecycle_hooks, toolExecutor]); // Only depend on appId, not full blueprint

  // Listen for UI spec updates from tools (e.g., hub grid population)
  const gridUpdateProcessedRef = React.useRef<string | null>(null);

  useEffect(() => {
    const handleMessage = (event: MessageEvent) => {
      const currentAppId = useAppStore.getState().appId;

      // Handle hub grid updates (legacy)
      if (
        event.data.type === "update_hub_grids" &&
        blueprint &&
        gridUpdateProcessedRef.current !== currentAppId
      ) {
        gridUpdateProcessedRef.current = currentAppId;

        logger.info("Updating hub grids with app data", {
          component: "DynamicRenderer",
          gridCount: Object.keys(event.data.grids).length,
          appId: currentAppId,
        });

        // Shallow clone for better performance
        const updatedSpec = {
          ...blueprint,
          components: blueprint.components.slice(),
        };

        // Find and update each grid component recursively
        const updateComponentChildren = (components: any[], depth = 0): any[] => {
          if (depth > 5) return components; // Prevent deep recursion

          return components.map((component) => {
            if (event.data.grids[component.id]) {
              // Update this grid's children
              logger.debug("Updating grid", {
                component: "DynamicRenderer",
                gridId: component.id,
                childCount: event.data.grids[component.id].length,
              });
              return {
                ...component,
                children: event.data.grids[component.id],
              };
            }
            // Check children recursively
            if (component.children && depth < 5) {
              return {
                ...component,
                children: updateComponentChildren(component.children, depth + 1),
              };
            }
            return component;
          });
        };

        updatedSpec.components = updateComponentChildren(updatedSpec.components);

        // Use requestAnimationFrame to batch the update
        requestAnimationFrame(() => {
          setBlueprint(updatedSpec, currentAppId || "");
        });
      }

      // Handle generalized dynamic list/grid updates
      if (event.data.type === "update_dynamic_lists" && blueprint) {
        logger.info("Updating dynamic lists", {
          component: "DynamicRenderer",
          listCount: Object.keys(event.data.lists || {}).length,
          appId: currentAppId,
        });

        // Shallow clone for better performance
        const updatedSpec = {
          ...blueprint,
          components: blueprint.components.slice(),
        };

        // Find and update each list/grid component recursively
        const updateComponentChildren = (components: any[], depth = 0): any[] => {
          if (depth > 5) return components; // Prevent deep recursion

          return components.map((component) => {
            if (event.data.lists[component.id]) {
              // Update this list/grid's children
              logger.debug("Updating list/grid", {
                component: "DynamicRenderer",
                componentId: component.id,
                childCount: event.data.lists[component.id].length,
              });
              return {
                ...component,
                children: event.data.lists[component.id],
              };
            }
            // Check children recursively
            if (component.children && depth < 5) {
              return {
                ...component,
                children: updateComponentChildren(component.children, depth + 1),
              };
            }
            return component;
          });
        };

        updatedSpec.components = updateComponentChildren(updatedSpec.components);

        // Use requestAnimationFrame to batch the update
        requestAnimationFrame(() => {
          setBlueprint(updatedSpec, currentAppId || "");
        });
      }
    };

    window.addEventListener("message", handleMessage);
    return () => {
      window.removeEventListener("message", handleMessage);
    };
  }, [blueprint, setBlueprint]);

  // Cleanup: Execute on_unmount hooks when app changes or component unmounts
  useEffect(() => {
    return () => {
      // Clean up timers first
      toolExecutor.cleanup();

      // Reset mounted app ID when this app unmounts
      if (mountedAppIdRef.current === appId) {
        mountedAppIdRef.current = null;
      }

      // Then execute lifecycle hooks
      if (blueprint?.lifecycle_hooks?.on_unmount) {
        logger.info("Executing on_unmount hooks", {
          component: "DynamicRenderer",
          hookCount: blueprint.lifecycle_hooks.on_unmount.length,
        });
        blueprint.lifecycle_hooks.on_unmount.forEach(async (toolId: string) => {
          try {
            await toolExecutor.execute(toolId, {});
          } catch (error) {
            logger.error(`Error executing on_unmount hook: ${toolId}`, error as Error, {
              component: "DynamicRenderer",
              toolId,
            });
          }
        });
      }
    };
  }, [appId, toolExecutor]); // Only depend on appId, not blueprint

  // DynamicRenderer is now "headless" - no UI rendering, only WebSocket message handling
  // All UI rendering is handled by BuilderView (for builder windows) and WindowManager (for apps)

  // DynamicRenderer is now a headless component - no UI rendering
  // All UI is handled by:
  // - BuilderView (for builder windows showing build progress)
  // - WindowManager (for displaying completed apps in windows)
  // - Desktop (for the main desktop interface)

  return null;
};

export default DynamicRenderer;
