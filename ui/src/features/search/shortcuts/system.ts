/**
 * Spotlight Keyboard Shortcuts
 * System-wide shortcuts for spotlight search
 */

import type { ShortcutConfig } from "../../input";

export const spotlightShortcuts: ShortcutConfig[] = [
  {
    id: "spotlight.toggle",
    sequence: "$mod+Space",
    label: "Spotlight Search",
    description: "Open global search",
    category: "system",
    priority: "critical",
    scope: "global",
    handler: () => {
      // Handler will be provided by Desktop component
    },
  },
];

