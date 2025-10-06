package scraper

import (
	"context"
	"fmt"
	"strings"

	"github.com/GriffinCanCode/AgentOS/backend/internal/shared/types"
)

// PatternsOps handles pattern-based extraction with cached regex
type PatternsOps struct {
	*ScraperOps
}

// Common regex patterns
const (
	EmailPattern     = `[a-zA-Z0-9._%+\-]+@[a-zA-Z0-9.\-]+\.[a-zA-Z]{2,}`
	PhoneUSPattern   = `\+?1?\s*\(?([0-9]{3})\)?[\s.-]?([0-9]{3})[\s.-]?([0-9]{4})`
	PhoneIntlPattern = `\+?([0-9]{1,3})?[\s.-]?\(?([0-9]{2,4})\)?[\s.-]?([0-9]{3,4})[\s.-]?([0-9]{4})`
	URLPattern       = `https?://[^\s<>"{}|\\^` + "`" + `\[\]]+`
	IPv4Pattern      = `\b(?:(?:25[0-5]|2[0-4][0-9]|[01]?[0-9][0-9]?)\.){3}(?:25[0-5]|2[0-4][0-9]|[01]?[0-9][0-9]?)\b`
	SSNPattern       = `\b\d{3}-\d{2}-\d{4}\b`
	ZipCodePattern   = `\b\d{5}(?:-\d{4})?\b`
)

// GetTools returns pattern tool definitions
func (p *PatternsOps) GetTools() []types.Tool {
	return []types.Tool{
		{
			ID:          "scraper.phones",
			Name:        "Extract Phone Numbers",
			Description: "Find phone numbers (US and international formats)",
			Parameters: []types.Parameter{
				{Name: "html", Type: "string", Description: "HTML content", Required: true},
			},
			Returns: "object",
		},
		{
			ID:          "scraper.pattern",
			Name:        "Match Pattern",
			Description: "Extract text matching regex pattern (cached)",
			Parameters: []types.Parameter{
				{Name: "html", Type: "string", Description: "HTML content", Required: true},
				{Name: "pattern", Type: "string", Description: "Regex pattern", Required: true},
				{Name: "unique", Type: "boolean", Description: "Return unique matches (default: true)", Required: false},
			},
			Returns: "object",
		},
		{
			ID:          "scraper.between",
			Name:        "Extract Between",
			Description: "Get text between two delimiter strings",
			Parameters: []types.Parameter{
				{Name: "html", Type: "string", Description: "HTML content", Required: true},
				{Name: "start", Type: "string", Description: "Start delimiter", Required: true},
				{Name: "end", Type: "string", Description: "End delimiter", Required: true},
				{Name: "limit", Type: "number", Description: "Max results (default: 100)", Required: false},
			},
			Returns: "object",
		},
		{
			ID:          "scraper.ips",
			Name:        "Extract IP Addresses",
			Description: "Find IPv4 addresses in content",
			Parameters: []types.Parameter{
				{Name: "html", Type: "string", Description: "HTML content", Required: true},
			},
			Returns: "object",
		},
	}
}

// ExtractPhones finds phone numbers using cached regex
func (p *PatternsOps) ExtractPhones(ctx context.Context, params map[string]interface{}, appCtx *types.Context) (*types.Result, error) {
	html, ok := GetString(params, "html")
	if !ok || html == "" {
		return Failure("html parameter required")
	}

	doc, err := LoadHTML(html)
	if err != nil {
		return Failure(fmt.Sprintf("parse failed: %v", err))
	}

	text := doc.Text()

	patterns := []string{PhoneUSPattern, PhoneIntlPattern}
	phoneMap := make(map[string]bool)
	var phones []string

	for _, pattern := range patterns {
		re, err := p.GetCachedRegex(pattern)
		if err != nil {
			continue
		}

		matches := re.FindAllString(text, -1)
		for _, match := range matches {
			normalized := strings.TrimSpace(match)
			if !phoneMap[normalized] {
				phoneMap[normalized] = true
				phones = append(phones, normalized)
			}
		}
	}

	return Success(map[string]interface{}{
		"phones": phones,
		"count":  len(phones),
	})
}

// MatchPattern extracts text matching custom regex
func (p *PatternsOps) MatchPattern(ctx context.Context, params map[string]interface{}, appCtx *types.Context) (*types.Result, error) {
	html, ok := GetString(params, "html")
	if !ok || html == "" {
		return Failure("html parameter required")
	}

	pattern, ok := GetString(params, "pattern")
	if !ok || pattern == "" {
		return Failure("pattern parameter required")
	}

	unique := GetBool(params, "unique", true)

	doc, err := LoadHTML(html)
	if err != nil {
		return Failure(fmt.Sprintf("parse failed: %v", err))
	}

	text := doc.Text()

	re, err := p.GetCachedRegex(pattern)
	if err != nil {
		return Failure(fmt.Sprintf("invalid regex: %v", err))
	}

	matches := re.FindAllString(text, -1)

	if unique {
		matches = Deduplicate(matches)
	}

	return Success(map[string]interface{}{
		"matches": matches,
		"count":   len(matches),
		"unique":  unique,
	})
}

// ExtractBetween extracts text between delimiters
func (p *PatternsOps) ExtractBetween(ctx context.Context, params map[string]interface{}, appCtx *types.Context) (*types.Result, error) {
	html, ok := GetString(params, "html")
	if !ok || html == "" {
		return Failure("html parameter required")
	}

	start, ok := GetString(params, "start")
	if !ok || start == "" {
		return Failure("start parameter required")
	}

	end, ok := GetString(params, "end")
	if !ok || end == "" {
		return Failure("end parameter required")
	}

	limit, hasLimit := GetInt(params, "limit")
	if !hasLimit {
		limit = 100
	}

	doc, err := LoadHTML(html)
	if err != nil {
		return Failure(fmt.Sprintf("parse failed: %v", err))
	}

	text := doc.Text()

	var results []string
	currentPos := 0

	for len(results) < limit {
		startIdx := strings.Index(text[currentPos:], start)
		if startIdx == -1 {
			break
		}
		startIdx += currentPos + len(start)

		endIdx := strings.Index(text[startIdx:], end)
		if endIdx == -1 {
			break
		}
		endIdx += startIdx

		extracted := strings.TrimSpace(text[startIdx:endIdx])
		if extracted != "" {
			results = append(results, extracted)
		}

		currentPos = endIdx + len(end)
	}

	return Success(map[string]interface{}{
		"results": results,
		"count":   len(results),
	})
}

// ExtractIPs finds IPv4 addresses
func (p *PatternsOps) ExtractIPs(ctx context.Context, params map[string]interface{}, appCtx *types.Context) (*types.Result, error) {
	html, ok := GetString(params, "html")
	if !ok || html == "" {
		return Failure("html parameter required")
	}

	doc, err := LoadHTML(html)
	if err != nil {
		return Failure(fmt.Sprintf("parse failed: %v", err))
	}

	text := doc.Text()

	re, err := p.GetCachedRegex(IPv4Pattern)
	if err != nil {
		return Failure(fmt.Sprintf("regex compilation failed: %v", err))
	}

	matches := re.FindAllString(text, -1)
	ips := Deduplicate(matches)

	return Success(map[string]interface{}{
		"ips":   ips,
		"count": len(ips),
	})
}
