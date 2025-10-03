package http

import (
	"net/http"

	"github.com/gin-gonic/gin"
	"github.com/GriffinCanCode/AgentOS/backend/internal/app"
	"github.com/GriffinCanCode/AgentOS/backend/internal/grpc"
	"github.com/GriffinCanCode/AgentOS/backend/internal/service"
	"github.com/GriffinCanCode/AgentOS/backend/internal/types"
)

// Handlers contains all HTTP handlers
type Handlers struct {
	appManager *app.Manager
	registry   *service.Registry
	aiClient   *grpc.AIClient
	kernel     *grpc.KernelClient
}

// NewHandlers creates a new handler set
func NewHandlers(
	appManager *app.Manager,
	registry *service.Registry,
	aiClient *grpc.AIClient,
	kernel *grpc.KernelClient,
) *Handlers {
	return &Handlers{
		appManager: appManager,
		registry:   registry,
		aiClient:   aiClient,
		kernel:     kernel,
	}
}

// Root handles health check
func (h *Handlers) Root(c *gin.Context) {
	c.JSON(http.StatusOK, gin.H{
		"status":  "online",
		"service": "AI-OS Service (Go)",
		"version": "0.2.0",
	})
}

// Health handles detailed health check
func (h *Handlers) Health(c *gin.Context) {
	c.JSON(http.StatusOK, gin.H{
		"status":           "healthy",
		"app_manager":      h.appManager.Stats(),
		"service_registry": h.registry.Stats(),
		"kernel":           gin.H{"connected": h.kernel != nil},
		"ai_service":       gin.H{"connected": h.aiClient != nil},
	})
}

// ListApps lists all running apps
func (h *Handlers) ListApps(c *gin.Context) {
	apps := h.appManager.List(nil)
	stats := h.appManager.Stats()

	c.JSON(http.StatusOK, gin.H{
		"apps":  apps,
		"stats": stats,
	})
}

// FocusApp brings an app to foreground
func (h *Handlers) FocusApp(c *gin.Context) {
	appID := c.Param("id")
	success := h.appManager.Focus(appID)

	c.JSON(http.StatusOK, gin.H{
		"success": success,
		"app_id":  appID,
	})
}

// CloseApp closes and destroys an app
func (h *Handlers) CloseApp(c *gin.Context) {
	appID := c.Param("id")
	success := h.appManager.Close(appID)

	c.JSON(http.StatusOK, gin.H{
		"success": success,
		"app_id":  appID,
	})
}

// ListServices lists all available services
func (h *Handlers) ListServices(c *gin.Context) {
	categoryStr := c.Query("category")
	var category *types.Category
	if categoryStr != "" {
		cat := types.Category(categoryStr)
		category = &cat
	}

	services := h.registry.List(category)
	stats := h.registry.Stats()

	c.JSON(http.StatusOK, gin.H{
		"services": services,
		"stats":    stats,
	})
}

// DiscoverServices discovers relevant services for a request
func (h *Handlers) DiscoverServices(c *gin.Context) {
	var req types.ChatRequest
	if err := c.ShouldBindJSON(&req); err != nil {
		c.JSON(http.StatusBadRequest, gin.H{"error": err.Error()})
		return
	}

	services := h.registry.Discover(req.Message, 5)

	c.JSON(http.StatusOK, gin.H{
		"query":    req.Message,
		"services": services,
	})
}

// ExecuteService executes a service tool
func (h *Handlers) ExecuteService(c *gin.Context) {
	var req types.ExecuteRequest
	if err := c.ShouldBindJSON(&req); err != nil {
		c.JSON(http.StatusBadRequest, gin.H{"error": err.Error()})
		return
	}

	var ctx *types.Context
	if req.AppID != nil {
		if app, ok := h.appManager.Get(*req.AppID); ok {
			ctx = &types.Context{
				AppID:      req.AppID,
				SandboxPID: app.SandboxPID,
			}
		}
	}

	result, err := h.registry.Execute(req.ToolID, req.Params, ctx)
	if err != nil {
		c.JSON(http.StatusInternalServerError, gin.H{"error": err.Error()})
		return
	}

	c.JSON(http.StatusOK, result)
}

// GenerateUI generates a UI specification (non-streaming)
func (h *Handlers) GenerateUI(c *gin.Context) {
	var req types.UIRequest
	if err := c.ShouldBindJSON(&req); err != nil {
		c.JSON(http.StatusBadRequest, gin.H{"error": err.Error()})
		return
	}

	// Convert context to string map
	contextMap := make(map[string]string)
	for k, v := range req.Context {
		if str, ok := v.(string); ok {
			contextMap[k] = str
		}
	}

	// Call AI service
	resp, err := h.aiClient.GenerateUI(req.Message, contextMap, req.ParentID)
	if err != nil {
		c.JSON(http.StatusInternalServerError, gin.H{
			"error":   err.Error(),
			"app_id":  nil,
			"ui_spec": nil,
		})
		return
	}

	if !resp.Success {
		errorStr := "unknown error"
		if resp.Error != nil {
			errorStr = *resp.Error
		}
		c.JSON(http.StatusInternalServerError, gin.H{
			"error":   errorStr,
			"app_id":  nil,
			"ui_spec": nil,
		})
		return
	}

	// Parse UI spec JSON
	var uiSpec map[string]interface{}
	if err := parseJSON(resp.UiSpecJson, &uiSpec); err != nil {
		c.JSON(http.StatusInternalServerError, gin.H{
			"error": "failed to parse UI spec",
		})
		return
	}

	// Register with app manager
	app, err := h.appManager.Spawn(req.Message, uiSpec, req.ParentID)
	if err != nil {
		c.JSON(http.StatusInternalServerError, gin.H{
			"error": err.Error(),
		})
		return
	}

	c.JSON(http.StatusOK, gin.H{
		"app_id":   app.ID,
		"ui_spec":  uiSpec,
		"thoughts": resp.Thoughts,
	})
}
