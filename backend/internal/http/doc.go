// Package http provides HTTP handlers and routing for the AI-OS REST API.
//
// This package implements all HTTP endpoints using the Gin framework, including
// health checks, app management, service execution, and session management.
//
// Endpoints:
//   - Health: / and /health
//   - Apps: /apps, /apps/:id/focus, /apps/:id
//   - Services: /services, /services/discover, /services/execute
//   - Registry: /registry/apps, /registry/apps/:id
//   - Sessions: /sessions, /sessions/:id/restore
//   - AI: /generate-ui
//
// Features:
//   - JSON request/response handling
//   - Proper HTTP status codes
//   - Error response formatting
//   - Request validation
//
// Example Usage:
//
//	handlers := http.NewHandlers(appMgr, registry, appRegistry, sessionMgr, aiClient, kernel)
//	router.GET("/health", handlers.Health)
//	router.POST("/apps/:id/focus", handlers.FocusApp)
package http
