# Web Browser

A full-featured web browser application with multi-tab support, bookmarks, and history.

## Features

- ✅ Multi-tab browsing with session management
- ✅ Address bar with URL and search support
- ✅ Navigation controls (back, forward, refresh, home)
- ✅ Hybrid rendering (iframe with proxy fallback)
- ✅ DuckDuckGo, Google, Bing search integration
- ✅ Safe URL validation
- ✅ Tab management (create, close, switch)
- ✅ Persistent storage for settings
- 🚧 Bookmarks (coming soon)
- 🚧 History tracking (coming soon)
- 🚧 Reader mode (coming soon)
- 🚧 Download manager (coming soon)

## Architecture

### Hybrid Rendering Approach

The browser uses a sophisticated hybrid approach to render web content:

1. **Iframe Mode (Primary)**: Attempts to load content in a sandboxed iframe
   - Fast, native rendering
   - Handles JavaScript-heavy sites
   - Respects X-Frame-Options

2. **Proxy Mode (Fallback)**: Fetches and sanitizes HTML
   - Works when iframe is blocked (X-Frame-Options)
   - Content is fetched via `http.get` provider
   - HTML is sanitized with DOMPurify
   - Good for static content

### Services Used

- **http**: Fetch web pages (`http.get`)
- **scraper**: Parse HTML and extract metadata (future)
- **storage**: Save bookmarks, history, and settings
- **filesystem**: Download files (future)

## Development

```bash
# Install dependencies
npm install

# Development mode with HMR
npm run dev

# Build for production
npm run build

# Type check
npm run type-check

# Lint
npm run lint
npm run lint:fix

# Format
npm run format
```

## Usage

Open the browser from the App Hub or via the system.

### Keyboard Shortcuts

- `Ctrl+T`: New tab
- `Ctrl+W`: Close tab
- `Ctrl+R`: Reload page
- `Alt+Left`: Go back
- `Alt+Right`: Go forward
- `Enter` in address bar: Navigate

### Search Engines

The browser supports multiple search engines:
- DuckDuckGo (default)
- Google
- Bing

Configure in settings (coming soon).

## Implementation Details

### URL Processing

The `url.ts` utility intelligently distinguishes between URLs and search queries:
- URLs: Anything with protocol, dots, or localhost
- Search queries: Everything else

### Security

- All URLs are validated before loading
- Dangerous protocols (javascript:, data:, etc.) are blocked
- HTML content is sanitized with DOMPurify
- Iframes use strict sandbox attributes

### Performance

- Lazy tab loading
- Efficient state management with hooks
- Minimal re-renders

## Future Enhancements

- [ ] Bookmarks with folders and tags
- [ ] History with search
- [ ] Reader mode for article extraction
- [ ] Download manager with progress tracking
- [ ] Tab grouping
- [ ] Private browsing mode
- [ ] Extensions support
- [ ] Developer tools

## Credits

Built with:
- React 18
- TypeScript
- DOMPurify for HTML sanitization
- OS SDK for service integration

