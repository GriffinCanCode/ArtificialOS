package scraper

import (
	"context"
	"fmt"
	"strings"

	"github.com/GriffinCanCode/AgentOS/backend/internal/shared/types"
	"github.com/PuerkitoBio/goquery"
)

// StructuredOps handles structured data extraction with enhanced parsing
type StructuredOps struct {
	*ScraperOps
}

// GetTools returns structured extraction tool definitions
func (s *StructuredOps) GetTools() []types.Tool {
	return []types.Tool{
		{
			ID:          "scraper.table",
			Name:        "Extract Table",
			Description: "Parse HTML table into structured data with headers",
			Parameters: []types.Parameter{
				{Name: "html", Type: "string", Description: "HTML content", Required: true},
				{Name: "selector", Type: "string", Description: "CSS selector for table", Required: false},
				{Name: "headers", Type: "boolean", Description: "First row as headers (default: auto-detect)", Required: false},
			},
			Returns: "object",
		},
		{
			ID:          "scraper.emails",
			Name:        "Extract Emails",
			Description: "Find all email addresses with deduplication",
			Parameters: []types.Parameter{
				{Name: "html", Type: "string", Description: "HTML content", Required: true},
			},
			Returns: "object",
		},
		{
			ID:          "scraper.urls",
			Name:        "Extract URLs",
			Description: "Find all URLs in text content",
			Parameters: []types.Parameter{
				{Name: "html", Type: "string", Description: "HTML content", Required: true},
				{Name: "validate", Type: "boolean", Description: "Validate URL format (default: true)", Required: false},
			},
			Returns: "object",
		},
		{
			ID:          "scraper.lists",
			Name:        "Extract Lists",
			Description: "Get all list items (ul/ol) with hierarchy",
			Parameters: []types.Parameter{
				{Name: "html", Type: "string", Description: "HTML content", Required: true},
				{Name: "type", Type: "string", Description: "List type: 'ul', 'ol', or 'all' (default: 'all')", Required: false},
			},
			Returns: "object",
		},
	}
}

// ExtractTable parses HTML table with enhanced features
func (s *StructuredOps) ExtractTable(ctx context.Context, params map[string]interface{}, appCtx *types.Context) (*types.Result, error) {
	html, ok := GetString(params, "html")
	if !ok || html == "" {
		return Failure("html parameter required")
	}

	doc, err := LoadHTML(html)
	if err != nil {
		return Failure(fmt.Sprintf("parse failed: %v", err))
	}

	selector := "table"
	if sel, ok := GetString(params, "selector"); ok && sel != "" {
		selector = sel
	}

	table := doc.Find(selector).First()
	if table.Length() == 0 {
		return Success(map[string]interface{}{
			"rows":    []interface{}{},
			"headers": []string{},
		})
	}

	// Auto-detect or use explicit headers flag
	hasHeaders := false
	if h, ok := params["headers"].(bool); ok {
		hasHeaders = h
	} else {
		// Auto-detect: check if first row has <th> elements
		firstRow := table.Find("tr").First()
		hasHeaders = firstRow.Find("th").Length() > 0
	}

	var headers []string
	var rows [][]string

	table.Find("tr").Each(func(i int, row *goquery.Selection) {
		var cells []string

		if i == 0 && hasHeaders {
			// Extract headers
			row.Find("th, td").Each(func(j int, cell *goquery.Selection) {
				headers = append(headers, strings.TrimSpace(cell.Text()))
			})
		} else {
			// Extract data cells
			row.Find("td, th").Each(func(j int, cell *goquery.Selection) {
				cells = append(cells, strings.TrimSpace(cell.Text()))
			})
			if len(cells) > 0 {
				rows = append(rows, cells)
			}
		}
	})

	result := map[string]interface{}{
		"rows":        rows,
		"row_count":   len(rows),
		"has_headers": hasHeaders,
	}

	if hasHeaders && len(headers) > 0 {
		result["headers"] = headers
		result["column_count"] = len(headers)
	} else if len(rows) > 0 {
		result["column_count"] = len(rows[0])
	}

	return Success(result)
}

// ExtractEmails finds email addresses using cached regex
func (s *StructuredOps) ExtractEmails(ctx context.Context, params map[string]interface{}, appCtx *types.Context) (*types.Result, error) {
	html, ok := GetString(params, "html")
	if !ok || html == "" {
		return Failure("html parameter required")
	}

	doc, err := LoadHTML(html)
	if err != nil {
		return Failure(fmt.Sprintf("parse failed: %v", err))
	}

	text := doc.Text()

	emailRe, err := s.GetCachedRegex(EmailPattern)
	if err != nil {
		return Failure(fmt.Sprintf("regex compilation failed: %v", err))
	}

	emails := emailRe.FindAllString(text, -1)
	emails = Deduplicate(emails)

	return Success(map[string]interface{}{
		"emails": emails,
		"count":  len(emails),
	})
}

// ExtractURLs finds URLs in text with validation
func (s *StructuredOps) ExtractURLs(ctx context.Context, params map[string]interface{}, appCtx *types.Context) (*types.Result, error) {
	html, ok := GetString(params, "html")
	if !ok || html == "" {
		return Failure("html parameter required")
	}

	validate := GetBool(params, "validate", true)

	doc, err := LoadHTML(html)
	if err != nil {
		return Failure(fmt.Sprintf("parse failed: %v", err))
	}

	text := doc.Text()

	urlRe, err := s.GetCachedRegex(URLPattern)
	if err != nil {
		return Failure(fmt.Sprintf("regex compilation failed: %v", err))
	}

	urls := urlRe.FindAllString(text, -1)

	if validate {
		// Filter out malformed URLs
		var validURLs []string
		for _, url := range urls {
			// Basic validation: must have protocol and domain
			if strings.Contains(url, "://") && strings.Contains(url, ".") {
				validURLs = append(validURLs, url)
			}
		}
		urls = validURLs
	}

	urls = Deduplicate(urls)

	return Success(map[string]interface{}{
		"urls":  urls,
		"count": len(urls),
	})
}

// ExtractLists extracts list items with hierarchy
func (s *StructuredOps) ExtractLists(ctx context.Context, params map[string]interface{}, appCtx *types.Context) (*types.Result, error) {
	html, ok := GetString(params, "html")
	if !ok || html == "" {
		return Failure("html parameter required")
	}

	listType := "all"
	if lt, ok := GetString(params, "type"); ok {
		listType = strings.ToLower(lt)
	}

	doc, err := LoadHTML(html)
	if err != nil {
		return Failure(fmt.Sprintf("parse failed: %v", err))
	}

	var selector string
	switch listType {
	case "ul":
		selector = "ul"
	case "ol":
		selector = "ol"
	default:
		selector = "ul, ol"
	}

	var lists []map[string]interface{}

	doc.Find(selector).Each(func(i int, list *goquery.Selection) {
		var items []string
		var listTypeName string

		if list.Is("ul") {
			listTypeName = "unordered"
		} else {
			listTypeName = "ordered"
		}

		list.Find("> li").Each(func(j int, item *goquery.Selection) {
			text := strings.TrimSpace(item.Clone().Children().Remove().End().Text())
			if text != "" {
				items = append(items, text)
			}
		})

		if len(items) > 0 {
			lists = append(lists, map[string]interface{}{
				"type":  listTypeName,
				"items": items,
				"count": len(items),
			})
		}
	})

	return Success(map[string]interface{}{
		"lists":      lists,
		"list_count": len(lists),
	})
}
