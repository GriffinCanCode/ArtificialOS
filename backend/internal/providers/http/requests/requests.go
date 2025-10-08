package requests

import (
	"context"
	"encoding/json"
	"fmt"
	"strings"

	"github.com/GriffinCanCode/AgentOS/backend/internal/providers/http/client"
	"github.com/GriffinCanCode/AgentOS/backend/internal/shared/types"
	"github.com/go-resty/resty/v2"
)

// KernelClient interface for permission checks and metrics
type KernelClient interface {
	ExecuteSyscall(ctx context.Context, pid uint32, syscallType string, params map[string]interface{}) ([]byte, error)
}

// extractHost extracts hostname from URL for permission checks
func extractHost(urlStr string) string {
	// Simple extraction: remove protocol and path
	host := urlStr
	if idx := strings.Index(host, "://"); idx != -1 {
		host = host[idx+3:]
	}
	if idx := strings.Index(host, "/"); idx != -1 {
		host = host[:idx]
	}
	if idx := strings.Index(host, "?"); idx != -1 {
		host = host[:idx]
	}
	return host
}

// RequestsOps handles HTTP request methods
type RequestsOps struct {
	*client.HTTPOps
	Kernel KernelClient
	PID    uint32
}

// GetTools returns HTTP request tool definitions
func (r *RequestsOps) GetTools() []types.Tool {
	return []types.Tool{
		{
			ID:          "http.get",
			Name:        "HTTP GET",
			Description: "Fetch data from URL with optional headers and params",
			Parameters: []types.Parameter{
				{Name: "url", Type: "string", Description: "Request URL", Required: true},
				{Name: "params", Type: "object", Description: "Query parameters", Required: false},
				{Name: "headers", Type: "object", Description: "HTTP headers", Required: false},
			},
			Returns: "object",
		},
		{
			ID:          "http.post",
			Name:        "HTTP POST",
			Description: "Send data to URL with optional headers",
			Parameters: []types.Parameter{
				{Name: "url", Type: "string", Description: "Request URL", Required: true},
				{Name: "data", Type: "object", Description: "Request body", Required: true},
				{Name: "headers", Type: "object", Description: "HTTP headers", Required: false},
				{Name: "json", Type: "boolean", Description: "Send as JSON (default: true)", Required: false},
			},
			Returns: "object",
		},
		{
			ID:          "http.put",
			Name:        "HTTP PUT",
			Description: "Update resource at URL",
			Parameters: []types.Parameter{
				{Name: "url", Type: "string", Description: "Request URL", Required: true},
				{Name: "data", Type: "object", Description: "Request body", Required: true},
				{Name: "headers", Type: "object", Description: "HTTP headers", Required: false},
			},
			Returns: "object",
		},
		{
			ID:          "http.patch",
			Name:        "HTTP PATCH",
			Description: "Partially update resource at URL",
			Parameters: []types.Parameter{
				{Name: "url", Type: "string", Description: "Request URL", Required: true},
				{Name: "data", Type: "object", Description: "Request body", Required: true},
				{Name: "headers", Type: "object", Description: "HTTP headers", Required: false},
			},
			Returns: "object",
		},
		{
			ID:          "http.delete",
			Name:        "HTTP DELETE",
			Description: "Delete resource at URL",
			Parameters: []types.Parameter{
				{Name: "url", Type: "string", Description: "Request URL", Required: true},
				{Name: "headers", Type: "object", Description: "HTTP headers", Required: false},
			},
			Returns: "object",
		},
		{
			ID:          "http.head",
			Name:        "HTTP HEAD",
			Description: "Get headers without downloading body",
			Parameters: []types.Parameter{
				{Name: "url", Type: "string", Description: "Request URL", Required: true},
				{Name: "headers", Type: "object", Description: "HTTP headers", Required: false},
			},
			Returns: "object",
		},
		{
			ID:          "http.options",
			Name:        "HTTP OPTIONS",
			Description: "Get allowed methods for URL",
			Parameters: []types.Parameter{
				{Name: "url", Type: "string", Description: "Request URL", Required: true},
				{Name: "headers", Type: "object", Description: "HTTP headers", Required: false},
			},
			Returns: "object",
		},
	}
}

// Get executes HTTP GET request with kernel permission checks
func (r *RequestsOps) Get(ctx context.Context, params map[string]interface{}, appCtx *types.Context) (*types.Result, error) {
	urlStr, err := client.GetString(params, "url", true)
	if err != nil {
		return client.Failure(err.Error())
	}

	// Step 1: Permission check with kernel (if available)
	if r.Kernel != nil {
		host := extractHost(urlStr)

		// Check permission via kernel (fast, small payload)
		_, err := r.Kernel.ExecuteSyscall(ctx, r.PID, "check_permission", map[string]interface{}{
			"resource": "network",
			"action":   "connect",
			"target":   host,
		})

		if err != nil {
			// Permission denied
			return client.Failure(fmt.Sprintf("permission denied: %v", err))
		}
	}

	// Step 2: Execute HTTP request locally (full-featured, fast)
	req, err := r.Client.Request(ctx)
	if err != nil {
		return client.Failure(err.Error())
	}

	// Add query parameters
	if queryParams := client.GetMap(params, "params"); queryParams != nil {
		for k, v := range queryParams {
			req.SetQueryParam(k, fmt.Sprint(v))
		}
	}

	// Add custom headers
	if headers := client.GetMap(params, "headers"); headers != nil {
		for k, v := range headers {
			req.SetHeader(k, fmt.Sprint(v))
		}
	}

	// Execute with circuit breaker protection
	resp, err := r.Client.ExecuteWithBreaker(func() (*resty.Response, error) {
		return req.Get(urlStr)
	})
	if err != nil {
		return client.Failure(fmt.Sprintf("request failed: %v", err))
	}

	// Step 3: Report metrics to kernel (async, non-blocking)
	if r.Kernel != nil {
		go func() {
			r.Kernel.ExecuteSyscall(context.Background(), r.PID, "emit_network_metric", map[string]interface{}{
				"host":           extractHost(urlStr),
				"method":         "GET",
				"status":         resp.StatusCode(),
				"bytes_received": len(resp.Body()),
				"duration_ms":    resp.Time().Milliseconds(),
			})
		}()
	}

	return client.Success(client.ResponseToMap(resp))
}

// Post executes HTTP POST request with kernel permission checks
func (r *RequestsOps) Post(ctx context.Context, params map[string]interface{}, appCtx *types.Context) (*types.Result, error) {
	urlStr, err := client.GetString(params, "url", true)
	if err != nil {
		return client.Failure(err.Error())
	}

	data := params["data"]
	if data == nil {
		return client.Failure("data parameter required")
	}

	// Step 1: Permission check with kernel
	if r.Kernel != nil {
		host := extractHost(urlStr)
		_, err := r.Kernel.ExecuteSyscall(ctx, r.PID, "check_permission", map[string]interface{}{
			"resource": "network",
			"action":   "connect",
			"target":   host,
		})
		if err != nil {
			return client.Failure(fmt.Sprintf("permission denied: %v", err))
		}
	}

	// Step 2: Execute HTTP request locally
	req, err := r.Client.Request(ctx)
	if err != nil {
		return client.Failure(err.Error())
	}

	// Add custom headers
	if headers := client.GetMap(params, "headers"); headers != nil {
		for k, v := range headers {
			req.SetHeader(k, fmt.Sprint(v))
		}
	}

	// Send as JSON or form data
	useJSON := client.GetBool(params, "json", true)
	var bodySize int
	if useJSON {
		req.SetBody(data)
		if jsonData, err := json.Marshal(data); err == nil {
			bodySize = len(jsonData)
		}
	} else {
		if dataMap, ok := data.(map[string]interface{}); ok {
			formData := make(map[string]string)
			for k, v := range dataMap {
				formData[k] = fmt.Sprint(v)
			}
			req.SetFormData(formData)
		} else {
			return client.Failure("data must be object for form encoding")
		}
	}

	// Execute with circuit breaker protection
	resp, err := r.Client.ExecuteWithBreaker(func() (*resty.Response, error) {
		return req.Post(urlStr)
	})
	if err != nil {
		return client.Failure(fmt.Sprintf("request failed: %v", err))
	}

	// Step 3: Report metrics to kernel
	if r.Kernel != nil {
		go func() {
			r.Kernel.ExecuteSyscall(context.Background(), r.PID, "emit_network_metric", map[string]interface{}{
				"host":           extractHost(urlStr),
				"method":         "POST",
				"status":         resp.StatusCode(),
				"bytes_sent":     bodySize,
				"bytes_received": len(resp.Body()),
				"duration_ms":    resp.Time().Milliseconds(),
			})
		}()
	}

	return client.Success(client.ResponseToMap(resp))
}

// Put executes HTTP PUT request with kernel permission checks
func (r *RequestsOps) Put(ctx context.Context, params map[string]interface{}, appCtx *types.Context) (*types.Result, error) {
	urlStr, err := client.GetString(params, "url", true)
	if err != nil {
		return client.Failure(err.Error())
	}

	data := params["data"]
	if data == nil {
		return client.Failure("data parameter required")
	}

	// Step 1: Permission check
	if r.Kernel != nil {
		host := extractHost(urlStr)
		_, err := r.Kernel.ExecuteSyscall(ctx, r.PID, "check_permission", map[string]interface{}{
			"resource": "network",
			"action":   "connect",
			"target":   host,
		})
		if err != nil {
			return client.Failure(fmt.Sprintf("permission denied: %v", err))
		}
	}

	// Step 2: Execute locally
	req, err := r.Client.Request(ctx)
	if err != nil {
		return client.Failure(err.Error())
	}

	if headers := client.GetMap(params, "headers"); headers != nil {
		for k, v := range headers {
			req.SetHeader(k, fmt.Sprint(v))
		}
	}

	req.SetBody(data)
	var bodySize int
	if jsonData, err := json.Marshal(data); err == nil {
		bodySize = len(jsonData)
	}

	resp, err := r.Client.ExecuteWithBreaker(func() (*resty.Response, error) {
		return req.Put(urlStr)
	})
	if err != nil {
		return client.Failure(fmt.Sprintf("request failed: %v", err))
	}

	// Step 3: Report metrics
	if r.Kernel != nil {
		go func() {
			r.Kernel.ExecuteSyscall(context.Background(), r.PID, "emit_network_metric", map[string]interface{}{
				"host":           extractHost(urlStr),
				"method":         "PUT",
				"status":         resp.StatusCode(),
				"bytes_sent":     bodySize,
				"bytes_received": len(resp.Body()),
				"duration_ms":    resp.Time().Milliseconds(),
			})
		}()
	}

	return client.Success(client.ResponseToMap(resp))
}

// Patch executes HTTP PATCH request with kernel permission checks
func (r *RequestsOps) Patch(ctx context.Context, params map[string]interface{}, appCtx *types.Context) (*types.Result, error) {
	urlStr, err := client.GetString(params, "url", true)
	if err != nil {
		return client.Failure(err.Error())
	}

	data := params["data"]
	if data == nil {
		return client.Failure("data parameter required")
	}

	// Permission check
	if r.Kernel != nil {
		host := extractHost(urlStr)
		_, err := r.Kernel.ExecuteSyscall(ctx, r.PID, "check_permission", map[string]interface{}{
			"resource": "network",
			"action":   "connect",
			"target":   host,
		})
		if err != nil {
			return client.Failure(fmt.Sprintf("permission denied: %v", err))
		}
	}

	req, err := r.Client.Request(ctx)
	if err != nil {
		return client.Failure(err.Error())
	}

	if headers := client.GetMap(params, "headers"); headers != nil {
		for k, v := range headers {
			req.SetHeader(k, fmt.Sprint(v))
		}
	}

	req.SetBody(data)
	var bodySize int
	if jsonData, err := json.Marshal(data); err == nil {
		bodySize = len(jsonData)
	}

	resp, err := r.Client.ExecuteWithBreaker(func() (*resty.Response, error) {
		return req.Patch(urlStr)
	})
	if err != nil {
		return client.Failure(fmt.Sprintf("request failed: %v", err))
	}

	if r.Kernel != nil {
		go func() {
			r.Kernel.ExecuteSyscall(context.Background(), r.PID, "emit_network_metric", map[string]interface{}{
				"host":           extractHost(urlStr),
				"method":         "PATCH",
				"status":         resp.StatusCode(),
				"bytes_sent":     bodySize,
				"bytes_received": len(resp.Body()),
				"duration_ms":    resp.Time().Milliseconds(),
			})
		}()
	}

	return client.Success(client.ResponseToMap(resp))
}

// Delete executes HTTP DELETE request with kernel permission checks
func (r *RequestsOps) Delete(ctx context.Context, params map[string]interface{}, appCtx *types.Context) (*types.Result, error) {
	urlStr, err := client.GetString(params, "url", true)
	if err != nil {
		return client.Failure(err.Error())
	}

	// Permission check
	if r.Kernel != nil {
		host := extractHost(urlStr)
		_, err := r.Kernel.ExecuteSyscall(ctx, r.PID, "check_permission", map[string]interface{}{
			"resource": "network",
			"action":   "connect",
			"target":   host,
		})
		if err != nil {
			return client.Failure(fmt.Sprintf("permission denied: %v", err))
		}
	}

	req, err := r.Client.Request(ctx)
	if err != nil {
		return client.Failure(err.Error())
	}

	if headers := client.GetMap(params, "headers"); headers != nil {
		for k, v := range headers {
			req.SetHeader(k, fmt.Sprint(v))
		}
	}

	resp, err := r.Client.ExecuteWithBreaker(func() (*resty.Response, error) {
		return req.Delete(urlStr)
	})
	if err != nil {
		return client.Failure(fmt.Sprintf("request failed: %v", err))
	}

	if r.Kernel != nil {
		go func() {
			r.Kernel.ExecuteSyscall(context.Background(), r.PID, "emit_network_metric", map[string]interface{}{
				"host":           extractHost(urlStr),
				"method":         "DELETE",
				"status":         resp.StatusCode(),
				"bytes_received": len(resp.Body()),
				"duration_ms":    resp.Time().Milliseconds(),
			})
		}()
	}

	return client.Success(client.ResponseToMap(resp))
}

// Head executes HTTP HEAD request with kernel permission checks
func (r *RequestsOps) Head(ctx context.Context, params map[string]interface{}, appCtx *types.Context) (*types.Result, error) {
	urlStr, err := client.GetString(params, "url", true)
	if err != nil {
		return client.Failure(err.Error())
	}

	// Permission check
	if r.Kernel != nil {
		host := extractHost(urlStr)
		_, err := r.Kernel.ExecuteSyscall(ctx, r.PID, "check_permission", map[string]interface{}{
			"resource": "network",
			"action":   "connect",
			"target":   host,
		})
		if err != nil {
			return client.Failure(fmt.Sprintf("permission denied: %v", err))
		}
	}

	req, err := r.Client.Request(ctx)
	if err != nil {
		return client.Failure(err.Error())
	}

	if headers := client.GetMap(params, "headers"); headers != nil {
		for k, v := range headers {
			req.SetHeader(k, fmt.Sprint(v))
		}
	}

	resp, err := r.Client.ExecuteWithBreaker(func() (*resty.Response, error) {
		return req.Head(urlStr)
	})
	if err != nil {
		return client.Failure(fmt.Sprintf("request failed: %v", err))
	}

	result := map[string]interface{}{
		"status":         resp.StatusCode(),
		"status_text":    resp.Status(),
		"content_length": resp.Header().Get("Content-Length"),
		"content_type":   resp.Header().Get("Content-Type"),
	}

	headers := make(map[string]string)
	for k, v := range resp.Header() {
		if len(v) > 0 {
			headers[k] = v[0]
		}
	}
	result["headers"] = headers

	if r.Kernel != nil {
		go func() {
			r.Kernel.ExecuteSyscall(context.Background(), r.PID, "emit_network_metric", map[string]interface{}{
				"host":        extractHost(urlStr),
				"method":      "HEAD",
				"status":      resp.StatusCode(),
				"duration_ms": resp.Time().Milliseconds(),
			})
		}()
	}

	return client.Success(result)
}

// Options executes HTTP OPTIONS request with kernel permission checks
func (r *RequestsOps) Options(ctx context.Context, params map[string]interface{}, appCtx *types.Context) (*types.Result, error) {
	urlStr, err := client.GetString(params, "url", true)
	if err != nil {
		return client.Failure(err.Error())
	}

	// Permission check
	if r.Kernel != nil {
		host := extractHost(urlStr)
		_, err := r.Kernel.ExecuteSyscall(ctx, r.PID, "check_permission", map[string]interface{}{
			"resource": "network",
			"action":   "connect",
			"target":   host,
		})
		if err != nil {
			return client.Failure(fmt.Sprintf("permission denied: %v", err))
		}
	}

	req, err := r.Client.Request(ctx)
	if err != nil {
		return client.Failure(err.Error())
	}

	if headers := client.GetMap(params, "headers"); headers != nil {
		for k, v := range headers {
			req.SetHeader(k, fmt.Sprint(v))
		}
	}

	resp, err := r.Client.ExecuteWithBreaker(func() (*resty.Response, error) {
		return req.Options(urlStr)
	})
	if err != nil {
		return client.Failure(fmt.Sprintf("request failed: %v", err))
	}

	if r.Kernel != nil {
		go func() {
			r.Kernel.ExecuteSyscall(context.Background(), r.PID, "emit_network_metric", map[string]interface{}{
				"host":           extractHost(urlStr),
				"method":         "OPTIONS",
				"status":         resp.StatusCode(),
				"bytes_received": len(resp.Body()),
				"duration_ms":    resp.Time().Milliseconds(),
			})
		}()
	}

	return client.Success(client.ResponseToMap(resp))
}
