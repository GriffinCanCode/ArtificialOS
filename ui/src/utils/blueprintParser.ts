/**
 * Blueprint DSL Parser for Frontend
 * Parses Blueprint JSON format into UISpec during streaming
 */

import { BlueprintComponent } from "../store/appStore";

interface BlueprintDSLObject {
  [key: string]: any;
}

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
    // Must have type and id
    if (!comp.type) {
      console.warn("[PARSE] Component missing type field:", comp);
      return null;
    }

    const compType = comp.type;
    const compId = comp.id || `${compType}-${idCounter.value++}`;
    const props = comp.props || {};
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
        const lastBraceIdx = componentsStr.lastIndexOf('}');
        if (lastBraceIdx !== -1) {
          componentsStr = componentsStr.substring(0, lastBraceIdx + 1);
        }
        
        // Extract complete component objects (handles nested objects in props/on_event/children)
        // This regex matches: { ... } including nested objects like {props: {nested: {}}}
        const componentRegex = /\{(?:[^{}]|\{[^{}]*\})*\}/g;
        const matches = componentsStr.match(componentRegex);
        
        console.log("[PARSE] Regex found", matches?.length || 0, "component objects");
        
        if (matches) {
          const idCounter = { value: 0 };
          components = matches
            .map((m, idx) => {
              try {
                const parsed = JSON.parse(m);
                console.log(`[PARSE] Component ${idx + 1}:`, parsed.type, parsed.id, "children:", parsed.children?.length || 0);
                // Already in UISpec format, just validate it has required fields
                if (parsed.type && parsed.id) {
                  return parsed as BlueprintComponent;
                }
                // Try Blueprint format fallback
                return parseBlueprintComponent(parsed, idCounter);
              } catch (err) {
                console.log(`[PARSE] Failed to parse component ${idx + 1}:`, (err as Error).message);
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
