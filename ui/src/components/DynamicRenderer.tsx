/**
 * Dynamic Renderer Component
 * Renders AI-generated applications on the fly
 */

import React, { useState, useCallback, useEffect, useRef } from "react";
import { Construction, MessageCircle, Palette, Save, Sparkles, Zap } from "lucide-react";
import { useWebSocket } from "../contexts/WebSocketContext";
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
import { logger } from "../utils/logger";
import { useSaveApp } from "../hooks/useRegistryQueries";
import * as gsapAnimations from "../utils/gsapAnimations";
import {
  useFadeIn,
  useFloat,
  useSpin,
  useBuildPulse,
  useImperativeAnimation,
  useStaggerSlideUp,
  useBlurReveal,
  useElasticBounceIn,
  useGlowPulse,
  useWobble,
  useParticleBurst,
} from "../hooks/useGSAP";
import { SaveAppDialog } from "./SaveAppDialog";
import {
  buttonVariants,
  inputVariants,
  textVariants,
  containerVariants,
  gridVariants,
  cn,
} from "../utils/componentVariants";
import "./DynamicRenderer.css";

// ============================================================================
// Component State Manager
// ============================================================================

class ComponentState {
  private state: Map<string, any> = new Map();
  private listeners: Map<string, Set<(value: any) => void>> = new Map();

  get(key: string, defaultValue: any = null): any {
    return this.state.get(key) ?? defaultValue;
  }

  set(key: string, value: any): void {
    this.state.set(key, value);
    // Notify listeners
    const listeners = this.listeners.get(key);
    if (listeners) {
      listeners.forEach((listener) => listener(value));
    }
  }

  subscribe(key: string, listener: (value: any) => void): () => void {
    if (!this.listeners.has(key)) {
      this.listeners.set(key, new Set());
    }
    this.listeners.get(key)!.add(listener);

    // Return unsubscribe function
    return () => {
      const listeners = this.listeners.get(key);
      if (listeners) {
        listeners.delete(listener);
      }
    };
  }

  clear(): void {
    this.state.clear();
    this.listeners.clear();
  }
}

// ============================================================================
// Tool Execution
// ============================================================================

class ToolExecutor {
  private componentState: ComponentState;
  private appId: string | null = null;

  constructor(componentState: ComponentState) {
    this.componentState = componentState;
  }

  setAppId(appId: string): void {
    this.appId = appId;
  }

  async execute(toolId: string, params: Record<string, any> = {}): Promise<any> {
    const startTime = performance.now();
    try {
      logger.debug("Executing tool", {
        component: "ToolExecutor",
        toolId,
        paramsCount: Object.keys(params).length,
      });

      // Check if this is a service tool (contains service prefix like storage.*, auth.*, ai.*)
      const servicePrefixes = ["storage", "auth", "ai", "sync", "media"];
      const [category] = toolId.split(".");

      let result;
      if (servicePrefixes.includes(category)) {
        result = await this.executeServiceTool(toolId, params);
      } else {
        // Handle built-in tools
        switch (category) {
          case "calc":
            result = this.executeCalcTool(toolId.split(".")[1], params);
            break;
          case "ui":
            result = this.executeUITool(toolId.split(".")[1], params);
            break;
          case "system":
            result = this.executeSystemTool(toolId.split(".")[1], params);
            break;
          case "app":
            result = await this.executeAppTool(toolId.split(".")[1], params);
            break;
          case "http":
            result = await this.executeNetworkTool(toolId.split(".")[1], params);
            break;
          case "timer":
            result = this.executeTimerTool(toolId.split(".")[1], params);
            break;
          default:
            logger.warn("Unknown tool category", {
              component: "ToolExecutor",
              category,
              toolId,
            });
            result = null;
        }
      }

      const duration = performance.now() - startTime;
      logger.performance("Tool execution", duration, {
        component: "ToolExecutor",
        toolId,
      });

      return result;
    } catch (error) {
      const duration = performance.now() - startTime;
      logger.error("Tool execution failed", error as Error, {
        component: "ToolExecutor",
        toolId,
        duration,
      });
      this.componentState.set(
        "error",
        `Tool ${toolId} failed: ${error instanceof Error ? error.message : "Unknown error"}`
      );
      return null;
    }
  }

  private async executeServiceTool(toolId: string, params: Record<string, any>): Promise<any> {
    logger.debug("Executing service tool", {
      component: "ToolExecutor",
      toolId,
    });

    try {
      const response = await fetch("http://localhost:8000/services/execute", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({
          tool_id: toolId,
          params: params,
          app_id: this.appId,
        }),
      });

      if (!response.ok) {
        throw new Error(`Service call failed: ${response.statusText}`);
      }

      const result = await response.json();

      if (!result.success) {
        throw new Error(result.error || "Service execution failed");
      }

      logger.debug("Service tool executed successfully", {
        component: "ToolExecutor",
        toolId,
        hasData: !!result.data,
      });
      return result.data;
    } catch (error) {
      logger.error("Service tool execution failed", error as Error, {
        component: "ToolExecutor",
        toolId,
      });
      throw error;
    }
  }

  private executeCalcTool(action: string, params: Record<string, any>): any {
    const a = params.a || 0;
    const b = params.b || 0;

    switch (action) {
      case "add":
        return a + b;
      case "subtract":
        return a - b;
      case "multiply":
        return a * b;
      case "divide":
        return b !== 0 ? a / b : "Error";
      case "append_digit":
        const current = this.componentState.get("display", "0");
        const digit = params.digit || "";
        const newValue = current === "0" ? digit : current + digit;
        this.componentState.set("display", newValue);
        return newValue;
      case "clear":
        this.componentState.set("display", "0");
        return "0";
      case "evaluate":
        try {
          const expression = this.componentState.get("display", "0");
          // Simple eval (in production, use a proper math parser!)
          const result = eval(expression.replace("×", "*").replace("÷", "/").replace("−", "-"));
          this.componentState.set("display", String(result));
          return result;
        } catch {
          this.componentState.set("display", "Error");
          return "Error";
        }
      default:
        return null;
    }
  }

  private executeUITool(action: string, params: Record<string, any>): any {
    switch (action) {
      case "set_state":
        this.componentState.set(params.key, params.value);
        return params.value;
      case "get_state":
        return this.componentState.get(params.key);
      case "add_todo":
        const todos = this.componentState.get("todos", []);
        const newTask = this.componentState.get("task-input", "");
        if (newTask.trim()) {
          todos.push({ id: Date.now(), text: newTask, done: false });
          this.componentState.set("todos", [...todos]);
          this.componentState.set("task-input", "");
        }
        return todos;
      default:
        return null;
    }
  }

  private executeSystemTool(action: string, params: Record<string, any>): any {
    switch (action) {
      case "alert":
        alert(params.message);
        return true;
      case "log":
        logger.info("System log", { component: "SystemTool", message: params.message });
        return true;
      default:
        return null;
    }
  }

  private async executeAppTool(action: string, params: Record<string, any>): Promise<any> {
    switch (action) {
      case "spawn":
        logger.info("Spawning new app", {
          component: "AppTool",
          request: params.request,
        });
        // Use HTTP endpoint for app spawning instead of creating new WebSocket
        const response = await fetch("http://localhost:8000/generate-ui", {
          method: "POST",
          headers: { "Content-Type": "application/json" },
          body: JSON.stringify({
            message: params.request,
            context: { parent_app_id: this.componentState.get("app_id") },
          }),
        });
        const data = await response.json();

        if (data.error) {
          throw new Error(data.error);
        }

        // Notify parent component to render new app
        window.postMessage(
          {
            type: "spawn_app",
            app_id: data.app_id,
            ui_spec: data.ui_spec,
          },
          "*"
        );

        return data.ui_spec;

      case "close":
        logger.info("Closing current app", { component: "AppTool" });
        // Notify parent to close this app
        window.postMessage({ type: "close_app" }, "*");
        return true;

      case "list":
        logger.info("Listing apps", { component: "AppTool" });
        const appsResponse = await fetch("http://localhost:8000/apps");
        const appsData = await appsResponse.json();
        return appsData.apps;

      default:
        return null;
    }
  }

  // Storage tools now handled by service system via executeServiceTool()

  private async executeNetworkTool(action: string, params: Record<string, any>): Promise<any> {
    switch (action) {
      case "get":
        const getResponse = await fetch(params.url);
        return await getResponse.json();
      case "post":
        const postResponse = await fetch(params.url, {
          method: "POST",
          headers: { "Content-Type": "application/json" },
          body: JSON.stringify(params.data),
        });
        return await postResponse.json();
      default:
        return null;
    }
  }

  private executeTimerTool(action: string, params: Record<string, any>): any {
    switch (action) {
      case "set":
        const timerId = setTimeout(() => {
          // Execute the action (would need tool executor reference)
          logger.debug("Executing delayed action", {
            component: "TimerTool",
            action: params.action,
          });
        }, params.delay);
        this.componentState.set(`timer_${timerId}`, timerId);
        return timerId;
      case "interval":
        const intervalId = setInterval(() => {
          logger.debug("Executing interval action", {
            component: "TimerTool",
            action: params.action,
          });
        }, params.interval);
        this.componentState.set(`interval_${intervalId}`, intervalId);
        return intervalId;
      case "clear":
        const id = this.componentState.get(params.timer_id);
        if (id) {
          clearTimeout(id);
          clearInterval(id);
        }
        return true;
      default:
        return null;
    }
  }
}

// ============================================================================
// Component Renderers
// ============================================================================

interface RendererProps {
  component: UIComponent;
  state: ComponentState;
  executor: ToolExecutor;
}

const ComponentRenderer: React.FC<RendererProps> = ({ component, state, executor }) => {
  const [, forceUpdate] = useState({});

  // Subscribe to state changes for this component
  React.useEffect(() => {
    if (component.id) {
      const unsubscribe = state.subscribe(component.id, () => {
        forceUpdate({});
      });
      return unsubscribe;
    }
  }, [component.id, state]);

  const handleEvent = useCallback(
    (eventName: string, eventData?: any) => {
      const toolId = component.on_event?.[eventName];
      if (toolId) {
        // Extract params from event or component
        const params = {
          ...eventData,
          componentId: component.id,
          digit: component.props?.text, // For calculator buttons
        };
        executor.execute(toolId, params);
      }
    },
    [component, executor]
  );

  // Render based on component type
  switch (component.type) {
    case "button":
      return (
        <button
          className={cn(
            buttonVariants({
              variant: component.props?.variant as any,
              size: component.props?.size as any,
              fullWidth: component.props?.fullWidth,
            })
          )}
          onClick={() => handleEvent("click")}
          style={component.props?.style}
        >
          {component.props?.text || "Button"}
        </button>
      );

    case "input":
      const value = state.get(component.id, component.props?.value || "");
      return (
        <input
          className={cn(
            inputVariants({
              variant: component.props?.variant as any,
              size: component.props?.size as any,
              error: component.props?.error,
              disabled: component.props?.disabled,
            })
          )}
          type={component.props?.type || "text"}
          placeholder={component.props?.placeholder}
          value={value}
          readOnly={component.props?.readonly}
          disabled={component.props?.disabled}
          onChange={(e) => state.set(component.id, e.target.value)}
          style={component.props?.style}
        />
      );

    case "text":
      const textVariant = component.props?.variant || "body";
      const Tag = textVariant === "h1" ? "h1" : textVariant === "h2" ? "h2" : textVariant === "h3" ? "h3" : "p";
      return (
        <Tag
          className={cn(
            textVariants({
              variant: textVariant as any,
              weight: component.props?.weight as any,
              color: component.props?.color as any,
              align: component.props?.align as any,
            })
          )}
          style={component.props?.style}
        >
          {component.props?.content}
        </Tag>
      );

    case "container":
      return (
        <div
          className={cn(
            containerVariants({
              layout: component.props?.layout as any,
              spacing: component.props?.spacing as any,
              padding: component.props?.padding as any,
              align: component.props?.align as any,
              justify: component.props?.justify as any,
            })
          )}
          style={{
            gap: component.props?.gap ? `${component.props.gap}px` : undefined,
            ...component.props?.style,
          }}
        >
          {component.children?.map((child: UIComponent, idx: number) => (
            <ComponentRenderer
              key={`${child.id}-${idx}`}
              component={child}
              state={state}
              executor={executor}
            />
          ))}
        </div>
      );

    case "grid":
      return (
        <div
          className={cn(
            gridVariants({
              columns: component.props?.columns as any,
              spacing: component.props?.spacing as any,
              responsive: component.props?.responsive,
            })
          )}
          style={{
            gridTemplateColumns: component.props?.columns
              ? `repeat(${component.props.columns}, 1fr)`
              : undefined,
            gap: component.props?.gap ? `${component.props.gap}px` : undefined,
            ...component.props?.style,
          }}
        >
          {component.children?.map((child: UIComponent, idx: number) => (
            <ComponentRenderer
              key={`${child.id}-${idx}`}
              component={child}
              state={state}
              executor={executor}
            />
          ))}
        </div>
      );

    default:
      return <div className="dynamic-unknown">Unknown component: {component.type}</div>;
  }
};

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
    return new ComponentState();
  });
  const [toolExecutor] = useState(() => {
    logger.info("Initializing tool executor", { component: "DynamicRenderer" });
    return new ToolExecutor(componentState);
  });

  // Track which component IDs we've already rendered to avoid re-animating
  // Use ref so it updates immediately without waiting for React re-render
  const renderedComponentIdsRef = React.useRef<Set<string>>(new Set());
  const [lastComponentCount, setLastComponentCount] = useState(0);

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
  const parsePartialJSON = useCallback((jsonStr: string) => {
    try {
      // Try to parse complete JSON first
      const parsed = JSON.parse(jsonStr);
      return { complete: true, data: parsed };
    } catch (e) {
      // If incomplete, try to extract what we can
      try {
        // Look for title
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
          const componentRegex = /\{(?:[^{}]|\{[^{}]*\})*\}/g;
          const matches = componentsStr.match(componentRegex);

          if (matches) {
            components = matches
              .map((m) => {
                try {
                  return JSON.parse(m);
                } catch {
                  return null;
                }
              })
              .filter((c): c is UIComponent => c !== null);
          }

          logger.debugThrottled("Partial JSON parsing progress", {
            component: "DynamicRenderer",
            componentsFound: components.length,
            charsProcessed: componentsStr.length,
          });
        }

        if (title || layout || components.length > 0) {
          return {
            complete: false,
            data: {
              title,
              layout,
              components,
              type: "app",
            },
          };
        }
      } catch (parseError) {
        // Silent fail - we'll try again with more data
      }
    }
    return { complete: false, data: null };
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

            // Try to parse partial JSON and update partial UI spec
            const currentPreview = useAppStore.getState().generationPreview + message.content;
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
          }
          break;

        case "ui_generated":
          logger.info("UI generated successfully", {
            component: "DynamicRenderer",
            title: message.ui_spec.title,
            appId: message.app_id,
            componentCount: message.ui_spec.components.length,
          });
          setUISpec(message.ui_spec, message.app_id);
          componentState.clear();
          componentState.set("app_id", message.app_id);
          toolExecutor.setAppId(message.app_id);

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
          break;

        case "error":
          logger.error("UI generation error", undefined, {
            component: "DynamicRenderer",
            message: message.message,
            previewLength: useAppStore.getState().generationPreview.length,
          });
          setError(message.message);
          setStreaming(false);
          break;

        default:
          // Ignore other message types
          break;
      }
    });

    return () => {
      logger.debug("Cleaning up WebSocket message listener", { component: "DynamicRenderer" });
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
  ]);

  // Cleanup: Execute on_unmount hooks when component unmounts or UI changes
  useEffect(() => {
    return () => {
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
  const { elementRef: wobbleRef, wobble } = useWobble<HTMLDivElement>();
  const buildContainerRef = useRef<HTMLDivElement>(null);
  const buildProgressRef = useRef<HTMLDivElement>(null);
  const renderedAppRef = useRef<HTMLDivElement>(null);
  const spinnerRef = useSpin<HTMLDivElement>(isLoading || isStreaming, { duration: 1.0 });
  const buildSpinnerRef = useSpin<HTMLDivElement>(true, { duration: 1.0 });
  const buildIconRef = useBuildPulse<HTMLDivElement>(isStreaming);
  const thoughtsListRef = useStaggerSlideUp<HTMLDivElement>('.thought-item', { stagger: 0.08, distance: 25 });
  const saveButtonRef = useGlowPulse<HTMLButtonElement>(false, { color: '99, 102, 241', intensity: 15 });
  const { elementRef: burstRef, burst } = useParticleBurst<HTMLDivElement>();
  
  // Animate error on appearance with wobble
  useEffect(() => {
    if (error && errorRef.elementRef.current) {
      errorRef.shake();
      setTimeout(() => wobble(), 300);
    }
  }, [error, wobble]);

  // Animate build container on appearance
  useEffect(() => {
    if (buildContainerRef.current && (isStreaming || partialUISpec)) {
      gsapAnimations.buildContainerAppear(buildContainerRef.current);
    }
  }, [isStreaming, partialUISpec]);

  // Animate rendered app on appearance with particle burst
  useEffect(() => {
    if (renderedAppRef.current && uiSpec) {
      gsapAnimations.appAppear(renderedAppRef.current);
      // Add particle burst after a short delay
      setTimeout(() => {
        if (burstRef.current) {
          burst({ count: 15, duration: 1.5 });
        }
      }, 400);
    }
  }, [uiSpec, burst]);

  // Animate progress bar width changes
  useEffect(() => {
    if (buildProgressRef.current) {
      gsapAnimations.setProps(buildProgressRef.current, { width: `${buildProgress || 20}%` });
    }
  }, [buildProgress]);

  // Animate components as they appear during build with enhanced animations
  const componentRefs = useRef<Map<string, HTMLDivElement>>(new Map());
  
  useEffect(() => {
    if (partialUISpec?.components) {
      partialUISpec.components.forEach((component, idx) => {
        const key = `${component.id}-${idx}`;
        const element = componentRefs.current.get(key);
        if (element && !element.dataset.animated) {
          // Use elastic bounce for more dynamic feel
          gsapAnimations.elasticBounceIn(element, { delay: idx * 0.06, from: 'bottom' });
          element.dataset.animated = 'true';
        }
      });
    }
  }, [partialUISpec?.components]);

  // Handle save app form submission
  const handleSaveApp = async (data: {
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
  };

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
        onClose={() => setShowSaveAppDialog(false)}
        onSave={handleSaveApp}
        isLoading={saveAppMutation.isPending}
      />
      
      <div className="renderer-canvas">
        {error && (
          <div ref={(el) => {
            if (el) {
              errorRef.elementRef.current = el;
              wobbleRef.current = el;
            }
          }} className="renderer-error">
            <strong>Error:</strong> {error}
          </div>
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
            <div ref={desktopMessageRef} className="desktop-message">
              <div ref={desktopIconRef} className="desktop-icon"><Sparkles size={48} /></div>
              <h2>Welcome to Griffin's AgentOS</h2>
              <p>Press ⌘K or click below to create something</p>
            </div>
          </div>
        )}

        {uiSpec && (
          <div ref={(el) => {
            if (el) {
              renderedAppRef.current = el;
              burstRef.current = el;
            }
          }} className="rendered-app" style={uiSpec.style}>
            <div className="app-header">
              <h2>{uiSpec.title}</h2>
              <button
                ref={saveButtonRef}
                className="save-app-btn"
                onClick={() => setShowSaveAppDialog(true)}
                onMouseEnter={() => {
                  if (saveButtonRef.current) {
                    gsapAnimations.buttonHoverIn(saveButtonRef.current);
                  }
                }}
                onMouseLeave={() => {
                  if (saveButtonRef.current) {
                    gsapAnimations.buttonHoverOut(saveButtonRef.current);
                  }
                }}
                title="Save this app to registry"
              >
                <Save size={16} style={{ marginRight: '6px', verticalAlign: 'middle' }} />
                Save
              </button>
            </div>
            <div className={`app-content app-layout-${uiSpec.layout}`}>
              {uiSpec.components.map((component, idx) => (
                <ComponentRenderer
                  key={`${component.id}-${idx}`}
                  component={component}
                  state={componentState}
                  executor={toolExecutor}
                />
              ))}
            </div>
          </div>
        )}
      </div>
    </div>
  );
};

export default DynamicRenderer;
