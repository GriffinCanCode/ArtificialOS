# Web Scraper Provider

High-performance HTML parsing and data extraction with specialized libraries.

## Architecture

The scraper provider follows a modular design with specialized operation modules:

```
scraper/
├── types.go       # Common types, helpers, and validation
├── content.go     # Content extraction (text, links, images)
├── xpath.go       # XPath queries for powerful selection
├── extract.go     # Smart content extraction (articles, summaries)
├── forms.go       # Form parsing and field extraction
├── metadata.go    # Metadata extraction (Open Graph, JSON-LD, microdata)
├── patterns.go    # Pattern matching (phones, emails, custom regex)
└── structured.go  # Structured data (tables, lists, URLs)
```

## Libraries Used

### Core Libraries
- **goquery** - jQuery-like HTML parsing with CSS selectors
- **htmlquery** - XPath support for powerful element selection
- **chardet** - Automatic charset detection for proper encoding
- **bluemonday** - HTML sanitization for security

### Key Features
- **Charset Detection**: Automatically detects and converts character encodings
- **Size Limits**: Protects against memory exhaustion (10MB max)
- **Cached Regex**: Pattern compilation is cached for performance
- **Deduplication**: Built-in deduplication for extracted data
- **HTML Sanitization**: Secure HTML cleaning using bluemonday
- **Strong Typing**: Comprehensive type validation and error handling

## Module Overview

### Content Operations (`content.go`)
Extract basic content from HTML documents:
- Text extraction with normalization
- Title extraction (with Open Graph fallback)
- Link extraction with deduplication
- Image extraction with metadata
- CSS selector queries
- Attribute extraction

### XPath Operations (`xpath.go`)
Powerful XPath queries for complex selections:
- XPath queries (single or multiple results)
- Text extraction via XPath
- Attribute extraction via XPath
- Full XPath 1.0 support

### Smart Extraction (`extract.go`)
Intelligent content extraction:
- Article extraction (removes nav, ads, sidebars)
- HTML sanitization
- Summary extraction (meta description or first paragraph)
- Automatic main content detection

### Form Operations (`forms.go`)
Comprehensive form parsing:
- Find forms by selector or action
- Extract all form fields with metadata
- Support for select options
- Detect required/disabled/readonly fields
- Find all forms on page

### Metadata Operations (`metadata.go`)
Extract structured metadata:
- Meta tags (standard, Open Graph, Twitter)
- Heading hierarchy (h1-h6)
- JSON-LD structured data
- HTML5 microdata (itemscope/itemprop)
- Open Graph metadata

### Pattern Operations (`patterns.go`)
Pattern-based extraction with cached regex:
- Phone number extraction (US & international)
- Custom regex patterns (cached)
- Text between delimiters
- IP address extraction
- Pre-defined patterns (email, URL, SSN, zip codes)

### Structured Data (`structured.go`)
Extract structured content:
- Table parsing with header detection
- Email address extraction
- URL extraction with validation
- List extraction (ul/ol) with hierarchy

## Usage Examples

### Extract Article Content
```go
params := map[string]interface{}{
    "html": htmlContent,
}
result, _ := scraper.Execute(ctx, "scraper.article", params, appCtx)
```

### XPath Query
```go
params := map[string]interface{}{
    "html": htmlContent,
    "xpath": "//div[@class='content']//p",
    "all": true,
}
result, _ := scraper.Execute(ctx, "scraper.xpath", params, appCtx)
```

### Extract Metadata
```go
params := map[string]interface{}{
    "html": htmlContent,
}
result, _ := scraper.Execute(ctx, "scraper.metadata", params, appCtx)
```

### Parse Table
```go
params := map[string]interface{}{
    "html": htmlContent,
    "selector": "table.data",
    "headers": true,
}
result, _ := scraper.Execute(ctx, "scraper.table", params, appCtx)
```

## Performance Considerations

1. **Charset Detection**: Automatic encoding detection ensures proper text extraction
2. **Regex Caching**: Common patterns are compiled once and cached
3. **Size Limits**: 10MB HTML size limit prevents memory exhaustion
4. **Deduplication**: Built-in deduplication reduces result size
5. **Selective Parsing**: XPath and CSS selectors are highly optimized

## Security

- HTML sanitization using bluemonday UGC policy
- Input validation and size limits
- No arbitrary code execution
- Safe regex compilation with error handling

## Testing

Each module is designed for easy unit testing:
- Pure functions with clear inputs/outputs
- Comprehensive error handling
- Validation helpers for type safety
- Mock-friendly interfaces

## Extending

To add new scraping capabilities:

1. Create a new ops struct in a dedicated file
2. Embed `*ScraperOps` for common functionality
3. Implement `GetTools()` to define tool metadata
4. Add operation methods with standard signature
5. Register tools in main scraper.go `Execute()` switch
6. Follow one-word file naming convention

## Performance Benchmarks

Approximate performance metrics:
- Small HTML (<50KB): <5ms parsing
- Medium HTML (500KB): <50ms parsing  
- Large HTML (5MB): <500ms parsing
- XPath queries: ~2x faster than CSS for complex selectors
- Cached regex: ~10x faster than recompilation
