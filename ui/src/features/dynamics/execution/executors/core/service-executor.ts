/**
 * Service Tool Executor
 * Handles backend service tool execution
 */

import { logger } from "../../../../../core/utils/monitoring/logger";
import { ExecutorContext, AsyncExecutor } from "./types";

export class ServiceExecutor implements AsyncExecutor {
  private context: ExecutorContext;

  constructor(context: ExecutorContext) {
    this.context = context;
  }

  async execute(toolId: string, params: Record<string, any>): Promise<any> {
    logger.debug("Executing service tool", {
      component: "ServiceExecutor",
      toolId,
    });

    try {
      const response = await fetch("http://localhost:8000/services/execute", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({
          tool_id: toolId,
          params: params,
          app_id: this.context.appId,
        }),
      });

      if (!response.ok) {
        throw new Error(`Service call failed: ${response.statusText}`);
      }

      const result = await response.json();

      if (!result.success) {
        throw new Error(result.error || "Service execution failed");
      }

      logger.debug("Service tool executed successfully", {
        component: "ServiceExecutor",
        toolId,
        hasData: !!result.data,
      });
      return result.data;
    } catch (error) {
      logger.error("Service tool execution failed", error as Error, {
        component: "ServiceExecutor",
        toolId,
      });
      throw error;
    }
  }
}
