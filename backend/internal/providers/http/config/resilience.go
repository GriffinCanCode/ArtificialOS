package config

import (
	"context"
	"github.com/GriffinCanCode/AgentOS/backend/internal/providers/http/client"
	"time"

	"github.com/GriffinCanCode/AgentOS/backend/internal/shared/types"
)

// ResilienceOps handles retry and rate limiting
type ResilienceOps struct {
	*client.HTTPOps
}

// GetTools returns resilience tool definitions
func (r *ResilienceOps) GetTools() []types.Tool {
	return []types.Tool{
		{
			ID:          "http.setRetry",
			Name:        "Set Retry Policy",
			Description: "Configure automatic retry on failure with exponential backoff",
			Parameters: []types.Parameter{
				{Name: "max_retries", Type: "number", Description: "Maximum retry attempts (0-10)", Required: true},
				{Name: "min_wait_seconds", Type: "number", Description: "Minimum wait between retries (default: 1)", Required: false},
				{Name: "max_wait_seconds", Type: "number", Description: "Maximum wait between retries (default: 30)", Required: false},
			},
			Returns: "boolean",
		},
		{
			ID:          "http.setRateLimit",
			Name:        "Set Rate Limit",
			Description: "Limit requests per second (thread-safe with token bucket)",
			Parameters: []types.Parameter{
				{Name: "requests_per_second", Type: "number", Description: "Max requests per second (0 = unlimited)", Required: true},
			},
			Returns: "boolean",
		},
		{
			ID:          "http.getRateLimit",
			Name:        "Get Rate Limit",
			Description: "Get current rate limit configuration",
			Parameters:  []types.Parameter{},
			Returns:     "object",
		},
	}
}

// SetRetry configures retry policy with exponential backoff
func (r *ResilienceOps) SetRetry(ctx context.Context, params map[string]interface{}, appCtx *types.Context) (*types.Result, error) {
	maxRetries, err := client.GetNumber(params, "max_retries", true)
	if err != nil {
		return client.Failure(err.Error())
	}

	if maxRetries < 0 || maxRetries > 10 {
		return client.Failure("max_retries must be between 0 and 10")
	}

	minWait := 1.0
	if mw, err := client.GetNumber(params, "min_wait_seconds", false); err == nil && mw > 0 {
		minWait = mw
	}

	maxWait := 30.0
	if mw, err := client.GetNumber(params, "max_wait_seconds", false); err == nil && mw > 0 {
		maxWait = mw
	}

	if minWait > maxWait {
		return client.Failure("min_wait_seconds cannot exceed max_wait_seconds")
	}

	minWaitDuration := time.Duration(minWait*1000) * time.Millisecond
	maxWaitDuration := time.Duration(maxWait*1000) * time.Millisecond

	r.Client.SetRetry(int(maxRetries), minWaitDuration, maxWaitDuration)

	return client.Success(map[string]interface{}{
		"set":              true,
		"max_retries":      int(maxRetries),
		"min_wait_seconds": minWait,
		"max_wait_seconds": maxWait,
		"strategy":         "exponential_backoff",
	})
}

// SetRateLimit configures rate limiting using token bucket algorithm
func (r *ResilienceOps) SetRateLimit(ctx context.Context, params map[string]interface{}, appCtx *types.Context) (*types.Result, error) {
	rps, err := client.GetNumber(params, "requests_per_second", true)
	if err != nil {
		return client.Failure(err.Error())
	}

	if rps < 0 {
		return client.Failure("requests_per_second cannot be negative")
	}

	r.Client.SetRateLimit(rps)

	result := map[string]interface{}{
		"set":                 true,
		"requests_per_second": rps,
		"algorithm":           "token_bucket",
		"thread_safe":         true,
	}

	if rps > 0 {
		result["delay_between_requests_ms"] = 1000.0 / rps
	} else {
		result["delay_between_requests_ms"] = 0
		result["unlimited"] = true
	}

	return client.Success(result)
}

// GetRateLimit returns current rate limit configuration
func (r *ResilienceOps) GetRateLimit(ctx context.Context, params map[string]interface{}, appCtx *types.Context) (*types.Result, error) {
	// Note: We can't directly access limiter config from resty,
	// so we return basic info
	return client.Success(map[string]interface{}{
		"configured":  true,
		"algorithm":   "token_bucket",
		"thread_safe": true,
	})
}
