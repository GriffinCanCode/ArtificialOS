package ws

import (
	"context"
	"encoding/json"
	"log"
	"net/http"
	"strings"
	"time"

	"github.com/GriffinCanCode/AgentOS/backend/internal/app"
	"github.com/GriffinCanCode/AgentOS/backend/internal/grpc"
	"github.com/GriffinCanCode/AgentOS/backend/internal/types"
	"github.com/GriffinCanCode/AgentOS/backend/internal/utils"
	"github.com/gin-gonic/gin"
	"github.com/gorilla/websocket"
)

var upgrader = websocket.Upgrader{
	CheckOrigin: func(r *http.Request) bool {
		return true // Allow all origins in dev
	},
}

// Handler manages WebSocket connections
type Handler struct {
	appManager *app.Manager
	aiClient   *grpc.AIClient
}

// NewHandler creates a new WebSocket handler
func NewHandler(appManager *app.Manager, aiClient *grpc.AIClient) *Handler {
	return &Handler{
		appManager: appManager,
		aiClient:   aiClient,
	}
}

// convertTokenType converts protobuf enum token types to frontend-expected format
func convertTokenType(protoType string) string {
	// Convert UPPERCASE to lowercase
	lower := strings.ToLower(protoType)

	// Handle special case: TOKEN in chat context should be "token"
	// but TOKEN in UI generation context is already handled as "generation_token" by the caller
	return lower
}

// HandleConnection handles WebSocket upgrade and messages
func (h *Handler) HandleConnection(c *gin.Context) {
	conn, err := upgrader.Upgrade(c.Writer, c.Request, nil)
	if err != nil {
		log.Printf("WebSocket upgrade failed: %v", err)
		return
	}
	defer conn.Close()

	// Get request context for propagation
	reqCtx := c.Request.Context()

	// Send welcome message
	h.send(conn, map[string]interface{}{
		"type":    "system",
		"message": "Connected to AI-OS Service (Go)",
	})

	// Listen for messages
	for {
		var msg types.WSMessage
		if err := conn.ReadJSON(&msg); err != nil {
			log.Printf("WebSocket read error: %v", err)
			break
		}

		switch msg.Type {
		case "chat":
			h.handleChat(conn, msg, reqCtx)
		case "generate_ui":
			h.handleGenerateUI(conn, msg, reqCtx)
		case "ping":
			h.send(conn, map[string]interface{}{"type": "pong"})
		default:
			h.sendError(conn, "unknown message type")
		}
	}
}

func (h *Handler) handleChat(conn *websocket.Conn, msg types.WSMessage, reqCtx context.Context) {
	// Convert context to string map
	contextMap := make(map[string]string)
	for k, v := range msg.Context {
		if str, ok := v.(string); ok {
			contextMap[k] = str
		}
	}

	// Add explicit timeout to prevent indefinite blocking on AI service
	// Derive from parent context to respect cancellations
	ctx, cancel := context.WithTimeout(reqCtx, 2*time.Minute)
	defer cancel()

	// Start streaming with timeout context
	stream, err := h.aiClient.StreamChat(ctx, msg.Message, contextMap, nil)
	if err != nil {
		h.sendError(conn, err.Error())
		return
	}

	// Forward tokens to client
	err = grpc.HandleChatStream(stream, func(tokenType, content string) error {
		return h.send(conn, map[string]interface{}{
			"type":      convertTokenType(tokenType),
			"content":   content,
			"timestamp": time.Now().Unix(),
		})
	})

	if err != nil {
		h.sendError(conn, err.Error())
		return
	}

	h.send(conn, map[string]interface{}{
		"type":      "complete",
		"timestamp": time.Now().Unix(),
	})
}

func (h *Handler) handleGenerateUI(conn *websocket.Conn, msg types.WSMessage, reqCtx context.Context) {
	// Convert context to string map
	contextMap := make(map[string]string)
	for k, v := range msg.Context {
		if str, ok := v.(string); ok {
			contextMap[k] = str
		}
	}

	// Validate context size to prevent DoS
	if err := utils.ValidateContext(contextMap); err != nil {
		log.Printf("Context validation failed: %v", err)
		h.sendError(conn, "Context too large")
		return
	}

	// Get parent ID if present
	var parentID *string
	if pid, ok := msg.Context["parent_app_id"].(string); ok {
		parentID = &pid
	}

	h.send(conn, map[string]interface{}{
		"type":      "generation_start",
		"message":   "Analyzing request...",
		"timestamp": time.Now().Unix(),
	})

	// Add explicit timeout for UI generation (longer than chat)
	// Derive from parent context to respect cancellations
	ctx, cancel := context.WithTimeout(reqCtx, 3*time.Minute)
	defer cancel()

	// Start streaming UI generation with timeout context
	stream, err := h.aiClient.StreamUI(ctx, msg.Message, contextMap, parentID)
	if err != nil {
		h.sendError(conn, err.Error())
		return
	}

	var uiSpecJSON string
	var thoughts []string

	// Collect tokens and forward to client
	err = grpc.HandleUIStream(stream, func(tokenType, content string) error {
		switch strings.ToUpper(tokenType) {
		case "GENERATION_START":
			// Reset buffer on new generation start (happens when LLM fails and falls back)
			uiSpecJSON = ""
			thoughts = []string{}
			return h.send(conn, map[string]interface{}{
				"type":      "generation_start",
				"message":   content,
				"timestamp": time.Now().Unix(),
			})
		case "THOUGHT":
			thoughts = append(thoughts, content)
			return h.send(conn, map[string]interface{}{
				"type":      "thought",
				"content":   content,
				"timestamp": time.Now().Unix(),
			})
		case "TOKEN":
			uiSpecJSON += content
			return h.send(conn, map[string]interface{}{
				"type":      "generation_token",
				"content":   content,
				"timestamp": time.Now().Unix(),
			})
		case "COMPLETE":
			return nil
		case "ERROR":
			return h.sendError(conn, content)
		}
		return nil
	})

	if err != nil {
		h.sendError(conn, err.Error())
		return
	}

	// Validate UI spec size and structure
	if err := utils.ValidateUISpec(uiSpecJSON); err != nil {
		log.Printf("UI spec validation failed: %v", err)
		h.sendError(conn, "UI spec validation failed: too large or malformed")
		return
	}

	// Parse UI spec
	var uiSpec map[string]interface{}
	if err := json.Unmarshal([]byte(uiSpecJSON), &uiSpec); err != nil {
		h.sendError(conn, "failed to parse UI spec")
		return
	}

	// Create fresh context for app spawning (AI context may be near timeout)
	spawnCtx, spawnCancel := context.WithTimeout(context.Background(), 10*time.Second)
	defer spawnCancel()

	// Register app with dedicated spawn context
	app, err := h.appManager.Spawn(spawnCtx, msg.Message, uiSpec, parentID)
	if err != nil {
		h.sendError(conn, err.Error())
		return
	}

	// Send UI generated message
	h.send(conn, map[string]interface{}{
		"type":      "ui_generated",
		"app_id":    app.ID,
		"ui_spec":   uiSpec,
		"timestamp": time.Now().Unix(),
	})

	h.send(conn, map[string]interface{}{
		"type":      "complete",
		"timestamp": time.Now().Unix(),
	})
}

func (h *Handler) send(conn *websocket.Conn, data interface{}) error {
	return conn.WriteJSON(data)
}

func (h *Handler) sendError(conn *websocket.Conn, msg string) error {
	return h.send(conn, map[string]interface{}{
		"type":      "error",
		"message":   msg,
		"timestamp": time.Now().Unix(),
	})
}
