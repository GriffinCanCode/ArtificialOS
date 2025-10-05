package registry

import (
	"context"
	"encoding/json"
	"log"
	"os"
	"path/filepath"
	"strings"

	"github.com/GriffinCanCode/AgentOS/backend/internal/blueprint"
	"github.com/GriffinCanCode/AgentOS/backend/internal/types"
)

// Seeder handles loading prebuilt apps from disk
type Seeder struct {
	manager *Manager
	appsDir string
}

// NewSeeder creates a new app seeder
func NewSeeder(manager *Manager, appsDir string) *Seeder {
	return &Seeder{
		manager: manager,
		appsDir: appsDir,
	}
}

// SeedApps loads all prebuilt apps from the apps directory
func (s *Seeder) SeedApps() error {
	log.Printf("📦 Seeding prebuilt apps from %s...", s.appsDir)

	// Check if apps directory exists
	if _, err := os.Stat(s.appsDir); os.IsNotExist(err) {
		log.Printf("⚠️  Apps directory not found: %s", s.appsDir)
		return nil
	}

	var loaded, failed int

	// Walk through all subdirectories
	err := filepath.Walk(s.appsDir, func(path string, info os.FileInfo, err error) error {
		if err != nil {
			return err
		}

		// Skip directories
		if info.IsDir() {
			return nil
		}

		// Process .aiapp and .bp (both JSON) files
		if !strings.HasSuffix(info.Name(), ".aiapp") && !strings.HasSuffix(info.Name(), ".bp") {
			return nil
		}

		// Load and register the app
		if err := s.loadApp(path); err != nil {
			log.Printf("  ✗ Failed to load %s: %v", info.Name(), err)
			failed++
		} else {
			log.Printf("  ✓ Loaded %s", info.Name())
			loaded++
		}

		return nil
	})

	if err != nil {
		return err
	}

	log.Printf("📊 Seeding complete: %d loaded, %d failed", loaded, failed)
	return nil
}

// loadApp loads a single .aiapp or .bp (both JSON) file and registers it
func (s *Seeder) loadApp(path string) error {
	// Read file
	data, err := os.ReadFile(path)
	if err != nil {
		return err
	}

	var pkg types.Package

	// Parse based on file extension
	if strings.HasSuffix(path, ".bp") {
		// Blueprint JSON format
		parsed, err := blueprint.ParseFile(data)
		if err != nil {
			return err
		}
		pkg = *parsed
	} else {
		// Legacy JSON format (.aiapp)
		if err := json.Unmarshal(data, &pkg); err != nil {
			return err
		}
	}

	// Validate required fields
	if pkg.ID == "" || pkg.Name == "" {
		return filepath.SkipDir
	}

	// Save to registry (this will update if already exists)
	ctx := context.Background()
	return s.manager.Save(ctx, &pkg)
}

// SeedDefaultApps creates essential system apps if they don't exist
func (s *Seeder) SeedDefaultApps() error {
	log.Println("🌱 Seeding default system apps...")

	defaults := []types.Package{
		{
			ID:          "app-launcher",
			Name:        "App Launcher",
			Description: "Browse and launch installed applications",
			Icon:        "🚀",
			Category:    "system",
			Version:     "1.0.0",
			Author:      "system",
			Services:    []string{},
			Permissions: []string{"STANDARD"},
			Tags:        []string{"launcher", "apps", "system"},
			Blueprint: map[string]interface{}{
				"type":   "app",
				"title":  "App Launcher",
				"layout": "vertical",
				"components": []interface{}{
					map[string]interface{}{
						"type": "text",
						"id":   "title",
						"props": map[string]interface{}{
							"content": "Applications",
							"variant": "h1",
						},
					},
					map[string]interface{}{
						"type": "input",
						"id":   "search",
						"props": map[string]interface{}{
							"placeholder": "Search apps...",
							"type":        "text",
						},
					},
					map[string]interface{}{
						"type": "grid",
						"id":   "app-grid",
						"props": map[string]interface{}{
							"columns": 4,
							"gap":     16,
						},
						"children": []interface{}{},
					},
				},
			},
		},
		{
			ID:          "settings",
			Name:        "Settings",
			Description: "System settings and preferences",
			Icon:        "⚙️",
			Category:    "system",
			Version:     "1.0.0",
			Author:      "system",
			Services:    []string{"storage", "system"},
			Permissions: []string{"STANDARD"},
			Tags:        []string{"settings", "config", "system"},
			Blueprint: map[string]interface{}{
				"type":   "app",
				"title":  "Settings",
				"layout": "horizontal",
				"components": []interface{}{
					map[string]interface{}{
						"type": "container",
						"id":   "sidebar",
						"props": map[string]interface{}{
							"layout": "vertical",
							"gap":    8,
							"style": map[string]interface{}{
								"width":       "200px",
								"borderRight": "1px solid rgba(255,255,255,0.1)",
							},
						},
						"children": []interface{}{
							map[string]interface{}{
								"type": "text",
								"id":   "settings-title",
								"props": map[string]interface{}{
									"content": "Settings",
									"variant": "h2",
								},
							},
						},
					},
					map[string]interface{}{
						"type": "container",
						"id":   "content",
						"props": map[string]interface{}{
							"layout": "vertical",
							"gap":    16,
							"style": map[string]interface{}{
								"flex":    "1",
								"padding": "24px",
							},
						},
						"children": []interface{}{
							map[string]interface{}{
								"type": "text",
								"id":   "section-title",
								"props": map[string]interface{}{
									"content": "General",
									"variant": "h3",
								},
							},
						},
					},
				},
			},
		},
		{
			ID:          "calculator",
			Name:        "Calculator",
			Description: "Basic calculator with standard operations",
			Icon:        "🧮",
			Category:    "productivity",
			Version:     "1.0.0",
			Author:      "system",
			Services:    []string{},
			Permissions: []string{"STANDARD"},
			Tags:        []string{"calculator", "math", "utility"},
			Blueprint: map[string]interface{}{
				"type":   "app",
				"title":  "Calculator",
				"layout": "vertical",
				"components": []interface{}{
					map[string]interface{}{
						"type": "input",
						"id":   "display",
						"props": map[string]interface{}{
							"value":    "0",
							"readonly": true,
							"style": map[string]interface{}{
								"fontSize":  "32px",
								"textAlign": "right",
								"padding":   "16px",
							},
						},
					},
					map[string]interface{}{
						"type": "grid",
						"id":   "buttons",
						"props": map[string]interface{}{
							"columns": 4,
							"gap":     8,
						},
						"children": []interface{}{
							map[string]interface{}{
								"type": "button",
								"id":   "btn-7",
								"props": map[string]interface{}{
									"text":    "7",
									"variant": "outline",
								},
								"on_event": map[string]interface{}{
									"click": "ui.append",
								},
							},
							map[string]interface{}{
								"type": "button",
								"id":   "btn-8",
								"props": map[string]interface{}{
									"text":    "8",
									"variant": "outline",
								},
								"on_event": map[string]interface{}{
									"click": "ui.append",
								},
							},
						},
					},
				},
			},
		},
	}

	var seeded int
	ctx := context.Background()
	for _, pkg := range defaults {
		// Only seed if doesn't exist
		if !s.manager.Exists(ctx, pkg.ID) {
			if err := s.manager.Save(ctx, &pkg); err != nil {
				log.Printf("  ✗ Failed to seed %s: %v", pkg.Name, err)
			} else {
				log.Printf("  ✓ Seeded %s", pkg.Name)
				seeded++
			}
		}
	}

	log.Printf("🌱 Seeded %d default apps", seeded)
	return nil
}
