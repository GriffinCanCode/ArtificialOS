package providers

import (
	"context"
	"fmt"

	scraperProvider "github.com/GriffinCanCode/AgentOS/backend/internal/providers/scraper"
	"github.com/GriffinCanCode/AgentOS/backend/internal/types"
)

// Scraper provides comprehensive web scraping operations
type Scraper struct {
	// Module instances
	content    *scraperProvider.ContentOps
	xpath      *scraperProvider.XPathOps
	extract    *scraperProvider.ExtractOps
	forms      *scraperProvider.FormsOps
	metadata   *scraperProvider.MetadataOps
	patterns   *scraperProvider.PatternsOps
	structured *scraperProvider.StructuredOps
}

// NewScraper creates a modular scraper provider with specialized libraries
func NewScraper() *Scraper {
	ops := scraperProvider.NewScraperOps()

	return &Scraper{
		content:    &scraperProvider.ContentOps{ScraperOps: ops},
		xpath:      &scraperProvider.XPathOps{ScraperOps: ops},
		extract:    &scraperProvider.ExtractOps{ScraperOps: ops},
		forms:      &scraperProvider.FormsOps{ScraperOps: ops},
		metadata:   &scraperProvider.MetadataOps{ScraperOps: ops},
		patterns:   &scraperProvider.PatternsOps{ScraperOps: ops},
		structured: &scraperProvider.StructuredOps{ScraperOps: ops},
	}
}

// Definition returns service metadata with all module tools
func (s *Scraper) Definition() types.Service {
	// Collect tools from all modules
	tools := []types.Tool{}
	tools = append(tools, s.content.GetTools()...)
	tools = append(tools, s.xpath.GetTools()...)
	tools = append(tools, s.extract.GetTools()...)
	tools = append(tools, s.forms.GetTools()...)
	tools = append(tools, s.metadata.GetTools()...)
	tools = append(tools, s.patterns.GetTools()...)
	tools = append(tools, s.structured.GetTools()...)

	return types.Service{
		ID:          "scraper",
		Name:        "Web Scraper Service",
		Description: "High-performance HTML parsing with XPath, content extraction, and structured data support",
		Category:    types.CategoryScraper,
		Capabilities: []string{
			"content_extraction",
			"xpath_queries",
			"css_selectors",
			"smart_extraction",
			"form_parsing",
			"metadata_extraction",
			"microdata",
			"json_ld",
			"open_graph",
			"pattern_matching",
			"structured_data",
			"charset_detection",
			"html_sanitization",
		},
		Tools: tools,
	}
}

// Execute routes to appropriate module
func (s *Scraper) Execute(ctx context.Context, toolID string, params map[string]interface{}, appCtx *types.Context) (*types.Result, error) {
	switch toolID {
	// Content operations
	case "scraper.text":
		return s.content.ExtractText(ctx, params, appCtx)
	case "scraper.title":
		return s.content.ExtractTitle(ctx, params, appCtx)
	case "scraper.links":
		return s.content.ExtractLinks(ctx, params, appCtx)
	case "scraper.images":
		return s.content.ExtractImages(ctx, params, appCtx)
	case "scraper.select":
		return s.content.Select(ctx, params, appCtx)
	case "scraper.attribute":
		return s.content.ExtractAttributes(ctx, params, appCtx)

	// XPath operations
	case "scraper.xpath":
		return s.xpath.Query(ctx, params, appCtx)
	case "scraper.xpath_text":
		return s.xpath.QueryText(ctx, params, appCtx)
	case "scraper.xpath_attr":
		return s.xpath.QueryAttribute(ctx, params, appCtx)

	// Smart extraction operations
	case "scraper.article":
		return s.extract.ExtractArticle(ctx, params, appCtx)
	case "scraper.clean":
		return s.extract.CleanHTML(ctx, params, appCtx)
	case "scraper.summary":
		return s.extract.ExtractSummary(ctx, params, appCtx)

	// Form operations
	case "scraper.form":
		return s.forms.FindForm(ctx, params, appCtx)
	case "scraper.form_fields":
		return s.forms.GetFormFields(ctx, params, appCtx)
	case "scraper.forms_all":
		return s.forms.FindAllForms(ctx, params, appCtx)

	// Metadata operations
	case "scraper.metadata":
		return s.metadata.ExtractMetadata(ctx, params, appCtx)
	case "scraper.headings":
		return s.metadata.ExtractHeadings(ctx, params, appCtx)
	case "scraper.jsonld":
		return s.metadata.ExtractJSONLD(ctx, params, appCtx)
	case "scraper.microdata":
		return s.metadata.ExtractMicrodata(ctx, params, appCtx)
	case "scraper.og":
		return s.metadata.ExtractOpenGraph(ctx, params, appCtx)

	// Pattern operations
	case "scraper.phones":
		return s.patterns.ExtractPhones(ctx, params, appCtx)
	case "scraper.pattern":
		return s.patterns.MatchPattern(ctx, params, appCtx)
	case "scraper.between":
		return s.patterns.ExtractBetween(ctx, params, appCtx)
	case "scraper.ips":
		return s.patterns.ExtractIPs(ctx, params, appCtx)

	// Structured data operations
	case "scraper.table":
		return s.structured.ExtractTable(ctx, params, appCtx)
	case "scraper.emails":
		return s.structured.ExtractEmails(ctx, params, appCtx)
	case "scraper.urls":
		return s.structured.ExtractURLs(ctx, params, appCtx)
	case "scraper.lists":
		return s.structured.ExtractLists(ctx, params, appCtx)

	default:
		return failure(fmt.Sprintf("unknown tool: %s", toolID))
	}
}
