/**
 * System Tool Executor
 * Handles system-level operations (alerts, undo/redo, state management)
 */

import { logger } from "../../../../../core/utils/monitoring/logger";
import { ExecutorContext, BaseExecutor } from "./types";

export class SystemExecutor implements BaseExecutor {
  private context: ExecutorContext;

  constructor(context: ExecutorContext) {
    this.context = context;
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

      default:
        return null;
    }
  }

  /**
   * Execute undo operation
   */
  private executeUndo(): boolean {
    const history = this.context.componentState.getHistory();
    if (history.length < 2) return false;

    // Get the timestamp before the last change
    const previousTimestamp = history[history.length - 2].timestamp;
    this.context.componentState.restoreToTimestamp(previousTimestamp);

    logger.info("Undo executed", {
      component: "SystemExecutor",
      restoredTo: previousTimestamp,
    });

    return true;
  }

  /**
   * Execute redo operation (requires maintaining separate redo stack)
   */
  private executeRedo(): boolean {
    // Note: Full redo requires maintaining a separate redo stack
    // This is a simplified version
    logger.warn("Redo not fully implemented", {
      component: "SystemExecutor",
    });
    return false;
  }
}
