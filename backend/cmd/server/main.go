package main

import (
	"flag"
	"os"
	"os/signal"
	"syscall"

	"github.com/GriffinCanCode/AgentOS/backend/internal/config"
	"github.com/GriffinCanCode/AgentOS/backend/internal/logging"
	"github.com/GriffinCanCode/AgentOS/backend/internal/server"
	"go.uber.org/zap"
)

func main() {
	// Parse flags (override environment variables)
	port := flag.String("port", "", "Server port")
	kernelAddr := flag.String("kernel", "", "Kernel gRPC address")
	aiAddr := flag.String("ai", "", "AI service gRPC address")
	dev := flag.Bool("dev", false, "Development mode")
	flag.Parse()

	// Load configuration
	cfg := config.LoadOrDefault()

	// Override with flags if provided
	if *port != "" {
		cfg.Server.Port = *port
	}
	if *kernelAddr != "" {
		cfg.Kernel.Address = *kernelAddr
	}
	if *aiAddr != "" {
		cfg.AI.Address = *aiAddr
	}
	if *dev {
		cfg.Logging.Development = true
	}

	// Create logger for main
	var logger *logging.Logger
	if cfg.Logging.Development {
		logger = logging.NewDevelopment()
	} else {
		logger = logging.NewDefault()
	}
	defer logger.Sync()

	logger.Info("=" + string(make([]byte, 60)) + "=")
	logger.Info("🤖 AI-Powered OS - Go Service")
	logger.Info("=" + string(make([]byte, 60)) + "=")

	// Create server
	srv, err := server.NewServer(cfg)
	if err != nil {
		logger.Fatal("Failed to create server", zap.Error(err))
	}

	// Handle graceful shutdown
	sigChan := make(chan os.Signal, 1)
	signal.Notify(sigChan, os.Interrupt, syscall.SIGTERM)

	// Start server in goroutine
	errChan := make(chan error, 1)
	go func() {
		if err := srv.Run(); err != nil {
			errChan <- err
		}
	}()

	// Wait for shutdown signal or error
	select {
	case <-sigChan:
		logger.Info("Received shutdown signal")
		if err := srv.Close(); err != nil {
			logger.Error("Error during shutdown", zap.Error(err))
			os.Exit(1)
		}
		logger.Info("✅ Shutdown complete")
	case err := <-errChan:
		logger.Fatal("Server error", zap.Error(err))
	}
}
