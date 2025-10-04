package middleware

import (
	"net/http"
	"sync"

	"github.com/gin-gonic/gin"
	"golang.org/x/time/rate"
)

// RateLimitConfig defines rate limiting configuration.
type RateLimitConfig struct {
	RequestsPerSecond int
	Burst             int
}

// DefaultRateLimitConfig returns production-ready rate limit configuration.
func DefaultRateLimitConfig() RateLimitConfig {
	return RateLimitConfig{
		RequestsPerSecond: 100,
		Burst:             200,
	}
}

// RateLimit creates a per-IP rate limiting middleware.
func RateLimit(cfg RateLimitConfig) gin.HandlerFunc {
	type client struct {
		limiter  *rate.Limiter
		lastSeen int64
	}

	var (
		mu      sync.RWMutex
		clients = make(map[string]*client)
	)

	return func(c *gin.Context) {
		ip := c.ClientIP()

		mu.Lock()
		if _, exists := clients[ip]; !exists {
			clients[ip] = &client{
				limiter: rate.NewLimiter(rate.Limit(cfg.RequestsPerSecond), cfg.Burst),
			}
		}
		limiter := clients[ip].limiter
		mu.Unlock()

		if !limiter.Allow() {
			c.JSON(http.StatusTooManyRequests, gin.H{
				"error": "rate limit exceeded",
			})
			c.Abort()
			return
		}

		c.Next()
	}
}

// GlobalRateLimit creates a global rate limiting middleware.
func GlobalRateLimit(cfg RateLimitConfig) gin.HandlerFunc {
	limiter := rate.NewLimiter(rate.Limit(cfg.RequestsPerSecond), cfg.Burst)

	return func(c *gin.Context) {
		if !limiter.Allow() {
			c.JSON(http.StatusTooManyRequests, gin.H{
				"error": "rate limit exceeded",
			})
			c.Abort()
			return
		}
		c.Next()
	}
}
