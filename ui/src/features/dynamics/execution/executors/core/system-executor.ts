/**
 * System Tool Executor
 * Handles system-level operations (alerts, undo/redo, state management)
 */

import { logger } from "../../../../../core/utils/monitoring/logger";
import { ExecutorContext, BaseExecutor } from "./types";

export class SystemExecutor implements BaseExecutor {
  private context: ExecutorContext;
  private redoStack: Array<{ timestamp: number; changes: Map<string, any> }> = [];
  private isUndoRedoOperation = false;

  constructor(context: ExecutorContext) {
    this.context = context;

    // Subscribe to state changes to clear redo stack on new changes
    this.context.componentState.subscribe("*", () => {
      // If we're not in the middle of undo/redo, clear the redo stack
      if (!this.isUndoRedoOperation) {
        this.redoStack = [];
      }
    });
  }

  execute(action: string, params: Record<string, any>): any {
    switch (action) {
      case "alert":
        alert(params.message);
        return true;

      case "log":
        logger.info("System log", { component: "SystemExecutor", message: params.message });
        return true;

      case "undo":
        return this.executeUndo();

      case "redo":
        return this.executeRedo();

      case "get_history":
        return this.context.componentState.getHistory();

      case "get_state_snapshot":
        return this.context.componentState.getAll();

      case "restore_state":
        // Restore state from snapshot
        if (params.snapshot) {
          this.context.componentState.batch(() => {
            this.context.componentState.clear();
            Object.entries(params.snapshot).forEach(([key, value]) => {
              this.context.componentState.set(key, value);
            });
          });
        }
        return true;

      case "get_redo_stack_size":
        return this.getRedoStackSize();

      case "clear_redo_stack":
        this.clearRedoStack();
        return true;

      default:
        return null;
    }
  }

  /**
   * Execute undo operation
   */
  private executeUndo(): boolean {
    const history = this.context.componentState.getHistory();
    if (history.length < 2) {
      logger.info("No history to undo", { component: "SystemExecutor" });
      return false;
    }

    // Get current state before undo
    const currentState = this.context.componentState.getAll();
    const currentTimestamp = Date.now();

    // Get the timestamp before the last change
    const previousTimestamp = history[history.length - 2].timestamp;

    // Push current state to redo stack
    this.redoStack.push({
      timestamp: currentTimestamp,
      changes: new Map(Object.entries(currentState)),
    });

    // Mark that we're performing an undo operation
    this.isUndoRedoOperation = true;

    try {
      // Restore to previous state
      this.context.componentState.restoreToTimestamp(previousTimestamp);

      logger.info("Undo executed", {
        component: "SystemExecutor",
        restoredTo: previousTimestamp,
        redoStackSize: this.redoStack.length,
      });

      return true;
    } finally {
      this.isUndoRedoOperation = false;
    }
  }

  /**
   * Execute redo operation
   * Restores state from the redo stack populated by undo operations
   */
  private executeRedo(): boolean {
    if (this.redoStack.length === 0) {
      logger.info("No changes to redo", { component: "SystemExecutor" });
      return false;
    }

    // Pop the last redo state
    const redoState = this.redoStack.pop();
    if (!redoState) return false;

    // Mark that we're performing a redo operation
    this.isUndoRedoOperation = true;

    try {
      // Restore the state from redo stack
      this.context.componentState.batch(() => {
        this.context.componentState.clear();
        redoState.changes.forEach((value, key) => {
          this.context.componentState.set(key, value);
        });
      });

      logger.info("Redo executed", {
        component: "SystemExecutor",
        redoStackSize: this.redoStack.length,
        timestamp: redoState.timestamp,
      });

      return true;
    } finally {
      this.isUndoRedoOperation = false;
    }
  }

  /**
   * Get redo stack size (useful for UI)
   */
  public getRedoStackSize(): number {
    return this.redoStack.length;
  }

  /**
   * Clear redo stack
   */
  public clearRedoStack(): void {
    this.redoStack = [];
    logger.debug("Redo stack cleared", { component: "SystemExecutor" });
  }
}
