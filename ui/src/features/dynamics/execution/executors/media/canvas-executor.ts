/**
 * Canvas Tool Executor
 * Handles canvas drawing operations
 */

import { logger } from "../../../../../core/utils/monitoring/logger";
import { ExecutorContext, BaseExecutor } from "../core/types";

export class CanvasExecutor implements BaseExecutor {
  private context: ExecutorContext;

  constructor(context: ExecutorContext) {
    this.context = context;
  }

  execute(action: string, params: Record<string, any>): any {
    const canvasId = params.canvas_id || "canvas";
    const canvas = this.context.componentState.get(`${canvasId}_canvas`) as
      | HTMLCanvasElement
      | undefined;

    if (!canvas) {
      logger.warn("Canvas element not found", {
        component: "CanvasExecutor",
        canvasId,
      });
      return null;
    }

    const ctx = canvas.getContext("2d");
    if (!ctx) return null;

    switch (action) {
      case "init":
        this.context.componentState.set(`${canvasId}_ctx`, ctx);
        this.context.componentState.set(`${canvasId}_tool`, "pen");
        this.context.componentState.set(`${canvasId}_color`, "black");
        this.context.componentState.set(`${canvasId}_brushSize`, 5);
        logger.debug("Canvas initialized", { component: "CanvasExecutor", canvasId });
        return true;

      case "draw":
        const operation = params.operation || "stroke";
        const data = params.data || {};

        ctx.strokeStyle = this.context.componentState.get(`${canvasId}_color`, "black");
        ctx.lineWidth = this.context.componentState.get(`${canvasId}_brushSize`, 5);
        ctx.lineCap = "round";

        if (operation === "stroke" && data.points) {
          ctx.beginPath();
          ctx.moveTo(data.points[0].x, data.points[0].y);
          data.points.forEach((point: { x: number; y: number }) => {
            ctx.lineTo(point.x, point.y);
          });
          ctx.stroke();
        }
        return true;

      case "clear":
        ctx.clearRect(0, 0, canvas.width, canvas.height);
        logger.debug("Canvas cleared", { component: "CanvasExecutor", canvasId });
        return true;

      case "setTool":
        this.context.componentState.set(`${canvasId}_tool`, params.tool);
        return params.tool;

      case "setColor":
        this.context.componentState.set(`${canvasId}_color`, params.color);
        return params.color;

      case "setBrushSize":
        this.context.componentState.set(`${canvasId}_brushSize`, params.size);
        return params.size;

      default:
        return null;
    }
  }
}
