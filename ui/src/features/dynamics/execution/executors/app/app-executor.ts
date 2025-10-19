/**
 * App Tool Executor
 * Handles app lifecycle operations (spawn, close, state persistence)
 */

import { logger } from "../../../../../core/monitoring/core/logger";
import { ExecutorContext, AsyncExecutor } from "../core/types";

export class AppExecutor implements AsyncExecutor {
  private context: ExecutorContext;

  constructor(context: ExecutorContext) {
    this.context = context;
  }

  async execute(action: string, params: Record<string, any>): Promise<any> {
    switch (action) {
      case "spawn":
        logger.info("Spawning new app", {
          component: "AppExecutor",
          request: params.request,
        });
        // Use HTTP endpoint for app spawning instead of creating new WebSocket
        const response = await fetch("http://localhost:8000/generate-ui", {
          method: "POST",
          headers: { "Content-Type": "application/json" },
          body: JSON.stringify({
            message: params.request,
            context: { parent_app_id: this.context.componentState.get("app_id") },
          }),
        });
        const data = await response.json();

        if (data.error) {
          throw new Error(data.error);
        }

        // Notify parent component to render new app
        window.postMessage(
          {
            type: "spawn_app",
            app_id: data.app_id,
            blueprint: data.blueprint,
          },
          "*"
        );

        return data.blueprint;

      case "close":
        logger.info("Closing current app", { component: "AppExecutor" });
        // Save state before closing if requested
        if (params.save_state) {
          await this.persistState();
        }
        // Notify parent to close this app
        window.postMessage({ type: "close_app" }, "*");
        return true;

      case "list":
        logger.info("Listing apps", { component: "AppExecutor" });
        const appsResponse = await fetch("http://localhost:8000/apps");
        const appsData = await appsResponse.json();
        return appsData.apps;

      case "save_state":
        return await this.persistState();

      case "load_state":
        return await this.loadPersistedState(params.app_id);

      default:
        return null;
    }
  }

  /**
   * Persist current state to backend storage
   */
  private async persistState(): Promise<boolean> {
    try {
      const appId = this.context.componentState.get("app_id");
      if (!appId) {
        logger.warn("Cannot persist state: no app_id", {
          component: "AppExecutor",
        });
        return false;
      }

      const stateSnapshot = this.context.componentState.getAll();

      const response = await fetch(`http://localhost:8000/apps/${appId}/state`, {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ state: stateSnapshot }),
      });

      if (!response.ok) {
        throw new Error("Failed to persist state");
      }

      logger.info("State persisted successfully", {
        component: "AppExecutor",
        appId,
        stateSize: Object.keys(stateSnapshot).length,
      });

      return true;
    } catch (error) {
      logger.error("Failed to persist state", error as Error, {
        component: "AppExecutor",
      });
      return false;
    }
  }

  /**
   * Load persisted state from backend
   */
  private async loadPersistedState(appId: string): Promise<boolean> {
    try {
      const response = await fetch(`http://localhost:8000/apps/${appId}/state`);

      if (!response.ok) {
        throw new Error("Failed to load state");
      }

      const data = await response.json();

      if (data.state) {
        this.context.componentState.batch(() => {
          Object.entries(data.state).forEach(([key, value]) => {
            this.context.componentState.set(key, value as any);
          });
        });

        logger.info("State loaded successfully", {
          component: "AppExecutor",
          appId,
          stateSize: Object.keys(data.state).length,
        });

        return true;
      }

      return false;
    } catch (error) {
      logger.error("Failed to load state", error as Error, {
        component: "AppExecutor",
        appId,
      });
      return false;
    }
  }
}
