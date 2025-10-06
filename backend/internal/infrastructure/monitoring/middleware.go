package monitoring

import (
	"strconv"
	"time"

	"github.com/gin-gonic/gin"
)

// Middleware creates a Gin middleware for metrics collection
func Middleware(metrics *Metrics) gin.HandlerFunc {
	return func(c *gin.Context) {
		start := time.Now()
		path := c.Request.URL.Path
		method := c.Request.Method

		// Get request size
		reqSize := c.Request.ContentLength
		if reqSize < 0 {
			reqSize = 0
		}

		// Process request
		c.Next()

		// Get response data
		duration := time.Since(start)
		status := strconv.Itoa(c.Writer.Status())
		respSize := int64(c.Writer.Size())

		// Record metrics
		metrics.RecordHTTPRequest(method, path, status, duration, reqSize, respSize)
	}
}

// Timer measures operation duration
type Timer struct {
	start   time.Time
	metrics *Metrics
	service string
	method  string
}

// NewTimer creates a new timer
func NewTimer(metrics *Metrics, service, method string) *Timer {
	return &Timer{
		start:   time.Now(),
		metrics: metrics,
		service: service,
		method:  method,
	}
}

// Stop stops the timer and records the duration
func (t *Timer) Stop(status string) {
	duration := time.Since(t.start)
	t.metrics.RecordServiceCall(t.service, t.method, status, duration)
}
