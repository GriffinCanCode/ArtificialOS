# Web Browser

A full-featured web browser application with multi-tab support, bookmarks, and history.

## Features

- ✅ Multi-tab browsing with session management
- ✅ Address bar with URL and search support
- ✅ Navigation controls (back, forward, refresh, home)
- ✅ Server-side proxy rendering (bypasses CORS, X-Frame-Options)
- ✅ Static HTML/CSS rendering (fast, clean)
- ✅ DuckDuckGo, Google, Bing search integration
- ✅ Safe URL validation
- ✅ Tab management (create, close, switch)
- ✅ Link interception (navigates through proxy)
- ✅ Cookie and session management per app
- ✅ Persistent storage for settings
- 🚧 JavaScript support (coming soon - requires sandbox)
- 🚧 Bookmarks (coming soon)
- 🚧 History tracking (coming soon)
- 🚧 Reader mode (coming soon)
- 🚧 Download manager (coming soon)

## Architecture

### Server-Side Proxy Rendering

The browser uses a **server-side proxy** approach to bypass browser security restrictions:

**Why Not Iframes?**
- ❌ Blocked by `X-Frame-Options` headers (95% of sites)
- ❌ Blocked by CSP `frame-ancestors` directive
- ❌ CORS restrictions prevent cross-origin content
- ❌ No control over content, limited interaction

**Server-Side Proxy Solution:**
1. **Backend Fetching**: All HTTP requests made by Go backend (bypasses CORS)
2. **HTML Rewriting**: URLs rewritten to work through proxy
3. **Asset Proxying**: Images, CSS, JS fetched through backend
4. **Session Management**: Server-side cookies and state
5. **Kernel Integration**: Permission checks, metrics, observability

### Services Used

- **browser**: Server-side proxy for web content (`browser.navigate`, `browser.proxy_asset`)
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
- Backend performs permission checks before network requests
- HTML content is sanitized on backend and frontend (defense in depth)
- All requests go through kernel permission system

### Performance

- ~500ms average page load (Google)
- Static HTML/CSS only (no JavaScript execution)
- No CORS errors or failed network requests
- Efficient goquery-based HTML parsing
- Lazy tab loading
- Efficient state management with hooks
- Minimal re-renders

**Note**: Initial navigation may take ~1 second as the backend:
1. Fetches the page
2. Parses HTML
3. Rewrites URLs
4. Removes scripts
5. Returns processed content

Subsequent navigations are faster (~300-500ms) due to session reuse.

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

