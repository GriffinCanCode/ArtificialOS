/**
 * Component Registry
 * Centralized registry for dynamic component renderers
 */

import type { ComponentRenderer } from "./types";
import { logger } from "../../../core/monitoring/core/logger";

// ============================================================================
// Registry Class
// ============================================================================

class ComponentRegistry {
  private renderers = new Map<string, ComponentRenderer>();

  /**
   * Register a component renderer
   */
  register(renderer: ComponentRenderer): void {
    if (this.renderers.has(renderer.type)) {
      logger.warn("Component renderer already registered, overwriting", {
        component: "ComponentRegistry",
        type: renderer.type,
      });
    }

    this.renderers.set(renderer.type, renderer);
    logger.debug("Component renderer registered", {
      component: "ComponentRegistry",
      type: renderer.type,
      category: renderer.category,
    });
  }

  /**
   * Register multiple renderers at once
   */
  registerAll(renderers: ComponentRenderer[]): void {
    renderers.forEach((renderer) => this.register(renderer));
  }

  /**
   * Get a renderer by component type
   */
  get(type: string): ComponentRenderer | undefined {
    return this.renderers.get(type);
  }

  /**
   * Check if a renderer is registered
   */
  has(type: string): boolean {
    return this.renderers.has(type);
  }

  /**
   * Get all registered component types
   */
  getTypes(): string[] {
    return Array.from(this.renderers.keys());
  }

  /**
   * Get all renderers in a category
   */
  getByCategory(category: ComponentRenderer["category"]): ComponentRenderer[] {
    return Array.from(this.renderers.values()).filter((r) => r.category === category);
  }

  /**
   * Unregister a component renderer
   */
  unregister(type: string): boolean {
    return this.renderers.delete(type);
  }

  /**
   * Clear all registered renderers
   */
  clear(): void {
    this.renderers.clear();
    logger.info("All component renderers cleared", {
      component: "ComponentRegistry",
    });
  }

  /**
   * Get registry statistics
   */
  getStats(): {
    total: number;
    byCategory: Record<string, number>;
  } {
    const byCategory: Record<string, number> = {};

    for (const renderer of this.renderers.values()) {
      const cat = renderer.category || "uncategorized";
      byCategory[cat] = (byCategory[cat] || 0) + 1;
    }

    return {
      total: this.renderers.size,
      byCategory,
    };
  }
}

// ============================================================================
// Singleton Instance
// ============================================================================

export const registry = new ComponentRegistry();
