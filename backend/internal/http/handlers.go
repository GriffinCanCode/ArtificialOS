package http

import (
	"net/http"
	"time"

	"github.com/GriffinCanCode/AgentOS/backend/internal/app"
	"github.com/GriffinCanCode/AgentOS/backend/internal/grpc"
	"github.com/GriffinCanCode/AgentOS/backend/internal/registry"
	"github.com/GriffinCanCode/AgentOS/backend/internal/service"
	"github.com/GriffinCanCode/AgentOS/backend/internal/types"
	"github.com/gin-gonic/gin"
)

// Handlers contains all HTTP handlers
type Handlers struct {
	appManager  *app.Manager
	registry    *service.Registry
	appRegistry *registry.Manager
	aiClient    *grpc.AIClient
	kernel      *grpc.KernelClient
}

// NewHandlers creates a new handler set
func NewHandlers(
	appManager *app.Manager,
	registry *service.Registry,
	appRegistry *registry.Manager,
	aiClient *grpc.AIClient,
	kernel *grpc.KernelClient,
) *Handlers {
	return &Handlers{
		appManager:  appManager,
		registry:    registry,
		appRegistry: appRegistry,
		aiClient:    aiClient,
		kernel:      kernel,
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

// SaveAppToRegistry saves a running app to the registry
func (h *Handlers) SaveAppToRegistry(c *gin.Context) {
	var req struct {
		AppID       string   `json:"app_id" binding:"required"`
		Description string   `json:"description"`
		Icon        string   `json:"icon"`
		Category    string   `json:"category"`
		Tags        []string `json:"tags"`
	}

	if err := c.ShouldBindJSON(&req); err != nil {
		c.JSON(http.StatusBadRequest, gin.H{"error": err.Error()})
		return
	}

	// Get app from manager
	app, ok := h.appManager.Get(req.AppID)
	if !ok {
		c.JSON(http.StatusNotFound, gin.H{"error": "app not found"})
		return
	}

	// Create package
	pkg := &types.Package{
		ID:          generatePackageID(app.Title),
		Name:        app.Title,
		Description: req.Description,
		Icon:        req.Icon,
		Category:    req.Category,
		Version:     "1.0.0",
		Author:      "user",
		CreatedAt:   time.Now(),
		UpdatedAt:   time.Now(),
		UISpec:      app.UISpec,
		Services:    app.Services,
		Permissions: []string{"STANDARD"},
		Tags:        req.Tags,
	}

	if pkg.Icon == "" {
		pkg.Icon = "📦"
	}
	if pkg.Category == "" {
		pkg.Category = "general"
	}

	// Save to registry
	if err := h.appRegistry.Save(pkg); err != nil {
		c.JSON(http.StatusInternalServerError, gin.H{"error": err.Error()})
		return
	}

	c.JSON(http.StatusOK, gin.H{
		"success":    true,
		"package_id": pkg.ID,
	})
}

// ListRegistryApps lists all apps in the registry
func (h *Handlers) ListRegistryApps(c *gin.Context) {
	categoryParam := c.Query("category")
	var category *string
	if categoryParam != "" {
		category = &categoryParam
	}

	// Get metadata only for performance
	metadata, err := h.appRegistry.ListMetadata(category)
	if err != nil {
		c.JSON(http.StatusInternalServerError, gin.H{"error": err.Error()})
		return
	}

	stats := h.appRegistry.Stats()

	c.JSON(http.StatusOK, gin.H{
		"apps":  metadata,
		"stats": stats,
	})
}

// LaunchRegistryApp launches an app from the registry
func (h *Handlers) LaunchRegistryApp(c *gin.Context) {
	packageID := c.Param("id")

	// Load package
	pkg, err := h.appRegistry.Load(packageID)
	if err != nil {
		c.JSON(http.StatusNotFound, gin.H{"error": "package not found"})
		return
	}

	// Spawn app directly from saved UISpec (no AI generation needed!)
	app, err := h.appManager.Spawn(
		"Launch "+pkg.Name,
		pkg.UISpec,
		nil,
	)
	if err != nil {
		c.JSON(http.StatusInternalServerError, gin.H{"error": err.Error()})
		return
	}

	// Add metadata
	app.Metadata["from_registry"] = true
	app.Metadata["package_id"] = pkg.ID

	c.JSON(http.StatusOK, gin.H{
		"app_id":  app.ID,
		"ui_spec": app.UISpec,
		"title":   app.Title,
	})
}

// DeleteRegistryApp deletes an app from the registry
func (h *Handlers) DeleteRegistryApp(c *gin.Context) {
	packageID := c.Param("id")

	if err := h.appRegistry.Delete(packageID); err != nil {
		c.JSON(http.StatusInternalServerError, gin.H{"error": err.Error()})
		return
	}

	c.JSON(http.StatusOK, gin.H{
		"success":    true,
		"package_id": packageID,
	})
}

// GetRegistryApp gets details of a specific registry app
func (h *Handlers) GetRegistryApp(c *gin.Context) {
	packageID := c.Param("id")

	pkg, err := h.appRegistry.Load(packageID)
	if err != nil {
		c.JSON(http.StatusNotFound, gin.H{"error": "package not found"})
		return
	}

	c.JSON(http.StatusOK, pkg)
}

// generatePackageID creates a unique package ID from app title
func generatePackageID(title string) string {
	// Simple implementation - normalize to lowercase, replace spaces with hyphens
	id := title
	id = sanitizeID(id)
	return id + "-" + time.Now().Format("20060102")
}

func sanitizeID(s string) string {
	// Convert to lowercase and replace spaces/special chars
	result := ""
	for _, r := range s {
		if (r >= 'a' && r <= 'z') || (r >= '0' && r <= '9') {
			result += string(r)
		} else if r >= 'A' && r <= 'Z' {
			result += string(r + 32) // to lowercase
		} else if r == ' ' || r == '_' {
			result += "-"
		}
	}
	return result
}
