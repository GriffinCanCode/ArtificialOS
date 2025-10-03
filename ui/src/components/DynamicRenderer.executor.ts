/**
 * DynamicRenderer Tool Executor
 * Handles execution of tools within dynamic components
 */

import { logger } from "../utils/monitoring/logger";
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
      // Validate numeric inputs
      if (event.key.includes('age') || event.key.includes('count') || event.key.includes('quantity')) {
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
      if (typeof event.newValue === 'string') {
        return {
          ...event,
          newValue: event.newValue.trim(),
        };
      }

      return event;
    });

    // Setup computed values for common patterns
    this.componentState.defineComputed('hasErrors', ['error.*'], (state) => {
      for (const [key, value] of state.entries()) {
        if (key.startsWith('error.') && value) {
          return true;
        }
      }
      return false;
    });

    // Log all state changes in development
    if (process.env.NODE_ENV === 'development') {
      this.componentState.subscribeWildcard('*', (event) => {
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
    this.activeTimers.forEach(timerId => {
      clearTimeout(timerId);
      clearInterval(timerId);
    });
    this.activeTimers.clear();
    logger.info("Cleaned up all active timers", {
      component: "ToolExecutor",
      count: this.activeTimers.size
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

      // Check if this is a service tool (contains service prefix like storage.*, auth.*, ai.*)
      const servicePrefixes = ["storage", "auth", "ai", "sync", "media"];
      const [category] = toolId.split(".");

      let result;
      if (servicePrefixes.includes(category)) {
        result = await this.executeServiceTool(toolId, params);
      } else {
        // Handle built-in tools
        switch (category) {
          case "calc":
            result = this.executeCalcTool(toolId.split(".")[1], params);
            break;
          case "ui":
            result = this.executeUITool(toolId.split(".")[1], params);
            break;
          case "system":
            result = this.executeSystemTool(toolId.split(".")[1], params);
            break;
          case "app":
            result = await this.executeAppTool(toolId.split(".")[1], params);
            break;
          case "http":
            result = await this.executeNetworkTool(toolId.split(".")[1], params);
            break;
          case "timer":
            result = this.executeTimerTool(toolId.split(".")[1], params);
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
        const current = this.componentState.get("display", "0");
        const digit = params.digit || "";
        const newValue = current === "0" ? digit : current + digit;
        this.componentState.set("display", newValue);
        return newValue;
      case "clear":
        this.componentState.set("display", "0");
        return "0";
      case "evaluate":
        try {
          const expression = this.componentState.get("display", "0");
          // Simple eval (in production, use a proper math parser!)
          const result = eval(expression.replace("×", "*").replace("÷", "/").replace("−", "-"));
          this.componentState.set("display", String(result));
          return result;
        } catch {
          this.componentState.set("display", "Error");
          return "Error";
        }
      default:
        return null;
    }
  }

  private executeUITool(action: string, params: Record<string, any>): any {
    switch (action) {
      case "set_state":
        this.componentState.set(params.key, params.value);
        return params.value;
      case "get_state":
        return this.componentState.get(params.key);
      case "set_multiple":
        // Batch update for efficiency
        this.componentState.setMultiple(params.values || {});
        return params.values;
      case "add_todo":
        // Use batch updates for multiple state changes
        this.componentState.batch(() => {
          const todos = this.componentState.get<Array<{ id: number; text: string; done: boolean }>>("todos", []);
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
          const todos = this.componentState.get<Array<{ id: number; text: string; done: boolean }>>("todos", []);
          const todo = todos.find(t => t.id === params.id);
          if (todo) {
            todo.done = !todo.done;
            this.componentState.set("todos", [...todos]);
          }
        });
        return this.componentState.get("todos");
      case "delete_todo":
        this.componentState.batch(() => {
          const todos = this.componentState.get<Array<{ id: number; text: string; done: boolean }>>("todos", []);
          const filtered = todos.filter(t => t.id !== params.id);
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
            ui_spec: data.ui_spec,
          },
          "*"
        );

        return data.ui_spec;

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
          // Execute the action (would need tool executor reference)
          logger.debug("Executing delayed action", {
            component: "TimerTool",
            action: params.action,
          });
          // Remove from active timers after execution
          this.activeTimers.delete(timerId);
        }, params.delay) as unknown as number;
        
        // Track this timer for cleanup
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
        
        // Track this interval for cleanup
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
}

