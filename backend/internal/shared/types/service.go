package types

// Category represents service categories
type Category string

const (
	CategoryStorage    Category = "storage"
	CategoryFilesystem Category = "filesystem"
	CategoryAI         Category = "ai"
	CategoryAuth       Category = "auth"
	CategorySystem     Category = "system"
	CategoryHTTP       Category = "http"
	CategoryScraper    Category = "scraper"
	CategoryMath       Category = "math"
)

// Service represents a service definition
type Service struct {
	ID           string      `json:"id"`
	Name         string      `json:"name"`
	Description  string      `json:"description"`
	Category     Category    `json:"category"`
	Capabilities []string    `json:"capabilities"`
	Tools        []Tool      `json:"tools"`
	DataModels   []DataModel `json:"data_models,omitempty"`
}

// Tool represents a service tool
type Tool struct {
	ID          string      `json:"id"`
	Name        string      `json:"name"`
	Description string      `json:"description"`
	Parameters  []Parameter `json:"parameters"`
	Returns     string      `json:"returns"`
}

// Parameter represents a tool parameter
type Parameter struct {
	Name        string `json:"name"`
	Type        string `json:"type"`
	Description string `json:"description"`
	Required    bool   `json:"required"`
}

// DataModel represents a data structure
type DataModel struct {
	Name   string            `json:"name"`
	Fields map[string]string `json:"fields"`
}

// Context provides execution context for services
type Context struct {
	AppID      *string `json:"app_id,omitempty"`
	SandboxPID *uint32 `json:"sandbox_pid,omitempty"`
	UserID     *string `json:"user_id,omitempty"`
}

// Result represents a service execution result
type Result struct {
	Success bool                   `json:"success"`
	Data    map[string]interface{} `json:"data,omitempty"`
	Error   *string                `json:"error,omitempty"`
}
