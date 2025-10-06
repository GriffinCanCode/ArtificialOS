/**
 * Navigation Tool Executor
 * Handles navigation operations (tabs, modals)
 */

import { logger } from "../../../../../core/utils/monitoring/logger";
import { ExecutorContext, BaseExecutor } from "../core/types";

export class NavigationExecutor implements BaseExecutor {
  private context: ExecutorContext;

  constructor(context: ExecutorContext) {
    this.context = context;
  }

  execute(action: string, params: Record<string, any>): any {
    const fullAction = action.includes(".") ? action : `navigation.${action}`;

    if (fullAction.startsWith("tabs.")) {
      const tabAction = fullAction.split(".")[1];
      const tabsId = params.tabs_id || "tabs";

      if (tabAction === "switch") {
        this.context.componentState.set(tabsId, params.tab_id);
        logger.debug("Tab switched", { component: "NavigationExecutor", tabId: params.tab_id });
        return params.tab_id;
      }
    }

    if (fullAction.startsWith("modal.")) {
      const modalAction = fullAction.split(".")[1];
      const modalId = params.modal_id || "modal";

      if (modalAction === "open") {
        this.context.componentState.set(modalId, true);
        logger.debug("Modal opened", { component: "NavigationExecutor", modalId });
        return true;
      } else if (modalAction === "close") {
        this.context.componentState.set(modalId, false);
        logger.debug("Modal closed", { component: "NavigationExecutor", modalId });
        return false;
      }
    }

    return null;
  }
}
