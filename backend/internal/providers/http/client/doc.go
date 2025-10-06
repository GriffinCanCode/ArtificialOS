// Package http provides HTTP client operations for AI-powered applications.
//
// This package is organized into specialized modules:
//   - requests: HTTP methods (GET, POST, PUT, PATCH, DELETE, HEAD, OPTIONS)
//   - config: Client configuration (headers, timeout, auth)
//   - downloads: File downloading with progress tracking
//   - uploads: File uploads (single and multiple)
//   - parse: Response parsing (JSON, XML, HTML)
//   - url: URL manipulation and encoding
//   - resilience: Retry logic and rate limiting
//   - connection: Connection settings (proxy, SSL, redirects, cookies)
//
// Built on go-resty/resty for production reliability:
//   - Automatic retries with exponential backoff
//   - Connection pooling and keep-alive
//   - Context-based cancellation
//   - Rate limiting per client instance
//
// Features:
//   - Thread-safe client instances
//   - Configurable timeouts and retries
//   - Automatic JSON/XML parsing
//   - Cookie jar support
//
// Example Usage:
//
//	client := http.NewClient()
//	requests := &http.RequestsOps{HTTPOps: &http.HTTPOps{Client: client}}
//	result, err := requests.Get(ctx, params, appCtx)
package client
