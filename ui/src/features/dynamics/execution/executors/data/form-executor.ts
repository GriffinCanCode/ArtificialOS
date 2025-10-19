/**
 * Form Tool Executor
 * Handles form validation and submission
 */

import { logger } from "../../../../../core/monitoring/core/logger";
import { ExecutorContext, BaseExecutor } from "../core/types";

export class FormExecutor implements BaseExecutor {
  private context: ExecutorContext;

  constructor(context: ExecutorContext) {
    this.context = context;
  }

  execute(action: string, params: Record<string, any>): any {
    const formId = params.form_id || "form";

    switch (action) {
      case "validate":
        const formData = this.context.componentState.get(`${formId}_data`, {});
        const errors: Record<string, string> = {};

        // Basic validation logic
        Object.entries(formData).forEach(([key, value]) => {
          if (!value || (typeof value === "string" && value.trim() === "")) {
            errors[key] = "This field is required";
          }
        });

        this.context.componentState.set(`${formId}_errors`, errors);
        this.context.componentState.set(`${formId}_valid`, Object.keys(errors).length === 0);

        logger.debug("Form validated", {
          component: "FormExecutor",
          formId,
          errorCount: Object.keys(errors).length,
        });
        return Object.keys(errors).length === 0;

      case "submit":
        const isValid = this.context.componentState.get(`${formId}_valid`, false);
        if (isValid) {
          const data = this.context.componentState.get(`${formId}_data`, {});
          logger.debug("Form submitted", { component: "FormExecutor", formId });
          return data;
        }
        return null;

      case "reset":
        this.context.componentState.batch(() => {
          this.context.componentState.set(`${formId}_data`, {});
          this.context.componentState.set(`${formId}_errors`, {});
          this.context.componentState.set(`${formId}_valid`, false);
        });
        logger.debug("Form reset", { component: "FormExecutor", formId });
        return true;

      default:
        return null;
    }
  }
}
