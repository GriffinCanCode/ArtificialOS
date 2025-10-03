package types

import "time"

// State represents app lifecycle states
type State string

const (
	StateSpawning   State = "spawning"
	StateActive     State = "active"
	StateBackground State = "background"
	StateSuspended  State = "suspended"
	StateDestroyed  State = "destroyed"
)

// App represents a running application instance
type App struct {
	ID         string                 `json:"id"`
	Title      string                 `json:"title"`
	UISpec     map[string]interface{} `json:"ui_spec"`
	State      State                  `json:"state"`
	ParentID   *string                `json:"parent_id,omitempty"`
	CreatedAt  time.Time              `json:"created_at"`
	Metadata   map[string]interface{} `json:"metadata"`
	Services   []string               `json:"services"`
	SandboxPID *uint32                `json:"sandbox_pid,omitempty"`
}

// Stats contains app manager statistics
type Stats struct {
	TotalApps      int     `json:"total_apps"`
	ActiveApps     int     `json:"active_apps"`
	BackgroundApps int     `json:"background_apps"`
	FocusedApp     *string `json:"focused_app,omitempty"`
}
