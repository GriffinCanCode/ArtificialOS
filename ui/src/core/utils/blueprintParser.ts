/**
 * Blueprint DSL Parser for Frontend
 * Parses Blueprint JSON format into UISpec during streaming
 */

import { BlueprintComponent } from "../store/appStore";

interface BlueprintDSLObject {
  [key: string]: any;
}

// Valid component types that can be rendered
// Includes layout shortcuts (row, col) that may not have been converted yet
const VALID_COMPONENT_TYPES = new Set([
  "button",
  "input",
  "text",
  "container",
  "grid",
  "select",
  "checkbox",
  "radio",
  "textarea",
  "image",
  "video",
  "audio",
  "canvas",
  "iframe",
  "slider",
  "progress",
  "badge",
  "divider",
  "tabs",
  "modal",
  "list",
  "card",
  "app-shortcut",
  // Layout shortcuts (converted to container by backend, but may arrive unconverted during streaming)
  "row",
  "col",
  "sidebar",
  "main",
  "editor",
  "header",
  "footer",
  "content",
  "section",
]);

/**
 * Parse Blueprint component - explicit format only
 * Format: {type: "button", id: "save", props: {...}, on_event: {...}, children: [...]}
 */
export function parseBlueprintComponent(
  comp: BlueprintDSLObject | string,
  idCounter: { value: number }
): BlueprintComponent | null {
  // Simple string becomes text component
  if (typeof comp === "string") {
    const id = `text-${idCounter.value++}`;
    return {
      type: "text",
      id,
      props: { content: comp },
    };
  }

  // Component object - explicit format only
  if (typeof comp === "object" && comp !== null) {
    // Must have type field - skip nested objects like props, on_event, etc.
    if (!comp.type || typeof comp.type !== "string") {
      // Silently skip - these are likely nested property objects during streaming
      console.log(
        `[PARSE] Filtering out object with no valid type field:`,
        Object.keys(comp).slice(0, 5)
      );
      return null;
    }

    // Validate that type is a recognized component type
    if (!VALID_COMPONENT_TYPES.has(comp.type)) {
      // Silently skip - invalid component type (likely nested object property)
      console.log(`[PARSE] Filtering out invalid component type: "${comp.type}"`);
      return null;
    }

    // Handle layout shortcuts: row -> container (horizontal), col -> container (vertical)
    let compType = comp.type;
    let props = comp.props || {};

    if (compType === "row") {
      compType = "container";
      props = { ...props, layout: props.layout || "horizontal" };
    } else if (compType === "col") {
      compType = "container";
      props = { ...props, layout: props.layout || "vertical" };
    } else if (
      ["sidebar", "main", "editor", "header", "footer", "content", "section"].includes(compType)
    ) {
      props = { ...props, role: compType, layout: props.layout || "vertical" };
      compType = "container";
    }

    const compId = comp.id || `${compType}-${idCounter.value++}`;
    const onEvent = comp.on_event;
    let children: BlueprintComponent[] | undefined;

    // Recursively parse children
    if (comp.children && Array.isArray(comp.children)) {
      console.log(`[PARSE] Component ${compId} has ${comp.children.length} children`);
      children = comp.children
        .map((child) => parseBlueprintComponent(child, idCounter))
        .filter((c): c is BlueprintComponent => c !== null);
      console.log(`[PARSE] Component ${compId} parsed ${children.length} children`);
    }

    const result: BlueprintComponent = {
      type: compType,
      id: compId,
      props,
    };

    if (onEvent && Object.keys(onEvent).length > 0) {
      result.on_event = onEvent;
    }

    if (children && children.length > 0) {
      result.children = children;
    }

    return result;
  }

  return null;
}

/**
 * Parse partial Blueprint JSON during streaming
 * Returns extracted components even if JSON is incomplete
 */
export function parsePartialBlueprint(jsonStr: string): {
  complete: boolean;
  data: {
    title?: string;
    layout?: string;
    components: BlueprintComponent[];
  } | null;
} {
  try {
    // Try to parse complete JSON first
    const parsed = JSON.parse(jsonStr);
    const uiSection = parsed.ui || {};

    const idCounter = { value: 0 };
    const components = (uiSection.components || [])
      .map((c: any) => parseBlueprintComponent(c, idCounter))
      .filter((c: BlueprintComponent | null): c is BlueprintComponent => c !== null);

    return {
      complete: true,
      data: {
        title: uiSection.title,
        layout: uiSection.layout || "vertical",
        components,
      },
    };
  } catch (e) {
    // If incomplete, try to extract what we can
    try {
      // Look for title
      const titleMatch = jsonStr.match(/"title"\s*:\s*"([^"]+)"/);
      const title = titleMatch ? titleMatch[1] : undefined;

      // Look for layout
      const layoutMatch = jsonStr.match(/"layout"\s*:\s*"([^"]+)"/);
      const layout = layoutMatch ? layoutMatch[1] : undefined;

      // Try to extract components array from "components": [ ... ]
      const componentsMatch = jsonStr.match(/"components"\s*:\s*\[([\s\S]*)/);
      let components: BlueprintComponent[] = [];

      if (componentsMatch) {
        let componentsStr = componentsMatch[1];
        console.log("[PARSE] Found components section, length:", componentsStr.length);

        // Remove trailing incomplete data after last complete object
        const lastBraceIdx = componentsStr.lastIndexOf("}");
        if (lastBraceIdx !== -1) {
          componentsStr = componentsStr.substring(0, lastBraceIdx + 1);
        }

        // Extract complete component objects (handles nested objects in props/on_event/children)
        // Strategy: Match objects that contain "type": to ensure we only get components, not nested props
        const componentRegex = /\{[^{}]*"type"\s*:\s*"[^"]*"[^{}]*(?:\{[^{}]*\}[^{}]*)*\}/g;
        let matches = componentsStr.match(componentRegex);

        // Fallback to simple matching if type-based regex fails (for malformed JSON during streaming)
        if (!matches || matches.length === 0) {
          // Try to extract any complete JSON objects, we'll filter by type presence later
          const simpleRegex = /\{(?:[^{}]|\{[^{}]*\})*\}/g;
          matches = componentsStr.match(simpleRegex);
        }

        console.log("[PARSE] Regex found", matches?.length || 0, "candidate objects");

        if (matches) {
          const idCounter = { value: 0 };
          components = matches
            .map((m, idx) => {
              try {
                const parsed = JSON.parse(m);

                // CRITICAL: Only process objects with a valid component "type" field
                // Skip nested objects like props, on_event, etc. and objects with invalid types
                if (!parsed.type || typeof parsed.type !== "string") {
                  console.log(
                    `[PARSE] Skipping object ${idx + 1} - no type field:`,
                    Object.keys(parsed).slice(0, 3).join(", ")
                  );
                  return null;
                }

                // Validate that type is a recognized component type
                if (!VALID_COMPONENT_TYPES.has(parsed.type)) {
                  console.log(
                    `[PARSE] Skipping object ${idx + 1} - invalid component type: "${parsed.type}"`
                  );
                  return null;
                }

                console.log(
                  `[PARSE] Component ${idx + 1}:`,
                  parsed.type,
                  parsed.id,
                  "children:",
                  parsed.children?.length || 0
                );

                // Already in UISpec format, just validate it has required fields
                if (parsed.type && parsed.id) {
                  return parsed as BlueprintComponent;
                }
                // Try Blueprint format fallback
                return parseBlueprintComponent(parsed, idCounter);
              } catch (err) {
                console.log(`[PARSE] Failed to parse object ${idx + 1}:`, (err as Error).message);
                return null;
              }
            })
            .filter((c): c is BlueprintComponent => c !== null);

          console.log("[PARSE] Successfully extracted", components.length, "BlueprintComponents");
        }
      } else {
        console.log("[PARSE] No components section found in:", jsonStr.substring(0, 200));
      }

      if (title || layout || components.length > 0) {
        return {
          complete: false,
          data: {
            title,
            layout: layout || "vertical",
            components,
          },
        };
      }
    } catch (parseError) {
      // Silent fail
    }
  }

  return { complete: false, data: null };
}

/**
 * Filter out invalid components from a blueprint
 * Used to clean up blueprints that bypass the parser (e.g., from backend)
 * Also converts layout shortcuts (row/col) to proper container types
 */
export function filterValidComponents(components: any[]): BlueprintComponent[] {
  console.log(`[FILTER] Starting filter with ${components?.length || 0} components`);

  if (!Array.isArray(components)) {
    console.log(`[FILTER] Input is not an array:`, typeof components);
    return [];
  }

  const filtered = components
    .filter((comp, idx) => {
      console.log(
        `[FILTER] Processing component ${idx + 1}/${components.length}:`,
        comp?.type,
        comp?.id
      );

      // Must be an object with a type field
      if (!comp || typeof comp !== "object" || !comp.type || typeof comp.type !== "string") {
        console.log(
          `[FILTER] ❌ Removing invalid component (no type):`,
          Object.keys(comp || {}).slice(0, 5)
        );
        return false;
      }

      // Must be a valid component type (including layout shortcuts)
      if (!VALID_COMPONENT_TYPES.has(comp.type)) {
        console.log(`[FILTER] ❌ Removing invalid component type: "${comp.type}"`);
        return false;
      }

      console.log(`[FILTER] ✅ Keeping component: ${comp.type} (${comp.id})`);
      return true;
    })
    .map((comp) => {
      // Apply layout shortcut conversions: row -> container (horizontal), col -> container (vertical)
      let compType = comp.type;
      let props = comp.props || {};

      if (compType === "row") {
        console.log(`[FILTER] Converting row -> container (horizontal):`, comp.id);
        compType = "container";
        props = { ...props, layout: props.layout || "horizontal" };
      } else if (compType === "col") {
        console.log(`[FILTER] Converting col -> container (vertical):`, comp.id);
        compType = "container";
        props = { ...props, layout: props.layout || "vertical" };
      } else if (
        ["sidebar", "main", "editor", "header", "footer", "content", "section"].includes(compType)
      ) {
        console.log(`[FILTER] Converting ${compType} -> container (with role):`, comp.id);
        props = { ...props, role: compType, layout: props.layout || "vertical" };
        compType = "container";
      }

      // Build filtered component with converted type
      const filtered: BlueprintComponent = {
        type: compType,
        id: comp.id,
        props,
      };

      if (comp.on_event && typeof comp.on_event === "object") {
        filtered.on_event = comp.on_event;
      }

      if (comp.children && Array.isArray(comp.children)) {
        console.log(`[FILTER] Filtering ${comp.children.length} children of ${comp.id}`);
        const validChildren = filterValidComponents(comp.children);
        if (validChildren.length > 0) {
          filtered.children = validChildren;
        }
      }

      return filtered;
    });

  console.log(
    `[FILTER] Completed: ${filtered.length} valid components out of ${components.length}`
  );
  return filtered;
}
