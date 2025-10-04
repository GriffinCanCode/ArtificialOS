package server

import (
	"context"
	"fmt"

	"github.com/GriffinCanCode/AgentOS/backend/internal/app"
	"github.com/GriffinCanCode/AgentOS/backend/internal/config"
	"github.com/GriffinCanCode/AgentOS/backend/internal/grpc"
	"github.com/GriffinCanCode/AgentOS/backend/internal/http"
	"github.com/GriffinCanCode/AgentOS/backend/internal/logging"
	"github.com/GriffinCanCode/AgentOS/backend/internal/middleware"
	"github.com/GriffinCanCode/AgentOS/backend/internal/providers"
	"github.com/GriffinCanCode/AgentOS/backend/internal/registry"
	"github.com/GriffinCanCode/AgentOS/backend/internal/service"
	"github.com/GriffinCanCode/AgentOS/backend/internal/session"
	"github.com/GriffinCanCode/AgentOS/backend/internal/ws"
	"github.com/gin-gonic/gin"
	"go.uber.org/zap"
)

// Server wraps the HTTP server and dependencies
type Server struct {
	router         *gin.Engine
	appManager     *app.Manager
	registry       *service.Registry
	appRegistry    *registry.Manager
	sessionManager *session.Manager
	aiClient       *grpc.AIClient
	kernel         *grpc.KernelClient
	logger         *logging.Logger
	config         *config.Config
}

// NewServer creates a new server instance
func NewServer(cfg *config.Config) (*Server, error) {
	// Initialize logger
	var logger *logging.Logger
	if cfg.Logging.Development {
		logger = logging.NewDevelopment()
	} else {
		logger = logging.NewDefault()
	}
	defer logger.Sync()

	logger.Info("🤖 Initializing AI-OS Server",
		zap.String("port", cfg.Server.Port),
		zap.String("kernel_addr", cfg.Kernel.Address),
		zap.String("ai_addr", cfg.AI.Address),
	)

	// Initialize kernel client (optional)
	var kernelClient *grpc.KernelClient
	if cfg.Kernel.Enabled && cfg.Kernel.Address != "" {
		client, err := grpc.NewKernelClient(cfg.Kernel.Address)
		if err != nil {
			logger.Warn("Failed to connect to kernel", zap.Error(err))
		} else {
			kernelClient = client
			logger.Info("✅ Connected to kernel", zap.String("addr", cfg.Kernel.Address))
		}
	}

	// Initialize AI client (required)
	aiClient, err := grpc.NewAIClient(cfg.AI.Address)
	if err != nil {
		// Clean up kernel client if AI client fails
		if kernelClient != nil {
			kernelClient.Close()
		}
		return nil, fmt.Errorf("failed to connect to AI service: %w", err)
	}
	logger.Info("✅ Connected to AI service", zap.String("addr", cfg.AI.Address))

	// Initialize app manager and service registry
	appManager := app.NewManager(kernelClient)
	serviceRegistry := service.NewRegistry()

	// Register service providers
	logger.Info("📦 Registering service providers...")
	registerProviders(serviceRegistry, kernelClient)

	// Initialize app registry with storage
	// Create a dummy PID for system storage operations
	var storagePID uint32 = 1
	if kernelClient != nil {
		// In production, create a dedicated process for storage
		ctx := context.Background()
		pid, err := kernelClient.CreateProcess(ctx, "storage-manager", 10, "PRIVILEGED")
		if err == nil && pid != nil {
			storagePID = *pid
		}
	}
	appRegistry := registry.NewManager(kernelClient, storagePID, "/tmp/ai-os-storage/system")

	// Seed prebuilt apps
	logger.Info("🌱 Loading prebuilt apps...")
	seeder := registry.NewSeeder(appRegistry, "../apps")
	if err := seeder.SeedApps(); err != nil {
		logger.Warn("Failed to seed prebuilt apps", zap.Error(err))
	}
	if err := seeder.SeedDefaultApps(); err != nil {
		logger.Warn("Failed to seed default apps", zap.Error(err))
	}

	// Initialize session manager
	sessionManager := session.NewManager(appManager, kernelClient, storagePID, "/tmp/ai-os-storage/system")

	// Create router
	if !cfg.Logging.Development {
		gin.SetMode(gin.ReleaseMode)
	}
	router := gin.New()

	// Add middleware
	router.Use(gin.Recovery())
	router.Use(middleware.CORS(middleware.DefaultCORSConfig()))
	if cfg.RateLimit.Enabled {
		logger.Info("⚡ Rate limiting enabled",
			zap.Int("rps", cfg.RateLimit.RequestsPerSecond),
			zap.Int("burst", cfg.RateLimit.Burst),
		)
		router.Use(middleware.RateLimit(middleware.RateLimitConfig{
			RequestsPerSecond: cfg.RateLimit.RequestsPerSecond,
			Burst:             cfg.RateLimit.Burst,
		}))
	}

	// Create handlers
	handlers := http.NewHandlers(appManager, serviceRegistry, appRegistry, sessionManager, aiClient, kernelClient)
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

	// Session endpoints
	router.POST("/sessions/save", handlers.SaveSession)
	router.POST("/sessions/save-default", handlers.SaveDefaultSession)
	router.GET("/sessions", handlers.ListSessions)
	router.GET("/sessions/:id", handlers.GetSession)
	router.POST("/sessions/:id/restore", handlers.RestoreSession)
	router.DELETE("/sessions/:id", handlers.DeleteSession)

	// WebSocket
	router.GET("/stream", wsHandler.HandleConnection)

	logger.Info("🚀 Server initialized successfully")

	return &Server{
		router:         router,
		appManager:     appManager,
		registry:       serviceRegistry,
		appRegistry:    appRegistry,
		sessionManager: sessionManager,
		aiClient:       aiClient,
		kernel:         kernelClient,
		logger:         logger,
		config:         cfg,
	}, nil
}

// Run starts the HTTP server
func (s *Server) Run() error {
	addr := s.config.Server.Host + ":" + s.config.Server.Port
	s.logger.Info("🎧 Starting HTTP server", zap.String("addr", addr))
	return s.router.Run(addr)
}

// Close gracefully shuts down the server
func (s *Server) Close() error {
	s.logger.Info("🛑 Shutting down server...")

	// Close gRPC connections
	if s.kernel != nil {
		if err := s.kernel.Close(); err != nil {
			s.logger.Error("Failed to close kernel client", zap.Error(err))
			return fmt.Errorf("failed to close kernel client: %w", err)
		}
		s.logger.Info("Closed kernel connection")
	}
	if s.aiClient != nil {
		if err := s.aiClient.Close(); err != nil {
			s.logger.Error("Failed to close AI client", zap.Error(err))
			return fmt.Errorf("failed to close AI client: %w", err)
		}
		s.logger.Info("Closed AI connection")
	}

	// Sync logger before exit
	s.logger.Sync()

	return nil
}

func registerProviders(registry *service.Registry, kernel *grpc.KernelClient) {
	storagePath := "/tmp/ai-os-storage"
	var storagePID uint32 = 1

	// Storage provider
	storageProvider := providers.NewStorage(kernel, storagePID, storagePath)
	if err := registry.Register(storageProvider); err != nil {
		// Using fmt package for now; logger not available in this context
		fmt.Printf("Warning: Failed to register storage provider: %v\n", err)
	}

	// Auth provider
	authProvider := providers.NewAuth(kernel, storagePID, storagePath)
	if err := registry.Register(authProvider); err != nil {
		fmt.Printf("Warning: Failed to register auth provider: %v\n", err)
	}

	// System provider
	systemProvider := providers.NewSystem()
	if err := registry.Register(systemProvider); err != nil {
		fmt.Printf("Warning: Failed to register system provider: %v\n", err)
	}

	// Filesystem provider
	filesystemProvider := providers.NewFilesystem(kernel, storagePID, storagePath)
	if err := registry.Register(filesystemProvider); err != nil {
		fmt.Printf("Warning: Failed to register filesystem provider: %v\n", err)
	}
}
