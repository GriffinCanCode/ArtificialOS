/**
 * Data Tool Executor
 * Handles data manipulation (filter, sort, search)
 */

import { logger } from "../../../../../core/utils/monitoring/logger";
import { compareTimestampsAsc, compareTimestampsDesc } from "../../../../../core/utils/dates";
import { ExecutorContext, BaseExecutor } from "../core/types";

export class DataExecutor implements BaseExecutor {
  constructor(_context: ExecutorContext) {
    // Context not currently used but kept for interface compatibility
  }

  execute(action: string, params: Record<string, any>): any {
    switch (action) {
      case "filter":
        const data = params.data || [];
        const filter = params.filter || "";
        const filtered = data.filter((item: any) =>
          JSON.stringify(item).toLowerCase().includes(filter.toLowerCase())
        );
        logger.debug("Data filtered", {
          component: "DataExecutor",
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

          // Smart sorting for timestamps (numeric date fields)
          if (typeof aVal === "number" && typeof bVal === "number") {
            // Check if field name suggests it's a timestamp
            const isTimestampField = /time|date|created|updated|modified/i.test(field);
            if (isTimestampField) {
              return order === "asc"
                ? compareTimestampsAsc(aVal, bVal)
                : compareTimestampsDesc(aVal, bVal);
            }
          }

          // Default comparison for other types
          if (order === "asc") {
            return aVal > bVal ? 1 : -1;
          } else {
            return aVal < bVal ? 1 : -1;
          }
        });

        logger.debug("Data sorted", {
          component: "DataExecutor",
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
          component: "DataExecutor",
          query,
          resultCount: results.length,
        });
        return results;

      default:
        return null;
    }
  }
}
