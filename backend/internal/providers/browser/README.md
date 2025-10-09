# Browser Proxy Provider

Server-side browser proxy that enables full web browsing by bypassing CORS, X-Frame-Options, and CSP restrictions.

## Why This Approach?

### The Problem with Traditional Methods

**Iframes** (Current approach):
- ❌ Blocked by `X-Frame-Options: DENY/SAMEORIGIN` (95% of websites)
- ❌ Blocked by CSP `frame-ancestors` directive
- ❌ No control over content, can't inject scripts or modify DOM
- ❌ Limited JavaScript interaction

**Direct Fetch**:
- ❌ Blocked by CORS policies
- ❌ Can't access cross-origin resources
- ❌ No cookie/session management

**Electron WebViews**:
- ❌ Not available in web-based environments
- ❌ Requires native application

### Server-Side Proxy Solution

The browser provider solves these problems by:

1. **Backend Fetching**: All HTTP requests made by Go backend (no CORS)
2. **HTML Rewriting**: Parse and rewrite all URLs to go through proxy
3. **Asset Proxying**: Images, CSS, JS fetched through backend
4. **Session Management**: Server-side cookies and state
5. **Form Handling**: POST requests through proxy
6. **Kernel Integration**: Permissions, metrics, observability

## Architecture

```
┌──────────────────────────────────────────────┐
│          Frontend Browser UI                 │
│  ┌────────┐  ┌─────────┐  ┌──────────┐     │
│  │ TabBar │  │ Address │  │ BrowserView │   │
│  └────────┘  └─────────┘  └──────────┘     │
└──────────────────┬───────────────────────────┘
                   │
                   ▼ Tool Execution
┌──────────────────────────────────────────────┐
│    Backend Browser Proxy Provider            │
│  ┌───────────┐  ┌────────────┐              │
│  │  Navigate │  │ Asset Proxy│              │
│  └───────────┘  └────────────┘              │
│  ┌───────────┐  ┌────────────┐              │
│  │   Forms   │  │   Sessions │              │
│  └───────────┘  └────────────┘              │
└──────────────────┬───────────────────────────┘
                   │
                   ▼ HTTP Client + Permission Checks
┌──────────────────────────────────────────────┐
│          Kernel (Permissions + Metrics)      │
└──────────────────┬───────────────────────────┘
                   │
                   ▼
              [Internet]
```

## Features

### ✅ Full Page Rendering
- Fetches complete HTML with all resources
- Rewrites URLs to work through proxy
- Preserves layout and styling
- Supports CSS and images

### ✅ Session Management
- Per-app browser sessions
- Cookie storage and management
- Navigation history
- Referer tracking
- Custom user agent

### ✅ Asset Proxying
- Images (PNG, JPG, GIF, SVG)
- Stylesheets (CSS)
- JavaScript files
- Fonts (WOFF, WOFF2)
- All assets go through backend

### ✅ Form Handling
- GET and POST form submissions
- Form data encoding
- Redirect following
- Session cookies maintained

### ✅ Security
- Kernel permission checks before network access
- Network metrics reported to kernel
- HTML content validation
- XSS protection through sanitization

### ✅ Kernel Integration
- Permission-first architecture
- Network observability
- Circuit breaker protection
- Retry logic with exponential backoff

## Tools

### `browser.navigate`
Load and render a web page with full HTML rewriting.

**Parameters:**
- `url` (string, required): URL to navigate to
- `session_id` (string, optional): Browser session ID

**Returns:**
```json
{
  "html": "<html>...</html>",
  "title": "Page Title",
  "url": "https://example.com",
  "status": 200,
  "content_type": "text/html",
  "session_id": "app-123"
}
```

### `browser.proxy_asset`
Fetch an asset (image, CSS, JS) through the backend.

**Parameters:**
- `url` (string, required): Asset URL
- `session_id` (string, optional): Browser session ID

**Returns:**
```json
{
  "data": "...",
  "content_type": "image/png",
  "url": "https://example.com/image.png"
}
```

### `browser.submit_form`
Submit a form through the proxy.

**Parameters:**
- `url` (string, required): Form action URL
- `method` (string, optional): HTTP method (GET/POST), default: POST
- `data` (object, required): Form data
- `session_id` (string, optional): Browser session ID

**Returns:**
```json
{
  "html": "<html>...</html>",
  "title": "Result Page",
  "url": "https://example.com/result",
  "session_id": "app-123"
}
```

### `browser.get_session`
Get current browser session information.

**Parameters:**
- `session_id` (string, required): Browser session ID

**Returns:**
```json
{
  "app_id": "app-123",
  "user_agent": "Mozilla/5.0...",
  "history": ["https://google.com", "https://github.com"],
  "referer": "https://google.com"
}
```

## URL Rewriting

The provider rewrites all URLs in HTML to work through the proxy:

### Before (Original HTML):
```html
<a href="/page">Link</a>
<img src="/image.png">
<link rel="stylesheet" href="/style.css">
<script src="/app.js"></script>
<form action="/submit" method="POST">
```

### After (Rewritten):
```html
<a href="javascript:void(window.parent.postMessage({action:'navigate',url:'https://example.com/page'},'*'))">Link</a>
<img src="[proxied through browser.proxy_asset]">
<link rel="stylesheet" href="[proxied through browser.proxy_asset]">
<script src="[proxied through browser.proxy_asset]"></script>
<form action="[proxied through browser.submit_form]">
```

## Session Lifecycle

1. **Creation**: Session created on first navigation per app ID
2. **Cookies**: Stored per-domain in session
3. **History**: All navigations tracked
4. **Cleanup**: Sessions cleaned up when app closes

## Performance

- **HTTP Client**: Resty with circuit breakers and retry
- **HTML Parsing**: goquery (jQuery-like, efficient)
- **Concurrent Sessions**: Thread-safe session management
- **Asset Caching**: TODO - Add backend caching layer

## Limitations & Future Work

### Current Limitations
- No JavaScript execution (static HTML only)
- No WebSocket proxying
- No Service Worker support
- Limited interactive features

### Planned Enhancements
- [ ] JavaScript sandbox with isolated execution
- [ ] WebSocket proxying for real-time apps
- [ ] Service Worker support for PWAs
- [ ] Backend caching for assets
- [ ] Download manager integration
- [ ] Developer tools (network inspector, console)
- [ ] Ad blocking and tracking protection

## Comparison with Alternatives

| Approach | CORS | X-Frame-Options | JS Support | Session Mgmt | Complexity |
|----------|------|-----------------|------------|--------------|------------|
| **Iframe** | ❌ | ❌ | ✅ | ❌ | Low |
| **Direct Fetch** | ❌ | N/A | ❌ | ❌ | Low |
| **Electron WebView** | ✅ | ✅ | ✅ | ✅ | High (native) |
| **Server Proxy** (This) | ✅ | ✅ | ⚠️ | ✅ | Medium |

## Integration with Kernel

The browser provider is deeply integrated with the kernel:

1. **Permission Checks**: Before every HTTP request
2. **Network Metrics**: Bytes sent/received, duration, status
3. **Observability**: All requests tracked in monitoring system
4. **Sandboxing**: Per-app network isolation (future)

## Usage in Frontend

```typescript
// Navigate to URL
const result = await context.executor.execute('browser.navigate', {
  url: 'https://example.com',
  session_id: context.appId
});

// Render HTML
document.getElementById('content').innerHTML = result.html;
```

## Security Considerations

- All network requests subject to kernel permissions
- HTML sanitization on backend
- Cookie security (HTTPOnly, Secure flags)
- XSS protection through content rewriting
- Rate limiting via HTTP provider

## Credits

Built with:
- **goquery**: HTML parsing and manipulation
- **resty**: HTTP client
- **Kernel integration**: Permissions and observability

