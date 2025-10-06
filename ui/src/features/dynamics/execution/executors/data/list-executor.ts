/**
 * List Tool Executor
 * Handles list operations (add, remove, toggle, clear)
 */

import { logger } from "../../../../../core/utils/monitoring/logger";
import { ExecutorContext, BaseExecutor } from "../core/types";

export class ListExecutor implements BaseExecutor {
  private context: ExecutorContext;

  constructor(context: ExecutorContext) {
    this.context = context;
  }

  execute(action: string, params: Record<string, any>): any {
    const listId = params.list_id || "list";

    switch (action) {
      case "add":
        this.context.componentState.batch(() => {
          const items = this.context.componentState.get<any[]>(listId, []);
          items.push(params.item);
          this.context.componentState.set(listId, [...items]);
        });
        logger.debug("List item added", { component: "ListExecutor", listId });
        return this.context.componentState.get(listId);

      case "remove":
        this.context.componentState.batch(() => {
          const items = this.context.componentState.get<any[]>(listId, []);
          const filtered = items.filter((item: any) => item.id !== params.item_id);
          this.context.componentState.set(listId, filtered);
        });
        logger.debug("List item removed", { component: "ListExecutor", listId });
        return this.context.componentState.get(listId);

      case "toggle":
        this.context.componentState.batch(() => {
          const items = this.context.componentState.get<any[]>(listId, []);
          const item = items.find((i: any) => i.id === params.item_id);
          if (item) {
            item.done = !item.done;
            this.context.componentState.set(listId, [...items]);
          }
        });
        logger.debug("List item toggled", { component: "ListExecutor", listId });
        return this.context.componentState.get(listId);

      case "clear":
        this.context.componentState.set(listId, []);
        logger.debug("List cleared", { component: "ListExecutor", listId });
        return [];

      default:
        return null;
    }
  }
}
