package scraper

import (
	"context"
	"fmt"
	"strings"

	"github.com/GriffinCanCode/AgentOS/backend/internal/shared/types"
	"github.com/PuerkitoBio/goquery"
)

// ExtractOps handles smart content extraction
type ExtractOps struct {
	*ScraperOps
}

// GetTools returns extraction tool definitions
func (e *ExtractOps) GetTools() []types.Tool {
	return []types.Tool{
		{
			ID:          "scraper.article",
			Name:        "Extract Article",
			Description: "Extract main article content (removes nav, ads, etc.)",
			Parameters: []types.Parameter{
				{Name: "html", Type: "string", Description: "HTML content", Required: true},
			},
			Returns: "object",
		},
		{
			ID:          "scraper.clean",
			Name:        "Clean HTML",
			Description: "Sanitize HTML and remove unwanted elements",
			Parameters: []types.Parameter{
				{Name: "html", Type: "string", Description: "HTML content", Required: true},
			},
			Returns: "object",
		},
		{
			ID:          "scraper.summary",
			Name:        "Extract Summary",
			Description: "Get page summary from meta description or first paragraph",
			Parameters: []types.Parameter{
				{Name: "html", Type: "string", Description: "HTML content", Required: true},
				{Name: "max_length", Type: "number", Description: "Max summary length (default: 300)", Required: false},
			},
			Returns: "object",
		},
	}
}

// ExtractArticle extracts main article content using heuristics
func (e *ExtractOps) ExtractArticle(ctx context.Context, params map[string]interface{}, appCtx *types.Context) (*types.Result, error) {
	html, ok := GetString(params, "html")
	if !ok || html == "" {
		return Failure("html parameter required")
	}

	doc, err := LoadHTML(html)
	if err != nil {
		return Failure(fmt.Sprintf("parse failed: %v", err))
	}

	// Remove unwanted elements
	doc.Find("script, style, nav, header, footer, aside, iframe, .ad, .advertisement, .sidebar").Remove()

	// Try to find main content container
	var mainContent *goquery.Selection

	// Check for semantic HTML5 tags
	if main := doc.Find("main, article").First(); main.Length() > 0 {
		mainContent = main
	} else if role := doc.Find("[role='main'], [role='article']").First(); role.Length() > 0 {
		mainContent = role
	} else if content := doc.Find("#content, #main, .content, .main, .article").First(); content.Length() > 0 {
		mainContent = content
	} else {
		// Fallback to body
		mainContent = doc.Find("body")
	}

	text := strings.TrimSpace(mainContent.Text())
	text = NormalizeWhitespace(text)

	htmlContent, _ := mainContent.Html()

	return Success(map[string]interface{}{
		"text":       text,
		"html":       htmlContent,
		"length":     len(text),
		"word_count": len(strings.Fields(text)),
	})
}

// CleanHTML sanitizes HTML content
func (e *ExtractOps) CleanHTML(ctx context.Context, params map[string]interface{}, appCtx *types.Context) (*types.Result, error) {
	html, ok := GetString(params, "html")
	if !ok || html == "" {
		return Failure("html parameter required")
	}

	if err := ValidateHTML(html); err != nil {
		return Failure(err.Error())
	}

	cleaned := e.SanitizeHTML(html)

	return Success(map[string]interface{}{
		"html":          cleaned,
		"original_size": len(html),
		"cleaned_size":  len(cleaned),
		"reduction_pct": float64(len(html)-len(cleaned)) / float64(len(html)) * 100,
	})
}

// ExtractSummary extracts page summary
func (e *ExtractOps) ExtractSummary(ctx context.Context, params map[string]interface{}, appCtx *types.Context) (*types.Result, error) {
	html, ok := GetString(params, "html")
	if !ok || html == "" {
		return Failure("html parameter required")
	}

	maxLength, hasMax := GetInt(params, "max_length")
	if !hasMax {
		maxLength = 300
	}

	doc, err := LoadHTML(html)
	if err != nil {
		return Failure(fmt.Sprintf("parse failed: %v", err))
	}

	var summary string
	var source string

	// Try meta description first
	if desc := doc.Find("meta[name='description'], meta[property='og:description']").First(); desc.Length() > 0 {
		summary = desc.AttrOr("content", "")
		source = "meta"
	}

	// Fallback to first paragraph
	if summary == "" {
		if p := doc.Find("p").First(); p.Length() > 0 {
			summary = strings.TrimSpace(p.Text())
			source = "paragraph"
		}
	}

	// Normalize and truncate
	summary = NormalizeWhitespace(summary)
	if len(summary) > maxLength {
		summary = TruncateText(summary, maxLength)
	}

	return Success(map[string]interface{}{
		"summary": summary,
		"source":  source,
		"length":  len(summary),
	})
}
