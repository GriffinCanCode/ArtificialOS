/**
 * Clipboard Tool Executor
 * Handles copy/paste operations with backend service integration
 */

import { logger } from "../../../../../core/monitoring/core/logger";
import { ExecutorContext, AsyncExecutor } from "../core/types";

export class ClipboardExecutor implements AsyncExecutor {
  private serviceExecutor: any;

  constructor(_context: ExecutorContext, serviceExecutor?: any) {
    // Context not currently used but kept for interface compatibility
    this.serviceExecutor = serviceExecutor;
  }

  async execute(action: string, params: Record<string, any>): Promise<any> {
    const service = this.serviceExecutor;

    switch (action) {
      case "copy":
        return this.handleCopy(params, service);

      case "paste":
        return this.handlePaste(params, service);

      case "history":
        return this.handleHistory(params, service);

      case "clear":
        return this.handleClear(params, service);

      case "subscribe":
        return this.handleSubscribe(params, service);

      case "unsubscribe":
        return this.handleUnsubscribe(service);

      case "stats":
        return this.handleStats(service);

      default:
        logger.warn("Unknown clipboard action", { action, component: "ClipboardExecutor" });
        return null;
    }
  }

  private async handleCopy(params: Record<string, any>, service: any): Promise<any> {
    const text = params.text || params.data;
    if (!text) {
      logger.error("Copy requires text parameter", null, { component: "ClipboardExecutor" });
      return false;
    }

    try {
      // Use browser clipboard first for immediate feedback
      if (navigator.clipboard) {
        await navigator.clipboard.writeText(text);
      }

      // Sync with backend if available
      if (service) {
        const result = await service.execute("clipboard.copy", {
          data: text,
          format: params.format || "text",
          global: params.global || false,
        });

        if (result?.success) {
          logger.debug("Copied to clipboard", {
            entryId: result.data?.entry_id,
            component: "ClipboardExecutor",
          });
          return result.data?.entry_id || true;
        }
      }

      return true;
    } catch (error) {
      logger.error("Failed to copy to clipboard", error as Error, {
        component: "ClipboardExecutor",
      });
      return false;
    }
  }

  private async handlePaste(params: Record<string, any>, service: any): Promise<any> {
    try {
      // Try backend first
      if (service) {
        const result = await service.execute("clipboard.paste", {
          global: params.global || false,
        });

        if (result?.success && result.data) {
          logger.debug("Pasted from clipboard", {
            entryId: result.data.id,
            component: "ClipboardExecutor",
          });
          return result.data;
        }
      }

      // Fallback to browser clipboard
      if (navigator.clipboard) {
        const text = await navigator.clipboard.readText();
        logger.debug("Pasted from browser clipboard", { component: "ClipboardExecutor" });
        return { data: { type: "Text", data: text } };
      }

      return null;
    } catch (error) {
      logger.error("Failed to paste from clipboard", error as Error, {
        component: "ClipboardExecutor",
      });
      return null;
    }
  }

  private async handleHistory(params: Record<string, any>, service: any): Promise<any> {
    if (!service) return [];

    try {
      const result = await service.execute("clipboard.history", {
        limit: params.limit,
        global: params.global || false,
      });

      if (result?.success && result.data?.entries) {
        logger.debug("Fetched clipboard history", {
          count: result.data.entries.length,
          component: "ClipboardExecutor",
        });
        return result.data.entries;
      }

      return [];
    } catch (error) {
      logger.error("Failed to fetch clipboard history", error as Error, {
        component: "ClipboardExecutor",
      });
      return [];
    }
  }

  private async handleClear(params: Record<string, any>, service: any): Promise<any> {
    try {
      if (service) {
        await service.execute("clipboard.clear", {
          global: params.global || false,
        });
      }

      logger.debug("Cleared clipboard", { component: "ClipboardExecutor" });
      return true;
    } catch (error) {
      logger.error("Failed to clear clipboard", error as Error, {
        component: "ClipboardExecutor",
      });
      return false;
    }
  }

  private async handleSubscribe(params: Record<string, any>, service: any): Promise<any> {
    if (!service) return false;

    try {
      await service.execute("clipboard.subscribe", {
        formats: params.formats || [],
      });

      logger.debug("Subscribed to clipboard", { component: "ClipboardExecutor" });
      return true;
    } catch (error) {
      logger.error("Failed to subscribe to clipboard", error as Error, {
        component: "ClipboardExecutor",
      });
      return false;
    }
  }

  private async handleUnsubscribe(service: any): Promise<any> {
    if (!service) return false;

    try {
      await service.execute("clipboard.unsubscribe", {});
      logger.debug("Unsubscribed from clipboard", { component: "ClipboardExecutor" });
      return true;
    } catch (error) {
      logger.error("Failed to unsubscribe from clipboard", error as Error, {
        component: "ClipboardExecutor",
      });
      return false;
    }
  }

  private async handleStats(service: any): Promise<any> {
    if (!service) return null;

    try {
      const result = await service.execute("clipboard.stats", {});

      if (result?.success && result.data) {
        logger.debug("Fetched clipboard stats", {
          stats: result.data,
          component: "ClipboardExecutor",
        });
        return result.data;
      }

      return null;
    } catch (error) {
      logger.error("Failed to fetch clipboard stats", error as Error, {
        component: "ClipboardExecutor",
      });
      return null;
    }
  }
}
