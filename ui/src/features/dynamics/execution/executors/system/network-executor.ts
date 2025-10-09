/**
 * Network Tool Executor
 * Handles HTTP requests
 */

import { ExecutorContext, AsyncExecutor } from "../core/types";

export class NetworkExecutor implements AsyncExecutor {
  constructor(_context: ExecutorContext) {
    // Context not currently used but kept for interface compatibility
  }

  async execute(action: string, params: Record<string, any>): Promise<any> {
    switch (action) {
      case "get":
        const getResponse = await fetch(params.url);
        return await getResponse.json();

      case "post":
        const postResponse = await fetch(params.url, {
          method: "POST",
          headers: { "Content-Type": "application/json" },
          body: JSON.stringify(params.data),
        });
        return await postResponse.json();

      default:
        return null;
    }
  }
}
