declare module "opentype.js" {
  export interface Glyph {
    name: string;
    unicode?: number;
    index: number;
    advanceWidth: number;
    leftSideBearing: number;
    xMin?: number;
    yMin?: number;
    xMax?: number;
    yMax?: number;
    getPath(x: number, y: number, fontSize: number): Path;
  }

  export interface Path {
    toPathData(decimals?: number): string;
    toSVG(decimals?: number): string;
  }

  export interface Font {
    unitsPerEm: number;
    ascender: number;
    descender: number;
    tables: {
      gsub?: any;
      gpos?: any;
    };
    charToGlyph(char: string): Glyph;
    getPath(text: string, x: number, y: number, fontSize: number, options?: any): Path;
    getKerningValue(leftGlyph: Glyph, rightGlyph: Glyph): number;
  }

  export function load(url: string): Promise<Font>;

  const opentype: {
    load: typeof load;
  };

  export default opentype;
}
