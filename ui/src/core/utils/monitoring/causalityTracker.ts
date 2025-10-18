/**
 * Causality Chain Tracking System
 *
 * Tracks cause-and-effect relationships across complex UI interactions.
 * Essential for debugging multi-step operations, async flows, and understanding
 * how user actions propagate through the system.
 *
 * Features:
 * - Automatic causality chain creation and propagation
 * - Cross-boundary tracking (components, async operations, WebSocket messages)
 * - Timeline reconstruction for debugging
 * - Performance impact tracking
 * - Automatic cleanup to prevent memory leaks
 */

import type { LogContext } from './logger';

// ============================================================================
// Types
// ============================================================================

export interface CausalityChain {
  /** Unique chain ID */
  id: string;

  /** Root cause that started this chain */
  rootCause: CausalEvent;

  /** All events in this chain */
  events: CausalEvent[];

  /** Chain metadata */
  metadata: {
    startTime: number;
    endTime?: number;
    totalDuration?: number;
    eventCount: number;
    maxDepth: number;
    tags: string[];
  };
}

export interface CausalEvent {
  /** Unique event ID */
  id: string;

  /** Chain this event belongs to */
  chainId: string;

  /** Parent event that caused this one (null for root) */
  parentId?: string;

  /** Child events caused by this one */
  childIds: string[];

  /** Event type (user_action, api_call, state_change, etc.) */
  type: CausalEventType;

  /** Event description */
  description: string;

  /** Event timing */
  timing: {
    startTime: number;
    endTime?: number;
    duration?: number;
  };

  /** Context information */
  context: {
    component?: string;
    windowId?: string;
    appId?: string;
    userId?: string;
    [key: string]: unknown;
  };

  /** Additional metadata */
  metadata: {
    depth: number;           // Depth in the causal tree
    severity: 'low' | 'medium' | 'high'; // Importance level
    tags: string[];          // Custom tags for filtering
    error?: Error;           // Error if this event failed
    data?: unknown;         // Additional event data
  };
}

export type CausalEventType =
  | 'user_action'      // User clicks, types, etc.
  | 'api_call'         // HTTP/WebSocket API calls
  | 'state_change'     // State updates (Zustand, React state)
  | 'render'           // Component renders
  | 'async_operation'  // Async tasks (timers, promises)
  | 'system_event'     // System-level events
  | 'error'           // Error occurrences
  | 'performance'     // Performance-related events
  | 'navigation'      // Route/page changes
  | 'websocket'       // WebSocket messages
  | 'custom';         // Custom event types

export interface CausalityOptions {
  /** Maximum chain length before auto-termination */
  maxChainLength: number;

  /** Maximum time before chain auto-expires (ms) */
  maxChainDuration: number;

  /** Maximum chains to keep in memory */
  maxChainsInMemory: number;

  /** Whether to automatically track React state changes */
  autoTrackStateChanges: boolean;

  /** Whether to automatically track API calls */
  autoTrackApiCalls: boolean;

  /** Whether to automatically track user interactions */
  autoTrackUserActions: boolean;

  /** Cleanup interval (ms) */
  cleanupIntervalMs: number;
}

// ============================================================================
// Causality Tracker Implementation
// ============================================================================

export class CausalityTracker {
  private chains = new Map<string, CausalityChain>();
  private events = new Map<string, CausalEvent>();
  private activeChainId: string | null = null;
  private cleanupTimer: number | null = null;
  private options: CausalityOptions;

  constructor(options: Partial<CausalityOptions> = {}) {
    this.options = {
      maxChainLength: 100,
      maxChainDuration: 30000, // 30 seconds
      maxChainsInMemory: 50,
      autoTrackStateChanges: true,
      autoTrackApiCalls: true,
      autoTrackUserActions: true,
      cleanupIntervalMs: 60000, // 1 minute
      ...options,
    };

    this.startCleanupTimer();
  }

  /**
   * Start a new causality chain
   */
  startChain(
    type: CausalEventType,
    description: string,
    context: Partial<CausalEvent['context']> = {},
    metadata: Partial<CausalEvent['metadata']> = {}
  ): string {
    const chainId = this.generateId('chain');
    const eventId = this.generateId('event');
    const now = performance.now();

    const rootEvent: CausalEvent = {
      id: eventId,
      chainId,
      childIds: [],
      type,
      description,
      timing: {
        startTime: now,
      },
      context,
      metadata: {
        depth: 0,
        severity: 'medium',
        tags: [],
        ...metadata,
      },
    };

    const chain: CausalityChain = {
      id: chainId,
      rootCause: rootEvent,
      events: [rootEvent],
      metadata: {
        startTime: now,
        eventCount: 1,
        maxDepth: 0,
        tags: metadata.tags || [],
      },
    };

    this.chains.set(chainId, chain);
    this.events.set(eventId, rootEvent);
    this.activeChainId = chainId;

    this.cleanupIfNeeded();
    return chainId;
  }

  /**
   * Add an event to the current or specified chain
   */
  addEvent(
    type: CausalEventType,
    description: string,
    context: Partial<CausalEvent['context']> = {},
    metadata: Partial<CausalEvent['metadata']> = {},
    chainId?: string,
    parentEventId?: string
  ): string {
    const targetChainId = chainId || this.activeChainId;
    if (!targetChainId) {
      // Auto-start a chain if none exists
      return this.startChain(type, description, context, metadata);
    }

    const chain = this.chains.get(targetChainId);
    if (!chain) {
      throw new Error(`Chain ${targetChainId} not found`);
    }

    // Find parent event (use last event if not specified)
    let parentEvent: CausalEvent | undefined;
    if (parentEventId) {
      parentEvent = this.events.get(parentEventId);
    } else {
      // Use the most recent event in the chain as parent
      parentEvent = chain.events[chain.events.length - 1];
    }

    const eventId = this.generateId('event');
    const now = performance.now();

    const event: CausalEvent = {
      id: eventId,
      chainId: targetChainId,
      parentId: parentEvent?.id,
      childIds: [],
      type,
      description,
      timing: {
        startTime: now,
      },
      context,
      metadata: {
        depth: (parentEvent?.metadata.depth || 0) + 1,
        severity: 'medium',
        tags: [],
        ...metadata,
      },
    };

    // Update parent's children
    if (parentEvent) {
      parentEvent.childIds.push(eventId);
    }

    // Update chain
    chain.events.push(event);
    chain.metadata.eventCount = chain.events.length;
    chain.metadata.maxDepth = Math.max(chain.metadata.maxDepth, event.metadata.depth);

    // Store event
    this.events.set(eventId, event);

    // Check limits
    if (chain.events.length >= this.options.maxChainLength) {
      this.endChain(targetChainId);
    }

    return eventId;
  }

  /**
   * Mark an event as completed
   */
  completeEvent(eventId: string, error?: Error): void {
    const event = this.events.get(eventId);
    if (!event) return;

    const now = performance.now();
    event.timing.endTime = now;
    event.timing.duration = now - event.timing.startTime;

    if (error) {
      event.metadata.error = error;
      event.metadata.severity = 'high';
    }
  }

  /**
   * End a causality chain
   */
  endChain(chainId: string): void {
    const chain = this.chains.get(chainId);
    if (!chain) return;

    const now = performance.now();
    chain.metadata.endTime = now;
    chain.metadata.totalDuration = now - chain.metadata.startTime;

    if (this.activeChainId === chainId) {
      this.activeChainId = null;
    }
  }

  /**
   * Get the current active chain ID
   */
  getCurrentChainId(): string | null {
    return this.activeChainId;
  }

  /**
   * Get a specific chain
   */
  getChain(chainId: string): CausalityChain | undefined {
    return this.chains.get(chainId);
  }

  /**
   * Get all chains matching criteria
   */
  getChains(filter?: {
    type?: CausalEventType;
    timeRange?: { start: number; end: number };
    tags?: string[];
    hasError?: boolean;
  }): CausalityChain[] {
    const chains = Array.from(this.chains.values());

    if (!filter) return chains;

    return chains.filter(chain => {
      if (filter.type && chain.rootCause.type !== filter.type) {
        return false;
      }

      if (filter.timeRange) {
        const { start, end } = filter.timeRange;
        if (chain.metadata.startTime < start || chain.metadata.startTime > end) {
          return false;
        }
      }

      if (filter.tags) {
        const chainTags = new Set(chain.metadata.tags);
        if (!filter.tags.some(tag => chainTags.has(tag))) {
          return false;
        }
      }

      if (filter.hasError !== undefined) {
        const hasError = chain.events.some(event => event.metadata.error);
        if (hasError !== filter.hasError) {
          return false;
        }
      }

      return true;
    });
  }

  /**
   * Get causality context for logging
   */
  getCausalityContext(): LogContext {
    if (!this.activeChainId) {
      return {};
    }

    const chain = this.chains.get(this.activeChainId);
    if (!chain) {
      return {};
    }

    const lastEvent = chain.events[chain.events.length - 1];

    return {
      causalityChainId: chain.id,
      causalityEventId: lastEvent.id,
      causalityDepth: lastEvent.metadata.depth,
      causalityRootCause: chain.rootCause.description,
      causalityEventCount: chain.metadata.eventCount,
      causalityChainDuration: Date.now() - chain.metadata.startTime,
    };
  }

  /**
   * Create a timeline view of a chain
   */
  getChainTimeline(chainId: string): Array<{
    event: CausalEvent;
    depth: number;
    duration?: number;
    children: string[];
  }> {
    const chain = this.chains.get(chainId);
    if (!chain) return [];

    // Sort events by start time
    const sortedEvents = [...chain.events].sort(
      (a, b) => a.timing.startTime - b.timing.startTime
    );

    return sortedEvents.map(event => ({
      event,
      depth: event.metadata.depth,
      duration: event.timing.duration,
      children: event.childIds,
    }));
  }

  /**
   * Get performance impact of a chain
   */
  getChainPerformanceImpact(chainId: string): {
    totalDuration: number;
    slowestEvent: CausalEvent | null;
    errorCount: number;
    averageEventDuration: number;
  } {
    const chain = this.chains.get(chainId);
    if (!chain) {
      return { totalDuration: 0, slowestEvent: null, errorCount: 0, averageEventDuration: 0 };
    }

    const eventsWithDuration = chain.events.filter(e => e.timing.duration !== undefined);
    const totalDuration = chain.metadata.totalDuration || 0;
    const errorCount = chain.events.filter(e => e.metadata.error).length;

    let slowestEvent: CausalEvent | null = null;
    let maxDuration = 0;

    eventsWithDuration.forEach(event => {
      if (event.timing.duration! > maxDuration) {
        maxDuration = event.timing.duration!;
        slowestEvent = event;
      }
    });

    const averageEventDuration = eventsWithDuration.length > 0
      ? eventsWithDuration.reduce((sum, e) => sum + e.timing.duration!, 0) / eventsWithDuration.length
      : 0;

    return {
      totalDuration,
      slowestEvent,
      errorCount,
      averageEventDuration,
    };
  }

  /**
   * Export chain data for debugging
   */
  exportChain(chainId: string): {
    chain: CausalityChain;
    timeline: ReturnType<CausalityTracker['getChainTimeline']>;
    performance: ReturnType<CausalityTracker['getChainPerformanceImpact']>;
  } | null {
    const chain = this.chains.get(chainId);
    if (!chain) return null;

    return {
      chain,
      timeline: this.getChainTimeline(chainId),
      performance: this.getChainPerformanceImpact(chainId),
    };
  }

  /**
   * Clear all chains and events
   */
  clear(): void {
    this.chains.clear();
    this.events.clear();
    this.activeChainId = null;
  }

  /**
   * Cleanup expired chains and events
   */
  cleanup(): void {
    const now = performance.now();
    const expiredChains: string[] = [];

    for (const [chainId, chain] of this.chains) {
      const age = now - chain.metadata.startTime;
      if (age > this.options.maxChainDuration) {
        expiredChains.push(chainId);
      }
    }

    // Remove expired chains
    expiredChains.forEach(chainId => {
      const chain = this.chains.get(chainId);
      if (chain) {
        // Remove all events in the chain
        chain.events.forEach(event => {
          this.events.delete(event.id);
        });
        this.chains.delete(chainId);
      }
    });

    // Limit memory usage
    if (this.chains.size > this.options.maxChainsInMemory) {
      const sortedChains = Array.from(this.chains.values())
        .sort((a, b) => a.metadata.startTime - b.metadata.startTime);

      const chainsToRemove = sortedChains.slice(0, this.chains.size - this.options.maxChainsInMemory);

      chainsToRemove.forEach(chain => {
        chain.events.forEach(event => {
          this.events.delete(event.id);
        });
        this.chains.delete(chain.id);
      });
    }
  }

  /**
   * Destroy the tracker and cleanup resources
   */
  destroy(): void {
    if (this.cleanupTimer) {
      clearInterval(this.cleanupTimer);
      this.cleanupTimer = null;
    }
    this.clear();
  }

  // ============================================================================
  // Private Methods
  // ============================================================================

  private generateId(prefix: string): string {
    return `${prefix}_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`;
  }

  private cleanupIfNeeded(): void {
    // Simple cleanup trigger
    if (this.chains.size > this.options.maxChainsInMemory * 1.2) {
      this.cleanup();
    }
  }

  private startCleanupTimer(): void {
    this.cleanupTimer = window.setInterval(() => {
      this.cleanup();
    }, this.options.cleanupIntervalMs);
  }
}

// ============================================================================
// Singleton Instance & Helpers
// ============================================================================

export const causalityTracker = new CausalityTracker();

/**
 * Start a new causality chain
 */
export function startCausalChain(
  type: CausalEventType,
  description: string,
  context?: Partial<CausalEvent['context']>,
  metadata?: Partial<CausalEvent['metadata']>
): string {
  return causalityTracker.startChain(type, description, context, metadata);
}

/**
 * Add an event to the current chain
 */
export function addCausalEvent(
  type: CausalEventType,
  description: string,
  context?: Partial<CausalEvent['context']>,
  metadata?: Partial<CausalEvent['metadata']>
): string {
  return causalityTracker.addEvent(type, description, context, metadata);
}

/**
 * Complete an event
 */
export function completeCausalEvent(eventId: string, error?: Error): void {
  causalityTracker.completeEvent(eventId, error);
}

/**
 * End the current chain
 */
export function endCurrentChain(): void {
  const currentChainId = causalityTracker.getCurrentChainId();
  if (currentChainId) {
    causalityTracker.endChain(currentChainId);
  }
}

/**
 * Get causality context for logging
 */
export function getCausalityLogContext(): LogContext {
  return causalityTracker.getCausalityContext();
}

/**
 * Wrap an async function with causality tracking
 */
export function withCausality<T extends (...args: unknown[]) => Promise<unknown>>(
  fn: T,
  type: CausalEventType,
  description: string,
  context?: Partial<CausalEvent['context']>
): T {
  return (async (...args: unknown[]) => {
    const eventId = addCausalEvent(type, description, context);

    try {
      const result = await fn(...args);
      completeCausalEvent(eventId);
      return result;
    } catch (error) {
      completeCausalEvent(eventId, error as Error);
      throw error;
    }
  }) as T;
}

/**
 * React hook for causality tracking in components
 * Note: This requires React to be available in the environment
 */
export function useCausality(componentName: string) {
  // We'll create a simple version without React.useCallback for now
  // In a real React environment, you'd import React and use useCallback
  const addEvent = (
    type: CausalEventType,
    description: string,
    metadata?: Partial<CausalEvent['metadata']>
  ) => {
    return addCausalEvent(type, description, { component: componentName }, metadata);
  };

  const completeEvent = (eventId: string, error?: Error) => {
    completeCausalEvent(eventId, error);
  };

  const trackUserAction = (action: string) => {
    return addEvent('user_action', action, { severity: 'high' });
  };

  return { addEvent, completeEvent, trackUserAction };
}

// Cleanup on page unload
if (typeof window !== 'undefined') {
  window.addEventListener('beforeunload', () => {
    causalityTracker.destroy();
  });
}
