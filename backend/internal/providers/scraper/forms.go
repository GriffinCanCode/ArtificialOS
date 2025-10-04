package scraper

import (
	"context"
	"fmt"
	"strings"

	"github.com/GriffinCanCode/AgentOS/backend/internal/types"
	"github.com/PuerkitoBio/goquery"
)

// FormsOps handles form extraction and analysis
type FormsOps struct {
	*ScraperOps
}

// GetTools returns form tool definitions
func (f *FormsOps) GetTools() []types.Tool {
	return []types.Tool{
		{
			ID:          "scraper.form",
			Name:        "Find Form",
			Description: "Find form by selector or action attribute",
			Parameters: []types.Parameter{
				{Name: "html", Type: "string", Description: "HTML content", Required: true},
				{Name: "selector", Type: "string", Description: "CSS selector", Required: false},
				{Name: "action", Type: "string", Description: "Form action URL", Required: false},
			},
			Returns: "object",
		},
		{
			ID:          "scraper.form_fields",
			Name:        "Get Form Fields",
			Description: "Extract all input fields from form with metadata",
			Parameters: []types.Parameter{
				{Name: "html", Type: "string", Description: "HTML content", Required: true},
				{Name: "selector", Type: "string", Description: "Form CSS selector", Required: false},
			},
			Returns: "object",
		},
		{
			ID:          "scraper.forms_all",
			Name:        "Find All Forms",
			Description: "Extract all forms from page with their fields",
			Parameters: []types.Parameter{
				{Name: "html", Type: "string", Description: "HTML content", Required: true},
			},
			Returns: "object",
		},
	}
}

// FindForm finds form element with enhanced detection
func (f *FormsOps) FindForm(ctx context.Context, params map[string]interface{}, appCtx *types.Context) (*types.Result, error) {
	html, ok := GetString(params, "html")
	if !ok || html == "" {
		return Failure("html parameter required")
	}

	doc, err := LoadHTML(html)
	if err != nil {
		return Failure(fmt.Sprintf("parse failed: %v", err))
	}

	var form *goquery.Selection

	// Try selector first
	if selector, ok := GetString(params, "selector"); ok && selector != "" {
		form = doc.Find(selector).First()
	} else if action, ok := GetString(params, "action"); ok && action != "" {
		// Find form by action
		doc.Find("form").EachWithBreak(func(i int, s *goquery.Selection) bool {
			if attr, exists := s.Attr("action"); exists && strings.Contains(attr, action) {
				form = s
				return false
			}
			return true
		})
	} else {
		// Default to first form
		form = doc.Find("form").First()
	}

	if form == nil || form.Length() == 0 {
		return Failure("form not found")
	}

	action := form.AttrOr("action", "")
	method := strings.ToUpper(form.AttrOr("method", "GET"))
	name := form.AttrOr("name", "")
	id := form.AttrOr("id", "")
	enctype := form.AttrOr("enctype", "application/x-www-form-urlencoded")

	return Success(map[string]interface{}{
		"found":   true,
		"action":  action,
		"method":  method,
		"name":    name,
		"id":      id,
		"enctype": enctype,
	})
}

// GetFormFields extracts form fields with full metadata
func (f *FormsOps) GetFormFields(ctx context.Context, params map[string]interface{}, appCtx *types.Context) (*types.Result, error) {
	html, ok := GetString(params, "html")
	if !ok || html == "" {
		return Failure("html parameter required")
	}

	doc, err := LoadHTML(html)
	if err != nil {
		return Failure(fmt.Sprintf("parse failed: %v", err))
	}

	selector := "form"
	if sel, ok := GetString(params, "selector"); ok && sel != "" {
		selector = sel
	}

	form := doc.Find(selector).First()
	if form.Length() == 0 {
		return Failure("form not found")
	}

	var fields []map[string]interface{}

	form.Find("input, textarea, select, button").Each(func(i int, s *goquery.Selection) {
		name := s.AttrOr("name", "")
		id := s.AttrOr("id", "")

		// Skip if no name or id
		if name == "" && id == "" {
			return
		}

		fieldType := s.AttrOr("type", "text")
		value := s.AttrOr("value", "")
		placeholder := s.AttrOr("placeholder", "")
		required := s.AttrOr("required", "") != ""
		disabled := s.AttrOr("disabled", "") != ""
		readonly := s.AttrOr("readonly", "") != ""

		field := map[string]interface{}{
			"name":        name,
			"id":          id,
			"type":        fieldType,
			"value":       value,
			"placeholder": placeholder,
			"required":    required,
			"disabled":    disabled,
			"readonly":    readonly,
		}

		// Handle select options
		if s.Is("select") {
			var options []map[string]string
			s.Find("option").Each(func(j int, opt *goquery.Selection) {
				options = append(options, map[string]string{
					"value": opt.AttrOr("value", ""),
					"text":  strings.TrimSpace(opt.Text()),
				})
			})
			field["options"] = options
		}

		fields = append(fields, field)
	})

	return Success(map[string]interface{}{
		"fields": fields,
		"count":  len(fields),
	})
}

// FindAllForms extracts all forms from page
func (f *FormsOps) FindAllForms(ctx context.Context, params map[string]interface{}, appCtx *types.Context) (*types.Result, error) {
	html, ok := GetString(params, "html")
	if !ok || html == "" {
		return Failure("html parameter required")
	}

	doc, err := LoadHTML(html)
	if err != nil {
		return Failure(fmt.Sprintf("parse failed: %v", err))
	}

	var forms []map[string]interface{}

	doc.Find("form").Each(func(i int, form *goquery.Selection) {
		action := form.AttrOr("action", "")
		method := strings.ToUpper(form.AttrOr("method", "GET"))
		name := form.AttrOr("name", "")
		id := form.AttrOr("id", "")

		// Count fields
		fieldCount := form.Find("input, textarea, select").Length()

		forms = append(forms, map[string]interface{}{
			"action":      action,
			"method":      method,
			"name":        name,
			"id":          id,
			"field_count": fieldCount,
		})
	})

	return Success(map[string]interface{}{
		"forms": forms,
		"count": len(forms),
	})
}
