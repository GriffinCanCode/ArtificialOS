/**
 * Toast Store
 * Centralized state management for toast notifications
 */

import { create } from "zustand";
import { devtools } from "zustand/middleware";
import type { ToastPosition, ActiveToast } from "./types";

interface ToastStoreState {
  toasts: Map<string | number, ActiveToast>;
  position: ToastPosition;
  setPosition: (position: ToastPosition) => void;
  addToast: (toast: ActiveToast) => void;
  removeToast: (id: string | number) => void;
  clearAll: () => void;
}

export const useToastStore = create<ToastStoreState>()(
  devtools(
    (set) => ({
      toasts: new Map(),
      position: "bottom-right",

      setPosition: (position) =>
        set({ position }, false, "toast/setPosition"),

      addToast: (toast) =>
        set(
          (state) => {
            const newToasts = new Map(state.toasts);
            newToasts.set(toast.id, toast);
            return { toasts: newToasts };
          },
          false,
          "toast/addToast"
        ),

      removeToast: (id) =>
        set(
          (state) => {
            const newToasts = new Map(state.toasts);
            newToasts.delete(id);
            return { toasts: newToasts };
          },
          false,
          "toast/removeToast"
        ),

      clearAll: () =>
        set({ toasts: new Map() }, false, "toast/clearAll"),
    }),
    { name: "ToastStore" }
  )
);

// Selectors for performance optimization
export const useToastPosition = () => useToastStore((state) => state.position);
export const useToastActions = () => useToastStore((state) => ({
  setPosition: state.setPosition,
  addToast: state.addToast,
  removeToast: state.removeToast,
  clearAll: state.clearAll,
}));
