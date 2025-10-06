package types

import "time"

// Package represents an installable/savable app package
type Package struct {
	ID          string                 `json:"id"`
	Name        string                 `json:"name"`
	Description string                 `json:"description"`
	Icon        string                 `json:"icon"`
	Category    string                 `json:"category"`
	Version     string                 `json:"version"`
	Author      string                 `json:"author"`
	CreatedAt   time.Time              `json:"created_at"`
	UpdatedAt   time.Time              `json:"updated_at"`
	Blueprint   map[string]interface{} `json:"blueprint"`
	Services    []string               `json:"services"`
	Permissions []string               `json:"permissions"`
	Tags        []string               `json:"tags"`
}

// PackageMetadata contains summary information about a package
type PackageMetadata struct {
	ID          string    `json:"id"`
	Name        string    `json:"name"`
	Description string    `json:"description"`
	Icon        string    `json:"icon"`
	Category    string    `json:"category"`
	Version     string    `json:"version"`
	Author      string    `json:"author"`
	CreatedAt   time.Time `json:"created_at"`
	Tags        []string  `json:"tags"`
}

// ToMetadata extracts metadata from a package
func (p *Package) ToMetadata() PackageMetadata {
	return PackageMetadata{
		ID:          p.ID,
		Name:        p.Name,
		Description: p.Description,
		Icon:        p.Icon,
		Category:    p.Category,
		Version:     p.Version,
		Author:      p.Author,
		CreatedAt:   p.CreatedAt,
		Tags:        p.Tags,
	}
}

// RegistryStats contains registry statistics
type RegistryStats struct {
	TotalPackages int            `json:"total_packages"`
	Categories    map[string]int `json:"categories"`
	LastUpdated   *time.Time     `json:"last_updated,omitempty"`
}
