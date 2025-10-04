package scraper

import (
	"context"
	"fmt"
	"strings"

	"github.com/GriffinCanCode/AgentOS/backend/internal/types"
	"github.com/antchfx/htmlquery"
)

// XPathOps handles XPath queries for powerful element selection
type XPathOps struct {
	*ScraperOps
}

// GetTools returns XPath tool definitions
func (x *XPathOps) GetTools() []types.Tool {
	return []types.Tool{
		{
			ID:          "scraper.xpath",
			Name:        "XPath Query",
			Description: "Query HTML using XPath expressions",
			Parameters: []types.Parameter{
				{Name: "html", Type: "string", Description: "HTML content", Required: true},
				{Name: "xpath", Type: "string", Description: "XPath expression", Required: true},
				{Name: "all", Type: "boolean", Description: "Get all matches (default: false)", Required: false},
			},
			Returns: "object",
		},
		{
			ID:          "scraper.xpath_text",
			Name:        "XPath Text",
			Description: "Extract text content using XPath",
			Parameters: []types.Parameter{
				{Name: "html", Type: "string", Description: "HTML content", Required: true},
				{Name: "xpath", Type: "string", Description: "XPath expression", Required: true},
			},
			Returns: "object",
		},
		{
			ID:          "scraper.xpath_attr",
			Name:        "XPath Attribute",
			Description: "Extract attribute value using XPath",
			Parameters: []types.Parameter{
				{Name: "html", Type: "string", Description: "HTML content", Required: true},
				{Name: "xpath", Type: "string", Description: "XPath expression", Required: true},
				{Name: "attribute", Type: "string", Description: "Attribute name", Required: true},
			},
			Returns: "object",
		},
	}
}

// Query executes XPath query
func (x *XPathOps) Query(ctx context.Context, params map[string]interface{}, appCtx *types.Context) (*types.Result, error) {
	html, ok := GetString(params, "html")
	if !ok || html == "" {
		return Failure("html parameter required")
	}

	xpath, ok := GetString(params, "xpath")
	if !ok || xpath == "" {
		return Failure("xpath parameter required")
	}

	doc, err := LoadHTMLNode(html)
	if err != nil {
		return Failure(fmt.Sprintf("parse failed: %v", err))
	}

	all := GetBool(params, "all", false)

	if all {
		nodes, err := htmlquery.QueryAll(doc, xpath)
		if err != nil {
			return Failure(fmt.Sprintf("xpath query failed: %v", err))
		}

		results := make([]map[string]interface{}, 0, len(nodes))
		for _, node := range nodes {
			text := ExtractText(node)
			html := htmlquery.OutputHTML(node, true)
			results = append(results, map[string]interface{}{
				"text": text,
				"html": html,
			})
		}

		return Success(map[string]interface{}{
			"elements": results,
			"count":    len(results),
		})
	}

	// Single element
	node, err := htmlquery.Query(doc, xpath)
	if err != nil {
		return Failure(fmt.Sprintf("xpath query failed: %v", err))
	}

	if node == nil {
		return Success(map[string]interface{}{
			"found": false,
			"text":  "",
			"html":  "",
		})
	}

	text := ExtractText(node)
	htmlContent := htmlquery.OutputHTML(node, true)

	return Success(map[string]interface{}{
		"found": true,
		"text":  text,
		"html":  htmlContent,
	})
}

// QueryText extracts text using XPath
func (x *XPathOps) QueryText(ctx context.Context, params map[string]interface{}, appCtx *types.Context) (*types.Result, error) {
	html, ok := GetString(params, "html")
	if !ok || html == "" {
		return Failure("html parameter required")
	}

	xpath, ok := GetString(params, "xpath")
	if !ok || xpath == "" {
		return Failure("xpath parameter required")
	}

	doc, err := LoadHTMLNode(html)
	if err != nil {
		return Failure(fmt.Sprintf("parse failed: %v", err))
	}

	nodes, err := htmlquery.QueryAll(doc, xpath)
	if err != nil {
		return Failure(fmt.Sprintf("xpath query failed: %v", err))
	}

	texts := make([]string, 0, len(nodes))
	for _, node := range nodes {
		text := strings.TrimSpace(ExtractText(node))
		if text != "" {
			texts = append(texts, text)
		}
	}

	return Success(map[string]interface{}{
		"texts": texts,
		"count": len(texts),
	})
}

// QueryAttribute extracts attribute using XPath
func (x *XPathOps) QueryAttribute(ctx context.Context, params map[string]interface{}, appCtx *types.Context) (*types.Result, error) {
	html, ok := GetString(params, "html")
	if !ok || html == "" {
		return Failure("html parameter required")
	}

	xpath, ok := GetString(params, "xpath")
	if !ok || xpath == "" {
		return Failure("xpath parameter required")
	}

	attribute, ok := GetString(params, "attribute")
	if !ok || attribute == "" {
		return Failure("attribute parameter required")
	}

	doc, err := LoadHTMLNode(html)
	if err != nil {
		return Failure(fmt.Sprintf("parse failed: %v", err))
	}

	nodes, err := htmlquery.QueryAll(doc, xpath)
	if err != nil {
		return Failure(fmt.Sprintf("xpath query failed: %v", err))
	}

	values := make([]string, 0, len(nodes))
	for _, node := range nodes {
		for _, attr := range node.Attr {
			if attr.Key == attribute && attr.Val != "" {
				values = append(values, attr.Val)
			}
		}
	}

	return Success(map[string]interface{}{
		"values": values,
		"count":  len(values),
	})
}
