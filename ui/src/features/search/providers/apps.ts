/**
 * App Search Provider
 * Registers app search context for Spotlight
 */

import { useEffect } from "react";
import { useSearchContexts } from "../store/store";

export interface AppItem {
  id: string;
  name: string;
  description?: string;
  icon?: string;
  category?: string;
}

/**
 * Hook to register app search context
 */
export function useAppSearch(apps: AppItem[]) {
  const { registerContext, unregisterContext } = useSearchContexts();

  useEffect(() => {
    registerContext(
      "apps",
      "Applications",
      apps,
      {
        keys: [
          { name: "name", weight: 0.8 },
          { name: "description", weight: 0.2 },
        ],
        threshold: 0.2,
      },
      20 // Higher priority than files
    );

    return () => {
      unregisterContext("apps");
    };
  }, [apps, registerContext, unregisterContext]);
}

