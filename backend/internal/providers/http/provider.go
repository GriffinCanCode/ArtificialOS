package http

import (
	"context"
	"fmt"

	"github.com/GriffinCanCode/AgentOS/backend/internal/providers/http/client"
	"github.com/GriffinCanCode/AgentOS/backend/internal/providers/http/config"
	"github.com/GriffinCanCode/AgentOS/backend/internal/providers/http/files"
	"github.com/GriffinCanCode/AgentOS/backend/internal/providers/http/requests"
	"github.com/GriffinCanCode/AgentOS/backend/internal/providers/http/utils"
	"github.com/GriffinCanCode/AgentOS/backend/internal/shared/types"
)

// Provider implements comprehensive HTTP client operations
type Provider struct {
	// Module instances
	requestsOps   *requests.RequestsOps
	configOps     *config.ConfigOps
	downloadsOps  *files.DownloadsOps
	uploadsOps    *files.UploadsOps
	parseOps      *utils.ParseOps
	urlOps        *utils.URLOps
	resilienceOps *config.ResilienceOps
	connectionOps *config.ConnectionOps
}

// NewProvider creates a modular HTTP provider with specialized libraries
func NewProvider() *Provider {
	httpClient := client.NewClient()
	ops := &client.HTTPOps{Client: httpClient}

	return &Provider{
		requestsOps:   &requests.RequestsOps{HTTPOps: ops},
		configOps:     &config.ConfigOps{HTTPOps: ops},
		downloadsOps:  &files.DownloadsOps{HTTPOps: ops},
		uploadsOps:    &files.UploadsOps{HTTPOps: ops},
		parseOps:      &utils.ParseOps{HTTPOps: ops},
		urlOps:        &utils.URLOps{HTTPOps: ops},
		resilienceOps: &config.ResilienceOps{HTTPOps: ops},
		connectionOps: &config.ConnectionOps{HTTPOps: ops},
	}
}

// Definition returns service metadata with all module tools
func (h *Provider) Definition() types.Service {
	// Collect tools from all modules
	tools := []types.Tool{}
	tools = append(tools, h.requestsOps.GetTools()...)
	tools = append(tools, h.configOps.GetTools()...)
	tools = append(tools, h.downloadsOps.GetTools()...)
	tools = append(tools, h.uploadsOps.GetTools()...)
	tools = append(tools, h.parseOps.GetTools()...)
	tools = append(tools, h.urlOps.GetTools()...)
	tools = append(tools, h.resilienceOps.GetTools()...)
	tools = append(tools, h.connectionOps.GetTools()...)

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
func (h *Provider) Execute(ctx context.Context, toolID string, params map[string]interface{}, appCtx *types.Context) (*types.Result, error) {
	switch toolID {
	// Request operations
	case "http.get":
		return h.requestsOps.Get(ctx, params, appCtx)
	case "http.post":
		return h.requestsOps.Post(ctx, params, appCtx)
	case "http.put":
		return h.requestsOps.Put(ctx, params, appCtx)
	case "http.patch":
		return h.requestsOps.Patch(ctx, params, appCtx)
	case "http.delete":
		return h.requestsOps.Delete(ctx, params, appCtx)
	case "http.head":
		return h.requestsOps.Head(ctx, params, appCtx)
	case "http.options":
		return h.requestsOps.Options(ctx, params, appCtx)

	// Config operations
	case "http.setHeader":
		return h.configOps.SetHeader(ctx, params, appCtx)
	case "http.removeHeader":
		return h.configOps.RemoveHeader(ctx, params, appCtx)
	case "http.getHeaders":
		return h.configOps.GetHeaders(ctx, params, appCtx)
	case "http.setTimeout":
		return h.configOps.SetTimeout(ctx, params, appCtx)
	case "http.setAuth":
		return h.configOps.SetAuth(ctx, params, appCtx)
	case "http.clearAuth":
		return h.configOps.ClearAuth(ctx, params, appCtx)

	// Download operations
	case "http.download":
		return h.downloadsOps.Download(ctx, params, appCtx)

	// Upload operations
	case "http.uploadFile":
		return h.uploadsOps.UploadFile(ctx, params, appCtx)
	case "http.uploadMultiple":
		return h.uploadsOps.UploadMultiple(ctx, params, appCtx)

	// Parse operations
	case "http.parseJSON":
		return h.parseOps.ParseJSON(ctx, params, appCtx)
	case "http.parseXML":
		return h.parseOps.ParseXML(ctx, params, appCtx)
	case "http.stringifyJSON":
		return h.parseOps.StringifyJSON(ctx, params, appCtx)

	// URL operations
	case "http.buildURL":
		return h.urlOps.BuildURL(ctx, params, appCtx)
	case "http.parseURL":
		return h.urlOps.ParseURL(ctx, params, appCtx)
	case "http.joinPath":
		return h.urlOps.JoinPath(ctx, params, appCtx)
	case "http.encodeQuery":
		return h.urlOps.EncodeQuery(ctx, params, appCtx)
	case "http.decodeQuery":
		return h.urlOps.DecodeQuery(ctx, params, appCtx)

	// Resilience operations
	case "http.setRetry":
		return h.resilienceOps.SetRetry(ctx, params, appCtx)
	case "http.setRateLimit":
		return h.resilienceOps.SetRateLimit(ctx, params, appCtx)
	case "http.getRateLimit":
		return h.resilienceOps.GetRateLimit(ctx, params, appCtx)

	// Connection operations
	case "http.setProxy":
		return h.connectionOps.SetProxy(ctx, params, appCtx)
	case "http.removeProxy":
		return h.connectionOps.RemoveProxy(ctx, params, appCtx)
	case "http.setVerifySSL":
		return h.connectionOps.SetVerifySSL(ctx, params, appCtx)
	case "http.setFollowRedirects":
		return h.connectionOps.SetFollowRedirects(ctx, params, appCtx)
	case "http.setCookieJar":
		return h.connectionOps.SetCookieJar(ctx, params, appCtx)

	default:
		msg := fmt.Sprintf("unknown tool: %s", toolID)
		return &types.Result{Success: false, Error: &msg}, nil
	}
}
