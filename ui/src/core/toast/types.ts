/**
 * Toast Types
 * Type definitions for toast notifications
 */

import type { ExternalToast } from "sonner";

export type ToastType = "success" | "error" | "warning" | "info" | "loading" | "promise";

export type ToastPosition =
  | "top-left"
  | "top-center"
  | "top-right"
  | "bottom-left"
  | "bottom-center"
  | "bottom-right";

export interface ToastAction {
  label: string;
  onClick: () => void;
}

// Use Sonner's ExternalToast type directly for compatibility
export type ToastOptions = ExternalToast;

export interface PromiseToastOptions<T = any> {
  loading: string;
  success: string | ((data: T) => string);
  error: string | ((error: any) => string);
  duration?: number;
  position?: ToastPosition;
}

export interface ToastState {
  toasts: Map<string | number, ActiveToast>;
  position: ToastPosition;
}

export interface ActiveToast {
  id: string | number;
  type: ToastType;
  message: string;
  options?: ToastOptions;
  timestamp: number;
}
