/**
 * Date Formatting Utilities
 * Powered by date-fns for robust, tree-shakeable date handling
 *
 * Benefits over custom implementation:
 * - Handles edge cases (leap years, DST, month boundaries)
 * - Internationalization support (40+ locales)
 * - Timezone support via date-fns-tz
 * - Battle-tested with millions of downloads
 * - Tree-shakeable (only imports what you use)
 *
 * Organization:
 * - format: Basic date/time formatting
 * - relative: Relative time ("2 hours ago")
 * - duration: Duration formatting ("2h 30m")
 * - parse: Date parsing and validation
 * - calculations: Add/subtract time units
 * - comparisons: Date comparison utilities
 * - timezone: Timezone conversion and formatting
 * - i18n: Internationalization support
 * - advanced: Advanced date utilities
 * - constants: Common timezone presets
 */

// Re-export all modules from domain folders
export * from "./format/format";
export * from "./format/relative";
export * from "./format/duration";
export * from "./parse/parse";
export * from "./calculate/calculations";
export * from "./calculate/advanced";
export * from "./compare/comparisons";
export * from "./timezone/timezone";
export * from "./timezone/constants";
export * from "./i18n/i18n";
