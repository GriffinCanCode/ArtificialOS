/**
 * Dynamic Renderer Component
 * Renders AI-generated applications on the fly
 */

import React, { useState, useCallback, useEffect, useRef } from "react";
import { Construction, MessageCircle, Palette, Save, Sparkles, Zap } from "lucide-react";
import { useWebSocket } from "../../contexts/WebSocketContext";
import { animated } from "@react-spring/web";
import {
  useSpringFadeIn,
  useMagneticHover,
  useJelloWobble,
  useBounceAttention,
} from "../../hooks/useSpring";
import {
  useUISpec,
  usePartialUISpec,
  useIsLoading,
  useIsStreaming,
  useBuildProgress,
  useError,
  useGenerationThoughts,
  useGenerationPreview,
  useAppActions,
  useAppStore,
  UIComponent,
  UISpec,
} from "../../store/appStore";
import { logger } from "../../utils/monitoring/logger";
import { startPerf, endPerf } from "../../utils/monitoring/performanceMonitor";
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

// ============================================================================
// Main DynamicRenderer Component
// ============================================================================

const DynamicRenderer: React.FC = () => {
  const { client } = useWebSocket();

  // Zustand store hooks - only re-render when these specific values change
  const uiSpec = useUISpec();
  const partialUISpec = usePartialUISpec();
  const isLoading = useIsLoading();
  const isStreaming = useIsStreaming();
  const buildProgress = useBuildProgress();
  const error = useError();
  const generationThoughts = useGenerationThoughts();
  const generationPreview = useGenerationPreview();
  const {
    setUISpec,
    setPartialUISpec,
    setLoading,
    setStreaming,
    setBuildProgress,
    setError,
    addGenerationThought,
    appendGenerationPreview,
  } = useAppActions();

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

  // Debounce partial JSON parsing
  const parseDebounceTimerRef = React.useRef<number | null>(null);
  
  // Track last state update time to prevent too-frequent flickering updates
  const lastStateUpdateRef = React.useRef<number>(0);
  const lastChildrenCountRef = React.useRef<Map<string, number>>(new Map());

  // Cache last parse result to avoid redundant parsing during streaming
  const lastParseResultRef = React.useRef<{
    preview: string;
    result: any;
  } | null>(null);

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

  // Helper to extract partial UI spec from streaming JSON
  // Optimized with result caching to avoid redundant parsing
  // Helper function to extract components with deep nesting support
  const extractComponentsRecursively = useCallback((componentsStr: string): UIComponent[] => {
    const components: UIComponent[] = [];
    let depth = 0;
    let currentObj = "";
    let inString = false;
    let escapeNext = false;

    for (let i = 0; i < componentsStr.length; i++) {
      const char = componentsStr[i];
      const prevChar = i > 0 ? componentsStr[i - 1] : "";

      // Handle string escaping
      if (escapeNext) {
        escapeNext = false;
        currentObj += char;
        continue;
      }

      if (char === "\\") {
        escapeNext = true;
        currentObj += char;
        continue;
      }

      // Track if we're inside a string
      if (char === '"' && prevChar !== "\\") {
        inString = !inString;
        currentObj += char;
        continue;
      }

      // Only count braces outside of strings
      if (!inString) {
        if (char === "{") {
          depth++;
          currentObj += char;
        } else if (char === "}") {
          currentObj += char;
          depth--;

          // When we complete a top-level object, try to parse it
          if (depth === 0 && currentObj.trim()) {
            try {
              const parsed = JSON.parse(currentObj.trim());
              if (parsed && typeof parsed === "object" && parsed.type) {
                components.push(parsed as UIComponent);
              }
            } catch (e) {
              // Skip invalid JSON fragments
            }
            currentObj = "";
          }
        } else {
          currentObj += char;
        }
      } else {
        currentObj += char;
      }
    }

    // AGGRESSIVE PARSING: Try to extract partial component even if not fully closed
    // This allows rendering containers with incomplete children arrays in real-time
    if (currentObj.trim() && depth > 0) {
      logger.info("Attempting aggressive parse", {
        component: "DynamicRenderer",
        depth,
        currentObjLength: currentObj.length,
        hasChildren: currentObj.includes('"children"'),
      });

      // Try to close the incomplete JSON and parse it
      // Remove leading comma (from array separator) and whitespace
      let attemptedClose = currentObj.trim().replace(/^,\s*/, "");

      // If we have an unclosed children array, close it
      if (attemptedClose.includes('"children"')) {
        // Count unclosed brackets
        const openBrackets = (attemptedClose.match(/\[/g) || []).length;
        const closeBrackets = (attemptedClose.match(/\]/g) || []).length;
        const missingBrackets = openBrackets - closeBrackets;

        // Close children array if needed
        for (let i = 0; i < missingBrackets; i++) {
          attemptedClose += "]";
        }
      }

      // Close remaining braces
      for (let i = 0; i < depth; i++) {
        attemptedClose += "}";
      }

      try {
        const parsed = JSON.parse(attemptedClose);
        if (parsed && typeof parsed === "object" && parsed.type) {
          components.push(parsed as UIComponent);
          logger.info("Extracted partial component with incomplete structure", {
            component: "DynamicRenderer",
            type: parsed.type,
            hasChildren: !!parsed.children,
            childrenCount: parsed.children?.length || 0,
            depth,
          });
        }
      } catch (e) {
        // Failed to parse even with closing - that's okay, we'll try again next time
        logger.info("Could not extract partial component", {
          component: "DynamicRenderer",
          depth,
          error: e instanceof Error ? e.message : String(e),
          firstChars: attemptedClose.slice(0, 100),
          lastChars: attemptedClose.slice(-50),
        });
      }
    }

    logger.info("Extraction complete", {
      component: "DynamicRenderer",
      totalComponents: components.length,
      componentDetails: components.map((c) => ({
        type: c.type,
        id: c.id,
        childrenCount: c.children?.length || 0,
      })),
    });

    return components;
  }, []);

  const parsePartialJSON = useCallback(
    (jsonStr: string) => {
      // Performance monitoring - use consistent metadata-free key
      startPerf("json_parse");

      // During streaming, the string grows, so we can't use a simple hash
      // Instead, check if the new string starts with the old one (streaming append)
      const hasCache = !!lastParseResultRef.current?.preview;
      const startsWithPrev = hasCache && lastParseResultRef.current ? jsonStr.startsWith(lastParseResultRef.current.preview) : false;
      const isLonger = hasCache && lastParseResultRef.current ? jsonStr.length > lastParseResultRef.current.preview.length : false;
      const isStreamingAppend = hasCache && startsWithPrev && isLonger;

      // Debug streaming detection
      if (hasCache && !isStreamingAppend && lastParseResultRef.current) {
        logger.debugThrottled("Streaming detection failed", {
          component: "DynamicRenderer",
          hasCache,
          startsWithPrev,
          isLonger,
          prevLen: lastParseResultRef.current.preview.length,
          newLen: jsonStr.length,
          preview: jsonStr.slice(0, 100),
        });
      }

      // If exact same string, return cached result
      if (lastParseResultRef.current?.preview === jsonStr) {
        endPerf("json_parse");
        return lastParseResultRef.current.result;
      }

      let result;

      try {
        // Try to parse complete JSON first
        const parsed = JSON.parse(jsonStr);
        result = { complete: true, data: parsed };
      } catch (e) {
        // If incomplete, try to extract what we can
        try {
          // Look for title (cached regex for performance)
          const titleMatch = jsonStr.match(/"title"\s*:\s*"([^"]+)"/);
          const title = titleMatch ? titleMatch[1] : undefined;

          // Look for layout
          const layoutMatch = jsonStr.match(/"layout"\s*:\s*"([^"]+)"/);
          const layout = layoutMatch ? layoutMatch[1] : undefined;

          // Try to extract components from array (even if unclosed during streaming)
          // Match: "components": [ ... (may be unclosed)
          const componentsMatch = jsonStr.match(/"components"\s*:\s*\[([\s\S]*)/);
          let components: UIComponent[] = [];

          if (componentsMatch) {
            let componentsStr = componentsMatch[1];
            // Remove trailing incomplete data after last complete object
            const lastBraceIdx = componentsStr.lastIndexOf("}");
            if (lastBraceIdx !== -1) {
              componentsStr = componentsStr.substring(0, lastBraceIdx + 1);
            }

            // Extract complete component objects using a better approach for deep nesting
            // Optimize for streaming: reuse previously parsed components
            const cachedComponents = lastParseResultRef.current?.result?.data?.components;
            const previousComponents = (isStreamingAppend && cachedComponents) || [];
            const previousCount = previousComponents.length;

            // Debug: Why is cache not working?
            if (isStreamingAppend && previousCount === 0) {
              logger.info("Cache empty despite streaming", {
                component: "DynamicRenderer",
                isStreamingAppend,
                hasCachedResult: !!lastParseResultRef.current?.result,
                hasCachedData: !!lastParseResultRef.current?.result?.data,
                hasCachedComponents: !!cachedComponents,
                cachedComponentCount: cachedComponents?.length || 0,
              });
            }

            // Extract all components (this will include old + new)
            const allComponents = extractComponentsRecursively(componentsStr);

            // Debug extraction results
            logger.info("Component extraction results", {
              component: "DynamicRenderer",
              allComponentsCount: allComponents.length,
              previousCount,
              isStreamingAppend,
              componentsStrLength: componentsStr.length,
            });

            // Reuse cached components and merge updates
            if (isStreamingAppend) {
              // CRITICAL: Never go backwards - if we extracted fewer components than before,
              // keep the previous ones (aggressive parse may have failed)
              if (allComponents.length < previousCount) {
                logger.info("Extraction regressed - keeping previous components", {
                  component: "DynamicRenderer",
                  previousCount,
                  newCount: allComponents.length,
                });
                components = previousComponents;
              } else {
                // Merge all components to handle both new components AND updated children
                components = allComponents.map((newComp, idx) => {
                  const existing = previousComponents[idx];
                  if (existing && existing.id === newComp.id) {
                    // Same component by ID and position
                    const childrenChanged = newComp.children?.length !== existing.children?.length;
                    
                    if (childrenChanged && newComp.children) {
                      // Children changed - check if we're just appending or replacing
                      const existingChildren = existing.children || [];
                      const newChildren = newComp.children;
                      
                      // Never go backwards on children either
                      if (newChildren.length < existingChildren.length) {
                        return existing; // Keep old version with more children
                      }
                      
                      // If all existing children are still present in the same order, we're appending
                      const isAppending = 
                        newChildren.length > existingChildren.length &&
                        existingChildren.every((ec: UIComponent, i: number) => ec.id === newChildren[i]?.id);
                      
                      if (isAppending) {
                        // Just append new children, keep existing array reference for old ones
                        const appendedChildren = [
                          ...existingChildren, // Reuse existing array slice
                          ...newChildren.slice(existingChildren.length) // Add new ones
                        ];
                        return { ...existing, children: appendedChildren };
                      } else {
                        // Children were reordered or replaced - merge by ID
                        const mergedChildren = newChildren.map((newChild) => {
                          const existingChild = existingChildren.find((c: UIComponent) => c.id === newChild.id);
                          return existingChild || newChild;
                        });
                        return { ...existing, children: mergedChildren };
                      }
                    }
                    // No change
                    return existing;
                  }
                  // New component
                  return newComp;
                });
              }
            } else {
              // Not streaming or components changed, use all extracted
              // But merge updates for existing components to maintain object identity
              components = allComponents.map((newComp) => {
                const existing = previousComponents.find((c: UIComponent) => c.id === newComp.id);
                if (existing) {
                  // Same component, check if children changed
                  const childrenChanged = 
                    newComp.children?.length !== existing.children?.length;
                  
                  if (childrenChanged && newComp.children) {
                    // Merge children: reuse existing children objects where possible
                    const mergedChildren = newComp.children.map((newChild) => {
                      const existingChild = existing.children?.find((c: UIComponent) => c.id === newChild.id);
                      // Reuse existing child if it has the same ID
                      return existingChild || newChild;
                    });
                    
                    // Return new object with merged children
                    return { ...existing, children: mergedChildren };
                  }
                  // No change, reuse existing object reference
                  return existing;
                }
                // New component
                return newComp;
              });
            }

            if (components.length > previousCount) {
              // Debug: Check what components we're extracting
              const newComponents = components.slice(previousCount);
              logger.info("Partial JSON parsing progress", {
                component: "DynamicRenderer",
                componentsFound: components.length,
                newComponents: newComponents.length,
                cached: previousCount,
                isStreaming: isStreamingAppend,
                charsProcessed: componentsStr.length,
                newComponentTypes: newComponents.map(
                  (c) => `${c.type}(children:${c.children?.length || 0})`
                ),
                sampleComponent: newComponents[0]
                  ? {
                      type: newComponents[0].type,
                      id: newComponents[0].id,
                      hasChildren: !!newComponents[0].children,
                      childrenCount: newComponents[0].children?.length || 0,
                    }
                  : null,
              });
            }
          }

        if (title || layout || components.length > 0) {
          // Reuse the previous components array if it hasn't changed
          const prevComponents = lastParseResultRef.current?.result?.data?.components;
          const componentsToUse = 
            prevComponents && 
            prevComponents.length === components.length &&
            prevComponents.every((c: UIComponent, i: number) => c === components[i])
              ? prevComponents  // Reuse array reference
              : components;     // New array
          
          result = {
            complete: false,
            data: {
              title,
              layout,
              components: componentsToUse,
              type: "app",
            },
          };
        } else {
          result = { complete: false, data: null };
        }
      } catch (parseError) {
          // Silent fail - we'll try again with more data
          result = { complete: false, data: null };
        }
      }

      // Cache result for next call
      lastParseResultRef.current = {
        preview: jsonStr,
        result,
      };

      endPerf("json_parse");

      return result;
    },
    [extractComponentsRecursively]
  );

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
          setPartialUISpec({ components: [] });
          renderedComponentIdsRef.current = new Set(); // Reset for new generation
          setLastComponentCount(0);
          lastParseResultRef.current = null; // Clear parse cache for new generation
          lastAnimatedCountRef.current = 0; // Reset animation counter for new generation
          animatedComponentIds.current.clear(); // Reset animated component tracking
          previousComponentsRef.current = []; // Reset previous components for animation tracking
          buildContainerAnimatedRef.current = false; // Reset build container animation
          lastStateUpdateRef.current = 0; // Reset update timing
          lastChildrenCountRef.current.clear(); // Reset children count tracking
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
          logger.verboseThrottled("Generation token received", {
            component: "DynamicRenderer",
            contentLength: message.content?.length || 0,
          });
          // Accumulate tokens for real-time display
          if (message.content) {
            appendGenerationPreview(message.content);

            // Debounce partial JSON parsing to reduce CPU usage
            if (parseDebounceTimerRef.current !== null) {
              window.clearTimeout(parseDebounceTimerRef.current);
            }

            parseDebounceTimerRef.current = window.setTimeout(() => {
              // Try to parse partial JSON and update partial UI spec
              const currentPreview = useAppStore.getState().generationPreview;
              const parseResult = parsePartialJSON(currentPreview);

              logger.debugThrottled("Parse result", {
                component: "DynamicRenderer",
                complete: parseResult.complete,
                hasData: !!parseResult.data,
                componentCount: parseResult.data?.components?.length || 0,
                previewLength: currentPreview.length,
              });

              if (parseResult.data) {
                const partial = parseResult.data as Partial<UISpec>;

                // Update partial UI spec if we have new data
                if (
                  partial.title ||
                  partial.layout ||
                  (partial.components && partial.components.length > 0)
                ) {
                  const currentCount = partial.components?.length || 0;
                  const prevCount = lastComponentCount;
                  const newComponentCount = currentCount - prevCount;

                  if (newComponentCount > 0) {
                    logger.info("New components parsed", {
                      component: "DynamicRenderer",
                      newComponents: newComponentCount,
                      totalComponents: currentCount,
                      previousCount: prevCount,
                    });
                    setLastComponentCount(currentCount);
                  }

                  logger.debugThrottled("Updating partial UI spec", {
                    component: "DynamicRenderer",
                    hasTitle: !!partial.title,
                    componentCount: currentCount,
                  });
                  
                  // Batch updates to prevent flickering
                  // Only update if enough has changed or enough time has passed
                  const now = Date.now();
                  const timeSinceLastUpdate = now - lastStateUpdateRef.current;
                  const MIN_UPDATE_INTERVAL = 100; // ms
                  
                  let shouldUpdate = false;
                  
                  // Always update if:
                  // 1. First update
                  // 2. Top-level component count changed
                  // 3. Enough time passed since last update
                  if (!useAppStore.getState().partialUISpec) {
                    shouldUpdate = true;
                  } else if (partial.components) {
                    const currentSpec = useAppStore.getState().partialUISpec;
                    const topLevelCountChanged = 
                      partial.components.length !== (currentSpec?.components?.length || 0);
                    
                    // Check if significant children changes (at least 3 new children in any component)
                    let significantChildrenChange = false;
                    partial.components.forEach((comp) => {
                      const lastCount = lastChildrenCountRef.current.get(comp.id) || 0;
                      const newCount = comp.children?.length || 0;
                      if (newCount - lastCount >= 3) {
                        significantChildrenChange = true;
                        lastChildrenCountRef.current.set(comp.id, newCount);
                      }
                    });
                    
                    shouldUpdate = 
                      topLevelCountChanged ||
                      significantChildrenChange ||
                      timeSinceLastUpdate >= MIN_UPDATE_INTERVAL;
                  }
                  
                  if (shouldUpdate) {
                    setPartialUISpec(partial);
                    lastStateUpdateRef.current = now;
                  }

                  // Track component IDs in ref (immediate, no re-render needed)
                  if (partial.components) {
                    partial.components.forEach((comp) => {
                      renderedComponentIdsRef.current.add(comp.id);
                    });
                  }

                  // Calculate progress based on components found
                  if (partial.components && partial.components.length > 0) {
                    // Estimate total components (we'll refine this as we get more data)
                    const estimatedTotal = Math.max(10, partial.components.length + 5);
                    const progress = Math.min(
                      90,
                      (partial.components.length / estimatedTotal) * 100
                    );
                    setBuildProgress(progress);
                  }
                }
              }

              parseDebounceTimerRef.current = null;
            }, PARSE_DEBOUNCE_MS);
          }
          break;

        case "ui_generated":
          try {
            // Validate UI spec size and depth
            const uiSpecStr = JSON.stringify(message.ui_spec);
            validateJSONSize(uiSpecStr);
            validateJSONDepth(message.ui_spec);

            logger.info("UI generated successfully", {
              component: "DynamicRenderer",
              title: message.ui_spec.title,
              appId: message.app_id,
              componentCount: message.ui_spec.components.length,
            });
            setUISpec(message.ui_spec, message.app_id);

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
          if (parseDebounceTimerRef.current !== null) {
            window.clearTimeout(parseDebounceTimerRef.current);
            parseDebounceTimerRef.current = null;
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
          if (parseDebounceTimerRef.current !== null) {
            window.clearTimeout(parseDebounceTimerRef.current);
            parseDebounceTimerRef.current = null;
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
      if (parseDebounceTimerRef.current !== null) {
        window.clearTimeout(parseDebounceTimerRef.current);
        parseDebounceTimerRef.current = null;
      }
      unsubscribe();
    };
  }, [
    client,
    componentState,
    toolExecutor,
    setUISpec,
    setLoading,
    setError,
    addGenerationThought,
    appendGenerationPreview,
    parsePartialJSON,
    setStreaming,
    setBuildProgress,
    setPartialUISpec,
    // Removed lastComponentCount - it's only used inside the effect, doesn't need to be a dependency
  ]);

  // Cleanup: Execute on_unmount hooks when component unmounts or UI changes
  useEffect(() => {
    return () => {
      // Clean up timers first
      toolExecutor.cleanup();

      // Then execute lifecycle hooks
      if (uiSpec?.lifecycle_hooks?.on_unmount) {
        logger.info("Executing on_unmount hooks", {
          component: "DynamicRenderer",
          hookCount: uiSpec.lifecycle_hooks.on_unmount.length,
        });
        uiSpec.lifecycle_hooks.on_unmount.forEach(async (toolId: string) => {
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
  }, [uiSpec, toolExecutor]);

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
  const iconBounce = useBounceAttention(!uiSpec && !isLoading && !isStreaming);
  const saveButtonMagnetic = useMagneticHover(0.15);
  const desktopFadeIn = useSpringFadeIn(!uiSpec && !isLoading && !isStreaming);

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
      (isStreaming || partialUISpec) &&
      !buildContainerAnimatedRef.current
    ) {
      gsapAnimations.buildContainerAppear(buildContainerRef.current);
      buildContainerAnimatedRef.current = true;
    }
  }, [isStreaming, partialUISpec]);

  // Animate rendered app on appearance with creative effects
  useEffect(() => {
    if (burstRef.current && uiSpec) {
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
  }, [uiSpec, burst, burstRef]);

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

  // Memoize filtered components to prevent re-filtering on every render
  const validPartialComponents = React.useMemo(() => {
    return partialUISpec?.components?.filter((c) => c && c.type && c.type !== "undefined") || [];
  }, [partialUISpec?.components]);

  const validUISpecComponents = React.useMemo(() => {
    return uiSpec?.components?.filter((c) => c && c.type && c.type !== "undefined") || [];
  }, [uiSpec?.components]);

  // Animate components as they appear during build - with varied creative effects
  const componentRefs = useRef<Map<string, HTMLDivElement>>(new Map());

  // Store previous components to detect new ones without triggering effect on every change
  const previousComponentsRef = useRef<UIComponent[]>([]);

  // Function to animate a single component
  const animateComponent = useCallback((component: UIComponent, idx: number) => {
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
    hasUISpec: !!uiSpec,
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

        {(isLoading || isStreaming || (partialUISpec && !uiSpec)) && (
          <>
            {/* Visual Build Mode - Show partial UI being constructed */}
            {isStreaming || partialUISpec ? (
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
                  {partialUISpec?.title && (
                    <h2 className="building-app-title">{partialUISpec.title}</h2>
                  )}
                  <div className={`app-content app-layout-${partialUISpec?.layout || "vertical"}`}>
                    {validPartialComponents.length >= VIRTUAL_SCROLL_THRESHOLD ? (
                      // Use virtual scrolling for large component lists during build
                      <VirtualizedList
                        children={validPartialComponents}
                        state={componentState}
                        executor={toolExecutor}
                        itemHeight={DEFAULT_ITEM_HEIGHT}
                        maxHeight={600}
                        layout={
                          (partialUISpec?.layout || "vertical") as
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

        {!uiSpec && !isLoading && !isStreaming && !error && (
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

        {uiSpec && (
          <div ref={burstRef} className="rendered-app" style={uiSpec.style}>
            <div className="app-header">
              <h2>{uiSpec.title}</h2>
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
            <div className={`app-content app-layout-${uiSpec.layout}`}>
              {validUISpecComponents.length >= VIRTUAL_SCROLL_THRESHOLD ? (
                // Use virtual scrolling for large apps
                <VirtualizedList
                  children={validUISpecComponents}
                  state={componentState}
                  executor={toolExecutor}
                  itemHeight={DEFAULT_ITEM_HEIGHT}
                  maxHeight={700}
                  layout={(uiSpec.layout || "vertical") as "vertical" | "horizontal" | "grid"}
                  className="app-components-list"
                  ComponentRenderer={ComponentRenderer}
                />
              ) : (
                // Normal rendering for small apps
                validUISpecComponents.map((component, idx) => (
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
