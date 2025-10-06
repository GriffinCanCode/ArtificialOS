/**
 * Notification Tool Executor
 * Handles browser notifications
 */

import { logger } from "../../../../../core/utils/monitoring/logger";
import { ExecutorContext, BaseExecutor } from "../core/types";

export class NotificationExecutor implements BaseExecutor {
  private context: ExecutorContext;

  constructor(context: ExecutorContext) {
    this.context = context;
  }

  execute(action: string, params: Record<string, any>): any {
    switch (action) {
      case "show":
        if ("Notification" in window) {
          if (Notification.permission === "granted") {
            new Notification(params.title, {
              body: params.message,
              icon: params.icon,
            });
          } else if (Notification.permission !== "denied") {
            Notification.requestPermission().then((permission) => {
              if (permission === "granted") {
                new Notification(params.title, {
                  body: params.message,
                  icon: params.icon,
                });
              }
            });
          }
        }
        logger.debug("Notification shown", { component: "NotificationExecutor" });
        return true;

      default:
        return null;
    }
  }
}
