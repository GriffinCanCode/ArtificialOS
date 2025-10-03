/**
 * Dynamic Renderer Component
 * Renders AI-generated applications on the fly
 */

import React, { useState, useCallback, useEffect, useRef } from "react";
import { Construction, MessageCircle, Palette, Save, Sparkles, Zap } from "lucide-react";
import { useWebSocket } from "../contexts/WebSocketContext";
import { animated } from '@react-spring/web';
import { 
  useSpringFadeIn,
  useMagneticHover,
  useJelloWobble,
  useBounceAttention,
} from "../hooks/useSpring";
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
} from "../store/appStore";
import { logger } from "../utils/monitoring/logger";
import { startPerf, endPerf } from "../utils/monitoring/performanceMonitor";
import { useSaveApp } from "../hooks/useRegistryQueries";
import * as gsapAnimations from "../utils/animation/gsapAnimations";
import {
  useFloat,
  useSpin,
  useBuildPulse,
  useImperativeAnimation,
  useStaggerSlideUp,
  useBlurReveal,
  useGlowPulse,
  useParticleBurst,
} from "../hooks/useGSAP";
import { SaveAppDialog } from "./dialogs/SaveAppDialog";
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

  const [componentState] = useState(() => {
    logger.info("Initializing component state", { component: "DynamicRenderer" });
    const state = new ComponentState();
    
    // Configure state management
    state.setMaxHistorySize(100); // Keep history for undo/redo
    
    // Setup global computed values
    state.defineComputed('formValid', ['error.*'], (stateMap) => {
      for (const [key, value] of stateMap.entries()) {
        if (key.startsWith('error.') && value) {
          return false;
        }
      }
      return true;
    });

    // Setup global wildcard listeners for debugging
    if (process.env.NODE_ENV === 'development') {
      state.subscribeWildcard('error.*', (event) => {
        logger.error("Error state changed", undefined, {
          component: "ComponentState",
          key: event.key,
          error: event.newValue,
        });
      });
    }

    return state;
  });
  
  const [toolExecutor] = useState(() => {
    logger.info("Initializing tool executor", { component: "DynamicRenderer" });
    return new ToolExecutor(componentState);
  });

  // Track which component IDs we've already rendered to avoid re-animating
  // Use ref so it updates immediately without waiting for React re-render
  const renderedComponentIdsRef = React.useRef<Set<string>>(new Set());
  const [lastComponentCount, setLastComponentCount] = useState(0);

  // Debounce partial JSON parsing
  const parseDebounceTimerRef = React.useRef<number | null>(null);
  
  // Cache last parse result to avoid redundant parsing
  const lastParseResultRef = React.useRef<{
    preview: string;
    result: any;
    hash: string;
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
  const parsePartialJSON = useCallback((jsonStr: string) => {
    // Performance monitoring
    startPerf('json_parse', { size: jsonStr.length });
    
    // Quick hash for cache check (simple but effective)
    const hash = `${jsonStr.length}-${jsonStr.slice(0, 50)}`;
    
    // Return cached result if input hasn't changed significantly
    if (lastParseResultRef.current?.hash === hash) {
      endPerf('json_parse', { cached: true });
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

          // Extract complete component objects (handles nested objects in props/on_event)
          // Optimized: Only parse new components beyond what we've already parsed
          const componentRegex = /\{(?:[^{}]|\{[^{}]*\})*\}/g;
          const matches = componentsStr.match(componentRegex);

          if (matches) {
            // Only parse components we haven't seen before
            const previousCount = lastParseResultRef.current?.result?.data?.components?.length || 0;
            const newMatches = matches.slice(previousCount);
            
            // Reuse previously parsed components if available
            const previousComponents = lastParseResultRef.current?.result?.data?.components || [];
            
            // Parse only new components for efficiency
            const newComponents = newMatches
              .map((m) => {
                try {
                  return JSON.parse(m);
                } catch {
                  return null;
                }
              })
              .filter((c): c is UIComponent => c !== null);
            
            components = [...previousComponents, ...newComponents];

            logger.debugThrottled("Partial JSON parsing progress", {
              component: "DynamicRenderer",
              componentsFound: components.length,
              newComponents: newComponents.length,
              cached: previousComponents.length,
              charsProcessed: componentsStr.length,
            });
          }
        }

        if (title || layout || components.length > 0) {
          result = {
            complete: false,
            data: {
              title,
              layout,
              components,
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
      hash,
    };
    
    endPerf('json_parse', { 
      cached: false,
      componentsFound: result?.data?.components?.length || 0 
    });
    
    return result;
  }, []);


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
                  setPartialUISpec(partial);

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
                    const progress = Math.min(90, (partial.components.length / estimatedTotal) * 100);
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
                componentState.defineComputed('componentCount', ['app_initialized'], () => {
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
    lastComponentCount,
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
  const thoughtsListRef = useStaggerSlideUp<HTMLDivElement>('.thought-item', { stagger: 0.08, distance: 25 });
  const saveButtonRef = useGlowPulse<HTMLButtonElement>(false, { color: '99, 102, 241', intensity: 15 });
  const { elementRef: burstRef, burst } = useParticleBurst<HTMLDivElement>();
  
  // React Spring animations
  const errorJello = useJelloWobble(!!error);
  const iconBounce = useBounceAttention(!uiSpec && !isLoading && !isStreaming);
  const saveButtonMagnetic = useMagneticHover(0.15);
  const desktopFadeIn = useSpringFadeIn(!uiSpec && !isLoading && !isStreaming);
  
  // Animate error with chromatic aberration and spring wobble
  useEffect(() => {
    if (error && errorRef.elementRef.current) {
      gsapAnimations.chromaticAberration(errorRef.elementRef.current, { intensity: 6, duration: 0.5 });
      setTimeout(() => {
        if (errorRef.elementRef.current) {
          errorRef.shake();
          gsapAnimations.wobble(errorRef.elementRef.current);
        }
      }, 300);
    }
  }, [error, errorRef]);

  // Animate build container on appearance
  useEffect(() => {
    if (buildContainerRef.current && (isStreaming || partialUISpec)) {
      gsapAnimations.buildContainerAppear(buildContainerRef.current);
    }
  }, [isStreaming, partialUISpec]);

  // Animate rendered app on appearance with creative effects
  useEffect(() => {
    if (burstRef.current && uiSpec) {
      // Use ink blot reveal for a more organic appearance
      gsapAnimations.inkBlotReveal(burstRef.current, { duration: 1.2, from: 'center' });
      
      // Add particle burst after a short delay
      setTimeout(() => {
        if (burstRef.current) {
          burst({ count: 18, duration: 1.8 });
        }
      }, 500);
      
      // Add subtle energy pulse to the app title
      const appTitle = burstRef.current.querySelector('.app-header h2');
      if (appTitle) {
        gsapAnimations.energyPulse(appTitle, { color: '139, 92, 246', count: 2 });
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
  const handleSaveApp = useCallback(async (data: {
    description: string;
    category: string;
    icon: string;
    tags: string[];
  }) => {
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
  }, [saveAppMutation]);

  const handleSaveDialogOpen = useCallback(() => {
    setShowSaveAppDialog(true);
  }, []);

  const handleSaveDialogClose = useCallback(() => {
    setShowSaveAppDialog(false);
  }, []);

  // Animate components as they appear during build - with varied creative effects
  const componentRefs = useRef<Map<string, HTMLDivElement>>(new Map());
  
  useEffect(() => {
    if (partialUISpec?.components) {
      partialUISpec.components.forEach((component, idx) => {
        const key = `${component.id}-${idx}`;
        const element = componentRefs.current.get(key);
        if (element && !element.dataset.animated) {
          // Vary animations for visual interest
          const animationType = idx % 3;
          const delay = idx * 0.05;
          
          switch (animationType) {
            case 0:
              gsapAnimations.vortexSpiral(element, { duration: 0.8, clockwise: idx % 2 === 0 });
              break;
            case 1:
              gsapAnimations.liquidMorph(element, { duration: 0.9, intensity: 12 });
              break;
            case 2:
              gsapAnimations.elasticBounceIn(element, { delay, from: 'bottom' });
              break;
          }
          
          element.dataset.animated = 'true';
        }
      });
    }
  }, [partialUISpec?.components]);

  // Clean desktop - no auto-loading

  // Debug: Log render state
  logger.verboseThrottled("Render state", {
    component: "DynamicRenderer",
    isLoading,
    isStreaming,
    hasPartialComponents: (partialUISpec?.components?.length || 0) > 0,
    partialCount: partialUISpec?.components?.length,
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
          <animated.div 
            ref={errorRef.elementRef} 
            className="renderer-error"
            style={errorJello}
          >
            <strong>Error:</strong> {error}
          </animated.div>
        )}

        {(isLoading || isStreaming || (partialUISpec && !uiSpec)) && (
          <>
            {/* Visual Build Mode - Show partial UI being constructed */}
            {isStreaming || partialUISpec ? (
              <div ref={buildContainerRef} className="building-app-preview">
                <div className="build-progress-header">
                  <div ref={buildIconRef} className="build-status-icon"><Construction size={24} /></div>
                  <div className="build-status-text">
                    <h3>Building Your App...</h3>
                    <p>{partialUISpec?.components?.length || 0} components assembled</p>
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
                    {partialUISpec?.components && partialUISpec.components.length >= VIRTUAL_SCROLL_THRESHOLD ? (
                      // Use virtual scrolling for large component lists during build
                      <VirtualizedList
                        children={partialUISpec.components}
                        state={componentState}
                        executor={toolExecutor}
                        itemHeight={DEFAULT_ITEM_HEIGHT}
                        maxHeight={600}
                        layout={(partialUISpec.layout || 'vertical') as 'vertical' | 'horizontal' | 'grid'}
                        className="building-components-list"
                        ComponentRenderer={ComponentRenderer}
                      />
                    ) : (
                      // Normal rendering for small lists
                      <>
                        {partialUISpec?.components?.map((component, idx) => (
                          <div
                            key={`building-${component.id}-${idx}`}
                            ref={(el) => {
                              if (el) componentRefs.current.set(`${component.id}-${idx}`, el);
                            }}
                            className="component-building"
                          >
                            <ComponentRenderer
                              component={component}
                              state={componentState}
                              executor={toolExecutor}
                            />
                          </div>
                        ))}
                        {(!partialUISpec?.components || partialUISpec.components.length === 0) && (
                          <div
                            className="component-assembling"
                            style={{ padding: "2rem", textAlign: "center" }}
                          >
                            <div ref={buildSpinnerRef} className="spinner" style={{ margin: "0 auto 1rem" }}></div>
                            <p style={{ opacity: 0.7 }}>Assembling components...</p>
                          </div>
                        )}
                      </>
                    )}
                  </div>
                </div>

                {/* Thoughts stream below */}
                {generationThoughts.length > 0 && (
                  <div ref={thoughtsListRef} className="thoughts-list" style={{ marginTop: "1rem", opacity: 0.7 }}>
                    {generationThoughts.slice(-3).map((thought, i) => (
                      <div key={i} className="thought-item">
                        <span className="thought-icon"><MessageCircle size={16} /></span>
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
                  <h3><Palette size={20} style={{ display: 'inline-block', marginRight: '8px', verticalAlign: 'middle' }} />Generating Application...</h3>
                </div>
                <div ref={thoughtsListRef} className="thoughts-list">
                  {generationThoughts.map((thought, i) => (
                    <div key={i} className="thought-item">
                      <span className="thought-icon"><MessageCircle size={16} /></span>
                      <span className="thought-text">{thought}</span>
                    </div>
                  ))}
                </div>
                {generationPreview && (
                  <div className="generation-preview">
                    <div className="preview-header">
                      <span className="preview-icon"><Zap size={16} /></span>
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
            <animated.div 
              ref={desktopMessageRef} 
              className="desktop-message"
              style={desktopFadeIn}
            >
              <animated.div 
                ref={desktopIconRef} 
                className="desktop-icon"
                style={iconBounce}
              >
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
                    (saveButtonRef as React.MutableRefObject<HTMLButtonElement | null>).current = el;
                  }
                  (saveButtonMagnetic.ref as React.MutableRefObject<HTMLButtonElement | null>).current = el;
                }}
                className="save-app-btn"
                onClick={handleSaveDialogOpen}
                {...saveButtonMagnetic.handlers}
                style={saveButtonMagnetic.style}
                title="Save this app to registry"
              >
                <Save size={16} style={{ marginRight: '6px', verticalAlign: 'middle' }} />
                Save
              </animated.button>
            </div>
            <div className={`app-content app-layout-${uiSpec.layout}`}>
              {uiSpec.components.length >= VIRTUAL_SCROLL_THRESHOLD ? (
                // Use virtual scrolling for large apps
                <VirtualizedList
                  children={uiSpec.components}
                  state={componentState}
                  executor={toolExecutor}
                  itemHeight={DEFAULT_ITEM_HEIGHT}
                  maxHeight={700}
                  layout={(uiSpec.layout || 'vertical') as 'vertical' | 'horizontal' | 'grid'}
                  className="app-components-list"
                  ComponentRenderer={ComponentRenderer}
                />
              ) : (
                // Normal rendering for small apps
                uiSpec.components.map((component, idx) => (
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

