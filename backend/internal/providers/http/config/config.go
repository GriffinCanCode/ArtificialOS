package config

import (
	"github.com/GriffinCanCode/AgentOS/backend/internal/providers/http/client"
	"context"
	"fmt"
	"time"

	"github.com/GriffinCanCode/AgentOS/backend/internal/shared/types"
)

// ConfigOps handles HTTP configuration
type ConfigOps struct {
	*client.HTTPOps
}

// GetTools returns config tool definitions
func (c *ConfigOps) GetTools() []types.Tool {
	return []types.Tool{
		{
			ID:          "http.setHeader",
			Name:        "Set Header",
			Description: "Set default header for subsequent requests",
			Parameters: []types.Parameter{
				{Name: "key", Type: "string", Description: "Header key", Required: true},
				{Name: "value", Type: "string", Description: "Header value", Required: true},
			},
			Returns: "boolean",
		},
		{
			ID:          "http.removeHeader",
			Name:        "Remove Header",
			Description: "Remove default header",
			Parameters: []types.Parameter{
				{Name: "key", Type: "string", Description: "Header key", Required: true},
			},
			Returns: "boolean",
		},
		{
			ID:          "http.getHeaders",
			Name:        "Get Headers",
			Description: "Get all default headers",
			Parameters:  []types.Parameter{},
			Returns:     "object",
		},
		{
			ID:          "http.setTimeout",
			Name:        "Set Timeout",
			Description: "Set request timeout in seconds",
			Parameters: []types.Parameter{
				{Name: "seconds", Type: "number", Description: "Timeout in seconds", Required: true},
			},
			Returns: "boolean",
		},
		{
			ID:          "http.setAuth",
			Name:        "Set Authentication",
			Description: "Configure request authentication",
			Parameters: []types.Parameter{
				{Name: "type", Type: "string", Description: "Auth type (basic|bearer|custom)", Required: true},
				{Name: "username", Type: "string", Description: "Username for basic auth", Required: false},
				{Name: "password", Type: "string", Description: "Password for basic auth", Required: false},
				{Name: "token", Type: "string", Description: "Token for bearer auth", Required: false},
				{Name: "header", Type: "string", Description: "Full auth header for custom", Required: false},
			},
			Returns: "boolean",
		},
		{
			ID:          "http.clearAuth",
			Name:        "Clear Authentication",
			Description: "Remove authentication configuration",
			Parameters:  []types.Parameter{},
			Returns:     "boolean",
		},
	}
}

// SetHeader sets default HTTP header
func (c *ConfigOps) SetHeader(ctx context.Context, params map[string]interface{}, appCtx *types.Context) (*types.Result, error) {
	key, err := client.GetString(params, "key", true)
	if err != nil {
		return client.Failure(err.Error())
	}

	value, err := client.GetString(params, "value", false)
	if err != nil {
		return client.Failure(err.Error())
	}

	c.Client.SetHeader(key, value)

	return client.Success(map[string]interface{}{
		"set": true,
		"key": key,
	})
}

// RemoveHeader removes default HTTP header
func (c *ConfigOps) RemoveHeader(ctx context.Context, params map[string]interface{}, appCtx *types.Context) (*types.Result, error) {
	key, err := client.GetString(params, "key", true)
	if err != nil {
		return client.Failure(err.Error())
	}

	c.Client.RemoveHeader(key)

	return client.Success(map[string]interface{}{
		"removed": true,
		"key":     key,
	})
}

// GetHeaders returns all default headers
func (c *ConfigOps) GetHeaders(ctx context.Context, params map[string]interface{}, appCtx *types.Context) (*types.Result, error) {
	headers := c.Client.GetHeaders()

	return client.Success(map[string]interface{}{
		"headers": headers,
		"count":   len(headers),
	})
}

// SetTimeout configures request timeout
func (c *ConfigOps) SetTimeout(ctx context.Context, params map[string]interface{}, appCtx *types.Context) (*types.Result, error) {
	seconds, err := client.GetNumber(params, "seconds", true)
	if err != nil {
		return client.Failure(err.Error())
	}

	if seconds <= 0 {
		return client.Failure("seconds must be positive number")
	}

	duration := time.Duration(seconds*1000) * time.Millisecond
	c.Client.SetTimeout(duration)

	return client.Success(map[string]interface{}{
		"set":     true,
		"seconds": seconds,
	})
}

// SetAuth configures authentication
func (c *ConfigOps) SetAuth(ctx context.Context, params map[string]interface{}, appCtx *types.Context) (*types.Result, error) {
	authType, err := client.GetString(params, "type", true)
	if err != nil {
		return client.Failure(err.Error())
	}

	switch authType {
	case "basic":
		username, err := client.GetString(params, "username", true)
		if err != nil {
			return client.Failure("username required for basic auth")
		}

		password, err := client.GetString(params, "password", true)
		if err != nil {
			return client.Failure("password required for basic auth")
		}

		c.Client.SetBasicAuth(username, password)

	case "bearer":
		token, err := client.GetString(params, "token", true)
		if err != nil {
			return client.Failure("token required for bearer auth")
		}

		c.Client.SetBearerAuth(token)

	case "custom":
		header, err := client.GetString(params, "header", true)
		if err != nil {
			return client.Failure("header required for custom auth")
		}

		c.Client.SetCustomAuth(header)

	default:
		return client.Failure(fmt.Sprintf("invalid auth type: %s (must be: basic, bearer, or custom)", authType))
	}

	return client.Success(map[string]interface{}{
		"set":  true,
		"type": authType,
	})
}

// ClearAuth removes authentication
func (c *ConfigOps) ClearAuth(ctx context.Context, params map[string]interface{}, appCtx *types.Context) (*types.Result, error) {
	c.Client.RemoveHeader("Authorization")

	return client.Success(map[string]interface{}{
		"cleared": true,
	})
}
