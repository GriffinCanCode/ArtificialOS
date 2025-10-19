/**
 * Toast Tool Executor
 * Handles toast notifications using Sonner
 */

import { logger } from "../../../../../core/monitoring/core/logger";
import { ExecutorContext, BaseExecutor } from "../core/types";
import { toast } from "../../../../../core/toast";
import type { ToastOptions, PromiseToastOptions } from "../../../../../core/toast";

export class ToastExecutor implements BaseExecutor {
  private activeToasts: Map<string, string | number> = new Map();
  private context: ExecutorContext;

  constructor(context: ExecutorContext) {
    this.context = context;
  }

  execute(action: string, params: Record<string, any>): any {
    switch (action) {
      case "success":
        return this.showSuccess(params);

      case "error":
        return this.showError(params);

      case "warning":
        return this.showWarning(params);

      case "info":
        return this.showInfo(params);

      case "loading":
        return this.showLoading(params);

      case "show":
        return this.show(params);

      case "promise":
        this.showPromise(params);
        return true;

      case "undo":
        return this.showUndo(params);

      case "progress":
        return this.showProgress(params);

      case "dismiss":
        return this.dismissToast(params);

      case "dismiss_all":
        return this.dismissAll();

      case "update":
        return this.updateToast(params);

      default:
        logger.warn("Unknown toast action", { component: "ToastExecutor", action });
        return null;
    }
  }

  private showSuccess(params: Record<string, any>): string | number {
    const { message, key, ...options } = params;
    const id = toast.success(message, this.buildOptions(options));
    if (key) {
      this.activeToasts.set(key, id);
    }
    logger.debug("Success toast shown", { component: "ToastExecutor", key, id });
    return id;
  }

  private showError(params: Record<string, any>): string | number {
    const { message, key, ...options } = params;
    const id = toast.error(message, this.buildOptions(options));
    if (key) {
      this.activeToasts.set(key, id);
    }
    logger.debug("Error toast shown", { component: "ToastExecutor", key, id });
    return id;
  }

  private showWarning(params: Record<string, any>): string | number {
    const { message, key, ...options } = params;
    const id = toast.warning(message, this.buildOptions(options));
    if (key) {
      this.activeToasts.set(key, id);
    }
    logger.debug("Warning toast shown", { component: "ToastExecutor", key, id });
    return id;
  }

  private showInfo(params: Record<string, any>): string | number {
    const { message, key, ...options } = params;
    const id = toast.info(message, this.buildOptions(options));
    if (key) {
      this.activeToasts.set(key, id);
    }
    logger.debug("Info toast shown", { component: "ToastExecutor", key, id });
    return id;
  }

  private showLoading(params: Record<string, any>): string | number {
    const { message, key, ...options } = params;
    const id = toast.loading(message, this.buildOptions(options));
    if (key) {
      this.activeToasts.set(key, id);
    }
    logger.debug("Loading toast shown", { component: "ToastExecutor", key, id });
    return id;
  }

  private show(params: Record<string, any>): string | number {
    const { message, key, ...options } = params;
    const id = toast.show(message, this.buildOptions(options));
    if (key) {
      this.activeToasts.set(key, id);
    }
    logger.debug("Toast shown", { component: "ToastExecutor", key, id });
    return id;
  }

  private showPromise(params: Record<string, any>): void {
    const { promise, loading, success, error, key } = params;

    // Create promise messages
    const messages: PromiseToastOptions = {
      loading,
      success,
      error,
    };

    logger.debug("Promise toast started", { component: "ToastExecutor", key });

    // Execute the promise toast
    toast.promise(promise, messages);
    // Note: Promise toasts manage their own lifecycle, can't track by key
  }

  private showUndo(params: Record<string, any>): string | number {
    const { message, on_undo, key, ...options } = params;

    // Build undo handler
    const undoHandler = () => {
      if (on_undo) {
        logger.info("Undo action triggered", { component: "ToastExecutor", action: on_undo });
        // Execute the undo action by triggering an event through component state
        this.executeActionHandler(on_undo, "undo");
      }
    };

    const id = toast.undo(message, undoHandler, this.buildOptions(options));
    if (key) {
      this.activeToasts.set(key, id);
    }
    logger.debug("Undo toast shown", { component: "ToastExecutor", key, id });
    return id;
  }

  private showProgress(params: Record<string, any>): string | number {
    const { message, percent, key, ...options } = params;
    const id = toast.progress(message, percent, this.buildOptions(options));
    if (key) {
      this.activeToasts.set(key, id);
    }
    logger.debug("Progress toast shown", { component: "ToastExecutor", key, id, percent });
    return id;
  }

  private dismissToast(params: Record<string, any>): void {
    const { id, key } = params;

    if (key && this.activeToasts.has(key)) {
      const toastId = this.activeToasts.get(key)!;
      toast.dismiss(toastId);
      this.activeToasts.delete(key);
      logger.debug("Toast dismissed by key", { component: "ToastExecutor", key });
    } else if (id) {
      toast.dismiss(id);
      logger.debug("Toast dismissed by id", { component: "ToastExecutor", id });
    } else {
      toast.dismiss();
      logger.debug("Current toast dismissed", { component: "ToastExecutor" });
    }
  }

  private dismissAll(): void {
    toast.dismiss();
    this.activeToasts.clear();
    logger.debug("All toasts dismissed", { component: "ToastExecutor" });
  }

  private updateToast(params: Record<string, any>): void {
    const { key, id: directId, message, type = "info", ...options } = params;

    // Find the toast ID
    let toastId: string | number | undefined;
    if (key && this.activeToasts.has(key)) {
      toastId = this.activeToasts.get(key);
    } else if (directId) {
      toastId = directId;
    }

    if (!toastId) {
      logger.warn("Cannot update toast: no ID found", { component: "ToastExecutor", key });
      return;
    }

    // Dismiss old and show new (Sonner doesn't have direct update)
    toast.dismiss(toastId);

    const newId = (toast as any)[type](message, this.buildOptions(options));
    if (key) {
      this.activeToasts.set(key, newId);
    }

    logger.debug("Toast updated", { component: "ToastExecutor", key, oldId: toastId, newId });
  }

  private buildOptions(params: Record<string, any>): ToastOptions {
    const options: ToastOptions = {};

    if (params.duration !== undefined) options.duration = params.duration;
    if (params.position) options.position = params.position;
    if (params.dismissible !== undefined) options.dismissible = params.dismissible;
    if (params.description) options.description = params.description;
    if (params.className) options.className = params.className;

    // Handle action button
    if (params.action_label && params.action_handler) {
      options.action = {
        label: params.action_label,
        onClick: () => {
          logger.info("Toast action clicked", {
            component: "ToastExecutor",
            handler: params.action_handler,
          });
          // Execute the action handler
          this.executeActionHandler(params.action_handler, "action");
        },
      };
    }

    // Handle cancel button
    if (params.cancel_label && params.cancel_handler) {
      options.cancel = {
        label: params.cancel_label,
        onClick: () => {
          logger.info("Toast cancel clicked", {
            component: "ToastExecutor",
            handler: params.cancel_handler,
          });
          // Execute the cancel handler
          this.executeActionHandler(params.cancel_handler, "cancel");
        },
      };
    }

    return options;
  }

  /**
   * Execute an action handler by triggering it through component state events
   * This allows toast actions to integrate with the broader execution system
   */
  private executeActionHandler(handler: string | Record<string, any>, handlerType: string): void {
    try {
      // Handler can be a string (tool ID) or an object with tool ID and params
      if (typeof handler === "string") {
        // Simple tool ID - trigger event
        this.context.componentState.set(`toast.${handlerType}.trigger`, {
          toolId: handler,
          timestamp: Date.now(),
        });
        logger.info("Action handler triggered", {
          component: "ToastExecutor",
          toolId: handler,
          type: handlerType,
        });
      } else if (typeof handler === "object" && handler.toolId) {
        // Object with tool ID and parameters
        this.context.componentState.set(`toast.${handlerType}.trigger`, {
          toolId: handler.toolId,
          params: handler.params || {},
          timestamp: Date.now(),
        });
        logger.info("Action handler triggered with params", {
          component: "ToastExecutor",
          toolId: handler.toolId,
          params: handler.params,
          type: handlerType,
        });
      } else {
        logger.warn("Invalid action handler format", {
          component: "ToastExecutor",
          handler,
          type: handlerType,
        });
      }
    } catch (error) {
      logger.error("Failed to execute action handler", {
        component: "ToastExecutor",
        error,
        handler,
        type: handlerType,
      });
    }
  }
}
