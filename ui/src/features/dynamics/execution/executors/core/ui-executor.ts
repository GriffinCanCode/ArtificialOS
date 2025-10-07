/**
 * UI Tool Executor
 * Handles generic UI state manipulation
 */

import { evaluateExpression } from "../../../../../core/utils/math";
import { logger } from "../../../../../core/utils/monitoring/logger";
import { ExecutorContext, BaseExecutor } from "./types";

export class UIExecutor implements BaseExecutor {
  private context: ExecutorContext;

  constructor(context: ExecutorContext) {
    this.context = context;
  }

  execute(action: string, params: Record<string, any>): any {
    switch (action) {
      case "set_state":
      case "set":
        const key = params.key || params.target || params.id;
        const value = params.value ?? params.data;
        if (!key) {
          logger.warn("set_state requires key parameter", { component: "UIExecutor", params });
          return null;
        }
        this.context.componentState.set(key, value);
        logger.debug("State set", { component: "UIExecutor", key, value });
        return value;

      case "get_state":
      case "get":
        const getKey = params.key || params.target || params.id;
        return this.context.componentState.get(getKey);

      case "set_multiple":
        // Batch update for efficiency
        this.context.componentState.setMultiple(params.values || {});
        return params.values;

      case "append":
        // Generic append - works for calculator, text fields, any string concatenation
        const appendKey = params.key || params.target || "display";
        const currentVal = String(this.context.componentState.get(appendKey, "0"));
        const appendVal = String(params.value ?? params.text ?? params.digit ?? "");

        if (!appendVal) {
          logger.warn("append called with empty value", { component: "UIExecutor", params });
          return currentVal;
        }

        // Smart appending: if current is "0" and appending a digit, replace it
        const isOperator = ["+", "-", "*", "/", "×", "÷", "−", "(", ")"].includes(appendVal);
        const isNumeric = /^\d+$/.test(appendVal);
        const newVal =
          currentVal === "0" && !isOperator && isNumeric ? appendVal : currentVal + appendVal;

        this.context.componentState.set(appendKey, newVal);
        logger.debug("Value appended", {
          component: "UIExecutor",
          key: appendKey,
          currentVal,
          appendVal,
          newVal,
        });
        return newVal;

      case "clear":
        // Generic clear - works for any field
        const clearKey = params.key || params.target || "display";
        const defaultVal = params.default || "0";
        this.context.componentState.set(clearKey, defaultVal);
        logger.debug("Value cleared", { component: "UIExecutor", key: clearKey, defaultVal });
        return defaultVal;

      case "compute":
      case "evaluate":
        // Generic expression evaluation
        const computeKey = params.key || params.target || "display";
        const expression = params.expression || this.context.componentState.get(computeKey, "");

        // Use secure math utility (replaces dangerous eval)
        // Supports advanced functions: sqrt, sin, cos, tan, log, exp, pi, e, etc.
        const resultStr = String(evaluateExpression(expression));

        this.context.componentState.set(computeKey, resultStr);
        logger.debug("Expression evaluated", {
          component: "UIExecutor",
          key: computeKey,
          expression,
          result: resultStr,
        });
        return resultStr;

      case "toggle":
        // Generic boolean toggle
        const toggleKey = params.key || params.target;
        if (!toggleKey) {
          logger.warn("toggle requires key parameter", { component: "UIExecutor", params });
          return null;
        }
        const currentToggle = this.context.componentState.get(toggleKey, false);
        const newToggle = !currentToggle;
        this.context.componentState.set(toggleKey, newToggle);
        logger.debug("Value toggled", {
          component: "UIExecutor",
          key: toggleKey,
          value: newToggle,
        });
        return newToggle;

      case "backspace":
        // Generic backspace - remove last character
        const backspaceKey = params.key || params.target || "display";
        const currentBack = this.context.componentState.get(backspaceKey, "0");
        const newBack = currentBack.length > 1 ? currentBack.slice(0, -1) : "0";
        this.context.componentState.set(backspaceKey, newBack);
        logger.debug("Backspace applied", {
          component: "UIExecutor",
          key: backspaceKey,
          newValue: newBack,
        });
        return newBack;

      case "add_todo":
        // Use batch updates for multiple state changes
        this.context.componentState.batch(() => {
          const todos = this.context.componentState.get<
            Array<{ id: number; text: string; done: boolean }>
          >("todos", []);
          const newTask = this.context.componentState.get<string>("task-input", "");
          if (newTask.trim()) {
            todos.push({ id: Date.now(), text: newTask, done: false });
            this.context.componentState.set("todos", [...todos]);
            this.context.componentState.set("task-input", "");
            this.context.componentState.set("todos.lastAdded", Date.now());
          }
        });
        return this.context.componentState.get("todos");

      case "toggle_todo":
        this.context.componentState.batch(() => {
          const todos = this.context.componentState.get<
            Array<{ id: number; text: string; done: boolean }>
          >("todos", []);
          const todo = todos.find((t) => t.id === params.id);
          if (todo) {
            todo.done = !todo.done;
            this.context.componentState.set("todos", [...todos]);
          }
        });
        return this.context.componentState.get("todos");

      case "delete_todo":
        this.context.componentState.batch(() => {
          const todos = this.context.componentState.get<
            Array<{ id: number; text: string; done: boolean }>
          >("todos", []);
          const filtered = todos.filter((t) => t.id !== params.id);
          this.context.componentState.set("todos", filtered);
          this.context.componentState.set("todos.lastDeleted", Date.now());
        });
        return this.context.componentState.get("todos");

      default:
        return null;
    }
  }
}
