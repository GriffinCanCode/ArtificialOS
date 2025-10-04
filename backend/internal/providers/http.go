package providers

import (
	"context"
	"fmt"

	httpProvider "github.com/GriffinCanCode/AgentOS/backend/internal/providers/http"
	"github.com/GriffinCanCode/AgentOS/backend/internal/types"
)

// HTTP provides comprehensive HTTP client operations
type HTTP struct {
	// Module instances
	requests   *httpProvider.RequestsOps
	config     *httpProvider.ConfigOps
	downloads  *httpProvider.DownloadsOps
	uploads    *httpProvider.UploadsOps
	parse      *httpProvider.ParseOps
	url        *httpProvider.URLOps
	resilience *httpProvider.ResilienceOps
	connection *httpProvider.ConnectionOps
}

// NewHTTP creates a modular HTTP provider with specialized libraries
func NewHTTP() *HTTP {
	client := httpProvider.NewClient()
	ops := &httpProvider.HTTPOps{Client: client}

	return &HTTP{
		requests:   &httpProvider.RequestsOps{HTTPOps: ops},
		config:     &httpProvider.ConfigOps{HTTPOps: ops},
		downloads:  &httpProvider.DownloadsOps{HTTPOps: ops},
		uploads:    &httpProvider.UploadsOps{HTTPOps: ops},
		parse:      &httpProvider.ParseOps{HTTPOps: ops},
		url:        &httpProvider.URLOps{HTTPOps: ops},
		resilience: &httpProvider.ResilienceOps{HTTPOps: ops},
		connection: &httpProvider.ConnectionOps{HTTPOps: ops},
	}
}

// Definition returns service metadata with all module tools
func (h *HTTP) Definition() types.Service {
	// Collect tools from all modules
	tools := []types.Tool{}
	tools = append(tools, h.requests.GetTools()...)
	tools = append(tools, h.config.GetTools()...)
	tools = append(tools, h.downloads.GetTools()...)
	tools = append(tools, h.uploads.GetTools()...)
	tools = append(tools, h.parse.GetTools()...)
	tools = append(tools, h.url.GetTools()...)
	tools = append(tools, h.resilience.GetTools()...)
	tools = append(tools, h.connection.GetTools()...)

	return types.Service{
		ID:          "http",
		Name:        "HTTP Service",
		Description: "High-performance HTTP client with retry, rate limiting, and advanced features",
		Category:    types.CategoryHTTP,
		Capabilities: []string{
			"requests", "get", "post", "put", "patch", "delete", "head", "options",
			"uploads", "downloads", "multipart",
			"parsing", "json", "xml",
			"authentication", "basic", "bearer", "custom",
			"resilience", "retry", "rate-limiting", "exponential-backoff",
			"connection", "proxy", "ssl", "redirects", "cookies",
			"url", "building", "parsing", "encoding",
		},
		Tools: tools,
	}
}

// Execute routes to appropriate module
func (h *HTTP) Execute(ctx context.Context, toolID string, params map[string]interface{}, appCtx *types.Context) (*types.Result, error) {
	switch toolID {
	// Request operations
	case "http.get":
		return h.requests.Get(ctx, params, appCtx)
	case "http.post":
		return h.requests.Post(ctx, params, appCtx)
	case "http.put":
		return h.requests.Put(ctx, params, appCtx)
	case "http.patch":
		return h.requests.Patch(ctx, params, appCtx)
	case "http.delete":
		return h.requests.Delete(ctx, params, appCtx)
	case "http.head":
		return h.requests.Head(ctx, params, appCtx)
	case "http.options":
		return h.requests.Options(ctx, params, appCtx)

	// Config operations
	case "http.setHeader":
		return h.config.SetHeader(ctx, params, appCtx)
	case "http.removeHeader":
		return h.config.RemoveHeader(ctx, params, appCtx)
	case "http.getHeaders":
		return h.config.GetHeaders(ctx, params, appCtx)
	case "http.setTimeout":
		return h.config.SetTimeout(ctx, params, appCtx)
	case "http.setAuth":
		return h.config.SetAuth(ctx, params, appCtx)
	case "http.clearAuth":
		return h.config.ClearAuth(ctx, params, appCtx)

	// Download operations
	case "http.download":
		return h.downloads.Download(ctx, params, appCtx)

	// Upload operations
	case "http.uploadFile":
		return h.uploads.UploadFile(ctx, params, appCtx)
	case "http.uploadMultiple":
		return h.uploads.UploadMultiple(ctx, params, appCtx)

	// Parse operations
	case "http.parseJSON":
		return h.parse.ParseJSON(ctx, params, appCtx)
	case "http.parseXML":
		return h.parse.ParseXML(ctx, params, appCtx)
	case "http.stringifyJSON":
		return h.parse.StringifyJSON(ctx, params, appCtx)

	// URL operations
	case "http.buildURL":
		return h.url.BuildURL(ctx, params, appCtx)
	case "http.parseURL":
		return h.url.ParseURL(ctx, params, appCtx)
	case "http.joinPath":
		return h.url.JoinPath(ctx, params, appCtx)
	case "http.encodeQuery":
		return h.url.EncodeQuery(ctx, params, appCtx)
	case "http.decodeQuery":
		return h.url.DecodeQuery(ctx, params, appCtx)

	// Resilience operations
	case "http.setRetry":
		return h.resilience.SetRetry(ctx, params, appCtx)
	case "http.setRateLimit":
		return h.resilience.SetRateLimit(ctx, params, appCtx)
	case "http.getRateLimit":
		return h.resilience.GetRateLimit(ctx, params, appCtx)

	// Connection operations
	case "http.setProxy":
		return h.connection.SetProxy(ctx, params, appCtx)
	case "http.removeProxy":
		return h.connection.RemoveProxy(ctx, params, appCtx)
	case "http.setVerifySSL":
		return h.connection.SetVerifySSL(ctx, params, appCtx)
	case "http.setFollowRedirects":
		return h.connection.SetFollowRedirects(ctx, params, appCtx)
	case "http.setCookieJar":
		return h.connection.SetCookieJar(ctx, params, appCtx)

	default:
		msg := fmt.Sprintf("unknown tool: %s", toolID)
		return &types.Result{Success: false, Error: &msg}, nil
	}
}
