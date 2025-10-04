package types

import "time"

// Session represents a saved workspace state
type Session struct {
	ID          string                 `json:"id"`
	Name        string                 `json:"name"`
	Description string                 `json:"description"`
	CreatedAt   time.Time              `json:"created_at"`
	UpdatedAt   time.Time              `json:"updated_at"`
	Workspace   Workspace              `json:"workspace"`
	Metadata    map[string]interface{} `json:"metadata"`
}

// Workspace contains the complete state of all apps and UI
type Workspace struct {
	Apps        []AppSnapshot `json:"apps"`
	FocusedID   *string       `json:"focused_app_id,omitempty"`
	FocusedHash *string       `json:"focused_app_hash,omitempty"` // Hash for restoration by properties
	ChatState   *ChatState    `json:"chat_state,omitempty"`
	UIState     *UIState      `json:"ui_state,omitempty"`
}

// AppSnapshot captures complete app state for restoration
type AppSnapshot struct {
	ID             string                 `json:"id"`
	Hash           string                 `json:"hash"` // Deterministic hash for matching
	Title          string                 `json:"title"`
	UISpec         map[string]interface{} `json:"ui_spec"`
	State          State                  `json:"state"`
	ParentID       *string                `json:"parent_id,omitempty"`
	CreatedAt      time.Time              `json:"created_at"`
	Metadata       map[string]interface{} `json:"metadata"`
	Services       []string               `json:"services"`
	ComponentState map[string]interface{} `json:"component_state,omitempty"`
}

// ChatState captures chat history
type ChatState struct {
	Messages []Message `json:"messages"`
	Thoughts []Thought `json:"thoughts"`
}

// Message represents a chat message
type Message struct {
	Type      string `json:"type"`
	Content   string `json:"content"`
	Timestamp int64  `json:"timestamp"`
}

// Thought represents a thought stream entry
type Thought struct {
	Content   string `json:"content"`
	Timestamp int64  `json:"timestamp"`
}

// UIState captures frontend UI state
type UIState struct {
	GenerationThoughts []string `json:"generation_thoughts,omitempty"`
	GenerationPreview  string   `json:"generation_preview,omitempty"`
	IsLoading          bool     `json:"is_loading"`
	Error              *string  `json:"error,omitempty"`
}

// SessionMetadata contains summary information
type SessionMetadata struct {
	ID          string    `json:"id"`
	Name        string    `json:"name"`
	Description string    `json:"description"`
	CreatedAt   time.Time `json:"created_at"`
	UpdatedAt   time.Time `json:"updated_at"`
	AppCount    int       `json:"app_count"`
}

// ToMetadata extracts metadata from session
func (s *Session) ToMetadata() SessionMetadata {
	return SessionMetadata{
		ID:          s.ID,
		Name:        s.Name,
		Description: s.Description,
		CreatedAt:   s.CreatedAt,
		UpdatedAt:   s.UpdatedAt,
		AppCount:    len(s.Workspace.Apps),
	}
}

// SessionStats contains session manager statistics
type SessionStats struct {
	TotalSessions int        `json:"total_sessions"`
	LastSaved     *time.Time `json:"last_saved,omitempty"`
	LastRestored  *time.Time `json:"last_restored,omitempty"`
}
