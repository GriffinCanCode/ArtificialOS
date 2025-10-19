/**
 * Toast Utilities
 * Helper functions for toast notifications
 */

import { toast as sonnerToast } from "sonner";
import type { ToastOptions, PromiseToastOptions } from "./types";
import { logger } from "../monitoring/core/logger";

/**
 * Show a success toast
 */
export function success(message: string, options?: ToastOptions): string | number {
  logger.debug("Toast: success", { component: "Toast", message });
  return sonnerToast.success(message, options);
}

/**
 * Show an error toast
 */
export function error(message: string, options?: ToastOptions): string | number {
  logger.debug("Toast: error", { component: "Toast", message });
  return sonnerToast.error(message, options);
}

/**
 * Show a warning toast
 */
export function warning(message: string, options?: ToastOptions): string | number {
  logger.debug("Toast: warning", { component: "Toast", message });
  return sonnerToast.warning(message, options);
}

/**
 * Show an info toast
 */
export function info(message: string, options?: ToastOptions): string | number {
  logger.debug("Toast: info", { component: "Toast", message });
  return sonnerToast.info(message, options);
}

/**
 * Show a loading toast
 */
export function loading(message: string, options?: ToastOptions): string | number {
  logger.debug("Toast: loading", { component: "Toast", message });
  return sonnerToast.loading(message, options);
}

/**
 * Show a generic toast
 */
export function show(message: string, options?: ToastOptions): string | number {
  logger.debug("Toast: show", { component: "Toast", message });
  return sonnerToast(message, options);
}

/**
 * Show a promise toast that updates based on promise state
 */
export function promise<T>(promise: Promise<T>, messages: PromiseToastOptions<T>): void {
  logger.debug("Toast: promise", { component: "Toast" });
  sonnerToast.promise(promise, messages);
}

/**
 * Dismiss a specific toast
 */
export function dismiss(id?: string | number): void {
  if (id) {
    logger.debug("Toast: dismiss", { component: "Toast", id });
    sonnerToast.dismiss(id);
  } else {
    logger.debug("Toast: dismiss all", { component: "Toast" });
    sonnerToast.dismiss();
  }
}

/**
 * Custom toast for undo actions
 */
export function undo(message: string, onUndo: () => void, options?: ToastOptions): string | number {
  logger.debug("Toast: undo", { component: "Toast", message });
  return sonnerToast(message, {
    ...options,
    action: {
      label: "Undo",
      onClick: () => {
        onUndo();
        logger.info("Undo action triggered", { component: "Toast" });
      },
    },
  });
}

/**
 * Custom toast for progress updates
 */
export function progress(
  message: string,
  percent: number,
  options?: ToastOptions
): string | number {
  const progressMessage = `${message} (${percent}%)`;
  logger.debug("Toast: progress", { component: "Toast", message: progressMessage, percent });
  return sonnerToast.loading(progressMessage, options);
}

/**
 * Toast namespace for organized API
 */
export const toast = {
  success,
  error,
  warning,
  info,
  loading,
  show,
  promise,
  dismiss,
  undo,
  progress,
};
