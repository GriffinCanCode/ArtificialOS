// Package server provides HTTP server setup and initialization for AI-OS.
//
// This package orchestrates all components:
//   - HTTP routing with Gin framework
//   - Middleware stack (CORS, rate limiting, recovery)
//   - gRPC client connections (kernel, AI)
//   - Service provider registration
//   - Manager initialization (app, session, registry)
//
// Server Lifecycle:
//  1. Load configuration from environment/flags
//  2. Initialize logger (production or development)
//  3. Connect to kernel and AI services
//  4. Create storage process for persistence
//  5. Register service providers
//  6. Setup HTTP routes and middleware
//  7. Start HTTP server
//  8. Graceful shutdown on signal
//
// Features:
//   - Configuration-driven setup
//   - Graceful shutdown handling
//   - Resource cleanup on exit
//   - Health check endpoints
//
// Example Usage:
//
//	cfg := config.LoadOrDefault()
//	srv, err := server.NewServer(cfg)
//	if err := srv.Run(); err != nil {
//	    log.Fatal(err)
//	}
package server
