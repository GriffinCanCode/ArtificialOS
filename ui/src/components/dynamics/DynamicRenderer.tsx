/**
 * Dynamic Renderer Component
 * Renders AI-generated applications on the fly
 */

import React, { useState, useCallback, useEffect, useRef } from "react";
import { Construction, MessageCircle, Palette, Save, Sparkles, Zap, ExternalLink } from "lucide-react";
import { useWebSocket } from "../../contexts/WebSocketContext";
import { animated } from "@react-spring/web";
import {
  useSpringFadeIn,
  useMagneticHover,
  useJelloWobble,
  useBounceAttention,
} from "../../hooks/useSpring";
import {
  useBlueprint,
  usePartialBlueprint,
  useIsLoading,
  useIsStreaming,
  useBuildProgress,
  useError,
  useGenerationThoughts,
  useGenerationPreview,
  useAppActions,
  useAppStore,
  BlueprintComponent,
  Blueprint,
} from "../../store/appStore";
import { useWindowActions } from "../../store/windowStore";
import { logger } from "../../utils/monitoring/logger";
import { useSaveApp } from "../../hooks/useRegistryQueries";
import * as gsapAnimations from "../../utils/animation/gsapAnimations";
import {
  useFloat,
  useSpin,
  useBuildPulse,
  useImperativeAnimation,
  useStaggerSlideUp,
  useBlurReveal,
  useGlowPulse,
  useParticleBurst,
} from "../../hooks/useGSAP";
import { SaveAppDialog } from "../dialogs/SaveAppDialog";
import "./DynamicRenderer.css";

// Import split modules
import {
  VIRTUAL_SCROLL_THRESHOLD,
  DEFAULT_ITEM_HEIGHT,
  PARSE_DEBOUNCE_MS,
  validateJSONSize,
  validateJSONDepth,
} from "./DynamicRenderer.constants";
import { ComponentState } from "./DynamicRenderer.state";
import { ToolExecutor } from "./DynamicRenderer.executor";
import { ComponentRenderer } from "./DynamicRenderer.renderer";
import { VirtualizedList } from "./DynamicRenderer.virtual";
import { parsePartialBlueprint } from "../../utils/blueprintParser";

// ============================================================================
// Main DynamicRenderer Component
// ============================================================================

const DynamicRenderer: React.FC = () => {
  const { client } = useWebSocket();

  // Zustand store hooks - only re-render when these specific values change
  const blueprint = useBlueprint();
  const partialBlueprint = usePartialBlueprint();
  const isLoading = useIsLoading();
  const isStreaming = useIsStreaming();
  const buildProgress = useBuildProgress();
  const error = useError();
  const generationThoughts = useGenerationThoughts();
  const generationPreview = useGenerationPreview();
  const {
    setBlueprint,
    setPartialBlueprint,
    setLoading,
    setStreaming,
    setBuildProgress,
    setError,
    addGenerationThought,
    appendGenerationPreview,
  } = useAppActions();

  const { openWindow } = useWindowActions();

  const [showSaveAppDialog, setShowSaveAppDialog] = useState(false);

  // Use TanStack Query for save operation
  const saveAppMutation = useSaveApp();

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

  // Track which component IDs we've already rendered to avoid re-animating
  // Use ref so it updates immediately without waiting for React re-render
  const renderedComponentIdsRef = React.useRef<Set<string>>(new Set());
  const [lastComponentCount, setLastComponentCount] = useState(0);

  // Track last animated component count to avoid re-animating existing components
  const lastAnimatedCountRef = useRef(0);

  // Track which components have been animated by ID (for incremental build animations)
  const animatedComponentIds = useRef<Set<string>>(new Set());

  // Track if build container has been animated (only once per generation)
  const buildContainerAnimatedRef = useRef(false);

  // Throttle partial JSON parsing
  const parseThrottleTimerRef = React.useRef<number | null>(null);
  const lastParseTimeRef = React.useRef<number>(0);

  // Auto-scroll preview as content is added
  const previewRef = React.useRef<HTMLPreElement>(null);
  React.useEffect(() => {
    logger.debugThrottled("Preview updated", {
      component: "DynamicRenderer",
      length: generationPreview.length,
    });
    if (previewRef.current && generationPreview) {
      previewRef.current.scrollTop = previewRef.current.scrollHeight;
    }
  }, [generationPreview]);

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
          setStreaming(true);
          setBuildProgress(0);
          setPartialBlueprint({ components: [] });
          renderedComponentIdsRef.current = new Set(); // Reset for new generation
          setLastComponentCount(0);
          lastAnimatedCountRef.current = 0; // Reset animation counter for new generation
          animatedComponentIds.current.clear(); // Reset animated component tracking
          previousComponentsRef.current = []; // Reset previous components for animation tracking
          buildContainerAnimatedRef.current = false; // Reset build container animation
          logger.debug("Generation state initialized", {
            component: "DynamicRenderer",
            isStreaming: true,
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
                  console.log("[STREAM] New components:", currentCount - prevCount, "Total:", currentCount);
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
            // Validate UI spec size and depth
            const blueprintStr = JSON.stringify(message.ui_spec);
            validateJSONSize(blueprintStr);
            validateJSONDepth(message.ui_spec);

            logger.info("UI generated successfully", {
              component: "DynamicRenderer",
              title: message.ui_spec.title,
              appId: message.app_id,
              componentCount: message.ui_spec.components.length,
            });
            setBlueprint(message.ui_spec, message.app_id);

            // Use batch update for initialization
            componentState.batch(() => {
              componentState.clear();
              componentState.set("app_id", message.app_id);
              componentState.set("app_title", message.ui_spec.title);
              componentState.set("app_initialized", Date.now());

              // Setup computed values specific to this app
              if (message.ui_spec.components) {
                componentState.defineComputed("componentCount", ["app_initialized"], () => {
                  return message.ui_spec.components.length;
                });
              }
            });

            toolExecutor.setAppId(message.app_id);
          } catch (validationError) {
            logger.error("UI spec validation failed", validationError as Error, {
              component: "DynamicRenderer",
            });
            setError(`UI spec validation failed: ${(validationError as Error).message}`);
            setLoading(false);
            setStreaming(false);
            break;
          }

          // Execute lifecycle hooks (on_mount)
          if (message.ui_spec.lifecycle_hooks?.on_mount) {
            logger.info("Executing on_mount lifecycle hooks", {
              component: "DynamicRenderer",
              hookCount: message.ui_spec.lifecycle_hooks.on_mount.length,
            });
            message.ui_spec.lifecycle_hooks.on_mount.forEach(async (toolId: string) => {
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
          break;

        case "complete":
          logger.info("UI generation complete", { component: "DynamicRenderer" });
          setLoading(false);
          setStreaming(false);
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
          setError(message.message);
          setStreaming(false);
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
    setStreaming,
    setBuildProgress,
    setPartialBlueprint,
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
      if (event.data.type === "update_hub_grids" && blueprint && gridUpdateProcessedRef.current !== currentAppId) {
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
            if (component.children && depth < 3) {
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

  // GSAP Animation refs and hooks
  const desktopIconRef = useFloat<HTMLDivElement>(true, { distance: 15, duration: 4.5 });
  const desktopMessageRef = useBlurReveal<HTMLDivElement>({ duration: 0.9 });
  const errorRef = useImperativeAnimation<HTMLDivElement>();
  const buildContainerRef = useRef<HTMLDivElement>(null);
  const buildProgressRef = useRef<HTMLDivElement>(null);
  const spinnerRef = useSpin<HTMLDivElement>(isLoading || isStreaming, { duration: 1.0 });
  const buildSpinnerRef = useSpin<HTMLDivElement>(true, { duration: 1.0 });
  const buildIconRef = useBuildPulse<HTMLDivElement>(isStreaming);
  const thoughtsListRef = useStaggerSlideUp<HTMLDivElement>(".thought-item", {
    stagger: 0.08,
    distance: 25,
  });
  const saveButtonRef = useGlowPulse<HTMLButtonElement>(false, {
    color: "99, 102, 241",
    intensity: 15,
  });
  const { elementRef: burstRef, burst } = useParticleBurst<HTMLDivElement>();

  // React Spring animations
  const errorJello = useJelloWobble(!!error);
  const iconBounce = useBounceAttention(!blueprint && !isLoading && !isStreaming);
  const saveButtonMagnetic = useMagneticHover(0.15);
  const desktopFadeIn = useSpringFadeIn(!blueprint && !isLoading && !isStreaming);

  // Animate error with chromatic aberration and spring wobble
  useEffect(() => {
    if (error && errorRef.elementRef.current) {
      gsapAnimations.chromaticAberration(errorRef.elementRef.current, {
        intensity: 6,
        duration: 0.5,
      });
      setTimeout(() => {
        if (errorRef.elementRef.current) {
          errorRef.shake();
          gsapAnimations.wobble(errorRef.elementRef.current);
        }
      }, 300);
    }
  }, [error, errorRef]);

  // Animate build container on appearance - only once per generation
  useEffect(() => {
    if (
      buildContainerRef.current &&
      (isStreaming || partialBlueprint) &&
      !buildContainerAnimatedRef.current
    ) {
      gsapAnimations.buildContainerAppear(buildContainerRef.current);
      buildContainerAnimatedRef.current = true;
    }
  }, [isStreaming, partialBlueprint]);

  // Animate rendered app on appearance with creative effects
  useEffect(() => {
    if (burstRef.current && blueprint) {
      // Use ink blot reveal for a more organic appearance
      gsapAnimations.inkBlotReveal(burstRef.current, { duration: 1.2, from: "center" });

      // Add particle burst after a short delay
      setTimeout(() => {
        if (burstRef.current) {
          burst({ count: 18, duration: 1.8 });
        }
      }, 500);

      // Add subtle energy pulse to the app title
      const appTitle = burstRef.current.querySelector(".app-header h2");
      if (appTitle) {
        gsapAnimations.energyPulse(appTitle, { color: "139, 92, 246", count: 2 });
      }
    }
  }, [blueprint, burst, burstRef]);

  // Animate progress bar width changes
  useEffect(() => {
    if (buildProgressRef.current) {
      gsapAnimations.setProps(buildProgressRef.current, { width: `${buildProgress || 20}%` });
    }
  }, [buildProgress]);

  // Handle save app form submission
  const handleSaveApp = useCallback(
    async (data: { description: string; category: string; icon: string; tags: string[] }) => {
      const appId = useAppStore.getState().appId;
      if (!appId) {
        throw new Error("No app ID found");
      }

      await saveAppMutation.mutateAsync({
        app_id: appId,
        description: data.description,
        category: data.category,
        icon: data.icon,
        tags: data.tags,
      });
    },
    [saveAppMutation]
  );

  const handleSaveDialogOpen = useCallback(() => {
    setShowSaveAppDialog(true);
  }, []);

  const handleSaveDialogClose = useCallback(() => {
    setShowSaveAppDialog(false);
  }, []);

  const handleOpenInWindow = useCallback(() => {
    if (blueprint) {
      const appId = useAppStore.getState().appId;
      if (appId) {
        openWindow(appId, blueprint.title, blueprint, "✨");
      }
    }
  }, [blueprint, openWindow]);

  // Memoize filtered components to prevent re-filtering on every render
  const validPartialComponents = React.useMemo(() => {
    return partialBlueprint?.components?.filter((c) => c && c.type && c.type !== "undefined") || [];
  }, [partialBlueprint?.components]);

  const validBlueprintComponents = React.useMemo(() => {
    return blueprint?.components?.filter((c) => c && c.type && c.type !== "undefined") || [];
  }, [blueprint?.components]);

  // Animate components as they appear during build - with varied creative effects
  const componentRefs = useRef<Map<string, HTMLDivElement>>(new Map());

  // Store previous components to detect new ones without triggering effect on every change
  const previousComponentsRef = useRef<BlueprintComponent[]>([]);

  // Function to animate a single component
  const animateComponent = useCallback((component: BlueprintComponent, idx: number) => {
    const componentId = component.id;

    // Skip if we've already animated this component
    if (animatedComponentIds.current.has(componentId)) {
      return;
    }

    const key = `${component.id}-${idx}`;
    const element = componentRefs.current.get(key);

    if (element && !element.dataset.animated) {
      // Ensure element is visible first (in case GSAP fails)
      element.style.opacity = "1";

      // TEMPORARILY DISABLED: Animations during streaming to prevent flicker
      // TODO: Re-enable with better synchronization
      /*
      // Vary animations for visual interest
      const animationType = idx % 3;
      const delay = 0.05; // Small delay for smoothness

      try {
        switch (animationType) {
          case 0:
            gsapAnimations.vortexSpiral(element, { duration: 0.8, clockwise: idx % 2 === 0 });
            break;
          case 1:
            gsapAnimations.liquidMorph(element, { duration: 0.9, intensity: 12 });
            break;
          case 2:
            gsapAnimations.elasticBounceIn(element, { delay, from: "bottom" });
            break;
        }
      } catch (error) {
        // If GSAP animation fails, component is still visible
        logger.debug("GSAP animation failed, component still visible", {
          component: "DynamicRenderer",
          componentId,
          error: error instanceof Error ? error.message : String(error),
        });
      }
      */

      element.dataset.animated = "true";
      animatedComponentIds.current.add(componentId);
    }
  }, []);

  // Check for new components and animate them - runs after render but doesn't depend on component array
  useEffect(() => {
    const currentComponents = validPartialComponents;
    const previousComponents = previousComponentsRef.current;

    // Only process if we have new components
    if (currentComponents.length > previousComponents.length) {
      // Get just the new components
      const newComponents = currentComponents.slice(previousComponents.length);

      // Animate them with a small delay so DOM elements are ready
      requestAnimationFrame(() => {
        newComponents.forEach((component, relativeIdx) => {
          const actualIdx = previousComponents.length + relativeIdx;
          animateComponent(component, actualIdx);
        });
      });

      // Update the previous components ref
      previousComponentsRef.current = currentComponents;
    }
  }, [validPartialComponents.length, animateComponent]); // Only depend on length, not the full array

  // Clean desktop - no auto-loading

  // Debug: Log render state
  logger.verboseThrottled("Render state", {
    component: "DynamicRenderer",
    isLoading,
    isStreaming,
    hasPartialComponents: validPartialComponents.length > 0,
    partialCount: validPartialComponents.length,
    hasBlueprint: !!blueprint,
    hasError: !!error,
  });

  return (
    <div className="dynamic-renderer fullscreen">
      <SaveAppDialog
        isOpen={showSaveAppDialog}
        onClose={handleSaveDialogClose}
        onSave={handleSaveApp}
        isLoading={saveAppMutation.isPending}
      />

      <div className="renderer-canvas">
        {error && (
          <animated.div ref={errorRef.elementRef} className="renderer-error" style={errorJello}>
            <strong>Error:</strong> {error}
          </animated.div>
        )}

        {(isLoading || isStreaming || (partialBlueprint && !blueprint)) && (
          <>
            {/* Visual Build Mode - Show partial UI being constructed */}
            {isStreaming || partialBlueprint ? (
              <div ref={buildContainerRef} className="building-app-preview">
                <div className="build-progress-header">
                  <div ref={buildIconRef} className="build-status-icon">
                    <Construction size={24} />
                  </div>
                  <div className="build-status-text">
                    <h3>Building Your App...</h3>
                    <p>{validPartialComponents.length} components assembled</p>
                  </div>
                </div>

                <div className="build-progress-bar">
                  <div
                    ref={buildProgressRef}
                    className="build-progress-fill"
                    style={{ width: `${buildProgress || 20}%` }}
                  />
                </div>

                <div className="building-app-container">
                  {partialBlueprint?.title && (
                    <h2 className="building-app-title">{partialBlueprint.title}</h2>
                  )}
                  <div className={`app-content app-layout-${partialBlueprint?.layout || "vertical"}`}>
                    {validPartialComponents.length >= VIRTUAL_SCROLL_THRESHOLD ? (
                      // Use virtual scrolling for large component lists during build
                      <VirtualizedList
                        children={validPartialComponents}
                        state={componentState}
                        executor={toolExecutor}
                        itemHeight={DEFAULT_ITEM_HEIGHT}
                        maxHeight={600}
                        layout={
                          (partialBlueprint?.layout || "vertical") as
                            | "vertical"
                            | "horizontal"
                            | "grid"
                        }
                        className="building-components-list"
                        ComponentRenderer={ComponentRenderer}
                      />
                    ) : (
                      // Normal rendering for small lists
                      <>
                        {validPartialComponents.map((component, idx) => {
                          // Debug: Log what we're about to render
                          if (process.env.NODE_ENV === "development") {
                            logger.debugThrottled(`Rendering component ${component.id}`, {
                              component: "DynamicRenderer",
                              type: component.type,
                              hasChildren: !!component.children,
                              childrenCount: component.children?.length || 0,
                            });
                          }
                          return (
                            <div
                              key={`building-${component.id}`}
                              ref={(el) => {
                                if (el) componentRefs.current.set(`${component.id}-${idx}`, el);
                              }}
                              className="component-building"
                            >
                              <ComponentRenderer
                                key={component.id}
                                component={component}
                                state={componentState}
                                executor={toolExecutor}
                              />
                            </div>
                          );
                        })}
                        {validPartialComponents.length === 0 && (
                          <div
                            className="component-assembling"
                            style={{ padding: "2rem", textAlign: "center" }}
                          >
                            <div
                              ref={buildSpinnerRef}
                              className="spinner"
                              style={{ margin: "0 auto 1rem" }}
                            ></div>
                            <p style={{ opacity: 0.7 }}>Assembling components...</p>
                          </div>
                        )}
                      </>
                    )}
                  </div>
                </div>

                {/* Thoughts stream below */}
                {generationThoughts.length > 0 && (
                  <div
                    ref={thoughtsListRef}
                    className="thoughts-list"
                    style={{ marginTop: "1rem", opacity: 0.7 }}
                  >
                    {generationThoughts.slice(-3).map((thought, i) => (
                      <div key={i} className="thought-item">
                        <span className="thought-icon">
                          <MessageCircle size={16} />
                        </span>
                        <span className="thought-text">{thought}</span>
                      </div>
                    ))}
                  </div>
                )}
              </div>
            ) : (
              /* Fallback to traditional loading screen if no partial data yet */
              <div className="generation-progress">
                <div className="generation-header">
                  <div ref={spinnerRef} className="spinner"></div>
                  <h3>
                    <Palette
                      size={20}
                      style={{
                        display: "inline-block",
                        marginRight: "8px",
                        verticalAlign: "middle",
                      }}
                    />
                    Generating Application...
                  </h3>
                </div>
                <div ref={thoughtsListRef} className="thoughts-list">
                  {generationThoughts.map((thought, i) => (
                    <div key={i} className="thought-item">
                      <span className="thought-icon">
                        <MessageCircle size={16} />
                      </span>
                      <span className="thought-text">{thought}</span>
                    </div>
                  ))}
                </div>
                {generationPreview && (
                  <div className="generation-preview">
                    <div className="preview-header">
                      <span className="preview-icon">
                        <Zap size={16} />
                      </span>
                      <span className="preview-label">
                        Live Generation ({generationPreview.length} chars)
                      </span>
                    </div>
                    <pre className="preview-content" ref={previewRef}>
                      <code>{generationPreview}</code>
                    </pre>
                  </div>
                )}
              </div>
            )}
          </>
        )}

        {!blueprint && !isLoading && !isStreaming && !error && (
          <div className="desktop-empty-state">
            <animated.div ref={desktopMessageRef} className="desktop-message" style={desktopFadeIn}>
              <animated.div ref={desktopIconRef} className="desktop-icon" style={iconBounce}>
                <Sparkles size={48} />
              </animated.div>
              <h2>Welcome to Griffin's AgentOS</h2>
              <p>Press ⌘K or click below to create something</p>
            </animated.div>
          </div>
        )}

        {blueprint && (
          <div ref={burstRef} className="rendered-app" style={blueprint.style}>
            <div className="app-header">
              <h2>{blueprint.title}</h2>
              <div className="app-header-actions">
                <button
                  className="app-action-btn"
                  onClick={handleOpenInWindow}
                  title="Open in window"
                >
                  <ExternalLink size={16} style={{ marginRight: "6px", verticalAlign: "middle" }} />
                  Open in Window
                </button>
                <animated.button
                  ref={(el: HTMLButtonElement) => {
                    if (saveButtonRef.current !== el && el) {
                      (saveButtonRef as React.MutableRefObject<HTMLButtonElement | null>).current =
                        el;
                    }
                    (
                      saveButtonMagnetic.ref as React.MutableRefObject<HTMLButtonElement | null>
                    ).current = el;
                  }}
                  className="save-app-btn"
                  onClick={handleSaveDialogOpen}
                  {...saveButtonMagnetic.handlers}
                  style={saveButtonMagnetic.style}
                  title="Save this app to registry"
                >
                  <Save size={16} style={{ marginRight: "6px", verticalAlign: "middle" }} />
                  Save
                </animated.button>
              </div>
            </div>
            <div className={`app-content app-layout-${blueprint.layout}`}>
              {validBlueprintComponents.length >= VIRTUAL_SCROLL_THRESHOLD ? (
                // Use virtual scrolling for large apps
                <VirtualizedList
                  children={validBlueprintComponents}
                  state={componentState}
                  executor={toolExecutor}
                  itemHeight={DEFAULT_ITEM_HEIGHT}
                  maxHeight={700}
                  layout={(blueprint.layout || "vertical") as "vertical" | "horizontal" | "grid"}
                  className="app-components-list"
                  ComponentRenderer={ComponentRenderer}
                />
              ) : (
                // Normal rendering for small apps
                validBlueprintComponents.map((component, idx) => (
                  <ComponentRenderer
                    key={`${component.id}-${idx}`}
                    component={component}
                    state={componentState}
                    executor={toolExecutor}
                  />
                ))
              )}
            </div>
          </div>
        )}
      </div>
    </div>
  );
};

export default DynamicRenderer;
