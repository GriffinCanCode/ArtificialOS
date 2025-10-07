/**
 * Filesystem Tool Executor
 * Handles filesystem operations with dynamic UI updates
 */

import { logger } from "../../../../../core/utils/monitoring/logger";
import { toRgbaString, UI_COLORS, ALPHA_VALUES } from "../../../../../core/utils/color";
import { ExecutorContext, AsyncExecutor } from "../core/types";

export class FilesystemExecutor implements AsyncExecutor {
  private context: ExecutorContext;
  private serviceExecutor: any; // Reference to service executor for delegation

  constructor(context: ExecutorContext, serviceExecutor: any) {
    this.context = context;
    this.serviceExecutor = serviceExecutor;
  }

  async execute(action: string, params: Record<string, any>): Promise<any> {
    switch (action) {
      case "list":
        return await this.listDirectory(params);

      case "mkdir":
      case "create":
      case "read":
      case "write":
      case "delete":
      case "stat":
      case "exists":
      case "move":
      case "copy":
        // For other filesystem operations, delegate to the service executor
        return await this.serviceExecutor.execute(`filesystem.${action}`, params);

      default:
        return null;
    }
  }

  private async listDirectory(params: Record<string, any>): Promise<any> {
    // Handle path from multiple sources: explicit param, component state, or clicked file
    let path =
      params.path ||
      this.context.componentState.get("path-input") ||
      this.context.componentState.get("current-path") ||
      "/tmp/ai-os-storage";

    // If a file/directory was clicked, extract the name and append to current path
    if (params.value && params.value.includes("📁")) {
      const currentPath = this.context.componentState.get("current-path") || "/tmp/ai-os-storage";
      const dirName = params.value.replace("📁 ", "").trim();
      path = `${currentPath}/${dirName}`.replace(/\/+/g, "/"); // Clean up double slashes
    }

    logger.info("Listing directory", { component: "FilesystemExecutor", path });

    try {
      const response = await fetch("http://localhost:8000/services/execute", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({
          tool_id: "filesystem.list",
          params: { path },
          app_id: this.context.appId,
        }),
      });

      if (!response.ok) {
        throw new Error(`Failed to list directory: ${response.statusText}`);
      }

      const result = await response.json();

      if (!result.success) {
        throw new Error(result.error || "Directory listing failed");
      }

      const data = result.data;
      const files = data.files || [];

      // Update state with current path and file count
      this.context.componentState.set("current-path", path);
      this.context.componentState.set("path-input", path);
      this.context.componentState.set("item-count", `${files.length} items`);

      logger.info("Directory listed successfully", {
        component: "FilesystemExecutor",
        path,
        fileCount: files.length,
      });

      // Generate file/folder row components dynamically
      const createFileRow = (file: string) => {
        const isDirectory = file.endsWith("/");
        const displayName = isDirectory ? file.slice(0, -1) : file;

        return {
          type: "container",
          id: `file-row-${displayName}`,
          props: {
            layout: "horizontal",
            gap: 16,
            padding: "small",
            align: "center",
            style: {
              borderBottom: `1px solid ${toRgbaString(UI_COLORS.text.primary, ALPHA_VALUES.ghost)}`,
              cursor: "pointer",
              transition: "background-color 0.2s",
            },
          },
          children: [
            {
              type: "text",
              id: `file-name-${displayName}`,
              props: {
                content: `${isDirectory ? "📁" : "📄"} ${displayName}`,
                variant: "body",
                style: { flex: 1 },
              },
            },
            {
              type: "text",
              id: `file-modified-${displayName}`,
              props: {
                content: "—",
                variant: "caption",
                color: "secondary",
                style: { width: "180px" },
              },
            },
            {
              type: "text",
              id: `file-size-${displayName}`,
              props: {
                content: isDirectory ? "—" : "—",
                variant: "caption",
                color: "secondary",
                style: { width: "100px" },
              },
            },
            {
              type: "container",
              id: `file-actions-${displayName}`,
              props: {
                layout: "horizontal",
                gap: 4,
                style: { width: "120px" },
              },
              children: [
                {
                  type: "button",
                  id: `file-open-${displayName}`,
                  props: {
                    text: "Open",
                    variant: "ghost",
                    size: "small",
                  },
                  on_event: {
                    click: isDirectory ? "filesystem.list" : "filesystem.read",
                  },
                },
              ],
            },
          ],
          on_event: {
            click: isDirectory ? "filesystem.list" : "ui.set",
          },
        };
      };

      // Notify parent to update UI spec with populated file list
      requestAnimationFrame(() => {
        window.postMessage(
          {
            type: "update_dynamic_lists",
            lists: {
              "file-list": files.map(createFileRow),
            },
          },
          "*"
        );
      });

      return { files, path, count: files.length };
    } catch (error) {
      logger.error("Failed to list directory", error as Error, {
        component: "FilesystemExecutor",
        path,
      });
      this.context.componentState.set(
        "error",
        `Failed to list directory: ${error instanceof Error ? error.message : "Unknown error"}`
      );
      return { files: [], path, count: 0 };
    }
  }
}
