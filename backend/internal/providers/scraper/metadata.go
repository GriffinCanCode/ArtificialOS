package scraper

import (
	"context"
	"encoding/json"
	"fmt"
	"strings"

	"github.com/GriffinCanCode/AgentOS/backend/internal/types"
	"github.com/PuerkitoBio/goquery"
)

// MetadataOps handles metadata extraction with microdata support
type MetadataOps struct {
	*ScraperOps
}

// GetTools returns metadata tool definitions
func (m *MetadataOps) GetTools() []types.Tool {
	return []types.Tool{
		{
			ID:          "scraper.metadata",
			Name:        "Extract Metadata",
			Description: "Get meta tags (Open Graph, Twitter, etc.)",
			Parameters: []types.Parameter{
				{Name: "html", Type: "string", Description: "HTML content", Required: true},
			},
			Returns: "object",
		},
		{
			ID:          "scraper.headings",
			Name:        "Extract Headings",
			Description: "Get all heading elements (h1-h6) with hierarchy",
			Parameters: []types.Parameter{
				{Name: "html", Type: "string", Description: "HTML content", Required: true},
			},
			Returns: "object",
		},
		{
			ID:          "scraper.jsonld",
			Name:        "Extract JSON-LD",
			Description: "Parse JSON-LD structured data",
			Parameters: []types.Parameter{
				{Name: "html", Type: "string", Description: "HTML content", Required: true},
			},
			Returns: "object",
		},
		{
			ID:          "scraper.microdata",
			Name:        "Extract Microdata",
			Description: "Parse HTML5 microdata (itemscope/itemprop)",
			Parameters: []types.Parameter{
				{Name: "html", Type: "string", Description: "HTML content", Required: true},
			},
			Returns: "object",
		},
		{
			ID:          "scraper.og",
			Name:        "Extract Open Graph",
			Description: "Get Open Graph metadata for social sharing",
			Parameters: []types.Parameter{
				{Name: "html", Type: "string", Description: "HTML content", Required: true},
			},
			Returns: "object",
		},
	}
}

// ExtractMetadata extracts all meta tags organized by type
func (m *MetadataOps) ExtractMetadata(ctx context.Context, params map[string]interface{}, appCtx *types.Context) (*types.Result, error) {
	html, ok := GetString(params, "html")
	if !ok || html == "" {
		return Failure("html parameter required")
	}

	doc, err := LoadHTML(html)
	if err != nil {
		return Failure(fmt.Sprintf("parse failed: %v", err))
	}

	standard := make(map[string]string)
	openGraph := make(map[string]string)
	twitter := make(map[string]string)
	other := make(map[string]string)

	doc.Find("meta").Each(func(i int, s *goquery.Selection) {
		name := s.AttrOr("name", "")
		property := s.AttrOr("property", "")
		content := s.AttrOr("content", "")

		if content == "" {
			return
		}

		if property != "" {
			if strings.HasPrefix(property, "og:") {
				openGraph[property] = content
			} else {
				other[property] = content
			}
		} else if name != "" {
			if strings.HasPrefix(name, "twitter:") {
				twitter[name] = content
			} else {
				standard[name] = content
			}
		}
	})

	return Success(map[string]interface{}{
		"standard":    standard,
		"open_graph":  openGraph,
		"twitter":     twitter,
		"other":       other,
		"total_count": len(standard) + len(openGraph) + len(twitter) + len(other),
	})
}

// ExtractHeadings extracts heading elements with hierarchy
func (m *MetadataOps) ExtractHeadings(ctx context.Context, params map[string]interface{}, appCtx *types.Context) (*types.Result, error) {
	html, ok := GetString(params, "html")
	if !ok || html == "" {
		return Failure("html parameter required")
	}

	doc, err := LoadHTML(html)
	if err != nil {
		return Failure(fmt.Sprintf("parse failed: %v", err))
	}

	var headings []map[string]interface{}

	for level := 1; level <= 6; level++ {
		selector := fmt.Sprintf("h%d", level)
		doc.Find(selector).Each(func(i int, s *goquery.Selection) {
			text := strings.TrimSpace(s.Text())
			if text != "" {
				headings = append(headings, map[string]interface{}{
					"level": level,
					"text":  text,
					"id":    s.AttrOr("id", ""),
				})
			}
		})
	}

	return Success(map[string]interface{}{
		"headings": headings,
		"count":    len(headings),
	})
}

// ExtractJSONLD extracts JSON-LD structured data
func (m *MetadataOps) ExtractJSONLD(ctx context.Context, params map[string]interface{}, appCtx *types.Context) (*types.Result, error) {
	html, ok := GetString(params, "html")
	if !ok || html == "" {
		return Failure("html parameter required")
	}

	doc, err := LoadHTML(html)
	if err != nil {
		return Failure(fmt.Sprintf("parse failed: %v", err))
	}

	var jsonLDData []interface{}

	doc.Find("script[type='application/ld+json']").Each(func(i int, s *goquery.Selection) {
		content := strings.TrimSpace(s.Text())
		if content != "" {
			var data interface{}
			if err := json.Unmarshal([]byte(content), &data); err == nil {
				jsonLDData = append(jsonLDData, data)
			}
		}
	})

	return Success(map[string]interface{}{
		"data":  jsonLDData,
		"count": len(jsonLDData),
	})
}

// ExtractMicrodata extracts HTML5 microdata
func (m *MetadataOps) ExtractMicrodata(ctx context.Context, params map[string]interface{}, appCtx *types.Context) (*types.Result, error) {
	html, ok := GetString(params, "html")
	if !ok || html == "" {
		return Failure("html parameter required")
	}

	doc, err := LoadHTML(html)
	if err != nil {
		return Failure(fmt.Sprintf("parse failed: %v", err))
	}

	var items []map[string]interface{}

	doc.Find("[itemscope]").Each(func(i int, scope *goquery.Selection) {
		itemType := scope.AttrOr("itemtype", "")
		itemId := scope.AttrOr("itemid", "")

		properties := make(map[string]interface{})

		scope.Find("[itemprop]").Each(func(j int, prop *goquery.Selection) {
			propName := prop.AttrOr("itemprop", "")
			if propName == "" {
				return
			}

			var value interface{}

			// Get value based on element type
			if prop.Is("[itemscope]") {
				// Nested item
				value = map[string]interface{}{
					"type": prop.AttrOr("itemtype", ""),
				}
			} else if content := prop.AttrOr("content", ""); content != "" {
				value = content
			} else if href := prop.AttrOr("href", ""); href != "" {
				value = href
			} else if src := prop.AttrOr("src", ""); src != "" {
				value = src
			} else {
				value = strings.TrimSpace(prop.Text())
			}

			properties[propName] = value
		})

		if len(properties) > 0 {
			item := map[string]interface{}{
				"type":       itemType,
				"properties": properties,
			}
			if itemId != "" {
				item["id"] = itemId
			}
			items = append(items, item)
		}
	})

	return Success(map[string]interface{}{
		"items": items,
		"count": len(items),
	})
}

// ExtractOpenGraph extracts Open Graph metadata
func (m *MetadataOps) ExtractOpenGraph(ctx context.Context, params map[string]interface{}, appCtx *types.Context) (*types.Result, error) {
	html, ok := GetString(params, "html")
	if !ok || html == "" {
		return Failure("html parameter required")
	}

	doc, err := LoadHTML(html)
	if err != nil {
		return Failure(fmt.Sprintf("parse failed: %v", err))
	}

	og := make(map[string]interface{})

	doc.Find("meta[property^='og:']").Each(func(i int, s *goquery.Selection) {
		property := s.AttrOr("property", "")
		content := s.AttrOr("content", "")
		if property != "" && content != "" {
			// Remove og: prefix for cleaner keys
			key := strings.TrimPrefix(property, "og:")
			og[key] = content
		}
	})

	return Success(map[string]interface{}{
		"open_graph": og,
		"count":      len(og),
	})
}
