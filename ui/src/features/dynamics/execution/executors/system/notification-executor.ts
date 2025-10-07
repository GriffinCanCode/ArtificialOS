/**
 * Notification Tool Executor
 * Handles browser notifications (OS-level)
 * Note: For in-app toasts, use ToastExecutor instead
 */

import { logger } from "../../../../../core/utils/monitoring/logger";
import { ExecutorContext, BaseExecutor } from "../core/types";
import { toast } from "../../../../../core/toast";

export class NotificationExecutor implements BaseExecutor {
  private context: ExecutorContext;

  constructor(context: ExecutorContext) {
    this.context = context;
  }

  execute(action: string, params: Record<string, any>): any {
    switch (action) {
      case "show":
        return this.showNotification(params);

      case "request_permission":
        return this.requestPermission();

      default:
        logger.warn("Unknown notification action", { component: "NotificationExecutor", action });
        return null;
    }
  }

  private showNotification(params: Record<string, any>): boolean {
    const { title, message, icon, use_toast = false, ...toastOptions } = params;

    // Option to use in-app toast instead of OS notification
    if (use_toast) {
      toast.info(title, {
        description: message,
        ...toastOptions,
      });
      logger.debug("Toast shown instead of notification", { component: "NotificationExecutor" });
      return true;
    }

    // Try to show OS notification
    if ("Notification" in window) {
      if (Notification.permission === "granted") {
        new Notification(title, {
          body: message,
          icon: icon,
        });
        logger.debug("OS notification shown", { component: "NotificationExecutor" });
        return true;
      } else if (Notification.permission !== "denied") {
        Notification.requestPermission().then((permission) => {
          if (permission === "granted") {
            new Notification(title, {
              body: message,
              icon: icon,
            });
            logger.debug("OS notification shown after permission", {
              component: "NotificationExecutor",
            });
          } else {
            // Fall back to toast if permission denied
            toast.info(title, {
              description: message,
            });
            logger.debug("Toast shown after permission denied", {
              component: "NotificationExecutor",
            });
          }
        });
        return true;
      } else {
        // Permission denied, fall back to toast
        toast.info(title, {
          description: message,
        });
        logger.debug("Toast shown (permission denied)", { component: "NotificationExecutor" });
        return true;
      }
    }

    // Browser doesn't support notifications, use toast
    toast.info(title, {
      description: message,
    });
    logger.debug("Toast shown (no notification support)", { component: "NotificationExecutor" });
    return true;
  }

  private requestPermission(): Promise<NotificationPermission> {
    if ("Notification" in window) {
      return Notification.requestPermission();
    }
    return Promise.resolve("denied" as NotificationPermission);
  }
}
