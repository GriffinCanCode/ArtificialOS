/**
 * Builder View Component
 * Displays build progress UI for apps being generated
 * This is a pure presentation component that reads from the app store
 */

import React, { useRef, useEffect } from "react";
import { Construction, MessageCircle } from "lucide-react";
import {
  usePartialBlueprint,
  useBuildProgress,
  useGenerationThoughts,
  BlueprintComponent,
} from "../../../core/store/appStore";
import { ComponentState } from "../state/state";
import { ToolExecutor } from "../execution/executor";
import { ComponentRenderer } from "./renderer";
import { VirtualizedList } from "./virtual";
import { VIRTUAL_SCROLL_THRESHOLD, DEFAULT_ITEM_HEIGHT } from "../core/constants";
import { useSpin, useBuildPulse, useStaggerSlideUp } from "../../../ui/hooks/useGSAP";
import * as gsapAnimations from "../../../core/utils/animation/gsapAnimations";
import { logger } from "../../../core/monitoring/core/logger";
import "../core/styles.css";

interface BuilderViewProps {
  state: ComponentState;
  executor: ToolExecutor;
}

// Filter out malformed components (missing type, invalid type, nested objects)
const isValidComponent = (c: any): c is BlueprintComponent => {
  return (
    c &&
    typeof c === "object" &&
    c.type &&
    typeof c.type === "string" &&
    c.type !== "undefined" &&
    c.type !== "props" && // Filter out props objects that got misidentified
    c.type !== "on_event" && // Filter out event handler objects
    c.type !== "children" && // Filter out children arrays
    c.id // Must have an id
  );
};

export const BuilderView: React.FC<BuilderViewProps> = ({ state, executor }) => {
  const partialBlueprint = usePartialBlueprint();
  const buildProgress = useBuildProgress();
  const generationThoughts = useGenerationThoughts();

  // Animation refs
  const buildContainerRef = useRef<HTMLDivElement>(null);
  const buildProgressRef = useRef<HTMLDivElement>(null);
  const buildSpinnerRef = useSpin<HTMLDivElement>(true, { duration: 1.0 });
  const buildIconRef = useBuildPulse<HTMLDivElement>(true);
  const thoughtsListRef = useStaggerSlideUp<HTMLDivElement>(".thought-item", {
    stagger: 0.08,
    distance: 25,
  });

  // Track animated components and refs
  const componentRefs = useRef<Map<string, HTMLDivElement>>(new Map());
  const animatedComponentIds = useRef<Set<string>>(new Set());
  const previousComponentsRef = useRef<BlueprintComponent[]>([]);
  const buildContainerAnimatedRef = useRef(false);

  // Memoize valid components
  const validPartialComponents = React.useMemo(() => {
    return partialBlueprint?.components?.filter(isValidComponent) || [];
  }, [partialBlueprint?.components]);

  // Animate build container on appearance - only once
  useEffect(() => {
    if (buildContainerRef.current && partialBlueprint && !buildContainerAnimatedRef.current) {
      gsapAnimations.buildContainerAppear(buildContainerRef.current);
      buildContainerAnimatedRef.current = true;
    }
  }, [partialBlueprint]);

  // Animate progress bar width changes
  useEffect(() => {
    if (buildProgressRef.current) {
      gsapAnimations.setProps(buildProgressRef.current, { width: `${buildProgress || 20}%` });
    }
  }, [buildProgress]);

  // Function to animate a single component
  const animateComponent = React.useCallback((component: BlueprintComponent, idx: number) => {
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
      element.dataset.animated = "true";
      animatedComponentIds.current.add(componentId);
    }
  }, []);

  // Check for new components and animate them
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
  }, [validPartialComponents.length, animateComponent]);

  return (
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
              state={state}
              executor={executor}
              itemHeight={DEFAULT_ITEM_HEIGHT}
              maxHeight={600}
              layout={
                (partialBlueprint?.layout || "vertical") as "vertical" | "horizontal" | "grid"
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
                    component: "BuilderView",
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
                      state={state}
                      executor={executor}
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
  );
};
