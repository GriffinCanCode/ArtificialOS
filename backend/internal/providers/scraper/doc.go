// Package scraper provides HTML parsing and web scraping operations for AI applications.
//
// This package is organized into specialized modules:
//   - content: Content extraction (text, title, links, images, attributes)
//   - xpath: XPath queries for precise element selection
//   - extract: Article extraction and HTML cleaning
//   - forms: Form discovery and field extraction
//   - metadata: Metadata extraction (meta tags, Open Graph, JSON-LD)
//   - patterns: Pattern matching (emails, phones, IPs, regex)
//   - structured: Structured data extraction (tables, lists, URLs)
//
// Built on specialized libraries:
//   - goquery: jQuery-like CSS selectors
//   - htmlquery: XPath support for HTML
//   - bluemonday: HTML sanitization
//   - chardet: Character encoding detection
//
// Features:
//   - CSS selector and XPath support
//   - Automatic charset detection
//   - XSS protection via sanitization
//   - Regex caching for performance
//   - Table to JSON conversion
//
// Example Usage:
//
//	content := &scraper.ContentOps{ScraperOps: scraper.NewScraperOps()}
//	result, err := content.ExtractText(ctx, params, appCtx)
package scraper
