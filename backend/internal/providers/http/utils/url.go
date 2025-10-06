package utils

import (
	"github.com/GriffinCanCode/AgentOS/backend/internal/providers/http/client"
	"context"
	"fmt"
	"net/url"
	"strings"

	"github.com/GriffinCanCode/AgentOS/backend/internal/shared/types"
)

// URLOps handles URL manipulation
type URLOps struct {
	*client.HTTPOps
}

// GetTools returns URL tool definitions
func (u *URLOps) GetTools() []types.Tool {
	return []types.Tool{
		{
			ID:          "http.buildURL",
			Name:        "Build URL",
			Description: "Construct URL from components with proper encoding",
			Parameters: []types.Parameter{
				{Name: "base", Type: "string", Description: "Base URL", Required: true},
				{Name: "path", Type: "string", Description: "Path to append", Required: false},
				{Name: "params", Type: "object", Description: "Query parameters", Required: false},
				{Name: "fragment", Type: "string", Description: "URL fragment (#)", Required: false},
			},
			Returns: "string",
		},
		{
			ID:          "http.parseURL",
			Name:        "Parse URL",
			Description: "Parse URL into components",
			Parameters: []types.Parameter{
				{Name: "url", Type: "string", Description: "URL to parse", Required: true},
			},
			Returns: "object",
		},
		{
			ID:          "http.joinPath",
			Name:        "Join Path",
			Description: "Join URL path segments correctly",
			Parameters: []types.Parameter{
				{Name: "base", Type: "string", Description: "Base URL or path", Required: true},
				{Name: "segments", Type: "array", Description: "Path segments to join", Required: true},
			},
			Returns: "string",
		},
		{
			ID:          "http.encodeQuery",
			Name:        "Encode Query",
			Description: "Encode object as URL query string",
			Parameters: []types.Parameter{
				{Name: "params", Type: "object", Description: "Parameters to encode", Required: true},
			},
			Returns: "string",
		},
		{
			ID:          "http.decodeQuery",
			Name:        "Decode Query",
			Description: "Decode query string to object",
			Parameters: []types.Parameter{
				{Name: "query", Type: "string", Description: "Query string to decode", Required: true},
			},
			Returns: "object",
		},
	}
}

// BuildURL constructs URL from components with validation
func (u *URLOps) BuildURL(ctx context.Context, params map[string]interface{}, appCtx *types.Context) (*types.Result, error) {
	base, err := client.GetString(params, "base", true)
	if err != nil {
		return client.Failure(err.Error())
	}

	parsedURL, err := url.Parse(base)
	if err != nil {
		return client.Failure(fmt.Sprintf("invalid base URL: %v", err))
	}

	// Append path if provided
	if path, _ := client.GetString(params, "path", false); path != "" {
		// Ensure proper path joining
		if !strings.HasSuffix(parsedURL.Path, "/") && !strings.HasPrefix(path, "/") {
			parsedURL.Path += "/"
		}
		parsedURL.Path += strings.TrimPrefix(path, "/")
	}

	// Add query parameters
	if queryParams := client.GetMap(params, "params"); queryParams != nil {
		q := parsedURL.Query()
		for k, v := range queryParams {
			// Handle array values
			if arr, ok := v.([]interface{}); ok {
				for _, item := range arr {
					q.Add(k, fmt.Sprint(item))
				}
			} else {
				q.Set(k, fmt.Sprint(v))
			}
		}
		parsedURL.RawQuery = q.Encode()
	}

	// Set fragment if provided
	if fragment, _ := client.GetString(params, "fragment", false); fragment != "" {
		parsedURL.Fragment = strings.TrimPrefix(fragment, "#")
	}

	return client.Success(map[string]interface{}{
		"url": parsedURL.String(),
	})
}

// ParseURL parses URL into structured components
func (u *URLOps) ParseURL(ctx context.Context, params map[string]interface{}, appCtx *types.Context) (*types.Result, error) {
	urlStr, err := client.GetString(params, "url", true)
	if err != nil {
		return client.Failure(err.Error())
	}

	parsedURL, err := url.Parse(urlStr)
	if err != nil {
		return client.Failure(fmt.Sprintf("invalid URL: %v", err))
	}

	// Extract query params as map
	queryParams := make(map[string]interface{})
	for k, v := range parsedURL.Query() {
		if len(v) == 1 {
			queryParams[k] = v[0]
		} else {
			queryParams[k] = v
		}
	}

	result := map[string]interface{}{
		"scheme":   parsedURL.Scheme,
		"host":     parsedURL.Host,
		"path":     parsedURL.Path,
		"query":    queryParams,
		"fragment": parsedURL.Fragment,
		"raw":      urlStr,
	}

	// Extract hostname and port if available
	if parsedURL.Host != "" {
		result["hostname"] = parsedURL.Hostname()
		if port := parsedURL.Port(); port != "" {
			result["port"] = port
		}
	}

	// Include user info if present
	if parsedURL.User != nil {
		result["username"] = parsedURL.User.Username()
		if password, ok := parsedURL.User.Password(); ok {
			result["has_password"] = true
			// Don't expose actual password
			_ = password
		}
	}

	return client.Success(result)
}

// JoinPath joins URL path segments properly
func (u *URLOps) JoinPath(ctx context.Context, params map[string]interface{}, appCtx *types.Context) (*types.Result, error) {
	base, err := client.GetString(params, "base", true)
	if err != nil {
		return client.Failure(err.Error())
	}

	segments := client.GetArray(params, "segments")
	if len(segments) == 0 {
		return client.Failure("segments array required")
	}

	// Convert segments to strings
	strSegments := make([]string, 0, len(segments))
	for _, seg := range segments {
		strSegments = append(strSegments, fmt.Sprint(seg))
	}

	// Parse base to check if it's a full URL or path
	parsedURL, err := url.Parse(base)
	if err != nil {
		return client.Failure(fmt.Sprintf("invalid base: %v", err))
	}

	// Join path segments
	joinedPath := parsedURL.Path
	for _, seg := range strSegments {
		seg = strings.Trim(seg, "/")
		if seg != "" {
			if !strings.HasSuffix(joinedPath, "/") {
				joinedPath += "/"
			}
			joinedPath += seg
		}
	}

	var result string
	if parsedURL.Scheme != "" {
		// Full URL
		parsedURL.Path = joinedPath
		result = parsedURL.String()
	} else {
		// Just a path
		result = joinedPath
	}

	return client.Success(map[string]interface{}{
		"path": result,
	})
}

// EncodeQuery encodes params as query string
func (u *URLOps) EncodeQuery(ctx context.Context, params map[string]interface{}, appCtx *types.Context) (*types.Result, error) {
	queryParams := client.GetMap(params, "params")
	if queryParams == nil {
		return client.Failure("params object required")
	}

	q := url.Values{}
	for k, v := range queryParams {
		// Handle array values
		if arr, ok := v.([]interface{}); ok {
			for _, item := range arr {
				q.Add(k, fmt.Sprint(item))
			}
		} else {
			q.Set(k, fmt.Sprint(v))
		}
	}

	encoded := q.Encode()

	return client.Success(map[string]interface{}{
		"query":  encoded,
		"length": len(encoded),
	})
}

// DecodeQuery decodes query string to object
func (u *URLOps) DecodeQuery(ctx context.Context, params map[string]interface{}, appCtx *types.Context) (*types.Result, error) {
	queryStr, err := client.GetString(params, "query", true)
	if err != nil {
		return client.Failure(err.Error())
	}

	// Remove leading '?' if present
	queryStr = strings.TrimPrefix(queryStr, "?")

	parsed, err := url.ParseQuery(queryStr)
	if err != nil {
		return client.Failure(fmt.Sprintf("invalid query string: %v", err))
	}

	// Convert to map
	result := make(map[string]interface{})
	for k, v := range parsed {
		if len(v) == 1 {
			result[k] = v[0]
		} else {
			result[k] = v
		}
	}

	return client.Success(map[string]interface{}{
		"params": result,
		"count":  len(result),
	})
}
