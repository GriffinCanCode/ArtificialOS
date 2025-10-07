/**
 * Virtual Scrolling Hooks
 * Custom hooks for virtualization functionality
 */

import { useRef, useCallback, useEffect, useState } from "react";
import { useVirtualizer } from "@tanstack/react-virtual";
import type { VirtualizerOptions } from "@tanstack/react-virtual";
import { calculateOverscan, measureElement } from "./utils";

// ============================================================================
// Basic Hooks
// ============================================================================

/**
 * Hook for basic virtual list setup
 */
export const useVirtualList = <T = any>(
  items: T[],
  options: Partial<VirtualizerOptions<any, any>> = {}
) => {
  const parentRef = useRef<HTMLDivElement>(null);

  const virtualizer = useVirtualizer({
    count: items.length,
    getScrollElement: () => parentRef.current,
    estimateSize: () => 60,
    overscan: 5,
    ...options,
  });

  return {
    parentRef,
    virtualizer,
    virtualItems: virtualizer.getVirtualItems(),
    totalSize: virtualizer.getTotalSize(),
  };
};

/**
 * Hook for virtual grid setup
 */
export const useVirtualGrid = <T = any>(
  items: T[],
  columns: number,
  options: Partial<VirtualizerOptions<any, any>> = {}
) => {
  const parentRef = useRef<HTMLDivElement>(null);
  const rows = Math.ceil(items.length / columns);

  const virtualizer = useVirtualizer({
    count: rows,
    getScrollElement: () => parentRef.current,
    estimateSize: () => 200,
    overscan: 3,
    ...options,
  });

  return {
    parentRef,
    virtualizer,
    virtualRows: virtualizer.getVirtualItems(),
    rows,
    columns,
    totalSize: virtualizer.getTotalSize(),
  };
};

// ============================================================================
// Advanced Hooks
// ============================================================================

/**
 * Hook for dynamic size measurement
 */
export const useMeasure = () => {
  const [sizes, setSizes] = useState<Map<number, number>>(new Map());
  const measureRef = useCallback((index: number, element: HTMLElement | null) => {
    if (!element) return;

    const size = measureElement(element);
    setSizes((prev) => {
      const next = new Map(prev);
      next.set(index, size);
      return next;
    });
  }, []);

  const getSize = useCallback(
    (index: number, defaultSize = 60) => {
      return sizes.get(index) ?? defaultSize;
    },
    [sizes]
  );

  return {
    measureRef,
    getSize,
    sizes,
  };
};

/**
 * Hook for auto-calculating optimal overscan
 */
export const useAutoOverscan = (viewportSize: number, itemSize: number): number => {
  const [overscan, setOverscan] = useState(5);

  useEffect(() => {
    const calculated = calculateOverscan(viewportSize, itemSize);
    setOverscan(calculated);
  }, [viewportSize, itemSize]);

  return overscan;
};

/**
 * Hook for virtual scroll to specific item
 */
export const useScrollToItem = (virtualizer: ReturnType<typeof useVirtualizer>) => {
  const scrollToItem = useCallback(
    (index: number, options?: { align?: "start" | "center" | "end" | "auto" }) => {
      virtualizer.scrollToIndex(index, options);
    },
    [virtualizer]
  );

  const scrollToTop = useCallback(() => {
    scrollToItem(0, { align: "start" });
  }, [scrollToItem]);

  const scrollToBottom = useCallback(() => {
    const lastIndex = virtualizer.options.count - 1;
    scrollToItem(lastIndex, { align: "end" });
  }, [scrollToItem, virtualizer.options.count]);

  return {
    scrollToItem,
    scrollToTop,
    scrollToBottom,
  };
};

/**
 * Hook for infinite scroll loading
 */
export const useInfiniteScroll = (
  virtualizer: ReturnType<typeof useVirtualizer>,
  options: {
    hasMore: boolean;
    loadMore: () => void | Promise<void>;
    threshold?: number;
  }
) => {
  const { hasMore, loadMore, threshold = 5 } = options;
  const [isLoading, setIsLoading] = useState(false);

  useEffect(() => {
    const items = virtualizer.getVirtualItems();
    if (!items.length || !hasMore || isLoading) return;

    const lastItem = items[items.length - 1];
    const shouldLoad = lastItem.index >= virtualizer.options.count - threshold;

    if (shouldLoad) {
      setIsLoading(true);
      Promise.resolve(loadMore()).finally(() => {
        setIsLoading(false);
      });
    }
  }, [virtualizer, hasMore, loadMore, threshold, isLoading]);

  return { isLoading };
};

/**
 * Hook for virtual scroll metrics
 */
export const useVirtualMetrics = (virtualizer: ReturnType<typeof useVirtualizer>) => {
  const virtualItems = virtualizer.getVirtualItems();

  return {
    totalItems: virtualizer.options.count,
    visibleItems: virtualItems.length,
    scrollOffset: virtualizer.scrollOffset ?? 0,
    totalSize: virtualizer.getTotalSize(),
    firstVisibleIndex: virtualItems[0]?.index ?? 0,
    lastVisibleIndex: virtualItems[virtualItems.length - 1]?.index ?? 0,
  };
};
