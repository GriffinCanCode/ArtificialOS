package types

// ChatRequest represents a chat message request
type ChatRequest struct {
	Message string                 `json:"message" binding:"required"`
	Context map[string]interface{} `json:"context"`
}

// UIRequest represents a UI generation request
type UIRequest struct {
	Message  string                 `json:"message" binding:"required"`
	Context  map[string]interface{} `json:"context"`
	ParentID *string                `json:"parent_id,omitempty"`
}

// ExecuteRequest represents a service execution request
type ExecuteRequest struct {
	ToolID string                 `json:"tool_id" binding:"required"`
	Params map[string]interface{} `json:"params" binding:"required"`
	AppID  *string                `json:"app_id,omitempty"`
}

// WSMessage represents a WebSocket message
type WSMessage struct {
	Type    string                 `json:"type"`
	Message string                 `json:"message,omitempty"`
	Context map[string]interface{} `json:"context,omitempty"`
}
