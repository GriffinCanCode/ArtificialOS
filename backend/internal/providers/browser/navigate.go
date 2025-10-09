package browser

import (
	"context"
	"fmt"
	"net/url"
	"regexp"
	"strings"

	"github.com/GriffinCanCode/AgentOS/backend/internal/providers/http/client"
	"github.com/GriffinCanCode/AgentOS/backend/internal/shared/types"
	"github.com/PuerkitoBio/goquery"
	"github.com/go-resty/resty/v2"
)

// Navigate fetches and processes a web page
func (p *Provider) Navigate(ctx context.Context, params map[string]interface{}, appCtx *types.Context) (*types.Result, error) {
	// Extract parameters
	urlStr, ok := params["url"].(string)
	if !ok || urlStr == "" {
		return client.Failure("url parameter required")
	}

	sessionID, _ := params["session_id"].(string)
	enableJS, _ := params["enable_js"].(bool)

	appID := ""
	if appCtx.AppID != nil {
		appID = *appCtx.AppID
	}
	session := p.getOrCreateSession(sessionID, appID)

	// Fetch page via HTTP provider
	pageData, err := p.fetchPage(ctx, urlStr, session)
	if err != nil {
		return client.Failure(err.Error())
	}

	// Process HTML
	processed, err := p.processHTML(pageData.Body, urlStr, sessionID, enableJS)
	if err != nil {
		return client.Failure(fmt.Sprintf("failed to process HTML: %v", err))
	}

	// Update session
	session.mu.Lock()
	session.history = append(session.history, urlStr)
	session.referer = urlStr
	session.mu.Unlock()

	return client.Success(map[string]interface{}{
		"html":         processed.HTML,
		"title":        processed.Title,
		"url":          urlStr,
		"status":       pageData.Status,
		"content_type": pageData.ContentType,
		"session_id":   sessionID,
	})
}

// PageData holds fetched page information
type PageData struct {
	Body        string
	Status      int
	ContentType string
	Headers     map[string]string
}

// ProcessedHTML holds processed HTML data
type ProcessedHTML struct {
	HTML  string
	Title string
}

// fetchPage retrieves the raw page content
func (p *Provider) fetchPage(ctx context.Context, urlStr string, session *BrowserSession) (*PageData, error) {
	// Build headers
	headers := map[string]string{
		"User-Agent":                session.userAgent,
		"Accept":                    "text/html,application/xhtml+xml,application/xml;q=0.9,image/webp,*/*;q=0.8",
		"Accept-Language":           "en-US,en;q=0.9",
		"Accept-Encoding":           "gzip, deflate, br",
		"DNT":                       "1",
		"Connection":                "keep-alive",
		"Upgrade-Insecure-Requests": "1",
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

	// Make request through HTTP client
	req, err := p.httpClient.Request(ctx)
	if err != nil {
		return nil, fmt.Errorf("failed to create request: %w", err)
	}

	for k, v := range headers {
		req.SetHeader(k, v)
	}

	resp, err := p.httpClient.ExecuteWithBreaker(func() (*resty.Response, error) {
		return req.Get(urlStr)
	})
	if err != nil {
		return nil, fmt.Errorf("request failed: %w", err)
	}

	// Check response status
	statusCode := resp.StatusCode()
	if statusCode < 200 || statusCode >= 400 {
		return nil, fmt.Errorf("HTTP %d: %s (url: %s)", statusCode, resp.Status(), urlStr)
	}

	// Extract response body
	body := resp.String()

	if body == "" {
		return nil, fmt.Errorf("empty response body from %s (status: %d, content-type: %s)",
			urlStr, statusCode, resp.Header().Get("Content-Type"))
	}

	return &PageData{
		Body:        body,
		Status:      statusCode,
		ContentType: resp.Header().Get("Content-Type"),
		Headers:     headers,
	}, nil
}

// processHTML rewrites URLs and makes the page browseable
func (p *Provider) processHTML(html string, baseURL string, sessionID string, enableJS bool) (*ProcessedHTML, error) {
	doc, err := goquery.NewDocumentFromReader(strings.NewReader(html))
	if err != nil {
		return nil, fmt.Errorf("failed to parse HTML: %w", err)
	}

	parsedBase, err := url.Parse(baseURL)
	if err != nil {
		return nil, fmt.Errorf("invalid base URL: %w", err)
	}

	// Extract title
	title := doc.Find("title").First().Text()
	if title == "" {
		title = parsedBase.Host
	}

	// Always remove scripts for now - sandboxed execution needs more work
	// TODO: Implement full sandbox execution with proper DOM parsing
	doc.Find("script").Remove()
	_ = enableJS // Suppress unused warning

	// Remove inline event handlers (not safe even with sandboxing)
	doc.Find("[onclick]").RemoveAttr("onclick")
	doc.Find("[onload]").RemoveAttr("onload")
	doc.Find("[onerror]").RemoveAttr("onerror")
	doc.Find("[onsubmit]").RemoveAttr("onsubmit")

	// Rewrite all URLs to go through proxy
	p.rewriteLinks(doc, parsedBase, sessionID)
	p.rewriteImages(doc, parsedBase, sessionID)
	p.rewriteStylesheets(doc, parsedBase, sessionID)
	p.rewriteForms(doc, parsedBase, sessionID)

	// Add base tag for relative URLs
	if doc.Find("base").Length() == 0 {
		doc.Find("head").PrependHtml(fmt.Sprintf(`<base href="%s">`, baseURL))
	}

	// Get processed HTML
	processedHTML, err := doc.Html()
	if err != nil {
		return nil, fmt.Errorf("failed to render HTML: %w", err)
	}

	return &ProcessedHTML{
		HTML:  processedHTML,
		Title: title,
	}, nil
}

// rewriteLinks rewrites anchor tags
func (p *Provider) rewriteLinks(doc *goquery.Document, base *url.URL, sessionID string) {
	doc.Find("a[href]").Each(func(i int, s *goquery.Selection) {
		href, exists := s.Attr("href")
		if !exists || href == "" || strings.HasPrefix(href, "#") {
			return
		}

		absolute := p.resolveURL(href, base)
		if absolute != "" {
			// Rewrite to use our navigation
			proxyURL := p.createProxyURL("navigate", absolute, sessionID)
			s.SetAttr("href", proxyURL)
			s.SetAttr("data-original-href", absolute)
		}
	})
}

// rewriteImages rewrites image sources
func (p *Provider) rewriteImages(doc *goquery.Document, base *url.URL, sessionID string) {
	doc.Find("img[src]").Each(func(i int, s *goquery.Selection) {
		src, exists := s.Attr("src")
		if !exists || src == "" {
			return
		}

		absolute := p.resolveURL(src, base)
		if absolute != "" {
			proxyURL := p.createProxyURL("asset", absolute, sessionID)
			s.SetAttr("src", proxyURL)
			s.SetAttr("data-original-src", absolute)
		}
	})
}

// rewriteStylesheets rewrites CSS links
func (p *Provider) rewriteStylesheets(doc *goquery.Document, base *url.URL, sessionID string) {
	doc.Find("link[rel='stylesheet'][href]").Each(func(i int, s *goquery.Selection) {
		href, exists := s.Attr("href")
		if !exists || href == "" {
			return
		}

		absolute := p.resolveURL(href, base)
		if absolute != "" {
			proxyURL := p.createProxyURL("asset", absolute, sessionID)
			s.SetAttr("href", proxyURL)
		}
	})
}

// rewriteForms rewrites form actions
func (p *Provider) rewriteForms(doc *goquery.Document, base *url.URL, sessionID string) {
	doc.Find("form[action]").Each(func(i int, s *goquery.Selection) {
		action, exists := s.Attr("action")
		if !exists || action == "" {
			return
		}

		absolute := p.resolveURL(action, base)
		if absolute != "" {
			proxyURL := p.createProxyURL("form", absolute, sessionID)
			s.SetAttr("action", proxyURL)
			s.SetAttr("data-original-action", absolute)
		}
	})
}

// resolveURL converts relative URLs to absolute
func (p *Provider) resolveURL(href string, base *url.URL) string {
	// Skip unsafe URLs
	lowerHref := strings.ToLower(href)
	if strings.HasPrefix(lowerHref, "data:") ||
		strings.HasPrefix(lowerHref, "javascript:") ||
		strings.HasPrefix(lowerHref, "mailto:") ||
		strings.HasPrefix(lowerHref, "tel:") ||
		strings.HasPrefix(lowerHref, "vbscript:") {
		return ""
	}

	parsed, err := url.Parse(href)
	if err != nil {
		return ""
	}

	resolved := base.ResolveReference(parsed)
	return resolved.String()
}

// createProxyURL creates a URL that routes through our proxy
func (p *Provider) createProxyURL(action string, targetURL string, sessionID string) string {
	// For now, return the actual URL
	// TODO: Implement proper frontend interception for navigation
	// For links, we want them to trigger browser.navigate when clicked
	return targetURL
}

// buildCookieHeader builds Cookie header from session cookies
func (p *Provider) buildCookieHeader(urlStr string, session *BrowserSession) string {
	parsed, err := url.Parse(urlStr)
	if err != nil {
		return ""
	}

	var cookies []string
	if domainCookies, ok := session.cookies[parsed.Host]; ok {
		for _, cookie := range domainCookies {
			cookies = append(cookies, fmt.Sprintf("%s=%s", cookie.Name, cookie.Value))
		}
	}

	return strings.Join(cookies, "; ")
}

// parseCookies parses Set-Cookie headers
func (p *Provider) parseCookies(headers map[string]string, domain string) []*Cookie {
	var cookies []*Cookie

	setCookieHeader, ok := headers["Set-Cookie"]
	if !ok {
		return cookies
	}

	// Simple cookie parsing (could be enhanced)
	re := regexp.MustCompile(`([^=]+)=([^;]+)`)
	matches := re.FindAllStringSubmatch(setCookieHeader, -1)

	for _, match := range matches {
		if len(match) >= 3 {
			cookies = append(cookies, &Cookie{
				Name:   strings.TrimSpace(match[1]),
				Value:  strings.TrimSpace(match[2]),
				Domain: domain,
				Path:   "/",
			})
		}
	}

	return cookies
}
