/**
 * Browser Tool Executor
 * Handles iframe navigation and browser-like operations
 */

import { logger } from "../../../../../core/utils/monitoring/logger";
import { ExecutorContext, BaseExecutor } from "../core/types";

export class BrowserExecutor implements BaseExecutor {
  private context: ExecutorContext;

  constructor(context: ExecutorContext) {
    this.context = context;
  }

  execute(action: string, params: Record<string, any>): any {
    const iframeId = params.iframe_id || "webpage";

    switch (action) {
      case "navigate":
        // Get URL from params or from a URL input field in state
        let url = params.url;
        if (!url) {
          // Try common input field names
          url =
            this.context.componentState.get("url-input") ||
            this.context.componentState.get("search-input") ||
            this.context.componentState.get("browser-url") ||
            this.context.componentState.get("address-bar");
        }

        if (!url) {
          logger.warn("No URL provided for browser navigation", { component: "BrowserExecutor" });
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

        this.context.componentState.set(`${iframeId}_url`, url);
        logger.debug("Browser navigating", { component: "BrowserExecutor", url });
        return url;

      case "back":
        // Browser back - would need iframe history API or parent navigation
        logger.debug("Browser back", { component: "BrowserExecutor" });
        return true;

      case "forward":
        logger.debug("Browser forward", { component: "BrowserExecutor" });
        return true;

      case "refresh":
        const currentUrl = this.context.componentState.get(`${iframeId}_url`);
        this.context.componentState.set(`${iframeId}_url`, currentUrl + "?refresh=" + Date.now());
        logger.debug("Browser refresh", { component: "BrowserExecutor" });
        return true;

      default:
        return null;
    }
  }
}
