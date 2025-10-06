/**
 * Hub Tool Executor
 * Handles app registry operations (loading and launching apps)
 */

import { logger } from "../../../../../core/utils/monitoring/logger";
import { ExecutorContext, AsyncExecutor } from "../core/types";

export class HubExecutor implements AsyncExecutor {
  private context: ExecutorContext;

  constructor(context: ExecutorContext) {
    this.context = context;
  }

  async execute(action: string, params: Record<string, any>): Promise<any> {
    switch (action) {
      case "load_apps":
        return await this.loadApps();

      case "launch_app":
        return await this.launchApp(params);

      default:
        return null;
    }
  }

  private async loadApps(): Promise<any> {
    logger.info("Loading apps from registry", { component: "HubExecutor" });
    try {
      const response = await fetch("http://localhost:8000/registry/apps");
      const data = await response.json();

      logger.info("Apps fetched from registry", {
        component: "HubExecutor",
        appCount: data.apps?.length || 0,
        apps: data.apps,
      });

      // Store apps in state
      this.context.componentState.set("hub_apps", data.apps || []);
      this.context.componentState.set("hub_stats", data.stats || {});

      // Update count displays
      const allCount = data.apps?.length || 0;
      this.context.componentState.set("all-count", `${allCount} apps available`);

      // Filter by category
      const systemApps = (data.apps || []).filter((app: any) => app.category === "system");
      const productivityApps = (data.apps || []).filter(
        (app: any) => app.category === "productivity"
      );
      const utilitiesApps = (data.apps || []).filter((app: any) => app.category === "utilities");

      this.context.componentState.set("system-count", `${systemApps.length} apps`);
      this.context.componentState.set("productivity-count", `${productivityApps.length} apps`);

      logger.info("Apps loaded successfully", {
        component: "HubExecutor",
        count: allCount,
      });

      // Generate app-shortcut components dynamically
      const createAppShortcut = (app: any) => ({
        type: "app-shortcut",
        id: `app-${app.id}`,
        props: {
          app_id: app.id,
          name: app.name,
          icon: app.icon,
          description: app.description,
          category: app.category,
          variant: "card",
        },
        on_event: {
          click: "hub.launch_app",
        },
      });

      // Notify parent to update UI spec with populated grids (batched with requestAnimationFrame)
      requestAnimationFrame(() => {
        window.postMessage(
          {
            type: "update_hub_grids",
            grids: {
              "all-apps-grid": (data.apps || []).map(createAppShortcut),
              "system-apps-grid": systemApps.map(createAppShortcut),
              "productivity-apps-grid": productivityApps.map(createAppShortcut),
              "utilities-apps-grid": utilitiesApps.map(createAppShortcut),
            },
          },
          "*"
        );
      });

      return data.apps;
    } catch (error) {
      logger.error("Failed to load apps", error as Error, {
        component: "HubExecutor",
      });
      return [];
    }
  }

  private async launchApp(params: Record<string, any>): Promise<any> {
    const appId = params.app_id || params.id;
    if (!appId) {
      logger.warn("No app_id provided for launch", { component: "HubExecutor" });
      return null;
    }

    logger.info("Launching app from registry", {
      component: "HubExecutor",
      appId,
    });

    try {
      const response = await fetch(`http://localhost:8000/registry/apps/${appId}/launch`, {
        method: "POST",
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

      logger.info("App launched successfully", {
        component: "HubExecutor",
        appId,
        launchedAppId: data.app_id,
      });

      return data;
    } catch (error) {
      logger.error("Failed to launch app", error as Error, {
        component: "HubExecutor",
        appId,
      });
      return null;
    }
  }
}
