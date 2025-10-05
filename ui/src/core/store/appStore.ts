/**
 * Application State Store (Zustand)
 * Centralized state management with better performance and simpler API
 */

import { create } from "zustand";
import { devtools } from "zustand/middleware";
import { useShallow } from "zustand/react/shallow";
import { logger } from "../utils/monitoring/logger";

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

export interface BlueprintComponent {
  type: string;
  id: string;
  props: Record<string, any>;
  children?: BlueprintComponent[];
  on_event?: Record<string, string>;
}

export interface Blueprint {
  type: string;
  title: string;
  layout: string;
  components: BlueprintComponent[];
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
  blueprint: Blueprint | null;
  partialBlueprint: Partial<Blueprint> | null; // Accumulates during streaming
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
  setBlueprint: (blueprint: Blueprint, appId: string) => void;
  setPartialBlueprint: (partial: Partial<Blueprint>) => void;
  addComponentToPartial: (component: BlueprintComponent) => void;
  setLoading: (loading: boolean) => void;
  setStreaming: (streaming: boolean) => void;
  setBuildProgress: (progress: number) => void;
  setError: (error: string | null) => void;
  addGenerationThought: (thought: string) => void;
  appendGenerationPreview: (content: string) => void;
  clearGenerationPreview: () => void;
  clearGenerationThoughts: () => void;
  clearBlueprint: () => void;
  resetState: () => void;
}

// ============================================================================
// Initial State
// ============================================================================

const initialState = {
  messages: [],
  thoughts: [],
  blueprint: null,
  partialBlueprint: null,
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
      setBlueprint: (blueprint, appId) => {
        logger.info("Setting UI spec", {
          component: "AppStore",
          appId,
          uiType: blueprint.type,
          componentCount: blueprint.components.length,
        });
        return set(
          {
            blueprint,
            appId,
            isLoading: false,
            error: null,
          },
          false,
          "setBlueprint"
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
            logger.debugThrottled("Appending to generation preview", {
              component: "AppStore",
              newContentLength: content.length,
              currentPreviewLength: state.generationPreview.length,
            });
            const newPreview = state.generationPreview + content;
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

      clearBlueprint: () => {
        logger.info("Clearing UI spec", { component: "AppStore" });
        return set(
          {
            blueprint: null,
            partialBlueprint: null,
            appId: null,
            error: null,
            generationThoughts: [],
            generationPreview: "",
            buildProgress: 0,
            isStreaming: false,
          },
          false,
          "clearBlueprint"
        );
      },

      // Partial UI spec actions for streaming
      setPartialBlueprint: (partial) => {
        logger.debug("Setting partial UI spec", {
          component: "AppStore",
          hasTitle: !!partial.title,
          componentCount: partial.components?.length || 0,
        });
        return set({ partialBlueprint: partial }, false, "setPartialBlueprint");
      },

      addComponentToPartial: (component) => {
        logger.debug("Adding component to partial UI", {
          component: "AppStore",
          componentType: component.type,
          componentId: component.id,
        });
        return set(
          (state) => {
            const current = state.partialBlueprint || { components: [] };
            const components = [...(current.components || []), component];
            return {
              partialBlueprint: {
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
        return set({ isStreaming: streaming }, false, "setStreaming");
      },

      setBuildProgress: (progress) => {
        return set({ buildProgress: progress }, false, "setBuildProgress");
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
export const useBlueprint = () => useAppStore((state) => state.blueprint);
export const usePartialBlueprint = () => useAppStore((state) => state.partialBlueprint);

// Individual loading state selectors to prevent unnecessary re-renders
export const useIsLoading = () => useAppStore((state) => state.isLoading);
export const useIsStreaming = () => useAppStore((state) => state.isStreaming);
export const useBuildProgress = () => useAppStore((state) => state.buildProgress);
export const useError = () => useAppStore((state) => state.error);
export const useGenerationThoughts = () => useAppStore((state) => state.generationThoughts);
export const useGenerationPreview = () => useAppStore((state) => state.generationPreview);

// Stable selector function to prevent infinite loops
const appActionsSelector = (state: AppState) => ({
  addMessage: state.addMessage,
  appendToLastMessage: state.appendToLastMessage,
  addThought: state.addThought,
  setBlueprint: state.setBlueprint,
  setPartialBlueprint: state.setPartialBlueprint,
  addComponentToPartial: state.addComponentToPartial,
  setLoading: state.setLoading,
  setStreaming: state.setStreaming,
  setBuildProgress: state.setBuildProgress,
  setError: state.setError,
  addGenerationThought: state.addGenerationThought,
  appendGenerationPreview: state.appendGenerationPreview,
  clearGenerationPreview: state.clearGenerationPreview,
  clearGenerationThoughts: state.clearGenerationThoughts,
  clearBlueprint: state.clearBlueprint,
  resetState: state.resetState,
});

// Get all actions (actions are stable, using single selector with shallow comparison to prevent re-renders)
export const useAppActions = () => {
  return useAppStore(useShallow(appActionsSelector));
};
