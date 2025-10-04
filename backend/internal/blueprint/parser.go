package blueprint

import (
	"fmt"
	"strings"
	"time"

	"github.com/GriffinCanCode/AgentOS/backend/internal/types"
	"gopkg.in/yaml.v3"
)

// Blueprint represents the root structure of a .bp file
type Blueprint struct {
	App       AppMetadata            `yaml:"app"`
	Services  []interface{}          `yaml:"services"`
	UI        map[string]interface{} `yaml:"ui"`
	Templates map[string]interface{} `yaml:"templates,omitempty"`
	Config    map[string]interface{} `yaml:"config,omitempty"`
}

// AppMetadata contains app identification and metadata
type AppMetadata struct {
	ID          string   `yaml:"id"`
	Name        string   `yaml:"name"`
	Description string   `yaml:"description,omitempty"`
	Icon        string   `yaml:"icon,omitempty"`
	Category    string   `yaml:"category,omitempty"`
	Version     string   `yaml:"version"`
	Author      string   `yaml:"author"`
	Tags        []string `yaml:"tags,omitempty"`
	Permissions []string `yaml:"permissions"`
}

// Parser handles Blueprint to Package conversion
type Parser struct {
	templates map[string]interface{}
}

// NewParser creates a new Blueprint parser
func NewParser() *Parser {
	return &Parser{
		templates: make(map[string]interface{}),
	}
}

// Parse converts Blueprint YAML content to a Package
func (p *Parser) Parse(content []byte) (*types.Package, error) {
	var bp Blueprint
	if err := yaml.Unmarshal(content, &bp); err != nil {
		return nil, fmt.Errorf("failed to parse YAML: %w", err)
	}

	// Validate required fields
	if bp.App.ID == "" {
		return nil, fmt.Errorf("app.id is required")
	}
	if bp.App.Name == "" {
		return nil, fmt.Errorf("app.name is required")
	}

	// Default timestamps
	now := time.Now()

	// Store templates for component expansion
	if templates, ok := bp.UI["templates"].(map[string]interface{}); ok {
		p.templates = templates
	}

	// Expand services
	services := p.expandServices(bp.Services)

	// Expand UI components
	uiSpec := p.expandUISpec(bp.UI)

	return &types.Package{
		ID:          bp.App.ID,
		Name:        bp.App.Name,
		Description: bp.App.Description,
		Icon:        bp.App.Icon,
		Category:    bp.App.Category,
		Version:     bp.App.Version,
		Author:      bp.App.Author,
		CreatedAt:   now,
		UpdatedAt:   now,
		Services:    services,
		Permissions: bp.App.Permissions,
		Tags:        bp.App.Tags,
		UISpec:      uiSpec,
	}, nil
}

// expandServices converts service definitions to string array
// Supports: "storage", {storage: [get, set]}, {storage: "*"}
func (p *Parser) expandServices(services []interface{}) []string {
	result := make([]string, 0, len(services))

	for _, svc := range services {
		switch v := svc.(type) {
		case string:
			// Simple string: "storage" (all tools)
			result = append(result, v)
		case map[string]interface{}:
			// Object syntax
			for key, value := range v {
				// For now, just extract service name
				// Tool filtering can be implemented in service registry
				result = append(result, key)

				// TODO: Store tool restrictions in Package metadata
				// This will require updating types.Package to include tool info
				_ = value // Suppress unused warning
			}
		}
	}

	return result
}

// expandUISpec converts Blueprint UI to standard UISpec format
func (p *Parser) expandUISpec(ui map[string]interface{}) map[string]interface{} {
	title, _ := ui["title"].(string)
	layout, _ := ui["layout"].(string)
	if layout == "" {
		layout = "vertical"
	}

	lifecycle, _ := ui["lifecycle"].(map[string]interface{})
	components, _ := ui["components"].([]interface{})

	return map[string]interface{}{
		"type":            "app",
		"title":           title,
		"layout":          layout,
		"lifecycle_hooks": p.expandLifecycle(lifecycle),
		"components":      p.expandComponents(components),
	}
}

// expandLifecycle converts lifecycle hooks
func (p *Parser) expandLifecycle(lifecycle map[string]interface{}) map[string]interface{} {
	if lifecycle == nil {
		return map[string]interface{}{}
	}

	result := make(map[string]interface{})
	for hook, action := range lifecycle {
		// Convert hook names (on_mount -> on_mount)
		switch v := action.(type) {
		case string:
			// Single action
			result[hook] = []string{v}
		case []interface{}:
			// Multiple actions
			actions := make([]string, 0, len(v))
			for _, a := range v {
				if str, ok := a.(string); ok {
					actions = append(actions, str)
				}
			}
			result[hook] = actions
		}
	}

	return result
}

// expandComponents recursively expands Blueprint components
func (p *Parser) expandComponents(components []interface{}) []interface{} {
	result := make([]interface{}, 0, len(components))

	for _, comp := range components {
		expanded := p.expandComponent(comp)
		if expanded != nil {
			result = append(result, expanded)
		}
	}

	return result
}

// expandComponent expands a single component with all shortcuts
func (p *Parser) expandComponent(comp interface{}) map[string]interface{} {
	switch v := comp.(type) {
	case string:
		// Simple string: "Hello" -> text component
		return map[string]interface{}{
			"type": "text",
			"props": map[string]interface{}{
				"content": v,
			},
		}

	case map[string]interface{}:
		// Component object
		// Extract type and ID from the first key
		for key, props := range v {
			propsMap, ok := props.(map[string]interface{})
			if !ok {
				continue
			}

			// Parse "type#id" or just "type"
			parts := strings.Split(key, "#")
			compType := parts[0]
			var compID string
			if len(parts) > 1 {
				compID = parts[1]
			}

			// Handle layout shortcuts
			switch compType {
			case "row":
				compType = "container"
				propsMap["layout"] = "horizontal"
			case "col":
				compType = "container"
				propsMap["layout"] = "vertical"
			}

			// Check for template reference
			if templateName, ok := propsMap["$template"].(string); ok {
				if template, exists := p.templates[templateName]; exists {
					if templateMap, ok := template.(map[string]interface{}); ok {
						// Merge template with current props (current overrides template)
						merged := make(map[string]interface{})
						for k, v := range templateMap {
							merged[k] = v
						}
						for k, v := range propsMap {
							if k != "$template" {
								merged[k] = v
							}
						}
						propsMap = merged
					}
				}
			}

			// Extract event handlers, children, and directives
			events := make(map[string]interface{})
			cleanProps := make(map[string]interface{})
			var children []interface{}
			var conditional interface{}
			var loopConfig interface{}

			for k, v := range propsMap {
				if strings.HasPrefix(k, "@") {
					// Event handler: @click -> click
					eventName := strings.TrimPrefix(k, "@")
					events[eventName] = v
				} else if k == "children" {
					// Recursively expand children
					if childList, ok := v.([]interface{}); ok {
						children = p.expandComponents(childList)
					}
				} else if k == "$if" {
					conditional = v
				} else if k == "$for" {
					loopConfig = v
				} else if !strings.HasPrefix(k, "$") {
					// Regular props (skip other $ directives)
					cleanProps[k] = v
				}
			}

			// Build result
			result := map[string]interface{}{
				"type":  compType,
				"props": cleanProps,
			}

			if compID != "" {
				result["id"] = compID
			}

			if len(events) > 0 {
				result["on_event"] = events
			}

			if len(children) > 0 {
				result["children"] = children
			}

			// Add conditional/loop metadata (frontend will handle)
			if conditional != nil {
				result["$if"] = conditional
			}

			if loopConfig != nil {
				result["$for"] = loopConfig
			}

			return result
		}
	}

	return nil
}

// ParseFile is a convenience function for parsing Blueprint files
func ParseFile(content []byte) (*types.Package, error) {
	parser := NewParser()
	return parser.Parse(content)
}
