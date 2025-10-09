/**
 * DynamicRenderer Tool Executor
 * Main coordinator for tool execution using modular executors
 */

import { logger } from "../../../core/utils/monitoring/logger";
import { ComponentState } from "../state/state";
import {
  ExecutorContext,
  ServiceExecutor,
  UIExecutor,
  SystemExecutor,
  AppExecutor,
  TimerExecutor,
  CanvasExecutor,
  BrowserExecutor,
  PlayerExecutor,
  GameExecutor,
  ClipboardExecutor,
  NotificationExecutor,
  ToastExecutor,
  FormExecutor,
  DataExecutor,
  ListExecutor,
  NavigationExecutor,
  HubExecutor,
  FilesystemExecutor,
  CalcExecutor,
  NotesExecutor,
  AnalysisExecutor,
  ChartExecutor,
  GraphExecutor,
} from "./executors";

// ============================================================================
// Tool Execution
// ============================================================================

export class ToolExecutor {
  private componentState: ComponentState;
  private appId: string | null = null;
  private context: ExecutorContext;

  // Modular executors
  private serviceExecutor: ServiceExecutor;
  private uiExecutor: UIExecutor;
  private systemExecutor: SystemExecutor;
  private appExecutor: AppExecutor;
  private timerExecutor: TimerExecutor;
  private canvasExecutor: CanvasExecutor;
  private browserExecutor: BrowserExecutor;
  private playerExecutor: PlayerExecutor;
  private gameExecutor: GameExecutor;
  private clipboardExecutor: ClipboardExecutor;
  private notificationExecutor: NotificationExecutor;
  private toastExecutor: ToastExecutor;
  private formExecutor: FormExecutor;
  private dataExecutor: DataExecutor;
  private listExecutor: ListExecutor;
  private navigationExecutor: NavigationExecutor;
  private hubExecutor: HubExecutor;
  private filesystemExecutor: FilesystemExecutor;
  private calcExecutor: CalcExecutor;
  private notesExecutor: NotesExecutor;
  private analysisExecutor: AnalysisExecutor;
  private chartExecutor: ChartExecutor;
  private graphExecutor: GraphExecutor;

  constructor(componentState: ComponentState) {
    this.componentState = componentState;
    this.context = {
      componentState: this.componentState,
      appId: this.appId,
    };

    // Initialize all executors
    this.serviceExecutor = new ServiceExecutor(this.context);
    this.uiExecutor = new UIExecutor(this.context);
    this.systemExecutor = new SystemExecutor(this.context);
    this.appExecutor = new AppExecutor(this.context);
    this.timerExecutor = new TimerExecutor(this.context);
    this.canvasExecutor = new CanvasExecutor(this.context);
    this.browserExecutor = new BrowserExecutor(this.context);
    this.playerExecutor = new PlayerExecutor(this.context);
    this.gameExecutor = new GameExecutor(this.context);
    this.clipboardExecutor = new ClipboardExecutor(this.context, this.serviceExecutor);
    this.notificationExecutor = new NotificationExecutor(this.context);
    this.toastExecutor = new ToastExecutor(this.context);
    this.formExecutor = new FormExecutor(this.context);
    this.dataExecutor = new DataExecutor(this.context);
    this.listExecutor = new ListExecutor(this.context);
    this.navigationExecutor = new NavigationExecutor(this.context);
    this.hubExecutor = new HubExecutor(this.context);
    this.filesystemExecutor = new FilesystemExecutor(this.context, this.serviceExecutor);
    this.calcExecutor = new CalcExecutor(this.context);
    this.notesExecutor = new NotesExecutor(this.context, this.serviceExecutor);
    this.analysisExecutor = new AnalysisExecutor(this.context);
    this.chartExecutor = new ChartExecutor(this.context);
    this.graphExecutor = new GraphExecutor(this.context);

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
        (event.key.includes("age") || event.key.includes("quantity")) &&
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

      // DON'T trim string inputs during typing - it strips spaces!
      // Trimming should happen on blur/submit, not on every keystroke
      // Users need to be able to type spaces naturally

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
    // Note: Conditional logging disabled to avoid build issues
    // Enable via logger configuration if needed
    // this.componentState.subscribeWildcard("*", (event) => {
    //   logger.verboseThrottled("State change", {
    //     component: "ComponentState",
    //     key: event.key,
    //     oldValue: event.oldValue,
    //     newValue: event.newValue,
    //   });
    // });
  }

  setAppId(appId: string): void {
    this.appId = appId;
    this.context.appId = appId;
  }

  /**
   * Clean up all active timers (call on unmount)
   */
  cleanup(): void {
    this.timerExecutor.cleanup();
    this.analysisExecutor.cleanup();
  }

  async execute(toolId: string, params: Record<string, any> = {}): Promise<any> {
    const startTime = performance.now();
    try {
      // Skip debug logging for high-frequency polling operations
      const isHighFrequencyTool = toolId === "terminal.read";

      if (!isHighFrequencyTool) {
        logger.debug("Executing tool", {
          component: "ToolExecutor",
          toolId,
          paramsCount: Object.keys(params).length,
        });
      }

      // Check if this is a service tool (backend providers)
      const servicePrefixes = [
        "storage",
        "auth",
        "ai",
        "sync",
        "media",
        "scraper",
        "math",
        "terminal",
        "settings",
        "permissions",
        "theme",
        "monitor",
        "http",
      ];
      const [category] = toolId.split(".");

      let result;
      if (servicePrefixes.includes(category)) {
        result = await this.serviceExecutor.execute(toolId, params);
      } else if (category === "filesystem") {
        // Special handling for filesystem to support dynamic UI updates
        result = await this.filesystemExecutor.execute(toolId.split(".")[1], params);
      } else if (category === "system") {
        // Check if it's a service system tool or frontend system tool
        const action = toolId.split(".")[1];
        if (["log", "time", "info"].includes(action)) {
          result = await this.serviceExecutor.execute(toolId, params);
        } else {
          result = this.systemExecutor.execute(action, params);
        }
      } else {
        // Handle built-in tools - delegate to appropriate executor
        switch (category) {
          case "ui":
            result = this.uiExecutor.execute(toolId.split(".")[1], params);
            break;
          case "app":
            result = await this.appExecutor.execute(toolId.split(".")[1], params);
            break;
          case "hub":
            result = await this.hubExecutor.execute(toolId.split(".")[1], params);
            break;
          case "timer":
            result = this.timerExecutor.execute(toolId.split(".")[1], params);
            break;
          case "clipboard":
            result = await this.clipboardExecutor.execute(toolId.split(".")[1], params);
            break;
          case "notification":
            result = this.notificationExecutor.execute(toolId.split(".")[1], params);
            break;
          case "toast":
            result = this.toastExecutor.execute(toolId.split(".")[1], params);
            break;
          case "notes":
            result = await this.notesExecutor.execute(toolId.split(".")[1], params);
            break;
          case "analysis":
            result = await this.analysisExecutor.execute(toolId.split(".")[1], params);
            break;
          case "chart":
            result = this.chartExecutor.execute(toolId.split(".")[1], params);
            break;
          case "graph":
            result = this.graphExecutor.execute(toolId.split(".")[1], params);
            break;

          // Legacy specific tools - kept for backward compatibility but deprecated
          case "calc":
            logger.warn("calc.* tools are deprecated, use ui.* tools instead", {
              component: "ToolExecutor",
              toolId,
            });
            result = this.calcExecutor.execute(toolId.split(".")[1], params);
            break;
          case "canvas":
            result = this.canvasExecutor.execute(toolId.split(".")[1], params);
            break;
          case "browser":
            result = this.browserExecutor.execute(toolId.split(".")[1], params);
            break;
          case "player":
            result = this.playerExecutor.execute(toolId.split(".")[1], params);
            break;
          case "game":
            result = this.gameExecutor.execute(toolId.split(".")[1], params);
            break;
          case "form":
            result = this.formExecutor.execute(toolId.split(".")[1], params);
            break;
          case "data":
            result = this.dataExecutor.execute(toolId.split(".")[1], params);
            break;
          case "list":
            result = this.listExecutor.execute(toolId.split(".")[1], params);
            break;
          case "navigation":
            result = this.navigationExecutor.execute(toolId.split(".")[1], params);
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

      // Skip performance logging for high-frequency polling operations
      if (!isHighFrequencyTool) {
        logger.performance("Tool execution", duration, {
          component: "ToolExecutor",
          toolId,
        });
      }

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
}
