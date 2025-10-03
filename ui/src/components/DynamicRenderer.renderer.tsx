/**
 * DynamicRenderer Component Renderers
 * Individual component rendering logic
 */

import React, { useState, useCallback } from "react";
import { UIComponent } from "../store/appStore";
import { ComponentState, SubscriptionOptions } from "./DynamicRenderer.state";
import { ToolExecutor } from "./DynamicRenderer.executor";
import { VIRTUAL_SCROLL_THRESHOLD, DEFAULT_ITEM_HEIGHT } from "./DynamicRenderer.constants";
import {
  buttonVariants,
  inputVariants,
  textVariants,
  containerVariants,
  gridVariants,
  cn,
} from "../utils/animation/componentVariants";

// Forward declaration to avoid circular dependency
// VirtualizedList will be imported dynamically
type VirtualizedListType = any;

// ============================================================================
// Component Renderer
// ============================================================================

interface RendererProps {
  component: UIComponent;
  state: ComponentState;
  executor: ToolExecutor;
}

export const ComponentRenderer: React.FC<RendererProps> = React.memo(({ component, state, executor }) => {
  const [, forceUpdate] = useState({});
  const [localState, setLocalState] = useState<any>(null);

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
      if (component.type === 'input' && component.props?.type === 'text') {
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

  const handleInputChange = useCallback((e: React.ChangeEvent<HTMLInputElement>) => {
    state.set(component.id, e.target.value);
  }, [component.id, state]);

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
          onChange={handleInputChange}
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
      const containerClassName = cn(
        containerVariants({
          layout: component.props?.layout as any,
          spacing: component.props?.spacing as any,
          padding: component.props?.padding as any,
          align: component.props?.align as any,
          justify: component.props?.justify as any,
        })
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
              layout={(component.props?.layout || 'vertical') as 'vertical' | 'horizontal' | 'grid'}
              className=""
              ComponentRenderer={ComponentRenderer}
            />
          </div>
        );
      }

      // Normal rendering for small lists
      return (
        <div className={containerClassName} style={containerStyle}>
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
}, (prevProps, nextProps) => {
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
    if (typeof prevVal !== 'object' || typeof nextVal !== 'object') {
      if (prevVal !== nextVal) return false;
    } else {
      // For complex objects, fall back to JSON comparison (cached)
      if (JSON.stringify(prevVal) !== JSON.stringify(nextVal)) return false;
    }
  }
  
  return true;
});

ComponentRenderer.displayName = 'ComponentRenderer';

