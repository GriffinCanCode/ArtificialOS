/**
 * Typography utilities using OpenType.js
 * Advanced font manipulation and rendering
 */

import opentype, { Font } from 'opentype.js';

export interface TypographyOptions {
  fontSize?: number;
  letterSpacing?: number;
  lineHeight?: number;
  fontWeight?: number;
  fontStyle?: 'normal' | 'italic';
  tracking?: number; // Additional character spacing
  baseline?: number;
  features?: {
    liga?: boolean; // Ligatures
    kern?: boolean; // Kerning
    calt?: boolean; // Contextual alternates
    swsh?: boolean; // Swashes
    salt?: boolean; // Stylistic alternates
  };
}

export interface TextPathOptions extends TypographyOptions {
  x?: number;
  y?: number;
  align?: 'left' | 'center' | 'right';
}

class TypographyManager {
  private fonts: Map<string, Font> = new Map();
  private loadingFonts: Map<string, Promise<Font>> = new Map();

  /**
   * Load a font from a URL
   */
  async loadFont(name: string, url: string): Promise<Font> {
    if (this.fonts.has(name)) {
      return this.fonts.get(name)!;
    }

    if (this.loadingFonts.has(name)) {
      return this.loadingFonts.get(name)!;
    }

    const loadPromise = opentype.load(url).then((font: Font) => {
      this.fonts.set(name, font);
      this.loadingFonts.delete(name);
      return font;
    });

    this.loadingFonts.set(name, loadPromise);
    return loadPromise;
  }

  /**
   * Load a font from a system font (if available in public/fonts)
   */
  async loadSystemFont(name: string): Promise<Font> {
    const url = `/fonts/${name}.ttf`;
    return this.loadFont(name, url);
  }

  /**
   * Get a loaded font
   */
  getFont(name: string): Font | undefined {
    return this.fonts.get(name);
  }

  /**
   * Convert text to SVG path
   */
  textToPath(
    text: string,
    fontName: string,
    options: TextPathOptions = {}
  ): string | null {
    const font = this.fonts.get(fontName);
    if (!font) {
      console.warn(`Font ${fontName} not loaded`);
      return null;
    }

    const {
      fontSize = 72,
      x = 0,
      y = 0,
      letterSpacing = 0,
      tracking = 0,
      features = {},
    } = options;

    try {
      const path = font.getPath(text, x, y, fontSize, {
        kerning: features.kern !== false,
        features: {
          liga: features.liga !== false,
          calt: features.calt || false,
          swsh: features.swsh || false,
          salt: features.salt || false,
        },
      });

      // Apply additional letter spacing if needed
      if (letterSpacing !== 0 || tracking !== 0) {
        return this.applyLetterSpacing(
          font,
          text,
          x,
          y,
          fontSize,
          letterSpacing + tracking
        );
      }

      return path.toSVG(2); // 2 decimal places
    } catch (error) {
      console.error('Error generating text path:', error);
      return null;
    }
  }

  /**
   * Apply custom letter spacing to text
   */
  private applyLetterSpacing(
    font: Font,
    text: string,
    x: number,
    y: number,
    fontSize: number,
    spacing: number
  ): string {
    const scale = fontSize / font.unitsPerEm;
    let currentX = x;
    const paths: string[] = [];

    for (let i = 0; i < text.length; i++) {
      const glyph = font.charToGlyph(text[i]);
      const glyphPath = glyph.getPath(currentX, y, fontSize);
      paths.push(glyphPath.toPathData(2));

      // Advance to next character position
      currentX += (glyph.advanceWidth || 0) * scale + spacing;
    }

    return `<path d="${paths.join(' ')}" />`;
  }

  /**
   * Get detailed glyph information
   */
  getGlyphInfo(char: string, fontName: string): GlyphInfo | null {
    const font = this.fonts.get(fontName);
    if (!font) return null;

    const glyph = font.charToGlyph(char);
    return {
      name: glyph.name || '',
      unicode: glyph.unicode,
      index: glyph.index || 0,
      advanceWidth: glyph.advanceWidth || 0,
      leftSideBearing: glyph.leftSideBearing || 0,
      boundingBox: {
        x1: glyph.xMin || 0,
        y1: glyph.yMin || 0,
        x2: glyph.xMax || 0,
        y2: glyph.yMax || 0,
      },
    };
  }

  /**
   * Measure text dimensions
   */
  measureText(
    text: string,
    fontName: string,
    fontSize: number = 72
  ): { width: number; height: number; ascender: number; descender: number } {
    const font = this.fonts.get(fontName);
    if (!font) {
      return { width: 0, height: 0, ascender: 0, descender: 0 };
    }

    const scale = fontSize / (font.unitsPerEm || 1000);
    let width = 0;

    for (let i = 0; i < text.length; i++) {
      const glyph = font.charToGlyph(text[i]);
      const glyphWidth = glyph.advanceWidth ?? 0;
      width += glyphWidth * scale;

      if (i < text.length - 1) {
        const nextGlyph = font.charToGlyph(text[i + 1]);
        const kerningValue = font.getKerningValue(glyph, nextGlyph) || 0;
        width += kerningValue * scale;
      }
    }

    return {
      width,
      height: fontSize,
      ascender: (font.ascender || 0) * scale,
      descender: (font.descender || 0) * scale,
    };
  }

  /**
   * Create animated morph paths between two texts
   */
  createMorphPaths(
    fromText: string,
    toText: string,
    fontName: string,
    fontSize: number = 72
  ): { from: string; to: string } | null {
    const font = this.fonts.get(fontName);
    if (!font) return null;

    const fromPath = font.getPath(fromText, 0, 0, fontSize);
    const toPath = font.getPath(toText, 0, 0, fontSize);

    return {
      from: fromPath.toPathData(2),
      to: toPath.toPathData(2),
    };
  }

  /**
   * Get available OpenType features for a font
   */
  getFontFeatures(fontName: string): string[] {
    const font = this.fonts.get(fontName);
    if (!font) return [];

    const features: string[] = [];

    // Check common OpenType features
    if (font.tables.gsub) {
      features.push('liga', 'calt', 'dlig', 'salt', 'swsh', 'cswh', 'frac', 'ordn');
    }

    if (font.tables.gpos) {
      features.push('kern');
    }

    return features;
  }

  /**
   * Clear font cache
   */
  clearCache(): void {
    this.fonts.clear();
    this.loadingFonts.clear();
  }
}

export interface GlyphInfo {
  name: string;
  unicode?: number;
  index: number;
  advanceWidth: number;
  leftSideBearing: number;
  boundingBox: {
    x1: number;
    y1: number;
    x2: number;
    y2: number;
  };
}

// Singleton instance
export const typography = new TypographyManager();

// Preset typography styles
export const typographyPresets = {
  hero: {
    fontSize: 96,
    letterSpacing: -2,
    tracking: 0,
    features: { liga: true, kern: true, calt: true },
  },
  title: {
    fontSize: 48,
    letterSpacing: -1,
    tracking: 0,
    features: { liga: true, kern: true },
  },
  heading: {
    fontSize: 32,
    letterSpacing: -0.5,
    tracking: 0,
    features: { liga: true, kern: true },
  },
  body: {
    fontSize: 16,
    letterSpacing: 0,
    tracking: 0,
    features: { liga: true, kern: true },
  },
  caption: {
    fontSize: 14,
    letterSpacing: 0.5,
    tracking: 0.5,
    features: { liga: false, kern: true },
  },
  display: {
    fontSize: 128,
    letterSpacing: -4,
    tracking: 0,
    features: { liga: true, kern: true, swsh: true, salt: true },
  },
} as const;
