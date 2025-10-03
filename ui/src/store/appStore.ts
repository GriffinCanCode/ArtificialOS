/**
 * Application State Store (Zustand)
 * Centralized state management with better performance and simpler API
 */

import { create } from "zustand";
import { devtools } from "zustand/middleware";
import { useMemo } from "react";
import { logger } from "../utils/logger";

// ============================================================================
// Type Definitions
// ============================================================================

export interface Message {
  type: "user" | "assistant" | "system";
  content: string;
  timestamp: number;
}

export interface ThoughtStep {
  content: string;
  timestamp: number;
}

export interface UIComponent {
  type: string;
  id: string;
  props: Record<string, any>;
  children?: UIComponent[];
  on_event?: Record<string, string>;
}

export interface UISpec {
  type: string;
  title: string;
  layout: string;
  components: UIComponent[];
  style?: Record<string, any>;
  services?: string[];
  service_bindings?: Record<string, string>;
  lifecycle_hooks?: Record<string, string[]>;
}

// ============================================================================
// Store Interface
// ============================================================================

interface AppState {
  // Chat state
  messages: Message[];

  // Thought stream state
  thoughts: ThoughtStep[];

  // Dynamic renderer state
  uiSpec: UISpec | null;
  partialUISpec: Partial<UISpec> | null; // Accumulates during streaming
  isLoading: boolean;
  isStreaming: boolean; // True when actively building UI
  error: string | null;
  generationThoughts: string[];
  generationPreview: string; // Accumulates streaming tokens for real-time display
  buildProgress: number; // 0-100 representing build progress
  appId: string | null;

  // Actions
  addMessage: (message: Message) => void;
  appendToLastMessage: (content: string) => void;
  addThought: (thought: ThoughtStep) => void;
  setUISpec: (uiSpec: UISpec, appId: string) => void;
  setPartialUISpec: (partial: Partial<UISpec>) => void;
  addComponentToPartial: (component: UIComponent) => void;
  setLoading: (loading: boolean) => void;
  setStreaming: (streaming: boolean) => void;
  setBuildProgress: (progress: number) => void;
  setError: (error: string | null) => void;
  addGenerationThought: (thought: string) => void;
  appendGenerationPreview: (content: string) => void;
  clearGenerationPreview: () => void;
  clearGenerationThoughts: () => void;
  clearUISpec: () => void;
  resetState: () => void;
}

// ============================================================================
// Initial State
// ============================================================================

const initialState = {
  messages: [],
  thoughts: [],
  uiSpec: null,
  partialUISpec: null,
  isLoading: false,
  isStreaming: false,
  error: null,
  generationThoughts: [],
  generationPreview: "",
  buildProgress: 0,
  appId: null,
};

// ============================================================================
// Store
// ============================================================================

export const useAppStore = create<AppState>()(
  devtools(
    (set) => ({
      ...initialState,

      // Chat actions
      addMessage: (message) => {
        logger.debug("Adding message to store", {
          component: "AppStore",
          messageType: message.type,
          contentLength: message.content.length,
        });
        return set(
          (state) => ({
            messages: [...state.messages, message],
          }),
          false,
          "addMessage"
        );
      },

      appendToLastMessage: (content) => {
        logger.verboseThrottled("Appending to last message", {
          component: "AppStore",
          contentLength: content.length,
        });
        return set(
          (state) => {
            const lastMessage = state.messages[state.messages.length - 1];
            if (lastMessage && lastMessage.type === "assistant") {
              return {
                messages: [
                  ...state.messages.slice(0, -1),
                  { ...lastMessage, content: lastMessage.content + content },
                ],
              };
            }
            // If no assistant message exists, create one
            return {
              messages: [
                ...state.messages,
                {
                  type: "assistant",
                  content,
                  timestamp: Date.now(),
                },
              ],
            };
          },
          false,
          "appendToLastMessage"
        );
      },

      // Thought stream actions
      addThought: (thought) => {
        logger.verboseThrottled("Adding thought to stream", {
          component: "AppStore",
          contentLength: thought.content.length,
        });
        return set(
          (state) => ({
            thoughts: [...state.thoughts, thought],
          }),
          false,
          "addThought"
        );
      },

      // Dynamic renderer actions
      setUISpec: (uiSpec, appId) => {
        logger.info("Setting UI spec", {
          component: "AppStore",
          appId,
          uiType: uiSpec.type,
          componentCount: uiSpec.components.length,
        });
        return set(
          {
            uiSpec,
            appId,
            isLoading: false,
            error: null,
          },
          false,
          "setUISpec"
        );
      },

      setLoading: (loading) => {
        logger.debugThrottled("Setting loading state", {
          component: "AppStore",
          loading,
        });
        return set(
          (state) => ({
            isLoading: loading,
            error: loading ? null : state.error,
          }),
          false,
          "setLoading"
        );
      },

      setError: (error) => {
        if (error) {
          logger.error("Store error set", undefined, {
            component: "AppStore",
            error,
          });
        }
        return set(
          {
            error,
            isLoading: false,
          },
          false,
          "setError"
        );
      },

      addGenerationThought: (thought) => {
        logger.verboseThrottled("Adding generation thought", {
          component: "AppStore",
          thoughtLength: thought.length,
        });
        return set(
          (state) => ({
            generationThoughts: [...state.generationThoughts, thought],
          }),
          false,
          "addGenerationThought"
        );
      },

      appendGenerationPreview: (content) => {
        return set(
          (state) => {
            console.log("🎯 STORE: Appending to preview", {
              newContent: content,
              newLength: content.length,
              currentPreviewLength: state.generationPreview.length
            });
            const newPreview = state.generationPreview + content;
            console.log("✅ STORE: New preview length:", newPreview.length);
            return { generationPreview: newPreview };
          },
          false,
          "appendGenerationPreview"
        );
      },

      clearGenerationPreview: () => {
        logger.debug("Clearing generation preview", { component: "AppStore" });
        return set(
          {
            generationPreview: "",
          },
          false,
          "clearGenerationPreview"
        );
      },

      clearGenerationThoughts: () => {
        logger.debug("Clearing generation thoughts", { component: "AppStore" });
        return set(
          {
            generationThoughts: [],
          },
          false,
          "clearGenerationThoughts"
        );
      },

      clearUISpec: () => {
        logger.info("Clearing UI spec", { component: "AppStore" });
        return set(
          {
            uiSpec: null,
            partialUISpec: null,
            appId: null,
            error: null,
            generationThoughts: [],
            generationPreview: "",
            buildProgress: 0,
            isStreaming: false,
          },
          false,
          "clearUISpec"
        );
      },

      // Partial UI spec actions for streaming
      setPartialUISpec: (partial) => {
        logger.debug("Setting partial UI spec", {
          component: "AppStore",
          hasTitle: !!partial.title,
          componentCount: partial.components?.length || 0,
        });
        return set(
          { partialUISpec: partial },
          false,
          "setPartialUISpec"
        );
      },

      addComponentToPartial: (component) => {
        logger.debug("Adding component to partial UI", {
          component: "AppStore",
          componentType: component.type,
          componentId: component.id,
        });
        return set(
          (state) => {
            const current = state.partialUISpec || { components: [] };
            const components = [...(current.components || []), component];
            return {
              partialUISpec: {
                ...current,
                components,
              },
            };
          },
          false,
          "addComponentToPartial"
        );
      },

      setStreaming: (streaming) => {
        logger.debug("Setting streaming state", {
          component: "AppStore",
          streaming,
        });
        return set(
          { isStreaming: streaming },
          false,
          "setStreaming"
        );
      },

      setBuildProgress: (progress) => {
        return set(
          { buildProgress: progress },
          false,
          "setBuildProgress"
        );
      },

      // Reset entire state
      resetState: () => {
        logger.info("Resetting store state", { component: "AppStore" });
        return set(initialState, false, "resetState");
      },
    }),
    { name: "AppStore" }
  )
);

// ============================================================================
// Selectors (for better performance)
// ============================================================================

// Only re-render when messages change
export const useMessages = () => useAppStore((state) => state.messages);

// Only re-render when thoughts change
export const useThoughts = () => useAppStore((state) => state.thoughts);

// Only re-render when UI spec changes
export const useUISpec = () => useAppStore((state) => state.uiSpec);
export const usePartialUISpec = () => useAppStore((state) => state.partialUISpec);

// Individual loading state selectors to prevent unnecessary re-renders
export const useIsLoading = () => useAppStore((state) => state.isLoading);
export const useIsStreaming = () => useAppStore((state) => state.isStreaming);
export const useBuildProgress = () => useAppStore((state) => state.buildProgress);
export const useError = () => useAppStore((state) => state.error);
export const useGenerationThoughts = () => useAppStore((state) => state.generationThoughts);
export const useGenerationPreview = () => useAppStore((state) => state.generationPreview);

// Get all actions (actions are stable, memoized to prevent re-renders)
export const useAppActions = () => {
  const addMessage = useAppStore((state) => state.addMessage);
  const appendToLastMessage = useAppStore((state) => state.appendToLastMessage);
  const addThought = useAppStore((state) => state.addThought);
  const setUISpec = useAppStore((state) => state.setUISpec);
  const setLoading = useAppStore((state) => state.setLoading);
  const setError = useAppStore((state) => state.setError);
  const addGenerationThought = useAppStore((state) => state.addGenerationThought);
  const appendGenerationPreview = useAppStore((state) => state.appendGenerationPreview);
  const clearGenerationPreview = useAppStore((state) => state.clearGenerationPreview);
  const clearGenerationThoughts = useAppStore((state) => state.clearGenerationThoughts);
  const clearUISpec = useAppStore((state) => state.clearUISpec);
  const resetState = useAppStore((state) => state.resetState);

  return useMemo(
    () => ({
      addMessage,
      appendToLastMessage,
      addThought,
      setUISpec,
      setLoading,
      setError,
      addGenerationThought,
      appendGenerationPreview,
      clearGenerationPreview,
      clearGenerationThoughts,
      clearUISpec,
      resetState,
    }),
    [
      addMessage,
      appendToLastMessage,
      addThought,
      setUISpec,
      setLoading,
      setError,
      addGenerationThought,
      appendGenerationPreview,
      clearGenerationPreview,
      clearGenerationThoughts,
      clearUISpec,
      resetState,
    ]
  );
};
