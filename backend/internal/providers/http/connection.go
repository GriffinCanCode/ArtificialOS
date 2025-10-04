package http

import (
	"context"
	"crypto/tls"
	"fmt"
	"net/http"
	"net/url"

	"github.com/GriffinCanCode/AgentOS/backend/internal/types"
)

// ConnectionOps handles connection settings
type ConnectionOps struct {
	*HTTPOps
}

// GetTools returns connection tool definitions
func (c *ConnectionOps) GetTools() []types.Tool {
	return []types.Tool{
		{
			ID:          "http.setProxy",
			Name:        "Set Proxy",
			Description: "Configure HTTP/HTTPS proxy",
			Parameters: []types.Parameter{
				{Name: "proxy_url", Type: "string", Description: "Proxy URL (http://host:port)", Required: true},
			},
			Returns: "boolean",
		},
		{
			ID:          "http.removeProxy",
			Name:        "Remove Proxy",
			Description: "Remove proxy configuration",
			Parameters:  []types.Parameter{},
			Returns:     "boolean",
		},
		{
			ID:          "http.setVerifySSL",
			Name:        "Set SSL Verification",
			Description: "Enable/disable SSL certificate verification",
			Parameters: []types.Parameter{
				{Name: "verify", Type: "boolean", Description: "Verify SSL certificates", Required: true},
			},
			Returns: "boolean",
		},
		{
			ID:          "http.setFollowRedirects",
			Name:        "Set Redirect Policy",
			Description: "Configure automatic redirect following",
			Parameters: []types.Parameter{
				{Name: "follow", Type: "boolean", Description: "Follow redirects", Required: true},
				{Name: "max_redirects", Type: "number", Description: "Maximum redirects (default: 10)", Required: false},
			},
			Returns: "boolean",
		},
		{
			ID:          "http.setCookieJar",
			Name:        "Set Cookie Jar",
			Description: "Enable/disable automatic cookie handling",
			Parameters: []types.Parameter{
				{Name: "enabled", Type: "boolean", Description: "Enable cookie jar", Required: true},
			},
			Returns: "boolean",
		},
	}
}

// SetProxy configures HTTP proxy
func (c *ConnectionOps) SetProxy(ctx context.Context, params map[string]interface{}, appCtx *types.Context) (*types.Result, error) {
	proxyURL, err := GetString(params, "proxy_url", true)
	if err != nil {
		return Failure(err.Error())
	}

	// Validate proxy URL
	parsedProxy, err := url.Parse(proxyURL)
	if err != nil {
		return Failure(fmt.Sprintf("invalid proxy URL: %v", err))
	}

	if parsedProxy.Scheme != "http" && parsedProxy.Scheme != "https" {
		return Failure("proxy URL must use http or https scheme")
	}

	// Set proxy using resty
	c.Client.mu.Lock()
	c.Client.resty.SetProxy(proxyURL)
	c.Client.mu.Unlock()

	return Success(map[string]interface{}{
		"set":   true,
		"proxy": proxyURL,
		"host":  parsedProxy.Host,
	})
}

// RemoveProxy removes proxy configuration
func (c *ConnectionOps) RemoveProxy(ctx context.Context, params map[string]interface{}, appCtx *types.Context) (*types.Result, error) {
	c.Client.mu.Lock()
	c.Client.resty.RemoveProxy()
	c.Client.mu.Unlock()

	return Success(map[string]interface{}{
		"removed": true,
	})
}

// SetVerifySSL configures SSL certificate verification
func (c *ConnectionOps) SetVerifySSL(ctx context.Context, params map[string]interface{}, appCtx *types.Context) (*types.Result, error) {
	verify := GetBool(params, "verify", true)

	c.Client.mu.Lock()
	c.Client.resty.SetTLSClientConfig(&tls.Config{
		InsecureSkipVerify: !verify,
	})
	c.Client.mu.Unlock()

	warning := ""
	if !verify {
		warning = "SSL verification disabled - use only for testing/development"
	}

	result := map[string]interface{}{
		"set":    true,
		"verify": verify,
	}

	if warning != "" {
		result["warning"] = warning
	}

	return Success(result)
}

// SetFollowRedirects configures redirect policy
func (c *ConnectionOps) SetFollowRedirects(ctx context.Context, params map[string]interface{}, appCtx *types.Context) (*types.Result, error) {
	follow := GetBool(params, "follow", true)

	maxRedirects := 10
	if max, err := GetNumber(params, "max_redirects", false); err == nil && max > 0 {
		maxRedirects = int(max)
	}

	c.Client.mu.Lock()
	defer c.Client.mu.Unlock()

	if follow {
		// Enable redirects with custom policy
		c.Client.resty.SetRedirectPolicy(func(req *http.Request, via []*http.Request) error {
			if len(via) >= maxRedirects {
				return fmt.Errorf("stopped after %d redirects", maxRedirects)
			}
			return nil
		})
	} else {
		// Disable redirects
		c.Client.resty.SetRedirectPolicy(func(req *http.Request, via []*http.Request) error {
			return http.ErrUseLastResponse
		})
	}

	return Success(map[string]interface{}{
		"set":           true,
		"follow":        follow,
		"max_redirects": maxRedirects,
	})
}

// SetCookieJar enables/disables automatic cookie handling
func (c *ConnectionOps) SetCookieJar(ctx context.Context, params map[string]interface{}, appCtx *types.Context) (*types.Result, error) {
	enabled := GetBool(params, "enabled", true)

	c.Client.mu.Lock()
	defer c.Client.mu.Unlock()

	if enabled {
		// Resty enables cookie jar by default
		c.Client.resty.SetCookieJar(nil) // Use default jar
	} else {
		// Create a no-op cookie jar
		c.Client.resty.SetCookieJar(http.CookieJar(nil))
	}

	return Success(map[string]interface{}{
		"set":     true,
		"enabled": enabled,
	})
}
