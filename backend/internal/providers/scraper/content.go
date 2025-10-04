package scraper

import (
	"context"
	"fmt"
	"strings"

	"github.com/GriffinCanCode/AgentOS/backend/internal/types"
	"github.com/PuerkitoBio/goquery"
)

// ContentOps handles content extraction with enhanced capabilities
type ContentOps struct {
	*ScraperOps
}

// GetTools returns content extraction tool definitions
func (c *ContentOps) GetTools() []types.Tool {
	return []types.Tool{
		{
			ID:          "scraper.text",
			Name:        "Extract Text",
			Description: "Extract all visible text from HTML with charset detection",
			Parameters: []types.Parameter{
				{Name: "html", Type: "string", Description: "HTML content", Required: true},
				{Name: "separator", Type: "string", Description: "Text separator (default: space)", Required: false},
				{Name: "normalize", Type: "boolean", Description: "Normalize whitespace (default: true)", Required: false},
			},
			Returns: "object",
		},
		{
			ID:          "scraper.title",
			Name:        "Extract Title",
			Description: "Get page title from HTML",
			Parameters: []types.Parameter{
				{Name: "html", Type: "string", Description: "HTML content", Required: true},
			},
			Returns: "object",
		},
		{
			ID:          "scraper.links",
			Name:        "Extract Links",
			Description: "Get all links from HTML with deduplication",
			Parameters: []types.Parameter{
				{Name: "html", Type: "string", Description: "HTML content", Required: true},
				{Name: "absolute_only", Type: "boolean", Description: "Only absolute URLs (default: false)", Required: false},
			},
			Returns: "object",
		},
		{
			ID:          "scraper.images",
			Name:        "Extract Images",
			Description: "Get all image URLs with metadata",
			Parameters: []types.Parameter{
				{Name: "html", Type: "string", Description: "HTML content", Required: true},
			},
			Returns: "object",
		},
		{
			ID:          "scraper.select",
			Name:        "CSS Select",
			Description: "Find elements by CSS selector",
			Parameters: []types.Parameter{
				{Name: "html", Type: "string", Description: "HTML content", Required: true},
				{Name: "selector", Type: "string", Description: "CSS selector", Required: true},
				{Name: "all", Type: "boolean", Description: "Get all matches (default: false)", Required: false},
				{Name: "limit", Type: "number", Description: "Max results (default: 100)", Required: false},
			},
			Returns: "object",
		},
		{
			ID:          "scraper.attribute",
			Name:        "Extract Attributes",
			Description: "Extract specific attribute from elements",
			Parameters: []types.Parameter{
				{Name: "html", Type: "string", Description: "HTML content", Required: true},
				{Name: "selector", Type: "string", Description: "CSS selector", Required: true},
				{Name: "attribute", Type: "string", Description: "Attribute name", Required: true},
			},
			Returns: "object",
		},
	}
}

// ExtractText extracts visible text with normalization
func (c *ContentOps) ExtractText(ctx context.Context, params map[string]interface{}, appCtx *types.Context) (*types.Result, error) {
	html, ok := GetString(params, "html")
	if !ok || html == "" {
		return Failure("html parameter required")
	}

	doc, err := LoadHTML(html)
	if err != nil {
		return Failure(fmt.Sprintf("parse failed: %v", err))
	}

	separator := " "
	if sep, ok := GetString(params, "separator"); ok {
		separator = sep
	}

	normalize := GetBool(params, "normalize", true)

	text := doc.Find("body").Text()
	if normalize {
		text = NormalizeWhitespace(text)
	} else {
		text = strings.TrimSpace(text)
		text = strings.Join(strings.Fields(text), separator)
	}

	return Success(map[string]interface{}{
		"text":       text,
		"length":     len(text),
		"word_count": len(strings.Fields(text)),
	})
}

// ExtractTitle extracts page title
func (c *ContentOps) ExtractTitle(ctx context.Context, params map[string]interface{}, appCtx *types.Context) (*types.Result, error) {
	html, ok := GetString(params, "html")
	if !ok || html == "" {
		return Failure("html parameter required")
	}

	doc, err := LoadHTML(html)
	if err != nil {
		return Failure(fmt.Sprintf("parse failed: %v", err))
	}

	title := strings.TrimSpace(doc.Find("title").Text())

	// Fallback to og:title if no title
	if title == "" {
		title = doc.Find("meta[property='og:title']").AttrOr("content", "")
	}

	return Success(map[string]interface{}{
		"title": title,
		"empty": title == "",
	})
}

// ExtractLinks extracts all links with deduplication
func (c *ContentOps) ExtractLinks(ctx context.Context, params map[string]interface{}, appCtx *types.Context) (*types.Result, error) {
	html, ok := GetString(params, "html")
	if !ok || html == "" {
		return Failure("html parameter required")
	}

	doc, err := LoadHTML(html)
	if err != nil {
		return Failure(fmt.Sprintf("parse failed: %v", err))
	}

	absoluteOnly := GetBool(params, "absolute_only", false)

	var links []string
	doc.Find("a[href]").Each(func(i int, s *goquery.Selection) {
		if href, exists := s.Attr("href"); exists {
			href = strings.TrimSpace(href)
			if href != "" && href != "#" {
				if absoluteOnly {
					if strings.HasPrefix(href, "http://") || strings.HasPrefix(href, "https://") {
						links = append(links, href)
					}
				} else {
					links = append(links, href)
				}
			}
		}
	})

	// Deduplicate
	links = Deduplicate(links)

	return Success(map[string]interface{}{
		"links": links,
		"count": len(links),
	})
}

// ExtractImages extracts all image URLs with metadata
func (c *ContentOps) ExtractImages(ctx context.Context, params map[string]interface{}, appCtx *types.Context) (*types.Result, error) {
	html, ok := GetString(params, "html")
	if !ok || html == "" {
		return Failure("html parameter required")
	}

	doc, err := LoadHTML(html)
	if err != nil {
		return Failure(fmt.Sprintf("parse failed: %v", err))
	}

	type ImageInfo struct {
		Src    string `json:"src"`
		Alt    string `json:"alt"`
		Title  string `json:"title,omitempty"`
		Width  string `json:"width,omitempty"`
		Height string `json:"height,omitempty"`
	}

	var images []ImageInfo
	doc.Find("img[src]").Each(func(i int, s *goquery.Selection) {
		if src, exists := s.Attr("src"); exists && src != "" {
			img := ImageInfo{
				Src:    src,
				Alt:    s.AttrOr("alt", ""),
				Title:  s.AttrOr("title", ""),
				Width:  s.AttrOr("width", ""),
				Height: s.AttrOr("height", ""),
			}
			images = append(images, img)
		}
	})

	return Success(map[string]interface{}{
		"images": images,
		"count":  len(images),
	})
}

// Select finds elements by CSS selector
func (c *ContentOps) Select(ctx context.Context, params map[string]interface{}, appCtx *types.Context) (*types.Result, error) {
	html, ok := GetString(params, "html")
	if !ok || html == "" {
		return Failure("html parameter required")
	}

	selector, ok := GetString(params, "selector")
	if !ok || selector == "" {
		return Failure("selector parameter required")
	}

	doc, err := LoadHTML(html)
	if err != nil {
		return Failure(fmt.Sprintf("parse failed: %v", err))
	}

	all := GetBool(params, "all", false)
	limit, hasLimit := GetInt(params, "limit")
	if !hasLimit {
		limit = 100
	}

	if all {
		var elements []map[string]interface{}
		doc.Find(selector).EachWithBreak(func(i int, s *goquery.Selection) bool {
			if len(elements) >= limit {
				return false
			}
			htmlContent, _ := s.Html()
			elements = append(elements, map[string]interface{}{
				"text": strings.TrimSpace(s.Text()),
				"html": htmlContent,
			})
			return true
		})
		return Success(map[string]interface{}{
			"elements": elements,
			"count":    len(elements),
		})
	}

	// Single element
	s := doc.Find(selector).First()
	if s.Length() == 0 {
		return Success(map[string]interface{}{
			"found": false,
			"text":  "",
			"html":  "",
		})
	}

	htmlContent, _ := s.Html()
	return Success(map[string]interface{}{
		"found": true,
		"text":  strings.TrimSpace(s.Text()),
		"html":  htmlContent,
	})
}

// ExtractAttributes extracts specific attribute from elements
func (c *ContentOps) ExtractAttributes(ctx context.Context, params map[string]interface{}, appCtx *types.Context) (*types.Result, error) {
	html, ok := GetString(params, "html")
	if !ok || html == "" {
		return Failure("html parameter required")
	}

	selector, ok := GetString(params, "selector")
	if !ok || selector == "" {
		return Failure("selector parameter required")
	}

	attribute, ok := GetString(params, "attribute")
	if !ok || attribute == "" {
		return Failure("attribute parameter required")
	}

	doc, err := LoadHTML(html)
	if err != nil {
		return Failure(fmt.Sprintf("parse failed: %v", err))
	}

	var values []string
	doc.Find(selector).Each(func(i int, s *goquery.Selection) {
		if val, exists := s.Attr(attribute); exists && val != "" {
			values = append(values, val)
		}
	})

	return Success(map[string]interface{}{
		"values": values,
		"count":  len(values),
	})
}
