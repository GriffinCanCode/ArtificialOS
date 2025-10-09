/**
 * Icons Store
 * Zustand state management for desktop icons with persistence
 */

import { create } from "zustand";
import { devtools, persist } from "zustand/middleware";
import { useShallow } from "zustand/react/shallow";
import type { Icon, GridPosition, ArrangeStrategy, SelectionBox, IconBadge } from "../core/types";
import { buildCollisionMap, findNearestAvailable, findFirstAvailable } from "../core/collision";
import { arrange, compactLayout } from "../utils/arrange";
import { filterIcons, sortByRelevance } from "../utils/search";
import { getIconIdsInBox, getIconIdsInRange } from "../utils/selection";

// ============================================================================
// Store Interface
// ============================================================================

interface Store {
  // State
  icons: Icon[];
  selectedIds: Set<string>;
  anchorId: string | null; // Anchor for range selection
  draggedIds: Set<string>;
  nextZIndex: number;
  viewportDimensions: { width: number; height: number; rows: number; cols: number };
  selectionBox: SelectionBox | null;
  searchQuery: string;
  searchResults: string[]; // Filtered icon IDs

  // Icon CRUD
  add: (icon: Omit<Icon, "id" | "isSelected" | "isDragging" | "isHovered" | "zIndex" | "createdAt" | "updatedAt">) => string;
  remove: (iconId: string) => void;
  update: (iconId: string, updates: Partial<Omit<Icon, "id">>) => void;
  get: (iconId: string) => Icon | undefined;
  getAll: () => Icon[];

  // Position management
  updatePosition: (iconId: string, position: GridPosition) => void;
  updatePositions: (positions: Map<string, GridPosition>) => void;
  moveToPosition: (iconId: string, position: GridPosition, findNearest?: boolean) => void;

  // Selection
  select: (iconId: string, multi?: boolean) => void;
  selectRange: (startId: string, endId: string) => void;
  deselect: (iconId: string) => void;
  selectAll: () => void;
  clearSelection: () => void;
  getSelected: () => Icon[];

  // Selection Box
  startSelectionBox: (start: { x: number; y: number }) => void;
  updateSelectionBox: (current: { x: number; y: number }) => void;
  endSelectionBox: () => void;
  cancelSelectionBox: () => void;

  // Drag state
  startDrag: (iconIds: string[]) => void;
  endDrag: () => void;

  // Arrangement
  autoArrange: (strategy: ArrangeStrategy) => void;
  compact: () => void;

  // Viewport
  updateViewport: (width: number, height: number, rows: number, cols: number) => void;

  // Search
  setSearchQuery: (query: string) => void;
  getSearchResults: () => Icon[];

  // Badges
  setBadge: (iconId: string, badge: IconBadge | undefined) => void;
  clearBadge: (iconId: string) => void;

  // Utilities
  clearAll: () => void;
  fixOverlaps: () => void;
}

// ============================================================================
// Helper Functions
// ============================================================================

function generateId(): string {
  return `icon-${Date.now()}-${Math.random().toString(36).substring(2, 9)}`;
}

function now(): number {
  return Date.now();
}

// ============================================================================
// Store Implementation
// ============================================================================

export const useStore = create<Store>()(
  devtools(
    persist(
      (set, get) => ({
        // Initial state
        icons: [],
        selectedIds: new Set(),
        anchorId: null,
        draggedIds: new Set(),
        nextZIndex: 100,
        viewportDimensions: { width: 1920, height: 1080, rows: 8, cols: 16 },
        selectionBox: null,
        searchQuery: "",
        searchResults: [],

        // ====================================================================
        // Icon CRUD Operations
        // ====================================================================

        add: (iconData) => {
          const state = get();
          const id = generateId();
          const timestamp = now();

          // Find first available position
          const collisionMap = buildCollisionMap(state.icons);

          // Check if requested position is available
          let position = iconData.position;
          const isOccupied = collisionMap.occupied.has(`${position.row}:${position.col}`);

          if (isOccupied) {
            // Find nearest available position
            const nearest = findNearestAvailable(
              position,
              collisionMap,
              state.viewportDimensions.rows,
              state.viewportDimensions.cols
            );
            position = nearest || findFirstAvailable(
              collisionMap,
              state.viewportDimensions.rows,
              state.viewportDimensions.cols
            ) || position;
          }

          const newIcon: Icon = {
            id,
            ...iconData,
            position,
            isSelected: false,
            isDragging: false,
            isHovered: false,
            zIndex: state.nextZIndex,
            createdAt: timestamp,
            updatedAt: timestamp,
          };

          set(
            (state) => ({
              icons: [...state.icons, newIcon],
              nextZIndex: state.nextZIndex + 1,
            }),
            false,
            "add"
          );

          return id;
        },

        remove: (iconId) => {
          set(
            (state) => ({
              icons: state.icons.filter((i) => i.id !== iconId),
              selectedIds: new Set([...state.selectedIds].filter((id) => id !== iconId)),
              draggedIds: new Set([...state.draggedIds].filter((id) => id !== iconId)),
            }),
            false,
            "remove"
          );
        },

        update: (iconId, updates) => {
          set(
            (state) => ({
              icons: state.icons.map((icon) =>
                icon.id === iconId
                  ? {
                      ...icon,
                      ...updates,
                      updatedAt: now(),
                    }
                  : icon
              ),
            }),
            false,
            "update"
          );
        },

        get: (iconId) => {
          return get().icons.find((i) => i.id === iconId);
        },

        getAll: () => {
          return get().icons;
        },

        // ====================================================================
        // Position Management
        // ====================================================================

        updatePosition: (iconId, position) => {
          set(
            (state) => ({
              icons: state.icons.map((icon) =>
                icon.id === iconId
                  ? {
                      ...icon,
                      position,
                      updatedAt: now(),
                    }
                  : icon
              ),
            }),
            false,
            "updatePosition"
          );
        },

        updatePositions: (positions) => {
          set(
            (state) => {
              // Build collision map excluding icons being moved
              const movingIconIds = Array.from(positions.keys());
              const staticIcons = state.icons.filter((icon) => !movingIconIds.includes(icon.id));
              const collisionMap = buildCollisionMap(staticIcons);

              // Update positions with collision detection
              return {
                icons: state.icons.map((icon) => {
                  const newPosition = positions.get(icon.id);
                  if (!newPosition) return icon;

                  // Check if new position is occupied by a static icon
                  const posKey = `${newPosition.row}:${newPosition.col}`;
                  const isOccupied = collisionMap.occupied.has(posKey);

                  if (isOccupied) {
                    // Find nearest available position
                    const nearest = findNearestAvailable(
                      newPosition,
                      collisionMap,
                      state.viewportDimensions.rows,
                      state.viewportDimensions.cols
                    );

                    if (nearest) {
                      // Mark as occupied for subsequent icons
                      collisionMap.occupied.set(`${nearest.row}:${nearest.col}`, icon.id);
                      return {
                        ...icon,
                        position: nearest,
                        updatedAt: now(),
                      };
                    }
                  }

                  // Position is free or no alternative found
                  collisionMap.occupied.set(posKey, icon.id);
                  return {
                    ...icon,
                    position: newPosition,
                    updatedAt: now(),
                  };
                }),
              };
            },
            false,
            "updatePositions"
          );
        },

        moveToPosition: (iconId, position, findNearest = true) => {
          const state = get();
          const collisionMap = buildCollisionMap(state.icons.filter((i) => i.id !== iconId));

          let targetPosition = position;

          if (findNearest) {
            // Find nearest available if position is occupied
            const nearestPosition = findNearestAvailable(
              position,
              collisionMap,
              state.viewportDimensions.rows,
              state.viewportDimensions.cols
            );
            targetPosition = nearestPosition || position;
          }

          get().updatePosition(iconId, targetPosition);
        },

        // ====================================================================
        // Selection Management
        // ====================================================================

        select: (iconId, multi = false) => {
          set(
            (state) => {
              const newSelectedIds = multi
                ? new Set([...state.selectedIds, iconId])
                : new Set([iconId]);

              return {
                selectedIds: newSelectedIds,
                anchorId: iconId, // Set anchor for range selection
                icons: state.icons.map((icon) => ({
                  ...icon,
                  isSelected: newSelectedIds.has(icon.id),
                  // Don't change z-index on selection - only on drag
                })),
              };
            },
            false,
            "select"
          );
        },

        selectRange: (startId, endId) => {
          set(
            (state) => {
              const rangeIds = getIconIdsInRange(state.icons, startId, endId);
              const newSelectedIds = new Set([...state.selectedIds, ...rangeIds]);

              return {
                selectedIds: newSelectedIds,
                icons: state.icons.map((icon) => ({
                  ...icon,
                  isSelected: newSelectedIds.has(icon.id),
                })),
              };
            },
            false,
            "selectRange"
          );
        },

        deselect: (iconId) => {
          set(
            (state) => {
              const newSelectedIds = new Set(state.selectedIds);
              newSelectedIds.delete(iconId);

              return {
                selectedIds: newSelectedIds,
                icons: state.icons.map((icon) =>
                  icon.id === iconId ? { ...icon, isSelected: false } : icon
                ),
              };
            },
            false,
            "deselect"
          );
        },

        selectAll: () => {
          set(
            (state) => ({
              selectedIds: new Set(state.icons.map((i) => i.id)),
              icons: state.icons.map((icon) => ({ ...icon, isSelected: true })),
            }),
            false,
            "selectAll"
          );
        },

        clearSelection: () => {
          set(
            (state) => ({
              selectedIds: new Set(),
              anchorId: null,
              icons: state.icons.map((icon) => ({ ...icon, isSelected: false })),
            }),
            false,
            "clearSelection"
          );
        },

        getSelected: () => {
          const state = get();
          return state.icons.filter((i) => state.selectedIds.has(i.id));
        },

        // ====================================================================
        // Selection Box
        // ====================================================================

        startSelectionBox: (start) => {
          set(
            {
              selectionBox: {
                start,
                end: start,
                current: start,
                isActive: true,
              },
            },
            false,
            "startSelectionBox"
          );
        },

        updateSelectionBox: (current) => {
          set(
            (state) => {
              if (!state.selectionBox) return state;

              const updatedBox = {
                ...state.selectionBox,
                end: current,
                current,
              };

              // Get icons in box
              const iconsInBox = getIconIdsInBox(state.icons, updatedBox);
              const newSelectedIds = new Set(iconsInBox);

              return {
                selectionBox: updatedBox,
                selectedIds: newSelectedIds,
                icons: state.icons.map((icon) => ({
                  ...icon,
                  isSelected: newSelectedIds.has(icon.id),
                })),
              };
            },
            false,
            "updateSelectionBox"
          );
        },

        endSelectionBox: () => {
          set(
            {
              selectionBox: null,
            },
            false,
            "endSelectionBox"
          );
        },

        cancelSelectionBox: () => {
          set(
            (state) => ({
              selectionBox: null,
              selectedIds: new Set(),
              icons: state.icons.map((icon) => ({ ...icon, isSelected: false })),
            }),
            false,
            "cancelSelectionBox"
          );
        },

        // ====================================================================
        // Drag State
        // ====================================================================

        startDrag: (iconIds) => {
          set(
            (state) => ({
              draggedIds: new Set(iconIds),
              icons: state.icons.map((icon) => ({
                ...icon,
                isDragging: iconIds.includes(icon.id),
                // Don't change z-index - CSS will handle visual layering during drag
              })),
            }),
            false,
            "startDrag"
          );
        },

        endDrag: () => {
          set(
            (state) => ({
              draggedIds: new Set(),
              icons: state.icons.map((icon) => ({ ...icon, isDragging: false })),
            }),
            false,
            "endDrag"
          );
        },

        // ====================================================================
        // Arrangement
        // ====================================================================

        autoArrange: (strategy) => {
          const state = get();
          const newPositions = arrange(
            state.icons,
            strategy,
            state.viewportDimensions.rows,
            state.viewportDimensions.cols
          );
          get().updatePositions(newPositions);
        },

        compact: () => {
          const state = get();
          const newPositions = compactLayout(
            state.icons,
            state.viewportDimensions.rows,
            state.viewportDimensions.cols
          );
          get().updatePositions(newPositions);
        },

        // ====================================================================
        // Viewport
        // ====================================================================

        updateViewport: (width, height, rows, cols) => {
          set(
            {
              viewportDimensions: { width, height, rows, cols },
            },
            false,
            "updateViewport"
          );
        },

        // ====================================================================
        // Search
        // ====================================================================

        setSearchQuery: (query) => {
          set(
            (state) => {
              const results = query.trim() ? filterIcons(query, state.icons) : state.icons.map((i) => i.id);

              return {
                searchQuery: query,
                searchResults: results,
              };
            },
            false,
            "setSearchQuery"
          );
        },

        getSearchResults: () => {
          const state = get();
          if (!state.searchQuery.trim()) {
            return state.icons;
          }
          return sortByRelevance(state.icons, state.searchQuery).filter((icon) =>
            state.searchResults.includes(icon.id)
          );
        },

        // ====================================================================
        // Badges
        // ====================================================================

        setBadge: (iconId, badge) => {
          set(
            (state) => ({
              icons: state.icons.map((icon) =>
                icon.id === iconId
                  ? {
                      ...icon,
                      badge,
                      updatedAt: now(),
                    }
                  : icon
              ),
            }),
            false,
            "setBadge"
          );
        },

        clearBadge: (iconId) => {
          get().setBadge(iconId, undefined);
        },

        // ====================================================================
        // Utilities
        // ====================================================================

        clearAll: () => {
          set(
            {
              icons: [],
              selectedIds: new Set(),
              anchorId: null,
              draggedIds: new Set(),
              selectionBox: null,
              searchQuery: "",
              searchResults: [],
            },
            false,
            "clearAll"
          );
        },

        fixOverlaps: () => {
          set(
            (state) => {
              // Find icons at the same position
              const positionMap = new Map<string, Icon[]>();

              state.icons.forEach((icon) => {
                const key = `${icon.position.row}:${icon.position.col}`;
                const existing = positionMap.get(key) || [];
                existing.push(icon);
                positionMap.set(key, existing);
              });

              // Find overlaps
              const overlaps = Array.from(positionMap.entries()).filter(([_, icons]) => icons.length > 1);

              if (overlaps.length === 0) {
                return state; // No overlaps
              }

              console.warn(`Fixing ${overlaps.length} overlapping positions`);

              // Build collision map of non-overlapping icons
              const nonOverlappingIds = new Set(
                state.icons
                  .filter((icon) => {
                    const key = `${icon.position.row}:${icon.position.col}`;
                    const iconsAtPos = positionMap.get(key) || [];
                    return iconsAtPos.length === 1;
                  })
                  .map((i) => i.id)
              );

              const collisionMap = buildCollisionMap(
                state.icons.filter((i) => nonOverlappingIds.has(i.id))
              );

              // Fix each overlap by moving duplicates to nearest available positions
              const fixedIcons = state.icons.map((icon) => {
                const key = `${icon.position.row}:${icon.position.col}`;
                const iconsAtPos = positionMap.get(key) || [];

                if (iconsAtPos.length <= 1) {
                  return icon; // No overlap
                }

                // Keep first icon, move others
                const isFirst = iconsAtPos[0].id === icon.id;
                if (isFirst) {
                  return icon;
                }

                // Find nearest available position
                const nearest = findNearestAvailable(
                  icon.position,
                  collisionMap,
                  state.viewportDimensions.rows,
                  state.viewportDimensions.cols
                );

                if (nearest) {
                  // Mark as occupied
                  collisionMap.occupied.set(`${nearest.row}:${nearest.col}`, icon.id);
                  return {
                    ...icon,
                    position: nearest,
                    updatedAt: now(),
                  };
                }

                return icon;
              });

              return {
                ...state,
                icons: fixedIcons,
              };
            },
            false,
            "fixOverlaps"
          );
        },
      }),
      {
        name: "icons-storage",
        version: 1,
        partialize: (state) => ({
          icons: state.icons,
          nextZIndex: state.nextZIndex,
        }),
      }
    ),
    { name: "IconsStore" }
  )
);

// ============================================================================
// Hooks for Selective Subscriptions
// ============================================================================

/**
 * Subscribe to all icons
 */
export function useIcons() {
  return useStore((state) => state.icons);
}

/**
 * Subscribe to icon actions
 */
export function useActions() {
  return useStore(
    useShallow((state) => ({
      add: state.add,
      remove: state.remove,
      update: state.update,
      updatePosition: state.updatePosition,
      updatePositions: state.updatePositions,
      moveToPosition: state.moveToPosition,
      select: state.select,
      selectRange: state.selectRange,
      deselect: state.deselect,
      selectAll: state.selectAll,
      clearSelection: state.clearSelection,
      startSelectionBox: state.startSelectionBox,
      updateSelectionBox: state.updateSelectionBox,
      endSelectionBox: state.endSelectionBox,
      cancelSelectionBox: state.cancelSelectionBox,
      startDrag: state.startDrag,
      endDrag: state.endDrag,
      autoArrange: state.autoArrange,
      compact: state.compact,
      updateViewport: state.updateViewport,
      setSearchQuery: state.setSearchQuery,
      setBadge: state.setBadge,
      clearBadge: state.clearBadge,
      clearAll: state.clearAll,
      fixOverlaps: state.fixOverlaps,
    }))
  );
}

/**
 * Subscribe to selected icons
 */
export function useSelectedIcons() {
  return useStore(useShallow((state) => state.getSelected()));
}

/**
 * Subscribe to specific icon
 */
export function useIcon(iconId: string) {
  return useStore((state) => state.icons.find((i) => i.id === iconId));
}

/**
 * Subscribe to dragged icon IDs
 */
export function useDraggedIds() {
  return useStore((state) => state.draggedIds);
}

/**
 * Subscribe to viewport dimensions
 */
export function useViewport() {
  return useStore((state) => state.viewportDimensions);
}

/**
 * Subscribe to selection box
 */
export function useSelectionBox() {
  return useStore((state) => state.selectionBox);
}

/**
 * Subscribe to search state
 */
export function useSearchState() {
  return useStore(
    useShallow((state) => ({
      query: state.searchQuery,
      results: state.searchResults,
      isActive: state.searchQuery.length > 0,
    }))
  );
}

/**
 * Subscribe to search results (filtered icons)
 */
export function useSearchResults() {
  return useStore((state) => state.getSearchResults());
}

/**
 * Subscribe to anchor ID
 */
export function useAnchorId() {
  return useStore((state) => state.anchorId);
}

