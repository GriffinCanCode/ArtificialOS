/**
 * Calculator Tool Executor (Deprecated)
 * Kept for backward compatibility - use ui.* tools instead
 */

import { evaluateExpression } from "../../../../../core/utils/math";
import { logger } from "../../../../../core/monitoring/core/logger";
import { ExecutorContext, BaseExecutor } from "../core/types";

export class CalcExecutor implements BaseExecutor {
  private context: ExecutorContext;

  constructor(context: ExecutorContext) {
    this.context = context;
  }

  execute(action: string, params: Record<string, any>): any {
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
        const current = this.context.componentState.get("display", "0");
        const digit = params.digit || params.value || "";

        // If current display is "0" and we're appending a digit (not an operator), replace it
        const isOperator = ["+", "-", "*", "/", "×", "÷", "−"].includes(digit);
        const newValue = current === "0" && !isOperator ? digit : current + digit;

        this.context.componentState.set("display", newValue);
        logger.debug("Calculator append", { component: "CalcExecutor", current, digit, newValue });
        return newValue;

      case "clear":
        this.context.componentState.set("display", "0");
        logger.debug("Calculator cleared", { component: "CalcExecutor" });
        return "0";

      case "backspace":
        const currentDisplay = this.context.componentState.get("display", "0");
        const newDisplay = currentDisplay.length > 1 ? currentDisplay.slice(0, -1) : "0";
        this.context.componentState.set("display", newDisplay);
        return newDisplay;

      case "evaluate":
        const expression = this.context.componentState.get("display", "0");
        // Use secure math utility (replaces dangerous eval)
        const result = evaluateExpression(expression);
        this.context.componentState.set("display", String(result));
        logger.debug("Calculator evaluated", { component: "CalcExecutor", expression, result });
        return result;

      default:
        return null;
    }
  }
}
