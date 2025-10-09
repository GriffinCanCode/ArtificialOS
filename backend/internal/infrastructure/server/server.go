package server

import (
	"context"
	"fmt"
	"os"

	"github.com/gin-gonic/gin"
	"go.uber.org/zap"

	"github.com/GriffinCanCode/AgentOS/backend/internal/api/http"
	"github.com/GriffinCanCode/AgentOS/backend/internal/api/middleware"
	"github.com/GriffinCanCode/AgentOS/backend/internal/api/ws"
	"github.com/GriffinCanCode/AgentOS/backend/internal/domain/app"
	"github.com/GriffinCanCode/AgentOS/backend/internal/domain/registry"
	"github.com/GriffinCanCode/AgentOS/backend/internal/domain/service"
	"github.com/GriffinCanCode/AgentOS/backend/internal/domain/session"
	"github.com/GriffinCanCode/AgentOS/backend/internal/grpc"
	"github.com/GriffinCanCode/AgentOS/backend/internal/grpc/kernel"
	"github.com/GriffinCanCode/AgentOS/backend/internal/infrastructure/config"
	"github.com/GriffinCanCode/AgentOS/backend/internal/infrastructure/logging"
	"github.com/GriffinCanCode/AgentOS/backend/internal/infrastructure/monitoring"
	"github.com/GriffinCanCode/AgentOS/backend/internal/infrastructure/tracing"
	"github.com/GriffinCanCode/AgentOS/backend/internal/providers/auth"
	browserProvider "github.com/GriffinCanCode/AgentOS/backend/internal/providers/browser"
	"github.com/GriffinCanCode/AgentOS/backend/internal/providers/clipboard"
	"github.com/GriffinCanCode/AgentOS/backend/internal/providers/filesystem"
	httpProvider "github.com/GriffinCanCode/AgentOS/backend/internal/providers/http"
	httpclient "github.com/GriffinCanCode/AgentOS/backend/internal/providers/http/client"
	"github.com/GriffinCanCode/AgentOS/backend/internal/providers/ipc"
	mathProvider "github.com/GriffinCanCode/AgentOS/backend/internal/providers/math"
	"github.com/GriffinCanCode/AgentOS/backend/internal/providers/monitor"
	"github.com/GriffinCanCode/AgentOS/backend/internal/providers/permissions"
	"github.com/GriffinCanCode/AgentOS/backend/internal/providers/pipeline"
	scraperProvider "github.com/GriffinCanCode/AgentOS/backend/internal/providers/scraper"
	"github.com/GriffinCanCode/AgentOS/backend/internal/providers/settings"
	"github.com/GriffinCanCode/AgentOS/backend/internal/providers/storage"
	systemProvider "github.com/GriffinCanCode/AgentOS/backend/internal/providers/system"
	"github.com/GriffinCanCode/AgentOS/backend/internal/providers/terminal"
	"github.com/GriffinCanCode/AgentOS/backend/internal/providers/theme"
)

// Server wraps the HTTP server and dependencies
type Server struct {
	router         *gin.Engine
	appManager     *app.Manager
	registry       *service.Registry
	appRegistry    *registry.Manager
	sessionManager *session.Manager
	aiClient       *grpc.AIClient
	kernel         *kernel.KernelClient
	logger         *logging.Logger
	config         *config.Config
	metrics        *monitoring.Metrics
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

	logger.Info("Initializing AgentOS Server",
		zap.String("port", cfg.Server.Port),
		zap.String("kernel_addr", cfg.Kernel.Address),
		zap.String("ai_addr", cfg.AI.Address),
	)

	// Initialize metrics first (needed by other components)
	metrics := monitoring.NewMetrics()
	logger.Info("Performance monitoring initialized")

	// Initialize distributed tracing
	tracer := tracing.New("backend", logger.Logger)
	logger.Info("Distributed tracing initialized")

	// Initialize kernel client (optional)
	var kernelClient *kernel.KernelClient
	if cfg.Kernel.Enabled && cfg.Kernel.Address != "" {
		client, err := kernel.New(cfg.Kernel.Address)
		if err != nil {
			logger.Warn("Failed to connect to kernel", zap.Error(err))
		} else {
			kernelClient = client
			logger.Info("Connected to kernel", zap.String("addr", cfg.Kernel.Address))
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
	logger.Info("Connected to AI service", zap.String("addr", cfg.AI.Address))

	// Initialize app manager and service registry
	appManager := app.NewManager(kernelClient).WithMetrics(metrics)
	serviceRegistry := service.NewRegistry()

	// Register service providers
	logger.Info("Registering service providers...")
	registerProviders(serviceRegistry, kernelClient)

	// Initialize app registry with storage
	// Create a dummy PID for system storage operations
	var storagePID uint32 = 1
	if kernelClient != nil {
		// In production, create a dedicated process for storage
		ctx := context.Background()
		pid, _, err := kernelClient.CreateProcess(ctx, "storage-manager", 10, "PRIVILEGED", nil)
		if err == nil && pid != nil {
			storagePID = *pid
		}
	}
	// Use consistent storage path with kernel
	storagePath := os.Getenv("KERNEL_STORAGE_PATH")
	if storagePath == "" {
		storagePath = "/tmp/ai-os-storage"
	}
	appRegistry := registry.NewManager(kernelClient, storagePID, storagePath+"/system")

	// Seed prebuilt apps
	logger.Info("Loading prebuilt apps...")
	seeder := registry.NewSeeder(appRegistry, "../apps")
	if err := seeder.SeedApps(); err != nil {
		logger.Warn("Failed to seed prebuilt apps", zap.Error(err))
	}
	if err := seeder.SeedDefaultApps(); err != nil {
		logger.Warn("Failed to seed default apps", zap.Error(err))
	}

	// Initialize session manager
	sessionManager := session.NewManager(appManager, kernelClient, storagePID, storagePath+"/system")

	// Create router
	if !cfg.Logging.Development {
		gin.SetMode(gin.ReleaseMode)
	}
	router := gin.New()

	// Add middleware
	router.Use(gin.Recovery())
	router.Use(tracing.HTTPMiddleware(tracer)) // Add tracing middleware
	router.Use(monitoring.Middleware(metrics))
	router.Use(middleware.CORS(middleware.DefaultCORSConfig()))
	if cfg.RateLimit.Enabled {
		logger.Info("Rate limiting enabled",
			zap.Int("rps", cfg.RateLimit.RequestsPerSecond),
			zap.Int("burst", cfg.RateLimit.Burst),
		)
		router.Use(middleware.RateLimit(middleware.RateLimitConfig{
			RequestsPerSecond: cfg.RateLimit.RequestsPerSecond,
			Burst:             cfg.RateLimit.Burst,
		}))
	}

	// Create handler metrics wrapper
	handlerMetrics := http.NewHandlerMetrics(metrics)

	// Create handlers
	handlers := http.NewHandlers(appManager, serviceRegistry, appRegistry, sessionManager, aiClient, kernelClient, handlerMetrics, tracer)
	wsHandler := ws.NewHandler(appManager, aiClient)

	// Serve native app bundles (static files from apps/dist)
	router.Static("/native-apps", "../apps/dist")

	// Register routes
	router.GET("/", handlers.Root)
	router.GET("/health", handlers.Health)

	// App management
	router.GET("/apps", handlers.ListApps)
	router.POST("/apps/:id/focus", handlers.FocusApp)
	router.POST("/apps/:id/window", handlers.UpdateWindowState)
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

	// Kernel/Scheduler operations
	router.POST("/kernel/schedule-next", handlers.ScheduleNext)
	router.GET("/kernel/scheduler/stats", handlers.GetSchedulerStats)
	router.PUT("/kernel/scheduler/policy", handlers.SetSchedulingPolicy)

	// WebSocket
	router.GET("/stream", wsHandler.HandleConnection)

	// Create metrics aggregator
	metricsAggregator := http.NewMetricsAggregator(metrics, kernelClient)

	// Metrics endpoints
	router.GET("/metrics", func(c *gin.Context) {
		c.String(200, metrics.GetMetricsPrometheus())
	})
	router.GET("/metrics/json", metricsAggregator.GetAggregatedMetrics)
	router.GET("/metrics/dashboard", metricsAggregator.GetMetricsDashboard)
	router.GET("/metrics/kernel", metricsAggregator.ProxyKernelMetrics)
	router.GET("/metrics/ai", metricsAggregator.ProxyAIMetrics)

	logger.Info("Server initialized successfully")

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
		metrics:        metrics,
	}, nil
}

// Run starts the HTTP server
func (s *Server) Run() error {
	addr := s.config.Server.Host + ":" + s.config.Server.Port
	s.logger.Info("Starting HTTP server", zap.String("addr", addr))
	return s.router.Run(addr)
}

// Close gracefully shuts down the server
func (s *Server) Close() error {
	s.logger.Info("Shutting down server...")

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

func registerProviders(registry *service.Registry, kernel *kernel.KernelClient) {
	// Use consistent storage path with kernel
	storagePath := os.Getenv("KERNEL_STORAGE_PATH")
	if storagePath == "" {
		storagePath = "/tmp/ai-os-storage"
	}
	var storagePID uint32 = 1

	// Storage provider
	stProvider := storage.NewProvider(kernel, storagePID, storagePath)
	if err := registry.Register(stProvider); err != nil {
		fmt.Printf("Warning: Failed to register storage provider: %v\n", err)
	}

	// Auth provider
	auProvider := auth.NewProvider(kernel, storagePID, storagePath)
	if err := registry.Register(auProvider); err != nil {
		fmt.Printf("Warning: Failed to register auth provider: %v\n", err)
	}

	// System provider
	sysProvider := systemProvider.NewProvider()
	if err := registry.Register(sysProvider); err != nil {
		fmt.Printf("Warning: Failed to register system provider: %v\n", err)
	}

	// Filesystem provider
	fsProvider := filesystem.NewProvider(kernel, storagePID, storagePath)
	if err := registry.Register(fsProvider); err != nil {
		fmt.Printf("Warning: Failed to register filesystem provider: %v\n", err)
	}

	// HTTP provider (requires kernel for network syscalls)
	if kernel != nil {
		htProvider := httpProvider.NewProvider(kernel, storagePID)
		if err := registry.Register(htProvider); err != nil {
			fmt.Printf("Warning: Failed to register http provider: %v\n", err)
		}
	} else {
		fmt.Printf("Warning: HTTP provider requires kernel connection, skipping registration\n")
	}

	// Scraper provider
	scrProvider := scraperProvider.NewProvider()
	if err := registry.Register(scrProvider); err != nil {
		fmt.Printf("Warning: Failed to register scraper provider: %v\n", err)
	}

	// Math provider
	mthProvider := mathProvider.NewProvider()
	if err := registry.Register(mthProvider); err != nil {
		fmt.Printf("Warning: Failed to register math provider: %v\n", err)
	}

	// Terminal provider
	termProvider := terminal.NewProvider()
	if err := registry.Register(termProvider); err != nil {
		fmt.Printf("Warning: Failed to register terminal provider: %v\n", err)
	}

	// Clipboard provider (requires kernel)
	if kernel != nil {
		clipProvider := clipboard.NewProvider(kernel, storagePID)
		if err := registry.Register(clipProvider); err != nil {
			fmt.Printf("Warning: Failed to register clipboard provider: %v\n", err)
		}
	}

	// IPC provider (only if kernel is available)
	if kernel != nil {
		ipcProvider := ipc.NewProvider(kernel)
		if err := registry.Register(ipcProvider); err != nil {
			fmt.Printf("Warning: Failed to register IPC provider: %v\n", err)
		}

		// Pipeline provider (demonstrates IPC with multi-process ETL)
		pipelineProvider := pipeline.NewProvider(kernel)
		if err := registry.Register(pipelineProvider); err != nil {
			fmt.Printf("Warning: Failed to register pipeline provider: %v\n", err)
		}
	}

	// Settings provider
	settingsProvider := settings.NewProvider(kernel, storagePID, storagePath)
	if err := registry.Register(settingsProvider); err != nil {
		fmt.Printf("Warning: Failed to register settings provider: %v\n", err)
	}

	// Monitor provider
	monitorProvider := monitor.NewProvider(kernel, storagePID)
	if err := registry.Register(monitorProvider); err != nil {
		fmt.Printf("Warning: Failed to register monitor provider: %v\n", err)
	}

	// Permissions provider
	permissionsProvider := permissions.NewProvider(kernel, storagePID)
	if err := registry.Register(permissionsProvider); err != nil {
		fmt.Printf("Warning: Failed to register permissions provider: %v\n", err)
	}

	// Theme provider
	themeProvider := theme.NewProvider(kernel, storagePID, storagePath)
	if err := registry.Register(themeProvider); err != nil {
		fmt.Printf("Warning: Failed to register theme provider: %v\n", err)
	}

	// Browser provider (requires kernel and HTTP client)
	if kernel != nil {
		// Create HTTP client for browser provider
		httpClient := httpclient.NewClient()
		brProvider := browserProvider.New(httpClient, kernel, storagePID)
		if err := registry.Register(brProvider); err != nil {
			fmt.Printf("Warning: Failed to register browser provider: %v\n", err)
		}
	}
}
