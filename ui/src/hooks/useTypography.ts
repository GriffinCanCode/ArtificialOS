/**
 * Typography hook for loading and using custom fonts
 */

import { useEffect, useState } from "react";
import { typography } from "../core/utils/typography";

export interface UseTypographyOptions {
  fonts?: Array<{ name: string; url: string }>;
  preloadFonts?: boolean;
}

export const useTypography = (options: UseTypographyOptions = {}) => {
  const { fonts = [], preloadFonts = true } = options;
  const [fontsLoaded, setFontsLoaded] = useState(false);
  const [loadingError, setLoadingError] = useState<Error | null>(null);

  useEffect(() => {
    if (!preloadFonts || fonts.length === 0) {
      setFontsLoaded(true);
      return;
    }

    const loadFonts = async () => {
      try {
        await Promise.all(fonts.map(({ name, url }) => typography.loadFont(name, url)));
        setFontsLoaded(true);
      } catch (error) {
        console.error("Error loading fonts:", error);
        setLoadingError(error as Error);
        setFontsLoaded(true); // Still set to true to allow fallback rendering
      }
    };

    loadFonts();
  }, [fonts, preloadFonts]);

  return {
    fontsLoaded,
    loadingError,
    typography,
  };
};

// Hook for loading system fonts from Google Fonts or local
export const useSystemFont = (fontFamily: string = "Inter") => {
  const [loaded, setLoaded] = useState(false);

  useEffect(() => {
    // Try to load from local first, fallback to system
    const loadFont = async () => {
      try {
        // Check if font is already available
        const fonts = await (document as any).fonts?.ready;
        if (fonts) {
          setLoaded(true);
          return;
        }

        // Fallback to system font
        setLoaded(true);
      } catch (error) {
        console.warn("Font loading failed, using system font:", error);
        setLoaded(true);
      }
    };

    loadFont();
  }, [fontFamily]);

  return loaded;
};
