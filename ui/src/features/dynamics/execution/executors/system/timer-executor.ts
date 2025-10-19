/**
 * Timer Tool Executor
 * Handles setTimeout/setInterval operations
 */

import { logger } from "../../../../../core/monitoring/core/logger";
import { ExecutorContext, BaseExecutor } from "../core/types";

export class TimerExecutor implements BaseExecutor {
  private context: ExecutorContext;
  private activeTimers: Set<number> = new Set();

  constructor(context: ExecutorContext) {
    this.context = context;
  }

  execute(action: string, params: Record<string, any>): any {
    switch (action) {
      case "set":
        const timerId = setTimeout(() => {
          logger.debug("Executing delayed action", {
            component: "TimerExecutor",
            action: params.action,
          });
          this.activeTimers.delete(timerId);
        }, params.delay) as unknown as number;

        this.activeTimers.add(timerId);
        this.context.componentState.set(`timer_${timerId}`, timerId);
        return timerId;

      case "interval":
        const intervalId = setInterval(() => {
          logger.debug("Executing interval action", {
            component: "TimerExecutor",
            action: params.action,
          });
        }, params.interval) as unknown as number;

        this.activeTimers.add(intervalId);
        this.context.componentState.set(`interval_${intervalId}`, intervalId);
        return intervalId;

      case "clear":
        const id = this.context.componentState.get(params.timer_id);
        if (id) {
          clearTimeout(id);
          clearInterval(id);
          this.activeTimers.delete(id);
        }
        return true;

      default:
        return null;
    }
  }

  /**
   * Clean up all active timers (call on unmount)
   */
  cleanup(): void {
    this.activeTimers.forEach((timerId) => {
      clearTimeout(timerId);
      clearInterval(timerId);
    });
    this.activeTimers.clear();
    logger.info("Cleaned up all active timers", {
      component: "TimerExecutor",
      count: this.activeTimers.size,
    });
  }
}
