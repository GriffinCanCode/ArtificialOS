package server

import (
	"log"

	"github.com/GriffinCanCode/AgentOS/backend/internal/app"
	"github.com/GriffinCanCode/AgentOS/backend/internal/grpc"
	"github.com/GriffinCanCode/AgentOS/backend/internal/http"
	"github.com/GriffinCanCode/AgentOS/backend/internal/registry"
	"github.com/GriffinCanCode/AgentOS/backend/internal/service"
	"github.com/GriffinCanCode/AgentOS/backend/internal/ws"
	"github.com/gin-gonic/gin"
)

// Server wraps the HTTP server and dependencies
type Server struct {
	router      *gin.Engine
	appManager  *app.Manager
	registry    *service.Registry
	appRegistry *registry.Manager
	aiClient    *grpc.AIClient
	kernel      *grpc.KernelClient
}

// Config contains server configuration
type Config struct {
	Port          string
	KernelAddr    string
	AIServiceAddr string
}

// NewServer creates a new server instance
func NewServer(cfg Config) (*Server, error) {
	// Initialize kernel client (optional)
	var kernelClient *grpc.KernelClient
	if cfg.KernelAddr != "" {
		client, err := grpc.NewKernelClient(cfg.KernelAddr)
		if err != nil {
			log.Printf("Warning: Failed to connect to kernel: %v", err)
		} else {
			kernelClient = client
			log.Println("✅ Connected to kernel")
		}
	}

	// Initialize AI client (required)
	aiClient, err := grpc.NewAIClient(cfg.AIServiceAddr)
	if err != nil {
		return nil, err
	}
	log.Println("✅ Connected to AI service")

	// Initialize app manager and service registry
	appManager := app.NewManager(kernelClient)
	serviceRegistry := service.NewRegistry()

	// Initialize app registry with storage
	// Create a dummy PID for system storage operations
	var storagePID uint32 = 1
	if kernelClient != nil {
		// In production, create a dedicated process for storage
		pid, err := kernelClient.CreateProcess("storage-manager", 10, "PRIVILEGED")
		if err == nil && pid != nil {
			storagePID = *pid
		}
	}
	appRegistry := registry.NewManager(kernelClient, storagePID, "/tmp/ai-os-storage/system")

	// Create router
	router := gin.Default()

	// Enable CORS
	router.Use(corsMiddleware())

	// Create handlers
	handlers := http.NewHandlers(appManager, serviceRegistry, appRegistry, aiClient, kernelClient)
	wsHandler := ws.NewHandler(appManager, aiClient)

	// Register routes
	router.GET("/", handlers.Root)
	router.GET("/health", handlers.Health)

	// App management
	router.GET("/apps", handlers.ListApps)
	router.POST("/apps/:id/focus", handlers.FocusApp)
	router.DELETE("/apps/:id", handlers.CloseApp)

	// Service management
	router.GET("/services", handlers.ListServices)
	router.POST("/services/discover", handlers.DiscoverServices)
	router.POST("/services/execute", handlers.ExecuteService)

	// AI operations
	router.POST("/generate-ui", handlers.GenerateUI)

	// App Registry endpoints
	router.POST("/registry/save", handlers.SaveAppToRegistry)
	router.GET("/registry/apps", handlers.ListRegistryApps)
	router.GET("/registry/apps/:id", handlers.GetRegistryApp)
	router.POST("/registry/apps/:id/launch", handlers.LaunchRegistryApp)
	router.DELETE("/registry/apps/:id", handlers.DeleteRegistryApp)

	// WebSocket
	router.GET("/stream", wsHandler.HandleConnection)

	return &Server{
		router:      router,
		appManager:  appManager,
		registry:    serviceRegistry,
		appRegistry: appRegistry,
		aiClient:    aiClient,
		kernel:      kernelClient,
	}, nil
}

// Run starts the server
func (s *Server) Run(port string) error {
	log.Printf("🚀 Starting Go service on :%s", port)
	return s.router.Run(":" + port)
}

// Close cleans up resources
func (s *Server) Close() error {
	if s.aiClient != nil {
		if err := s.aiClient.Close(); err != nil {
			log.Printf("Error closing AI client: %v", err)
		}
	}
	if s.kernel != nil {
		if err := s.kernel.Close(); err != nil {
			log.Printf("Error closing kernel client: %v", err)
		}
	}
	return nil
}

func corsMiddleware() gin.HandlerFunc {
	return func(c *gin.Context) {
		c.Writer.Header().Set("Access-Control-Allow-Origin", "*")
		c.Writer.Header().Set("Access-Control-Allow-Credentials", "true")
		c.Writer.Header().Set("Access-Control-Allow-Headers", "Content-Type, Content-Length, Accept-Encoding, X-CSRF-Token, Authorization, accept, origin, Cache-Control, X-Requested-With")
		c.Writer.Header().Set("Access-Control-Allow-Methods", "POST, OPTIONS, GET, PUT, DELETE")

		if c.Request.Method == "OPTIONS" {
			c.AbortWithStatus(204)
			return
		}

		c.Next()
	}
}
