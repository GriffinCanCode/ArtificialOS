/**
 * DynamicRenderer Component Renderers
 * Individual component rendering logic
 */

import React, { useState, useCallback } from "react";
import { BlueprintComponent } from "../../store/appStore";
import { ComponentState, SubscriptionOptions } from "./DynamicRenderer.state";
import { ToolExecutor } from "./DynamicRenderer.executor";
import { VIRTUAL_SCROLL_THRESHOLD, DEFAULT_ITEM_HEIGHT } from "./DynamicRenderer.constants";
import {
  buttonVariants,
  inputVariants,
  textVariants,
  containerVariants,
  gridVariants,
  selectVariants,
  checkboxVariants,
  radioVariants,
  textareaVariants,
  imageVariants,
  sliderVariants,
  progressVariants,
  badgeVariants,
  dividerVariants,
  tabVariants,
  modalVariants,
  listVariants,
  canvasVariants,
  iframeVariants,
  videoVariants,
  audioVariants,
  cn,
} from "../../utils/animation/componentVariants";

// ============================================================================
// Component Renderer
// ============================================================================

interface RendererProps {
  component: BlueprintComponent;
  state: ComponentState;
  executor: ToolExecutor;
}

export const ComponentRenderer: React.FC<RendererProps> = React.memo(
  ({ component, state, executor }) => {
    const [, forceUpdate] = useState({});
    const [localState, setLocalState] = useState<any>(null);

    // Debounce timers for change events (e.g., text input, textarea)
    const changeDebounceTimerRef = React.useRef<number | null>(null);

    // Subscribe to state changes for this component with advanced options
    React.useEffect(() => {
      if (component.id) {
        // Initialize local state from component state manager
        setLocalState(state.get(component.id, component.props?.value));

        // Subscribe with options based on component type
        const subscriptionOptions: SubscriptionOptions = {
          immediate: true,
        };

        // Add debouncing for text inputs to reduce re-renders
        if (component.type === "input" && component.props?.type === "text") {
          subscriptionOptions.debounce = 100;
        }

        const unsubscribe = state.subscribe(
          component.id,
          (newValue, oldValue) => {
            // Only update if value actually changed (additional safety)
            if (newValue !== oldValue) {
              setLocalState(newValue);
              forceUpdate({});
            }
          },
          subscriptionOptions
        );

        return unsubscribe;
      }
    }, [component.id, component.type, state]);

    // Cleanup debounce timers on unmount
    React.useEffect(() => {
      return () => {
        if (changeDebounceTimerRef.current !== null) {
          window.clearTimeout(changeDebounceTimerRef.current);
        }
      };
    }, []);

  const handleEvent = useCallback(
    async (eventName: string, eventData?: any) => {
      const toolId = component.on_event?.[eventName];
      if (toolId) {
        // Extract params from event or component
        // Pass multiple parameter names for flexibility
        const params = {
          ...eventData,
          componentId: component.id,
          id: component.id,
          key: component.id,
          target: component.id,
          // Pass button text as multiple param names for generic tools
          value: component.props?.text || component.props?.value,
          text: component.props?.text,
          digit: component.props?.text,
        };
        try {
          await executor.execute(toolId, params);
        } catch (error) {
          console.error("Tool execution failed:", error);
        }
      }
    },
    [component, executor]
  );

    // Debounced event handler for change events (prevents firing on every keystroke)
    const handleDebouncedEvent = useCallback(
      (eventName: string, eventData?: any, debounceMs: number = 500) => {
        // Clear any existing timer
        if (changeDebounceTimerRef.current !== null) {
          window.clearTimeout(changeDebounceTimerRef.current);
        }

        // Set new timer
        changeDebounceTimerRef.current = window.setTimeout(() => {
          handleEvent(eventName, eventData);
          changeDebounceTimerRef.current = null;
        }, debounceMs);
      },
      [handleEvent]
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
        const value = localState ?? component.props?.value ?? "";
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
            onChange={(e) => {
              const newValue = e.target.value;
              // Update local state immediately for responsive typing
              state.set(component.id, newValue);
              // If there's a change event handler, debounce it
              if (component.on_event?.change) {
                handleDebouncedEvent("change", { value: newValue }, 500);
              }
            }}
            style={component.props?.style}
          />
        );

      case "text":
        const textVariant = component.props?.variant || "body";
        const Tag =
          textVariant === "h1"
            ? "h1"
            : textVariant === "h2"
              ? "h2"
              : textVariant === "h3"
                ? "h3"
                : "p";
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
        const containerClassName = cn(
          containerVariants({
            layout: component.props?.layout as any,
            spacing: component.props?.spacing as any,
            padding: component.props?.padding as any,
            align: component.props?.align as any,
            justify: component.props?.justify as any,
          }),
          // Add semantic role class if present (sidebar, main, editor, etc.)
          component.props?.role ? `semantic-${component.props.role}` : null
        );

        const containerStyle = {
          gap: component.props?.gap ? `${component.props.gap}px` : undefined,
          ...component.props?.style,
        };

        // Use virtual scrolling for large lists
        if (component.children && component.children.length >= VIRTUAL_SCROLL_THRESHOLD) {
          // Dynamic import to avoid circular dependency
          const VirtualizedList = require("./DynamicRenderer.virtual").VirtualizedList;
          return (
            <div className={containerClassName} style={containerStyle}>
              <VirtualizedList
                children={component.children}
                state={state}
                executor={executor}
                itemHeight={component.props?.itemHeight || DEFAULT_ITEM_HEIGHT}
                maxHeight={component.props?.maxHeight || 600}
                layout={
                  (component.props?.layout || "vertical") as "vertical" | "horizontal" | "grid"
                }
                className=""
                ComponentRenderer={ComponentRenderer}
              />
            </div>
          );
        }

        // Normal rendering for small lists
        return (
          <div 
            className={containerClassName} 
            style={containerStyle}
            data-role={component.props?.role || undefined}
          >
            {component.children?.map((child: BlueprintComponent) => (
              <ComponentRenderer
                key={child.id}
                component={child}
                state={state}
                executor={executor}
              />
            ))}
          </div>
        );

      case "grid":
        const gridClassName = cn(
          gridVariants({
            columns: component.props?.columns as any,
            spacing: component.props?.spacing as any,
            responsive: component.props?.responsive,
          })
        );

        const gridStyle = {
          gridTemplateColumns: component.props?.columns
            ? `repeat(${component.props.columns}, 1fr)`
            : undefined,
          gap: component.props?.gap ? `${component.props.gap}px` : undefined,
          ...component.props?.style,
        };

        // Use virtual scrolling for large grids
        if (component.children && component.children.length >= VIRTUAL_SCROLL_THRESHOLD) {
          // Dynamic import to avoid circular dependency
          const VirtualizedList = require("./DynamicRenderer.virtual").VirtualizedList;
          return (
            <div className={gridClassName} style={gridStyle}>
              <VirtualizedList
                children={component.children}
                state={state}
                executor={executor}
                itemHeight={component.props?.itemHeight || DEFAULT_ITEM_HEIGHT}
                maxHeight={component.props?.maxHeight || 600}
                layout="grid"
                columns={component.props?.columns || 3}
                className=""
                ComponentRenderer={ComponentRenderer}
              />
            </div>
          );
        }

        // Normal rendering for small grids
        return (
          <div className={gridClassName} style={gridStyle}>
            {component.children?.map((child: BlueprintComponent) => (
              <ComponentRenderer
                key={child.id}
                component={child}
                state={state}
                executor={executor}
              />
            ))}
          </div>
        );

      case "select":
        return (
          <select
            className={cn(
              selectVariants({
                variant: component.props?.variant as any,
                size: component.props?.size as any,
                error: component.props?.error,
                disabled: component.props?.disabled,
              })
            )}
            value={localState ?? component.props?.value ?? ""}
            disabled={component.props?.disabled}
            onChange={(e) => {
              const newValue = e.target.value;
              state.set(component.id, newValue);
              handleEvent("change", { value: newValue });
            }}
            style={component.props?.style}
          >
            {component.props?.options?.map((opt: any, idx: number) => (
              <option key={idx} value={opt.value || opt}>
                {opt.label || opt}
              </option>
            ))}
          </select>
        );

      case "checkbox":
        const checked = localState ?? component.props?.checked ?? false;
        return (
          <label className="dynamic-checkbox-wrapper" style={component.props?.style}>
            <input
              type="checkbox"
              className={cn(
                checkboxVariants({
                  size: component.props?.size as any,
                  variant: component.props?.variant as any,
                  disabled: component.props?.disabled,
                })
              )}
              checked={checked}
              disabled={component.props?.disabled}
              onChange={(e) => {
                const newValue = e.target.checked;
                state.set(component.id, newValue);
                handleEvent("change", { checked: newValue });
              }}
            />
            {component.props?.label && (
              <span className="checkbox-label">{component.props.label}</span>
            )}
          </label>
        );

      case "radio":
        return (
          <label className="dynamic-radio-wrapper" style={component.props?.style}>
            <input
              type="radio"
              className={cn(
                radioVariants({
                  size: component.props?.size as any,
                  disabled: component.props?.disabled,
                })
              )}
              name={component.props?.name}
              value={component.props?.value}
              checked={localState === component.props?.value}
              disabled={component.props?.disabled}
              onChange={(e) => {
                const newValue = e.target.value;
                state.set(component.id, newValue);
                handleEvent("change", { value: newValue });
              }}
            />
            {component.props?.label && <span className="radio-label">{component.props.label}</span>}
          </label>
        );

      case "textarea":
        return (
          <textarea
            className={cn(
              textareaVariants({
                variant: component.props?.variant as any,
                size: component.props?.size as any,
                error: component.props?.error,
                disabled: component.props?.disabled,
                resize: component.props?.resize as any,
              })
            )}
            placeholder={component.props?.placeholder}
            value={localState ?? component.props?.value ?? ""}
            disabled={component.props?.disabled}
            rows={component.props?.rows}
            onChange={(e) => {
              const newValue = e.target.value;
              // Update local state immediately for responsive typing
              state.set(component.id, newValue);
              // If there's a change event handler, debounce it to prevent lag
              if (component.on_event?.change) {
                handleDebouncedEvent("change", { value: newValue }, 500);
              }
            }}
            style={component.props?.style}
          />
        );

      case "image":
        return (
          <img
            className={cn(
              imageVariants({
                fit: component.props?.fit as any,
                rounded: component.props?.rounded as any,
              })
            )}
            src={component.props?.src}
            alt={component.props?.alt || ""}
            width={component.props?.width}
            height={component.props?.height}
            onClick={() => handleEvent("click")}
            style={component.props?.style}
          />
        );

      case "video":
        return (
          <video
            className={cn(
              videoVariants({
                fit: component.props?.fit as any,
                rounded: component.props?.rounded as any,
              })
            )}
            src={component.props?.src}
            controls={component.props?.controls !== false}
            autoPlay={component.props?.autoPlay}
            loop={component.props?.loop}
            muted={component.props?.muted}
            width={component.props?.width}
            height={component.props?.height}
            style={component.props?.style}
          />
        );

      case "audio":
        return (
          <audio
            className={cn(
              audioVariants({
                variant: component.props?.variant as any,
              })
            )}
            src={component.props?.src}
            controls={component.props?.controls !== false}
            autoPlay={component.props?.autoPlay}
            loop={component.props?.loop}
            style={component.props?.style}
          />
        );

      case "canvas":
        return (
          <canvas
            ref={(ref) => {
              if (ref && component.props?.onMount) {
                state.set(`${component.id}_canvas`, ref);
                handleEvent("mount", { canvas: ref });
              }
            }}
            className={cn(
              canvasVariants({
                bordered: component.props?.bordered,
              })
            )}
            width={component.props?.width || 300}
            height={component.props?.height || 150}
            style={component.props?.style}
          />
        );

      case "iframe":
        return (
          <iframe
            className={cn(
              iframeVariants({
                bordered: component.props?.bordered,
                rounded: component.props?.rounded,
              })
            )}
            src={component.props?.src}
            title={component.props?.title || "iframe"}
            width={component.props?.width || "100%"}
            height={component.props?.height || 400}
            style={component.props?.style}
            sandbox={component.props?.sandbox}
          />
        );

      case "slider":
        return (
          <input
            type="range"
            className={cn(
              sliderVariants({
                size: component.props?.size as any,
                variant: component.props?.variant as any,
                disabled: component.props?.disabled,
              })
            )}
            min={component.props?.min || 0}
            max={component.props?.max || 100}
            step={component.props?.step || 1}
            value={localState ?? component.props?.value ?? 0}
            disabled={component.props?.disabled}
            onChange={(e) => {
              const newValue = parseFloat(e.target.value);
              state.set(component.id, newValue);
              handleEvent("change", { value: newValue });
            }}
            style={component.props?.style}
          />
        );

      case "progress":
        const progressValue = component.props?.value || 0;
        const progressMax = component.props?.max || 100;
        const progressPercent = (progressValue / progressMax) * 100;
        return (
          <div
            className={cn(
              progressVariants({
                variant: component.props?.variant as any,
                size: component.props?.size as any,
              })
            )}
            style={component.props?.style}
          >
            <div className="progress-bar" style={{ width: `${progressPercent}%` }} />
          </div>
        );

      case "badge":
        return (
          <span
            className={cn(
              badgeVariants({
                variant: component.props?.variant as any,
                size: component.props?.size as any,
              })
            )}
            onClick={() => handleEvent("click")}
            style={component.props?.style}
          >
            {component.props?.text || component.props?.content}
          </span>
        );

      case "divider":
        return (
          <div
            className={cn(
              dividerVariants({
                orientation: component.props?.orientation as any,
                variant: component.props?.variant as any,
              })
            )}
            style={component.props?.style}
          />
        );

      case "tabs":
        const activeTab = localState ?? component.props?.defaultTab ?? component.children?.[0]?.id;
        return (
          <div className="dynamic-tabs" style={component.props?.style}>
            <div
              className={cn(
                tabVariants({
                  variant: component.props?.variant as any,
                  size: component.props?.size as any,
                }),
                "tabs-header"
              )}
            >
              {component.children?.map((tab) => (
                <button
                  key={tab.id}
                  className={activeTab === tab.id ? "tab-active" : "tab-inactive"}
                  onClick={() => {
                    state.set(component.id, tab.id);
                    handleEvent("tabChange", { tabId: tab.id });
                  }}
                >
                  {tab.props?.label || tab.id}
                </button>
              ))}
            </div>
            <div className="tabs-content">
              {component.children?.map((tab) =>
                activeTab === tab.id ? (
                  <div key={tab.id}>
                    {tab.children?.map((child) => (
                      <ComponentRenderer
                        key={child.id}
                        component={child}
                        state={state}
                        executor={executor}
                      />
                    ))}
                  </div>
                ) : null
              )}
            </div>
          </div>
        );

      case "modal":
        const isOpen = localState ?? component.props?.open ?? false;
        if (!isOpen) return null;
        return (
          <div
            className="modal-overlay"
            onClick={() => {
              state.set(component.id, false);
              handleEvent("close");
            }}
          >
            <div
              className={cn(
                modalVariants({
                  size: component.props?.size as any,
                  centered: component.props?.centered,
                })
              )}
              onClick={(e) => e.stopPropagation()}
              style={component.props?.style}
            >
              {component.props?.title && (
                <div className="modal-header">
                  <h3>{component.props.title}</h3>
                  <button
                    onClick={() => {
                      state.set(component.id, false);
                      handleEvent("close");
                    }}
                  >
                    ×
                  </button>
                </div>
              )}
              <div className="modal-body">
                {component.children?.map((child) => (
                  <ComponentRenderer
                    key={child.id}
                    component={child}
                    state={state}
                    executor={executor}
                  />
                ))}
              </div>
            </div>
          </div>
        );

      case "list":
        return (
          <div
            className={cn(
              listVariants({
                variant: component.props?.variant as any,
                spacing: component.props?.spacing as any,
              })
            )}
            style={component.props?.style}
          >
            {component.children?.map((child) => (
              <div key={child.id} className="list-item">
                <ComponentRenderer component={child} state={state} executor={executor} />
              </div>
            ))}
          </div>
        );

      case "card":
        return (
          <div className="dynamic-card" style={component.props?.style}>
            {component.props?.title && (
              <div className="card-header">
                <h4>{component.props.title}</h4>
              </div>
            )}
            <div className="card-body">
              {component.children?.map((child) => (
                <ComponentRenderer
                  key={child.id}
                  component={child}
                  state={state}
                  executor={executor}
                />
              ))}
            </div>
            {component.props?.footer && <div className="card-footer">{component.props.footer}</div>}
          </div>
        );

      case "app-shortcut":
        const { AppShortcut } = require("./AppShortcut");
        return (
          <AppShortcut
            id={component.props?.app_id || component.id}
            name={component.props?.name || "App"}
            icon={component.props?.icon || "📦"}
            description={component.props?.description}
            category={component.props?.category}
            variant={component.props?.variant as "icon" | "card" | "list"}
            onClick={(appId: string) => handleEvent("click", { app_id: appId })}
          />
        );

      default:
        // Log unknown component for debugging
        if (process.env.NODE_ENV === "development") {
          console.warn(`Unknown component type: "${component.type}"`, {
            id: component.id,
            props: component.props,
            component: component,
          });
        }
        // Return null to avoid rendering incomplete/invalid components
        // Valid components are filtered during parsing, so this should rarely be hit
        return null;
    }
  },
  (prevProps, nextProps) => {
    // Custom comparison for better memoization
    // Return true if props are equal (skip re-render)
    if (prevProps.component.id !== nextProps.component.id) return false;
    if (prevProps.component.type !== nextProps.component.type) return false;
    if (prevProps.component.children?.length !== nextProps.component.children?.length) return false;

    // Deep compare props - but only if they're different objects
    if (prevProps.component.props === nextProps.component.props) return true;

    // Compare props by key (more efficient than JSON.stringify for large objects)
    const prevKeys = Object.keys(prevProps.component.props || {});
    const nextKeys = Object.keys(nextProps.component.props || {});

    if (prevKeys.length !== nextKeys.length) return false;

    for (const key of prevKeys) {
      const prevVal = prevProps.component.props?.[key];
      const nextVal = nextProps.component.props?.[key];

      // For primitive values and simple objects
      if (typeof prevVal !== "object" || typeof nextVal !== "object") {
        if (prevVal !== nextVal) return false;
      } else {
        // For complex objects, fall back to JSON comparison (cached)
        if (JSON.stringify(prevVal) !== JSON.stringify(nextVal)) return false;
      }
    }

    return true;
  }
);

ComponentRenderer.displayName = "ComponentRenderer";
