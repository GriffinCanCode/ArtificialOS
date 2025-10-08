/**
 * File Search Provider
 * Registers file search context for Spotlight
 */

import { useEffect } from "react";
import { useSearchContexts } from "../store/store";

export interface FileItem {
  id: string;
  name: string;
  path: string;
  type: "file" | "directory";
  size?: number;
  modified?: Date;
}

/**
 * Hook to register file search context
 */
export function useFileSearch(files: FileItem[]) {
  const { registerContext, unregisterContext } = useSearchContexts();

  useEffect(() => {
    registerContext(
      "files",
      "Files",
      files,
      {
        keys: [
          { name: "name", weight: 0.7 },
          { name: "path", weight: 0.3 },
        ],
        threshold: 0.3,
      },
      10 // Priority
    );

    return () => {
      unregisterContext("files");
    };
  }, [files, registerContext, unregisterContext]);
}

