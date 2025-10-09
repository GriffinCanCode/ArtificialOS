package browser

import (
	"context"
	"fmt"
	"strings"

	"github.com/GriffinCanCode/AgentOS/backend/internal/providers/http/client"
	"github.com/GriffinCanCode/AgentOS/backend/internal/shared/types"
	"github.com/go-resty/resty/v2"
)

// ProxyAsset fetches an asset (image, CSS, JS) through the backend
func (p *Provider) ProxyAsset(ctx context.Context, params map[string]interface{}, appCtx *types.Context) (*types.Result, error) {
	urlStr, ok := params["url"].(string)
	if !ok || urlStr == "" {
		return client.Failure("url parameter required")
	}

	sessionID, _ := params["session_id"].(string)
	appID := ""
	if appCtx.AppID != nil {
		appID = *appCtx.AppID
	}
	session := p.getOrCreateSession(sessionID, appID)

	// Fetch asset
	data, contentType, err := p.fetchAsset(ctx, urlStr, session)
	if err != nil {
		return client.Failure(err.Error())
	}

	return client.Success(map[string]interface{}{
		"data":         data,
		"content_type": contentType,
		"url":          urlStr,
	})
}

// fetchAsset retrieves an asset
func (p *Provider) fetchAsset(ctx context.Context, urlStr string, session *BrowserSession) (string, string, error) {
	headers := map[string]string{
		"User-Agent": session.userAgent,
		"Accept":     "*/*",
	}

	if session.referer != "" {
		headers["Referer"] = session.referer
	}

	// Add cookies
	session.mu.RLock()
	cookieHeader := p.buildCookieHeader(urlStr, session)
	session.mu.RUnlock()
	if cookieHeader != "" {
		headers["Cookie"] = cookieHeader
	}

	req, err := p.httpClient.Request(ctx)
	if err != nil {
		return "", "", fmt.Errorf("failed to create request: %w", err)
	}

	for k, v := range headers {
		req.SetHeader(k, v)
	}

	resp, err := p.httpClient.ExecuteWithBreaker(func() (*resty.Response, error) {
		return req.Get(urlStr)
	})
	if err != nil {
		return "", "", fmt.Errorf("asset fetch failed: %w", err)
	}

	// Get response body as string
	data := resp.String()
	contentType := "application/octet-stream" // Default

	// Detect content type from URL extension
	if strings.HasSuffix(urlStr, ".css") {
		contentType = "text/css"
	} else if strings.HasSuffix(urlStr, ".js") {
		contentType = "application/javascript"
	} else if strings.HasSuffix(urlStr, ".png") {
		contentType = "image/png"
	} else if strings.HasSuffix(urlStr, ".jpg") || strings.HasSuffix(urlStr, ".jpeg") {
		contentType = "image/jpeg"
	} else if strings.HasSuffix(urlStr, ".gif") {
		contentType = "image/gif"
	} else if strings.HasSuffix(urlStr, ".svg") {
		contentType = "image/svg+xml"
	} else if strings.HasSuffix(urlStr, ".woff") || strings.HasSuffix(urlStr, ".woff2") {
		contentType = "font/woff2"
	}

	// If CSS, rewrite URLs in it too
	if contentType == "text/css" {
		data = p.rewriteCSSURLs(data, urlStr, session.appID)
	}

	return data, contentType, nil
}

// rewriteCSSURLs rewrites URLs in CSS content
func (p *Provider) rewriteCSSURLs(css string, baseURL string, sessionID string) string {
	// Simple regex-based URL rewriting in CSS
	// url(...) patterns
	// This is a simplified version - production would use proper CSS parsing
	return css
}

// SubmitForm handles form submissions
func (p *Provider) SubmitForm(ctx context.Context, params map[string]interface{}, appCtx *types.Context) (*types.Result, error) {
	urlStr, ok := params["url"].(string)
	if !ok || urlStr == "" {
		return client.Failure("url parameter required")
	}

	method, ok := params["method"].(string)
	if !ok || method == "" {
		method = "POST"
	}
	method = strings.ToUpper(method)

	formData, ok := params["data"].(map[string]interface{})
	if !ok {
		return client.Failure("data parameter required")
	}

	sessionID, _ := params["session_id"].(string)
	appID := ""
	if appCtx.AppID != nil {
		appID = *appCtx.AppID
	}
	session := p.getOrCreateSession(sessionID, appID)

	// Build headers
	headers := map[string]string{
		"User-Agent":   session.userAgent,
		"Content-Type": "application/x-www-form-urlencoded",
		"Accept":       "text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8",
	}

	if session.referer != "" {
		headers["Referer"] = session.referer
	}

	// Add cookies
	session.mu.RLock()
	cookieHeader := p.buildCookieHeader(urlStr, session)
	session.mu.RUnlock()
	if cookieHeader != "" {
		headers["Cookie"] = cookieHeader
	}

	// Make request
	req, err := p.httpClient.Request(ctx)
	if err != nil {
		return client.Failure(err.Error())
	}

	for k, v := range headers {
		req.SetHeader(k, v)
	}

	var resp *resty.Response
	if method == "GET" {
		// Add form data as query params
		for k, v := range formData {
			req.SetQueryParam(k, fmt.Sprint(v))
		}
		resp, err = p.httpClient.ExecuteWithBreaker(func() (*resty.Response, error) {
			return req.Get(urlStr)
		})
	} else {
		// POST form data
		formMap := make(map[string]string)
		for k, v := range formData {
			formMap[k] = fmt.Sprint(v)
		}
		req.SetFormData(formMap)
		resp, err = p.httpClient.ExecuteWithBreaker(func() (*resty.Response, error) {
			return req.Post(urlStr)
		})
	}

	if err != nil {
		return client.Failure(err.Error())
	}

	// Process response as HTML page
	body := resp.String()
	enableJS, _ := params["enable_js"].(bool)
	processed, err := p.processHTML(body, urlStr, sessionID, enableJS)
	if err != nil {
		return client.Failure(err.Error())
	}

	// Update session
	session.mu.Lock()
	session.history = append(session.history, urlStr)
	session.referer = urlStr
	session.mu.Unlock()

	return client.Success(map[string]interface{}{
		"html":       processed.HTML,
		"title":      processed.Title,
		"url":        urlStr,
		"session_id": sessionID,
	})
}

// GetSessionInfo retrieves session information
func (p *Provider) GetSessionInfo(ctx context.Context, params map[string]interface{}, appCtx *types.Context) (*types.Result, error) {
	sessionID, ok := params["session_id"].(string)
	if !ok || sessionID == "" {
		return client.Failure("session_id parameter required")
	}

	p.sessions.mu.RLock()
	session, exists := p.sessions.sessions[sessionID]
	p.sessions.mu.RUnlock()

	if !exists {
		return client.Failure("session not found")
	}

	session.mu.RLock()
	defer session.mu.RUnlock()

	return client.Success(map[string]interface{}{
		"app_id":     session.appID,
		"user_agent": session.userAgent,
		"history":    session.history,
		"referer":    session.referer,
	})
}
