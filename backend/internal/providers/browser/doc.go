/*
Package browser provides a server-side browser proxy service.

# Overview

The browser provider enables web browsing within the OS by proxying all HTTP requests through
the backend, bypassing browser security restrictions like CORS, X-Frame-Options, and CSP.

# Architecture

Instead of loading web pages directly in iframes (which fails for most sites), this provider:

 1. Fetches pages via the backend HTTP client (bypasses CORS)
 2. Parses and rewrites HTML to make it browseable
 3. Proxies all assets (images, CSS, JS) through the backend
 4. Manages cookies and sessions server-side
 5. Handles form submissions through the proxy

# Why Server-Side Proxy?

Traditional approaches fail because:

  - iframes: Blocked by X-Frame-Options and CSP headers (95% of sites)
  - Direct fetch: Blocked by CORS policies
  - Electron webviews: Not available in web-based environments

Server-side proxying solves all of these by:

  - ✅ Backend makes requests (no CORS restrictions)
  - ✅ Integrated with kernel permissions and observability
  - ✅ Full control over HTML rewriting and asset proxying
  - ✅ Session management with cookies
  - ✅ Form handling and redirects

# Session Management

Each browser instance (app) has its own session with:

  - Cookies (per-domain)
  - Navigation history
  - User agent string
  - Referer tracking

# Security

All requests go through kernel permission checks:

  - Network access controlled by kernel
  - Permission checks before HTTP requests
  - Observability metrics reported to kernel
  - HTML sanitization on backend

# Tools

Browser proxy provides these tools:

  - browser.navigate: Load and render a web page
  - browser.proxy_asset: Fetch images, CSS, JS through proxy
  - browser.submit_form: Submit forms through proxy
  - browser.get_session: Get current session info

# Usage Example

	// Navigate to a URL
	result := executor.Execute("browser.navigate", map[string]interface{}{
		"url": "https://example.com",
		"session_id": "my-browser-session",
	})

	// Display rendered HTML
	html := result.Data["html"]

# Performance

  - HTTP client with circuit breakers and retry logic
  - Streaming support for large pages
  - Efficient HTML parsing with goquery
  - Per-app session caching

# Future Enhancements

  - JavaScript execution sandboxing
  - WebSocket proxying
  - Service Worker support
  - Download management
  - Developer tools integration
*/
package browser
