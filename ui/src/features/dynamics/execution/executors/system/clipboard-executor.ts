/**
 * Clipboard Tool Executor
 * Handles copy/paste operations
 */

import { logger } from "../../../../../core/utils/monitoring/logger";
import { ExecutorContext, AsyncExecutor } from "../core/types";

export class ClipboardExecutor implements AsyncExecutor {
  private context: ExecutorContext;

  constructor(context: ExecutorContext) {
    this.context = context;
  }

  async execute(action: string, params: Record<string, any>): Promise<any> {
    switch (action) {
      case "copy":
        try {
          await navigator.clipboard.writeText(params.text);
          logger.debug("Text copied to clipboard", { component: "ClipboardExecutor" });
          return true;
        } catch (error) {
          logger.error("Failed to copy to clipboard", error as Error, {
            component: "ClipboardExecutor",
          });
          return false;
        }

      case "paste":
        try {
          const text = await navigator.clipboard.readText();
          logger.debug("Text pasted from clipboard", { component: "ClipboardExecutor" });
          return text;
        } catch (error) {
          logger.error("Failed to paste from clipboard", error as Error, {
            component: "ClipboardExecutor",
          });
          return null;
        }

      default:
        return null;
    }
  }
}
