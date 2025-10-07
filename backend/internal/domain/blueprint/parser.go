package blueprint

import (
	"encoding/json"
	"fmt"
	"strings"
	"time"

	"github.com/GriffinCanCode/AgentOS/backend/internal/shared/types"
)

// Blueprint represents the root structure of a .bp file
type Blueprint struct {
	App       AppMetadata            `json:"app"`
	Services  []interface{}          `json:"services"`
	UI        map[string]interface{} `json:"ui"`
	Templates map[string]interface{} `json:"templates,omitempty"`
	Config    map[string]interface{} `json:"config,omitempty"`
}

// AppMetadata contains app identification and metadata
type AppMetadata struct {
	ID          string   `json:"id"`
	Name        string   `json:"name"`
	Description string   `json:"description,omitempty"`
	Icon        string   `json:"icon,omitempty"`
	Category    string   `json:"category,omitempty"`
	Version     string   `json:"version"`
	Author      string   `json:"author"`
	Tags        []string `json:"tags,omitempty"`
	Permissions []string `json:"permissions"`
}

// Parser handles Blueprint to Package conversion
type Parser struct {
	templates map[string]interface{}
	idCounter int
}

// NewParser creates a new Blueprint parser
func NewParser() *Parser {
	return &Parser{
		templates: make(map[string]interface{}),
	}
}

// Parse converts Blueprint JSON content to a Package
func (p *Parser) Parse(content []byte) (*types.Package, error) {
	// Reset ID counter for each parse
	p.idCounter = 0

	var bp Blueprint
	if err := json.Unmarshal(content, &bp); err != nil {
		return nil, fmt.Errorf("failed to parse JSON: %w", err)
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

	// Expand services and extract tool restrictions
	services, restrictions := p.expandServices(bp.Services)

	// Expand UI components
	blueprint := p.expandUISpec(bp.UI)

	return &types.Package{
		ID:                  bp.App.ID,
		Name:                bp.App.Name,
		Description:         bp.App.Description,
		Icon:                bp.App.Icon,
		Category:            bp.App.Category,
		Version:             bp.App.Version,
		Author:              bp.App.Author,
		CreatedAt:           now,
		UpdatedAt:           now,
		Services:            services,
		Permissions:         bp.App.Permissions,
		Tags:                bp.App.Tags,
		ServiceRestrictions: restrictions,
		Blueprint:           blueprint,
	}, nil
}

// expandServices converts service definitions to string array
// Supports: "storage", {storage: [get, set]}, {storage: "*"}
// Returns both the service list and tool restrictions
func (p *Parser) expandServices(services []interface{}) ([]string, []types.ServiceToolRestriction) {
	serviceList := make([]string, 0, len(services))
	restrictions := make([]types.ServiceToolRestriction, 0)

	for _, svc := range services {
		switch v := svc.(type) {
		case string:
			// Simple string: "storage" (all tools allowed)
			serviceList = append(serviceList, v)
			restrictions = append(restrictions, types.ServiceToolRestriction{
				Service:  v,
				AllowAll: true,
			})
		case map[string]interface{}:
			// Object syntax: {storage: [get, set]} or {storage: "*"}
			for key, value := range v {
				serviceList = append(serviceList, key)

				restriction := types.ServiceToolRestriction{
					Service: key,
				}

				switch tools := value.(type) {
				case string:
					if tools == "*" {
						// All tools allowed
						restriction.AllowAll = true
					}
				case []interface{}:
					// Specific tool list
					toolNames := make([]string, 0, len(tools))
					for _, tool := range tools {
						if toolStr, ok := tool.(string); ok {
							toolNames = append(toolNames, toolStr)
						}
					}
					if len(toolNames) > 0 {
						restriction.AllowedTools = toolNames
						restriction.AllowAll = false
					}
				}

				restrictions = append(restrictions, restriction)
			}
		}
	}

	return serviceList, restrictions
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
		compID := fmt.Sprintf("text-%d", p.idCounter)
		p.idCounter++
		return map[string]interface{}{
			"type": "text",
			"id":   compID,
			"props": map[string]interface{}{
				"content": v,
			},
		}

	case map[string]interface{}:
		// Component object
		// Check if this is explicit format (has "type" key) or shorthand format
		if explicitType, hasType := v["type"].(string); hasType {
			// Explicit format: {"type": "button", "id": "...", "props": {...}, "on_event": {...}}
			compType := explicitType
			compID, _ := v["id"].(string)
			propsMap, _ := v["props"].(map[string]interface{})
			if propsMap == nil {
				propsMap = make(map[string]interface{})
			}
			onEventMap, _ := v["on_event"].(map[string]interface{})
			childrenList, _ := v["children"].([]interface{})

			// Build result with explicit format
			result := map[string]interface{}{
				"type":  compType,
				"props": propsMap,
			}

			// Ensure every component has an ID
			if compID != "" {
				result["id"] = compID
			} else {
				result["id"] = fmt.Sprintf("%s-%d", compType, p.idCounter)
				p.idCounter++
			}

			// Add event handlers if present
			if len(onEventMap) > 0 {
				result["on_event"] = onEventMap
			}

			// Recursively expand children if present
			if len(childrenList) > 0 {
				result["children"] = p.expandComponents(childrenList)
			}

			return result
		}

		// Shorthand format: {"button#id": {...props}}
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
			case "sidebar", "main", "editor", "header", "footer", "content", "section":
				// Semantic container shortcuts - preserve semantic meaning via role
				role := compType
				compType = "container"
				propsMap["role"] = role // Add role for styling/identification
				if _, hasLayout := propsMap["layout"]; !hasLayout {
					propsMap["layout"] = "vertical"
				}
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

			// Ensure every component has an ID (required by frontend)
			if compID != "" {
				result["id"] = compID
			} else {
				// Generate auto ID: type-N (e.g., "container-0", "button-1")
				result["id"] = fmt.Sprintf("%s-%d", compType, p.idCounter)
				p.idCounter++
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
