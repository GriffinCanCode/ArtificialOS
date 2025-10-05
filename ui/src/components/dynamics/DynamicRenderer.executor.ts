/**
 * DynamicRenderer Tool Executor
 * Handles execution of tools within dynamic components
 */

import { logger } from "../../utils/monitoring/logger";
import { ComponentState } from "./DynamicRenderer.state";

// ============================================================================
// Tool Execution
// ============================================================================

export class ToolExecutor {
  private componentState: ComponentState;
  private appId: string | null = null;
  private activeTimers: Set<number> = new Set(); // Track active timers for cleanup

  constructor(componentState: ComponentState) {
    this.componentState = componentState;
    this.setupStateManagement();
  }

  /**
   * Setup advanced state management features
   */
  private setupStateManagement(): void {
    // Add validation middleware
    this.componentState.use((event) => {
      // Validate numeric inputs - but only for specific numeric fields, not display text
      if (
        (event.key.includes("age") ||
        event.key.includes("quantity")) &&
        typeof event.newValue !== "string"
      ) {
        const num = Number(event.newValue);
        if (isNaN(num) || num < 0) {
          logger.warn("Invalid numeric value rejected", {
            component: "ToolExecutor",
            key: event.key,
            value: event.newValue,
          });
          return null; // Reject invalid values
        }
      }

      // Sanitize string inputs
      if (typeof event.newValue === "string") {
        return {
          ...event,
          newValue: event.newValue.trim(),
        };
      }

      return event;
    });

    // Setup computed values for common patterns
    this.componentState.defineComputed("hasErrors", ["error.*"], (state) => {
      for (const [key, value] of state.entries()) {
        if (key.startsWith("error.") && value) {
          return true;
        }
      }
      return false;
    });

    // Log all state changes in development
    if (process.env.NODE_ENV === "development") {
      this.componentState.subscribeWildcard("*", (event) => {
        logger.verboseThrottled("State change", {
          component: "ComponentState",
          key: event.key,
          oldValue: event.oldValue,
          newValue: event.newValue,
        });
      });
    }
  }

  setAppId(appId: string): void {
    this.appId = appId;
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
      component: "ToolExecutor",
      count: this.activeTimers.size,
    });
  }

  async execute(toolId: string, params: Record<string, any> = {}): Promise<any> {
    const startTime = performance.now();
    try {
      logger.debug("Executing tool", {
        component: "ToolExecutor",
        toolId,
        paramsCount: Object.keys(params).length,
      });

      // Check if this is a service tool (backend providers)
      const servicePrefixes = ["storage", "auth", "ai", "sync", "media", "scraper", "math"];
      const [category] = toolId.split(".");

      let result;
      if (servicePrefixes.includes(category)) {
        result = await this.executeServiceTool(toolId, params);
      } else if (category === "filesystem") {
        // Special handling for filesystem to support dynamic UI updates
        result = await this.executeFilesystemTool(toolId.split(".")[1], params);
      } else if (category === "system") {
        // Check if it's a service system tool or frontend system tool
        const action = toolId.split(".")[1];
        if (["log", "time", "info"].includes(action)) {
          result = await this.executeServiceTool(toolId, params);
        } else {
          result = this.executeSystemTool(action, params);
        }
      } else {
        // Handle built-in tools - prefer generic over specific
        switch (category) {
          case "ui":
            result = this.executeUITool(toolId.split(".")[1], params);
            break;
          case "system":
            result = this.executeSystemTool(toolId.split(".")[1], params);
            break;
          case "app":
            result = await this.executeAppTool(toolId.split(".")[1], params);
            break;
          case "hub":
            result = await this.executeHubTool(toolId.split(".")[1], params);
            break;
          case "http":
            result = await this.executeNetworkTool(toolId.split(".")[1], params);
            break;
          case "timer":
            result = this.executeTimerTool(toolId.split(".")[1], params);
            break;
          case "clipboard":
            result = await this.executeClipboardTool(toolId.split(".")[1], params);
            break;
          case "notification":
            result = this.executeNotificationTool(toolId.split(".")[1], params);
            break;
          
          // Legacy specific tools - kept for backward compatibility but deprecated
          case "calc":
            logger.warn("calc.* tools are deprecated, use ui.* tools instead", {
              component: "ToolExecutor",
              toolId,
            });
            result = this.executeCalcTool(toolId.split(".")[1], params);
            break;
          case "canvas":
            result = this.executeCanvasTool(toolId.split(".")[1], params);
            break;
          case "browser":
            result = this.executeBrowserTool(toolId.split(".")[1], params);
            break;
          case "player":
            result = this.executePlayerTool(toolId.split(".")[1], params);
            break;
          case "game":
            result = this.executeGameTool(toolId.split(".")[1], params);
            break;
          case "form":
            result = this.executeFormTool(toolId.split(".")[1], params);
            break;
          case "data":
            result = this.executeDataTool(toolId.split(".")[1], params);
            break;
          case "list":
            result = this.executeListTool(toolId.split(".")[1], params);
            break;
          case "navigation":
            result = this.executeNavigationTool(toolId.split(".")[1], params);
            break;
          default:
            logger.warn("Unknown tool category", {
              component: "ToolExecutor",
              category,
              toolId,
            });
            result = null;
        }
      }

      const duration = performance.now() - startTime;
      logger.performance("Tool execution", duration, {
        component: "ToolExecutor",
        toolId,
      });

      return result;
    } catch (error) {
      const duration = performance.now() - startTime;
      logger.error("Tool execution failed", error as Error, {
        component: "ToolExecutor",
        toolId,
        duration,
      });
      this.componentState.set(
        "error",
        `Tool ${toolId} failed: ${error instanceof Error ? error.message : "Unknown error"}`
      );
      return null;
    }
  }

  private async executeServiceTool(toolId: string, params: Record<string, any>): Promise<any> {
    logger.debug("Executing service tool", {
      component: "ToolExecutor",
      toolId,
    });

    try {
      const response = await fetch("http://localhost:8000/services/execute", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({
          tool_id: toolId,
          params: params,
          app_id: this.appId,
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
        component: "ToolExecutor",
        toolId,
        hasData: !!result.data,
      });
      return result.data;
    } catch (error) {
      logger.error("Service tool execution failed", error as Error, {
        component: "ToolExecutor",
        toolId,
      });
      throw error;
    }
  }

  private executeCalcTool(action: string, params: Record<string, any>): any {
    const a = params.a || 0;
    const b = params.b || 0;

    switch (action) {
      case "add":
        return a + b;
      case "subtract":
        return a - b;
      case "multiply":
        return a * b;
      case "divide":
        return b !== 0 ? a / b : "Error";
      case "append_digit":
      case "append":
        const current = this.componentState.get("display", "0");
        const digit = params.digit || params.value || "";
        
        // If current display is "0" and we're appending a digit (not an operator), replace it
        const isOperator = ["+", "-", "*", "/", "×", "÷", "−"].includes(digit);
        const newValue = (current === "0" && !isOperator) ? digit : current + digit;
        
        this.componentState.set("display", newValue);
        logger.debug("Calculator append", { component: "CalcTool", current, digit, newValue });
        return newValue;
      case "clear":
        this.componentState.set("display", "0");
        logger.debug("Calculator cleared", { component: "CalcTool" });
        return "0";
      case "backspace":
        const currentDisplay = this.componentState.get("display", "0");
        const newDisplay = currentDisplay.length > 1 ? currentDisplay.slice(0, -1) : "0";
        this.componentState.set("display", newDisplay);
        return newDisplay;
      case "evaluate":
        try {
          const expression = this.componentState.get("display", "0");
          // Simple eval (in production, use a proper math parser!)
          const sanitized = expression.replace(/×/g, "*").replace(/÷/g, "/").replace(/−/g, "-");
          const result = eval(sanitized);
          this.componentState.set("display", String(result));
          logger.debug("Calculator evaluated", { component: "CalcTool", expression, result });
          return result;
        } catch (error) {
          this.componentState.set("display", "Error");
          logger.error("Calculator evaluation error", error as Error, { component: "CalcTool" });
          return "Error";
        }
      default:
        return null;
    }
  }

  private executeUITool(action: string, params: Record<string, any>): any {
    switch (action) {
      case "set_state":
      case "set":
        const key = params.key || params.target || params.id;
        const value = params.value ?? params.data;
        if (!key) {
          logger.warn("set_state requires key parameter", { component: "UITool", params });
          return null;
        }
        this.componentState.set(key, value);
        logger.debug("State set", { component: "UITool", key, value });
        return value;
        
      case "get_state":
      case "get":
        const getKey = params.key || params.target || params.id;
        return this.componentState.get(getKey);
        
      case "set_multiple":
        // Batch update for efficiency
        this.componentState.setMultiple(params.values || {});
        return params.values;
        
      case "append":
        // Generic append - works for calculator, text fields, any string concatenation
        const appendKey = params.key || params.target || "display";
        const currentVal = String(this.componentState.get(appendKey, "0"));
        const appendVal = String(params.value ?? params.text ?? params.digit ?? "");
        
        if (!appendVal) {
          logger.warn("append called with empty value", { component: "UITool", params });
          return currentVal;
        }
        
        // Smart appending: if current is "0" and appending a digit, replace it
        const isOperator = ["+", "-", "*", "/", "×", "÷", "−", "(", ")"].includes(appendVal);
        const isNumeric = /^\d+$/.test(appendVal);
        const newVal = (currentVal === "0" && !isOperator && isNumeric) 
          ? appendVal 
          : currentVal + appendVal;
        
        this.componentState.set(appendKey, newVal);
        logger.debug("Value appended", { component: "UITool", key: appendKey, currentVal, appendVal, newVal });
        return newVal;
        
      case "clear":
        // Generic clear - works for any field
        const clearKey = params.key || params.target || "display";
        const defaultVal = params.default || "0";
        this.componentState.set(clearKey, defaultVal);
        logger.debug("Value cleared", { component: "UITool", key: clearKey, defaultVal });
        return defaultVal;
        
      case "compute":
      case "evaluate":
        // Generic expression evaluation
        const computeKey = params.key || params.target || "display";
        const expression = params.expression || this.componentState.get(computeKey, "");
        
        try {
          // Sanitize and evaluate
          const sanitized = String(expression)
            .replace(/×/g, "*")
            .replace(/÷/g, "/")
            .replace(/−/g, "-");
          
          // Basic security: only allow numbers, operators, parentheses, and whitespace
          if (!/^[\d\s+\-*/.()]+$/.test(sanitized)) {
            throw new Error("Invalid expression");
          }
          
          const result = eval(sanitized);
          const resultStr = String(result);
          
          this.componentState.set(computeKey, resultStr);
          logger.debug("Expression evaluated", { 
            component: "UITool", 
            key: computeKey, 
            expression, 
            result: resultStr 
          });
          return resultStr;
        } catch (error) {
          const errorMsg = "Error";
          this.componentState.set(computeKey, errorMsg);
          logger.error("Expression evaluation failed", error as Error, {
            component: "UITool",
            expression,
          });
          return errorMsg;
        }
        
      case "toggle":
        // Generic boolean toggle
        const toggleKey = params.key || params.target;
        if (!toggleKey) {
          logger.warn("toggle requires key parameter", { component: "UITool", params });
          return null;
        }
        const currentToggle = this.componentState.get(toggleKey, false);
        const newToggle = !currentToggle;
        this.componentState.set(toggleKey, newToggle);
        logger.debug("Value toggled", { component: "UITool", key: toggleKey, value: newToggle });
        return newToggle;
        
      case "backspace":
        // Generic backspace - remove last character
        const backspaceKey = params.key || params.target || "display";
        const currentBack = this.componentState.get(backspaceKey, "0");
        const newBack = currentBack.length > 1 ? currentBack.slice(0, -1) : "0";
        this.componentState.set(backspaceKey, newBack);
        logger.debug("Backspace applied", { component: "UITool", key: backspaceKey, newValue: newBack });
        return newBack;
      case "add_todo":
        // Use batch updates for multiple state changes
        this.componentState.batch(() => {
          const todos = this.componentState.get<Array<{ id: number; text: string; done: boolean }>>(
            "todos",
            []
          );
          const newTask = this.componentState.get<string>("task-input", "");
          if (newTask.trim()) {
            todos.push({ id: Date.now(), text: newTask, done: false });
            this.componentState.set("todos", [...todos]);
            this.componentState.set("task-input", "");
            this.componentState.set("todos.lastAdded", Date.now());
          }
        });
        return this.componentState.get("todos");
      case "toggle_todo":
        this.componentState.batch(() => {
          const todos = this.componentState.get<Array<{ id: number; text: string; done: boolean }>>(
            "todos",
            []
          );
          const todo = todos.find((t) => t.id === params.id);
          if (todo) {
            todo.done = !todo.done;
            this.componentState.set("todos", [...todos]);
          }
        });
        return this.componentState.get("todos");
      case "delete_todo":
        this.componentState.batch(() => {
          const todos = this.componentState.get<Array<{ id: number; text: string; done: boolean }>>(
            "todos",
            []
          );
          const filtered = todos.filter((t) => t.id !== params.id);
          this.componentState.set("todos", filtered);
          this.componentState.set("todos.lastDeleted", Date.now());
        });
        return this.componentState.get("todos");
      default:
        return null;
    }
  }

  private executeSystemTool(action: string, params: Record<string, any>): any {
    switch (action) {
      case "alert":
        alert(params.message);
        return true;
      case "log":
        logger.info("System log", { component: "SystemTool", message: params.message });
        return true;
      case "undo":
        return this.executeUndo();
      case "redo":
        return this.executeRedo();
      case "get_history":
        return this.componentState.getHistory();
      case "get_state_snapshot":
        return this.componentState.getAll();
      case "restore_state":
        // Restore state from snapshot
        if (params.snapshot) {
          this.componentState.batch(() => {
            this.componentState.clear();
            Object.entries(params.snapshot).forEach(([key, value]) => {
              this.componentState.set(key, value);
            });
          });
        }
        return true;
      default:
        return null;
    }
  }

  /**
   * Execute undo operation
   */
  private executeUndo(): boolean {
    const history = this.componentState.getHistory();
    if (history.length < 2) return false;

    // Get the timestamp before the last change
    const previousTimestamp = history[history.length - 2].timestamp;
    this.componentState.restoreToTimestamp(previousTimestamp);

    logger.info("Undo executed", {
      component: "ToolExecutor",
      restoredTo: previousTimestamp,
    });

    return true;
  }

  /**
   * Execute redo operation (requires maintaining separate redo stack)
   */
  private executeRedo(): boolean {
    // Note: Full redo requires maintaining a separate redo stack
    // This is a simplified version
    logger.warn("Redo not fully implemented", {
      component: "ToolExecutor",
    });
    return false;
  }

  private async executeAppTool(action: string, params: Record<string, any>): Promise<any> {
    switch (action) {
      case "spawn":
        logger.info("Spawning new app", {
          component: "AppTool",
          request: params.request,
        });
        // Use HTTP endpoint for app spawning instead of creating new WebSocket
        const response = await fetch("http://localhost:8000/generate-ui", {
          method: "POST",
          headers: { "Content-Type": "application/json" },
          body: JSON.stringify({
            message: params.request,
            context: { parent_app_id: this.componentState.get("app_id") },
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
        logger.info("Closing current app", { component: "AppTool" });
        // Save state before closing if requested
        if (params.save_state) {
          await this.persistState();
        }
        // Notify parent to close this app
        window.postMessage({ type: "close_app" }, "*");
        return true;

      case "list":
        logger.info("Listing apps", { component: "AppTool" });
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
      const appId = this.componentState.get("app_id");
      if (!appId) {
        logger.warn("Cannot persist state: no app_id", {
          component: "ToolExecutor",
        });
        return false;
      }

      const stateSnapshot = this.componentState.getAll();

      const response = await fetch(`http://localhost:8000/apps/${appId}/state`, {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ state: stateSnapshot }),
      });

      if (!response.ok) {
        throw new Error("Failed to persist state");
      }

      logger.info("State persisted successfully", {
        component: "ToolExecutor",
        appId,
        stateSize: Object.keys(stateSnapshot).length,
      });

      return true;
    } catch (error) {
      logger.error("Failed to persist state", error as Error, {
        component: "ToolExecutor",
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
        this.componentState.batch(() => {
          Object.entries(data.state).forEach(([key, value]) => {
            this.componentState.set(key, value as any);
          });
        });

        logger.info("State loaded successfully", {
          component: "ToolExecutor",
          appId,
          stateSize: Object.keys(data.state).length,
        });

        return true;
      }

      return false;
    } catch (error) {
      logger.error("Failed to load state", error as Error, {
        component: "ToolExecutor",
        appId,
      });
      return false;
    }
  }

  // Storage tools now handled by service system via executeServiceTool()

  private async executeNetworkTool(action: string, params: Record<string, any>): Promise<any> {
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

  private executeTimerTool(action: string, params: Record<string, any>): any {
    switch (action) {
      case "set":
        const timerId = setTimeout(() => {
          logger.debug("Executing delayed action", {
            component: "TimerTool",
            action: params.action,
          });
          this.activeTimers.delete(timerId);
        }, params.delay) as unknown as number;

        this.activeTimers.add(timerId);
        this.componentState.set(`timer_${timerId}`, timerId);
        return timerId;

      case "interval":
        const intervalId = setInterval(() => {
          logger.debug("Executing interval action", {
            component: "TimerTool",
            action: params.action,
          });
        }, params.interval) as unknown as number;

        this.activeTimers.add(intervalId);
        this.componentState.set(`interval_${intervalId}`, intervalId);
        return intervalId;

      case "clear":
        const id = this.componentState.get(params.timer_id);
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

  private executeCanvasTool(action: string, params: Record<string, any>): any {
    const canvasId = params.canvas_id || "canvas";
    const canvas = this.componentState.get(`${canvasId}_canvas`) as HTMLCanvasElement | undefined;

    if (!canvas) {
      logger.warn("Canvas element not found", {
        component: "CanvasTool",
        canvasId,
      });
      return null;
    }

    const ctx = canvas.getContext("2d");
    if (!ctx) return null;

    switch (action) {
      case "init":
        this.componentState.set(`${canvasId}_ctx`, ctx);
        this.componentState.set(`${canvasId}_tool`, "pen");
        this.componentState.set(`${canvasId}_color`, "black");
        this.componentState.set(`${canvasId}_brushSize`, 5);
        logger.debug("Canvas initialized", { component: "CanvasTool", canvasId });
        return true;

      case "draw":
        const operation = params.operation || "stroke";
        const data = params.data || {};

        ctx.strokeStyle = this.componentState.get(`${canvasId}_color`, "black");
        ctx.lineWidth = this.componentState.get(`${canvasId}_brushSize`, 5);
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
        logger.debug("Canvas cleared", { component: "CanvasTool", canvasId });
        return true;

      case "setTool":
        this.componentState.set(`${canvasId}_tool`, params.tool);
        return params.tool;

      case "setColor":
        this.componentState.set(`${canvasId}_color`, params.color);
        return params.color;

      case "setBrushSize":
        this.componentState.set(`${canvasId}_brushSize`, params.size);
        return params.size;

      default:
        return null;
    }
  }

  private executeBrowserTool(action: string, params: Record<string, any>): any {
    const iframeId = params.iframe_id || "webpage";

    switch (action) {
      case "navigate":
        // Get URL from params or from a URL input field in state
        let url = params.url;
        if (!url) {
          // Try common input field names
          url = this.componentState.get("url-input") || 
                this.componentState.get("search-input") ||
                this.componentState.get("browser-url") ||
                this.componentState.get("address-bar");
        }
        
        if (!url) {
          logger.warn("No URL provided for browser navigation", { component: "BrowserTool" });
          return null;
        }
        
        // Add https:// if no protocol specified and not a search query
        if (!url.startsWith("http://") && !url.startsWith("https://")) {
          // If it looks like a search query (has spaces or no dots), use Google search
          if (url.includes(" ") || !url.includes(".")) {
            url = `https://www.google.com/search?q=${encodeURIComponent(url)}`;
          } else {
            url = `https://${url}`;
          }
        }
        
        this.componentState.set(`${iframeId}_url`, url);
        logger.debug("Browser navigating", { component: "BrowserTool", url });
        return url;

      case "back":
        // Browser back - would need iframe history API or parent navigation
        logger.debug("Browser back", { component: "BrowserTool" });
        return true;

      case "forward":
        logger.debug("Browser forward", { component: "BrowserTool" });
        return true;

      case "refresh":
        const currentUrl = this.componentState.get(`${iframeId}_url`);
        this.componentState.set(`${iframeId}_url`, currentUrl + "?refresh=" + Date.now());
        logger.debug("Browser refresh", { component: "BrowserTool" });
        return true;

      default:
        return null;
    }
  }

  private executePlayerTool(action: string, params: Record<string, any>): any {
    const mediaId = params.media_id || "player";
    const mediaElement = this.componentState.get(`${mediaId}_element`) as
      | HTMLMediaElement
      | undefined;

    switch (action) {
      case "play":
        if (mediaElement) {
          mediaElement.play();
          this.componentState.set(`${mediaId}_playing`, true);
        }
        logger.debug("Media playing", { component: "PlayerTool", mediaId });
        return true;

      case "pause":
        if (mediaElement) {
          mediaElement.pause();
          this.componentState.set(`${mediaId}_playing`, false);
        }
        logger.debug("Media paused", { component: "PlayerTool", mediaId });
        return true;

      case "stop":
        if (mediaElement) {
          mediaElement.pause();
          mediaElement.currentTime = 0;
          this.componentState.set(`${mediaId}_playing`, false);
        }
        logger.debug("Media stopped", { component: "PlayerTool", mediaId });
        return true;

      case "next":
        this.componentState.set(
          `${mediaId}_trackIndex`,
          this.componentState.get(`${mediaId}_trackIndex`, 0) + 1
        );
        logger.debug("Next track", { component: "PlayerTool" });
        return true;

      case "previous":
        this.componentState.set(
          `${mediaId}_trackIndex`,
          Math.max(0, this.componentState.get(`${mediaId}_trackIndex`, 0) - 1)
        );
        logger.debug("Previous track", { component: "PlayerTool" });
        return true;

      case "seek":
        if (mediaElement) {
          mediaElement.currentTime = params.time;
        }
        return params.time;

      case "setVolume":
        if (mediaElement) {
          mediaElement.volume = params.volume / 100;
        }
        this.componentState.set(`${mediaId}_volume`, params.volume);
        return params.volume;

      default:
        return null;
    }
  }

  private executeGameTool(action: string, params: Record<string, any>): any {
    switch (action) {
      case "move":
        const position = params.position;
        const gameState = this.componentState.get("game_state", {});

        // Update game state with move
        this.componentState.set("game_state", { ...gameState, lastMove: position });
        this.componentState.set(`cell_${position}`, params.data);

        logger.debug("Game move", { component: "GameTool", position });
        return true;

      case "reset":
        this.componentState.set("game_state", {});
        this.componentState.batch(() => {
          // Clear all cell states
          for (let i = 0; i < 9; i++) {
            this.componentState.set(`cell_${i}`, "");
          }
        });
        logger.debug("Game reset", { component: "GameTool" });
        return true;

      case "score":
        const currentScore = this.componentState.get(`score_${params.player}`, 0);
        this.componentState.set(`score_${params.player}`, currentScore + params.score);
        return currentScore + params.score;

      default:
        return null;
    }
  }

  private async executeClipboardTool(action: string, params: Record<string, any>): Promise<any> {
    switch (action) {
      case "copy":
        try {
          await navigator.clipboard.writeText(params.text);
          logger.debug("Text copied to clipboard", { component: "ClipboardTool" });
          return true;
        } catch (error) {
          logger.error("Failed to copy to clipboard", error as Error, {
            component: "ClipboardTool",
          });
          return false;
        }

      case "paste":
        try {
          const text = await navigator.clipboard.readText();
          logger.debug("Text pasted from clipboard", { component: "ClipboardTool" });
          return text;
        } catch (error) {
          logger.error("Failed to paste from clipboard", error as Error, {
            component: "ClipboardTool",
          });
          return null;
        }

      default:
        return null;
    }
  }

  private executeNotificationTool(action: string, params: Record<string, any>): any {
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
        logger.debug("Notification shown", { component: "NotificationTool" });
        return true;

      default:
        return null;
    }
  }

  private executeFormTool(action: string, params: Record<string, any>): any {
    const formId = params.form_id || "form";

    switch (action) {
      case "validate":
        const formData = this.componentState.get(`${formId}_data`, {});
        const errors: Record<string, string> = {};

        // Basic validation logic
        Object.entries(formData).forEach(([key, value]) => {
          if (!value || (typeof value === "string" && value.trim() === "")) {
            errors[key] = "This field is required";
          }
        });

        this.componentState.set(`${formId}_errors`, errors);
        this.componentState.set(`${formId}_valid`, Object.keys(errors).length === 0);

        logger.debug("Form validated", {
          component: "FormTool",
          formId,
          errorCount: Object.keys(errors).length,
        });
        return Object.keys(errors).length === 0;

      case "submit":
        const isValid = this.componentState.get(`${formId}_valid`, false);
        if (isValid) {
          const data = this.componentState.get(`${formId}_data`, {});
          logger.debug("Form submitted", { component: "FormTool", formId });
          return data;
        }
        return null;

      case "reset":
        this.componentState.batch(() => {
          this.componentState.set(`${formId}_data`, {});
          this.componentState.set(`${formId}_errors`, {});
          this.componentState.set(`${formId}_valid`, false);
        });
        logger.debug("Form reset", { component: "FormTool", formId });
        return true;

      default:
        return null;
    }
  }

  private executeDataTool(action: string, params: Record<string, any>): any {
    switch (action) {
      case "filter":
        const data = params.data || [];
        const filter = params.filter || "";
        const filtered = data.filter((item: any) =>
          JSON.stringify(item).toLowerCase().includes(filter.toLowerCase())
        );
        logger.debug("Data filtered", {
          component: "DataTool",
          originalCount: data.length,
          filteredCount: filtered.length,
        });
        return filtered;

      case "sort":
        const dataToSort = params.data || [];
        const field = params.field || "id";
        const order = params.order || "asc";

        const sorted = [...dataToSort].sort((a, b) => {
          const aVal = a[field];
          const bVal = b[field];

          if (order === "asc") {
            return aVal > bVal ? 1 : -1;
          } else {
            return aVal < bVal ? 1 : -1;
          }
        });

        logger.debug("Data sorted", {
          component: "DataTool",
          field,
          order,
          count: sorted.length,
        });
        return sorted;

      case "search":
        const searchData = params.data || [];
        const query = params.query || "";
        const results = searchData.filter((item: any) =>
          JSON.stringify(item).toLowerCase().includes(query.toLowerCase())
        );
        logger.debug("Data searched", {
          component: "DataTool",
          query,
          resultCount: results.length,
        });
        return results;

      default:
        return null;
    }
  }

  private executeListTool(action: string, params: Record<string, any>): any {
    const listId = params.list_id || "list";

    switch (action) {
      case "add":
        this.componentState.batch(() => {
          const items = this.componentState.get<any[]>(listId, []);
          items.push(params.item);
          this.componentState.set(listId, [...items]);
        });
        logger.debug("List item added", { component: "ListTool", listId });
        return this.componentState.get(listId);

      case "remove":
        this.componentState.batch(() => {
          const items = this.componentState.get<any[]>(listId, []);
          const filtered = items.filter((item: any) => item.id !== params.item_id);
          this.componentState.set(listId, filtered);
        });
        logger.debug("List item removed", { component: "ListTool", listId });
        return this.componentState.get(listId);

      case "toggle":
        this.componentState.batch(() => {
          const items = this.componentState.get<any[]>(listId, []);
          const item = items.find((i: any) => i.id === params.item_id);
          if (item) {
            item.done = !item.done;
            this.componentState.set(listId, [...items]);
          }
        });
        logger.debug("List item toggled", { component: "ListTool", listId });
        return this.componentState.get(listId);

      case "clear":
        this.componentState.set(listId, []);
        logger.debug("List cleared", { component: "ListTool", listId });
        return [];

      default:
        return null;
    }
  }

  private executeNavigationTool(action: string, params: Record<string, any>): any {
    const fullAction = action.includes(".") ? action : `navigation.${action}`;

    if (fullAction.startsWith("tabs.")) {
      const tabAction = fullAction.split(".")[1];
      const tabsId = params.tabs_id || "tabs";

      if (tabAction === "switch") {
        this.componentState.set(tabsId, params.tab_id);
        logger.debug("Tab switched", { component: "NavigationTool", tabId: params.tab_id });
        return params.tab_id;
      }
    }

    if (fullAction.startsWith("modal.")) {
      const modalAction = fullAction.split(".")[1];
      const modalId = params.modal_id || "modal";

      if (modalAction === "open") {
        this.componentState.set(modalId, true);
        logger.debug("Modal opened", { component: "NavigationTool", modalId });
        return true;
      } else if (modalAction === "close") {
        this.componentState.set(modalId, false);
        logger.debug("Modal closed", { component: "NavigationTool", modalId });
        return false;
      }
    }

    return null;
  }

  private async executeHubTool(action: string, params: Record<string, any>): Promise<any> {
    switch (action) {
      case "load_apps":
        logger.info("Loading apps from registry", { component: "HubTool" });
        try {
          const response = await fetch("http://localhost:8000/registry/apps");
          const data = await response.json();
          
          // Store apps in state
          this.componentState.set("hub_apps", data.apps || []);
          this.componentState.set("hub_stats", data.stats || {});
          
          // Update count displays
          const allCount = data.apps?.length || 0;
          this.componentState.set("all-count", `${allCount} apps available`);
          
          // Filter by category
          const systemApps = (data.apps || []).filter((app: any) => app.category === "system");
          const productivityApps = (data.apps || []).filter((app: any) => app.category === "productivity");
          const utilitiesApps = (data.apps || []).filter((app: any) => app.category === "utilities");
          
          this.componentState.set("system-count", `${systemApps.length} apps`);
          this.componentState.set("productivity-count", `${productivityApps.length} apps`);
          
          logger.info("Apps loaded successfully", {
            component: "HubTool",
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
            component: "HubTool",
          });
          return [];
        }

      case "launch_app":
        const appId = params.app_id || params.id;
        if (!appId) {
          logger.warn("No app_id provided for launch", { component: "HubTool" });
          return null;
        }

        logger.info("Launching app from registry", {
          component: "HubTool",
          appId,
        });

        try {
          const response = await fetch(
            `http://localhost:8000/registry/apps/${appId}/launch`,
            { method: "POST" }
          );
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
            component: "HubTool",
            appId,
            launchedAppId: data.app_id,
          });

          return data;
        } catch (error) {
          logger.error("Failed to launch app", error as Error, {
            component: "HubTool",
            appId,
          });
          return null;
        }

      default:
        return null;
    }
  }

  private async executeFilesystemTool(action: string, params: Record<string, any>): Promise<any> {
    switch (action) {
      case "list":
        // Handle path from multiple sources: explicit param, component state, or clicked file
        let path = params.path || this.componentState.get("path-input") || this.componentState.get("current-path") || "/tmp/ai-os-storage";
        
        // If a file/directory was clicked, extract the name and append to current path
        if (params.value && params.value.includes("📁")) {
          const currentPath = this.componentState.get("current-path") || "/tmp/ai-os-storage";
          const dirName = params.value.replace("📁 ", "").trim();
          path = `${currentPath}/${dirName}`.replace(/\/+/g, "/"); // Clean up double slashes
        }
        
        logger.info("Listing directory", { component: "FilesystemTool", path });
        
        try {
          const response = await fetch("http://localhost:8000/services/execute", {
            method: "POST",
            headers: { "Content-Type": "application/json" },
            body: JSON.stringify({
              tool_id: "filesystem.list",
              params: { path },
              app_id: this.appId,
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
          this.componentState.set("current-path", path);
          this.componentState.set("path-input", path);
          this.componentState.set("item-count", `${files.length} items`);
          
          logger.info("Directory listed successfully", {
            component: "FilesystemTool",
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
                  borderBottom: "1px solid rgba(255,255,255,0.05)",
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
            component: "FilesystemTool",
            path,
          });
          this.componentState.set("error", `Failed to list directory: ${error instanceof Error ? error.message : "Unknown error"}`);
          return { files: [], path, count: 0 };
        }

      case "mkdir":
      case "create":
      case "read":
      case "write":
      case "delete":
      case "stat":
      case "exists":
      case "move":
      case "copy":
        // For other filesystem operations, use the generic service tool executor
        return await this.executeServiceTool(`filesystem.${action}`, params);

      default:
        return null;
    }
  }
}
