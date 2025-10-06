# Middleware

Production-ready HTTP middleware for the Go backend service.

## Available Middleware

### CORS (`cors.go`)

Cross-Origin Resource Sharing middleware using `gin-contrib/cors`.

**Features:**
- Configurable allowed origins
- Configurable methods and headers
- Credentials support
- Max age caching

**Usage:**
```go
import "github.com/GriffinCanCode/AgentOS/backend/internal/middleware"

// Use default configuration
router.Use(middleware.CORS(middleware.DefaultCORSConfig()))

// Custom configuration
router.Use(middleware.CORS(middleware.CORSConfig{
    AllowOrigins:     []string{"https://example.com"},
    AllowMethods:     []string{"GET", "POST"},
    AllowHeaders:     []string{"Content-Type"},
    AllowCredentials: true,
    MaxAge:           12 * time.Hour,
}))
```

### Rate Limiting (`rate.go`)

Per-IP and global rate limiting using token bucket algorithm.

**Features:**
- Per-IP rate limiting with independent limiters
- Global rate limiting for all requests
- Configurable requests per second and burst
- Automatic cleanup of stale clients

**Usage:**
```go
// Per-IP rate limiting (recommended)
router.Use(middleware.RateLimit(middleware.RateLimitConfig{
    RequestsPerSecond: 100,
    Burst:             200,
}))

// Global rate limiting
router.Use(middleware.GlobalRateLimit(middleware.RateLimitConfig{
    RequestsPerSecond: 1000,
    Burst:             2000,
}))
```

## Design Principles

1. **Configurable** - All middleware accepts configuration structs
2. **Extensible** - Easy to add new middleware
3. **Type-Safe** - Strong typing for all configurations
4. **Production-Ready** - Using battle-tested libraries
5. **Minimal** - Each file has one clear purpose

## Adding New Middleware

To add new middleware:

1. Create a new file in this directory (e.g., `auth.go`)
2. Define a configuration struct
3. Define a default configuration function
4. Implement the middleware function that returns `gin.HandlerFunc`
5. Export the middleware for use in server setup

Example:
```go
// auth.go
package middleware

import "github.com/gin-gonic/gin"

type AuthConfig struct {
    Secret string
}

func DefaultAuthConfig() AuthConfig {
    return AuthConfig{Secret: "default-secret"}
}

func Auth(cfg AuthConfig) gin.HandlerFunc {
    return func(c *gin.Context) {
        // Authentication logic
        c.Next()
    }
}
```

