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

// WindowPosition represents window position on screen
type WindowPosition struct {
	X int `json:"x"`
	Y int `json:"y"`
}

// WindowSize represents window dimensions
type WindowSize struct {
	Width  int `json:"width"`
	Height int `json:"height"`
}

// App represents a running application instance
type App struct {
	ID         string                 `json:"id"`
	Hash       string                 `json:"hash"` // Deterministic hash for identification
	Title      string                 `json:"title"`
	Blueprint  map[string]interface{} `json:"ui_spec"`
	State      State                  `json:"state"`
	ParentID   *string                `json:"parent_id,omitempty"`
	CreatedAt  time.Time              `json:"created_at"`
	Metadata   map[string]interface{} `json:"metadata"`
	Services   []string               `json:"services"`
	SandboxPID *uint32                `json:"sandbox_pid,omitempty"`
	OSPID      *uint32                `json:"os_pid,omitempty"` // Actual OS process ID if spawned

	// Window state (for session restoration)
	WindowID   *string         `json:"window_id,omitempty"`
	WindowPos  *WindowPosition `json:"window_pos,omitempty"`
	WindowSize *WindowSize     `json:"window_size,omitempty"`
}

// Stats contains app manager statistics
type Stats struct {
	TotalApps      int     `json:"total_apps"`
	ActiveApps     int     `json:"active_apps"`
	BackgroundApps int     `json:"background_apps"`
	FocusedAppID   *string `json:"focused_app_id,omitempty"`   // App ID
	FocusedAppHash *string `json:"focused_app_hash,omitempty"` // App hash for restoration
}
