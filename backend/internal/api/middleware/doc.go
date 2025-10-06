// Package middleware provides production-ready HTTP middleware for the AI-OS backend.
//
// Middleware stack includes:
//   - CORS: Cross-origin resource sharing with configurable origins
//   - RateLimit: Per-IP token bucket rate limiting
//   - Recovery: Panic recovery with graceful error responses
//   - Logging: Request/response logging (via Gin)
//
// CORS Configuration:
//   - AllowOrigins: Permitted origin domains
//   - AllowMethods: HTTP methods (GET, POST, etc.)
//   - AllowHeaders: Request headers
//   - AllowCredentials: Cookie/auth support
//   - MaxAge: Preflight cache duration
//
// Rate Limiting:
//   - Per-IP tracking with automatic cleanup
//   - Token bucket algorithm
//   - Configurable RPS and burst capacity
//   - Global rate limiting option
//
// Example Usage:
//
//	router.Use(middleware.CORS(middleware.DefaultCORSConfig()))
//	router.Use(middleware.RateLimit(middleware.DefaultRateLimitConfig()))
package middleware
