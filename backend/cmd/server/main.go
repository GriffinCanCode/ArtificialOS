package main

import (
	"flag"
	"log"
	"os"
	"os/signal"
	"syscall"

	"github.com/GriffinCanCode/AgentOS/backend/internal/server"
)

func main() {
	// Parse flags
	port := flag.String("port", "8000", "Server port")
	kernelAddr := flag.String("kernel", "localhost:50051", "Kernel gRPC address")
	aiAddr := flag.String("ai", "localhost:50052", "AI service gRPC address")
	flag.Parse()

	log.Println("=" + string(make([]byte, 60)) + "=")
	log.Println("🤖 AI-Powered OS - Go Service")
	log.Println("=" + string(make([]byte, 60)) + "=")

	// Create server
	srv, err := server.NewServer(server.Config{
		Port:          *port,
		KernelAddr:    *kernelAddr,
		AIServiceAddr: *aiAddr,
	})
	if err != nil {
		log.Fatalf("Failed to create server: %v", err)
	}

	// Handle graceful shutdown
	sigChan := make(chan os.Signal, 1)
	signal.Notify(sigChan, os.Interrupt, syscall.SIGTERM)

	// Start server in goroutine
	errChan := make(chan error, 1)
	go func() {
		if err := srv.Run(*port); err != nil {
			errChan <- err
		}
	}()

	// Wait for shutdown signal or error
	select {
	case <-sigChan:
		log.Println("\n🛑 Shutting down gracefully...")
		if err := srv.Close(); err != nil {
			log.Printf("Error during shutdown: %v", err)
		}
	case err := <-errChan:
		log.Fatalf("Server error: %v", err)
	}
}
