/**
 * Application State Store (Zustand)
 * Centralized state management with better performance and simpler API
 */

import { create } from 'zustand';
import { devtools } from 'zustand/middleware';
import { useMemo } from 'react';
import { logger } from '../utils/logger';

// ============================================================================
// Type Definitions
// ============================================================================

export interface Message {
  type: 'user' | 'assistant' | 'system';
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
  isLoading: boolean;
  error: string | null;
  generationThoughts: string[];
  appId: string | null;
  
  // Actions
  addMessage: (message: Message) => void;
  appendToLastMessage: (content: string) => void;
  addThought: (thought: ThoughtStep) => void;
  setUISpec: (uiSpec: UISpec, appId: string) => void;
  setLoading: (loading: boolean) => void;
  setError: (error: string | null) => void;
  addGenerationThought: (thought: string) => void;
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
  isLoading: false,
  error: null,
  generationThoughts: [],
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
        logger.debug('Adding message to store', {
          component: 'AppStore',
          messageType: message.type,
          contentLength: message.content.length
        });
        return set((state) => ({
          messages: [...state.messages, message],
        }), false, 'addMessage');
      },

      appendToLastMessage: (content) => {
        logger.verboseThrottled('Appending to last message', {
          component: 'AppStore',
          contentLength: content.length
        });
        return set((state) => {
          const lastMessage = state.messages[state.messages.length - 1];
          if (lastMessage && lastMessage.type === 'assistant') {
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
                type: 'assistant',
                content,
                timestamp: Date.now(),
              },
            ],
          };
        }, false, 'appendToLastMessage');
      },

      // Thought stream actions
      addThought: (thought) => {
        logger.verboseThrottled('Adding thought to stream', {
          component: 'AppStore',
          contentLength: thought.content.length
        });
        return set((state) => ({
          thoughts: [...state.thoughts, thought],
        }), false, 'addThought');
      },

      // Dynamic renderer actions
      setUISpec: (uiSpec, appId) => {
        logger.info('Setting UI spec', {
          component: 'AppStore',
          appId,
          uiType: uiSpec.type,
          componentCount: uiSpec.components.length
        });
        return set({
          uiSpec,
          appId,
          isLoading: false,
          error: null,
        }, false, 'setUISpec');
      },

      setLoading: (loading) => {
        logger.debugThrottled('Setting loading state', {
          component: 'AppStore',
          loading
        });
        return set((state) => ({
          isLoading: loading,
          error: loading ? null : state.error,
        }), false, 'setLoading');
      },

      setError: (error) => {
        if (error) {
          logger.error('Store error set', undefined, {
            component: 'AppStore',
            error
          });
        }
        return set({
          error,
          isLoading: false,
        }, false, 'setError');
      },

      addGenerationThought: (thought) => {
        logger.verboseThrottled('Adding generation thought', {
          component: 'AppStore',
          thoughtLength: thought.length
        });
        return set((state) => ({
          generationThoughts: [...state.generationThoughts, thought],
        }), false, 'addGenerationThought');
      },

      clearGenerationThoughts: () => {
        logger.debug('Clearing generation thoughts', { component: 'AppStore' });
        return set({
          generationThoughts: [],
        }, false, 'clearGenerationThoughts');
      },

      clearUISpec: () => {
        logger.info('Clearing UI spec', { component: 'AppStore' });
        return set({
          uiSpec: null,
          appId: null,
          error: null,
          generationThoughts: [],
        }, false, 'clearUISpec');
      },

      // Reset entire state
      resetState: () => {
        logger.info('Resetting store state', { component: 'AppStore' });
        return set(initialState, false, 'resetState');
      },
    }),
    { name: 'AppStore' }
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

// Individual loading state selectors to prevent unnecessary re-renders
export const useIsLoading = () => useAppStore((state) => state.isLoading);
export const useError = () => useAppStore((state) => state.error);
export const useGenerationThoughts = () => useAppStore((state) => state.generationThoughts);

// Get all actions (actions are stable, memoized to prevent re-renders)
export const useAppActions = () => {
  const addMessage = useAppStore((state) => state.addMessage);
  const appendToLastMessage = useAppStore((state) => state.appendToLastMessage);
  const addThought = useAppStore((state) => state.addThought);
  const setUISpec = useAppStore((state) => state.setUISpec);
  const setLoading = useAppStore((state) => state.setLoading);
  const setError = useAppStore((state) => state.setError);
  const addGenerationThought = useAppStore((state) => state.addGenerationThought);
  const clearGenerationThoughts = useAppStore((state) => state.clearGenerationThoughts);
  const clearUISpec = useAppStore((state) => state.clearUISpec);
  const resetState = useAppStore((state) => state.resetState);
  
  return useMemo(() => ({
    addMessage,
    appendToLastMessage,
    addThought,
    setUISpec,
    setLoading,
    setError,
    addGenerationThought,
    clearGenerationThoughts,
    clearUISpec,
    resetState,
  }), [addMessage, appendToLastMessage, addThought, setUISpec, setLoading, setError, addGenerationThought, clearGenerationThoughts, clearUISpec, resetState]);
};

