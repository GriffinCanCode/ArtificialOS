package http

import (
	"context"
	"encoding/base64"
	"fmt"
	"sync"
	"time"

	"github.com/GriffinCanCode/AgentOS/backend/internal/types"
	"github.com/go-resty/resty/v2"
	"github.com/hashicorp/go-retryablehttp"
	"golang.org/x/time/rate"
)

// Client wraps resty with rate limiting and advanced features
type Client struct {
	resty   *resty.Client
	limiter *rate.Limiter
	mu      sync.RWMutex
}

// RetryConfig defines retry behavior
type RetryConfig struct {
	MaxRetries int
	MinWait    time.Duration
	MaxWait    time.Duration
}

// HTTPOps provides base functionality for all HTTP modules
type HTTPOps struct {
	Client *Client
}

// NewClient creates production-ready HTTP client
func NewClient() *Client {
	// Create underlying retryable client
	retryClient := retryablehttp.NewClient()
	retryClient.RetryMax = 3
	retryClient.RetryWaitMin = 1 * time.Second
	retryClient.RetryWaitMax = 30 * time.Second
	retryClient.Logger = nil // Disable logging

	// Create resty client with retry support
	restyClient := resty.New()
	restyClient.
		SetTimeout(30*time.Second).
		SetRetryCount(3).
		SetRetryWaitTime(1*time.Second).
		SetRetryMaxWaitTime(30*time.Second).
		SetHeader("User-Agent", "AgentOS-HTTP/1.0")

	// Configure transport settings
	restyClient.SetTransport(retryClient.HTTPClient.Transport)

	return &Client{
		resty:   restyClient,
		limiter: rate.NewLimiter(rate.Inf, 0), // Unlimited by default
	}
}

// SetHeader adds default header
func (c *Client) SetHeader(key, value string) {
	c.mu.Lock()
	defer c.mu.Unlock()
	c.resty.SetHeader(key, value)
}

// RemoveHeader removes a default header
func (c *Client) RemoveHeader(key string) {
	c.mu.Lock()
	defer c.mu.Unlock()
	c.resty.Header.Del(key)
}

// GetHeaders returns copy of all headers
func (c *Client) GetHeaders() map[string]string {
	c.mu.RLock()
	defer c.mu.RUnlock()

	headers := make(map[string]string)
	for k, v := range c.resty.Header {
		if len(v) > 0 {
			headers[k] = v[0]
		}
	}
	return headers
}

// SetTimeout configures request timeout
func (c *Client) SetTimeout(duration time.Duration) {
	c.mu.Lock()
	defer c.mu.Unlock()
	c.resty.SetTimeout(duration)
}

// SetRetry configures retry behavior
func (c *Client) SetRetry(maxRetries int, minWait, maxWait time.Duration) {
	c.mu.Lock()
	defer c.mu.Unlock()
	c.resty.SetRetryCount(maxRetries).
		SetRetryWaitTime(minWait).
		SetRetryMaxWaitTime(maxWait)
}

// SetRateLimit configures rate limiting (requests per second)
func (c *Client) SetRateLimit(rps float64) {
	c.mu.Lock()
	defer c.mu.Unlock()
	if rps <= 0 {
		c.limiter = rate.NewLimiter(rate.Inf, 0)
	} else {
		c.limiter = rate.NewLimiter(rate.Limit(rps), int(rps))
	}
}

// SetBasicAuth configures basic authentication
func (c *Client) SetBasicAuth(username, password string) {
	c.mu.Lock()
	defer c.mu.Unlock()
	c.resty.SetBasicAuth(username, password)
}

// SetBearerAuth configures bearer token authentication
func (c *Client) SetBearerAuth(token string) {
	c.mu.Lock()
	defer c.mu.Unlock()
	c.resty.SetAuthToken(token)
}

// SetCustomAuth sets custom authorization header
func (c *Client) SetCustomAuth(header string) {
	c.SetHeader("Authorization", header)
}

// Request creates new request with rate limiting
func (c *Client) Request(ctx context.Context) (*resty.Request, error) {
	// Wait for rate limiter
	if err := c.limiter.Wait(ctx); err != nil {
		return nil, fmt.Errorf("rate limit error: %w", err)
	}

	c.mu.RLock()
	defer c.mu.RUnlock()
	return c.resty.R().SetContext(ctx), nil
}

// Success creates successful result
func Success(data map[string]interface{}) (*types.Result, error) {
	return &types.Result{Success: true, Data: data}, nil
}

// Failure creates failed result
func Failure(message string) (*types.Result, error) {
	msg := message
	return &types.Result{Success: false, Error: &msg}, nil
}

// GetString extracts string parameter
func GetString(params map[string]interface{}, key string, required bool) (string, error) {
	val, ok := params[key]
	if !ok || val == nil {
		if required {
			return "", fmt.Errorf("%s parameter required", key)
		}
		return "", nil
	}

	str, ok := val.(string)
	if !ok {
		return "", fmt.Errorf("%s must be string", key)
	}

	if required && str == "" {
		return "", fmt.Errorf("%s cannot be empty", key)
	}

	return str, nil
}

// GetBool extracts bool parameter
func GetBool(params map[string]interface{}, key string, defaultVal bool) bool {
	val, ok := params[key]
	if !ok {
		return defaultVal
	}

	b, ok := val.(bool)
	if !ok {
		return defaultVal
	}

	return b
}

// GetNumber extracts numeric parameter
func GetNumber(params map[string]interface{}, key string, required bool) (float64, error) {
	val, ok := params[key]
	if !ok || val == nil {
		if required {
			return 0, fmt.Errorf("%s parameter required", key)
		}
		return 0, nil
	}

	switch v := val.(type) {
	case float64:
		return v, nil
	case int:
		return float64(v), nil
	case int64:
		return float64(v), nil
	case float32:
		return float64(v), nil
	default:
		return 0, fmt.Errorf("%s must be number", key)
	}
}

// GetMap extracts map parameter
func GetMap(params map[string]interface{}, key string) map[string]interface{} {
	val, ok := params[key]
	if !ok {
		return nil
	}

	m, ok := val.(map[string]interface{})
	if !ok {
		return nil
	}

	return m
}

// GetArray extracts array parameter
func GetArray(params map[string]interface{}, key string) []interface{} {
	val, ok := params[key]
	if !ok {
		return nil
	}

	arr, ok := val.([]interface{})
	if !ok {
		return nil
	}

	return arr
}

// EncodeBasicAuth creates base64 encoded basic auth
func EncodeBasicAuth(username, password string) string {
	auth := username + ":" + password
	return "Basic " + base64.StdEncoding.EncodeToString([]byte(auth))
}

// ResponseToMap converts resty response to result map
func ResponseToMap(resp *resty.Response) map[string]interface{} {
	result := map[string]interface{}{
		"status":      resp.StatusCode(),
		"status_text": resp.Status(),
		"body":        resp.String(),
		"size":        len(resp.Body()),
		"time":        resp.Time().Milliseconds(),
	}

	// Convert headers to map
	headers := make(map[string]string)
	for k, v := range resp.Header() {
		if len(v) > 0 {
			headers[k] = v[0]
		}
	}
	result["headers"] = headers

	return result
}
