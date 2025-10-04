package middleware

import (
	"net/http"
	"net/http/httptest"
	"testing"
	"time"

	"github.com/gin-gonic/gin"
	"github.com/stretchr/testify/assert"
)

func setupTestRouter() *gin.Engine {
	gin.SetMode(gin.TestMode)
	return gin.New()
}

func TestCORS(t *testing.T) {
	router := setupTestRouter()

	cfg := DefaultCORSConfig()
	router.Use(CORS(cfg))

	router.GET("/test", func(c *gin.Context) {
		c.JSON(http.StatusOK, gin.H{"message": "success"})
	})

	tests := []struct {
		name           string
		method         string
		origin         string
		wantStatus     int
		checkHeaders   bool
		wantCORSHeader bool
	}{
		{
			name:           "simple GET request with origin",
			method:         "GET",
			origin:         "http://localhost:3000",
			wantStatus:     http.StatusOK,
			checkHeaders:   true,
			wantCORSHeader: true,
		},
		{
			name:           "preflight OPTIONS request",
			method:         "OPTIONS",
			origin:         "http://localhost:3000",
			wantStatus:     http.StatusNoContent,
			checkHeaders:   true,
			wantCORSHeader: true,
		},
		{
			name:           "no origin header",
			method:         "GET",
			origin:         "",
			wantStatus:     http.StatusOK,
			checkHeaders:   true,
			wantCORSHeader: false,
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			req := httptest.NewRequest(tt.method, "/test", nil)
			if tt.origin != "" {
				req.Header.Set("Origin", tt.origin)
			}

			w := httptest.NewRecorder()
			router.ServeHTTP(w, req)

			assert.Equal(t, tt.wantStatus, w.Code)

			if tt.checkHeaders {
				if tt.wantCORSHeader {
					// With wildcard origin, CORS should be set
					allowOrigin := w.Header().Get("Access-Control-Allow-Origin")
					assert.NotEmpty(t, allowOrigin, "CORS header should be set")
				} else {
					// Without origin header, CORS header should not be set
					assert.Empty(t, w.Header().Get("Access-Control-Allow-Origin"))
				}
			}
		})
	}
}

func TestCORSWithCustomConfig(t *testing.T) {
	// Test that custom CORS config can be created and applied
	cfg := CORSConfig{
		AllowOrigins:     []string{"https://example.com"},
		AllowMethods:     []string{"GET", "POST"},
		AllowHeaders:     []string{"Content-Type"},
		AllowCredentials: false,
		MaxAge:           1 * time.Hour,
	}

	// Verify config values
	assert.Equal(t, []string{"https://example.com"}, cfg.AllowOrigins)
	assert.Equal(t, []string{"GET", "POST"}, cfg.AllowMethods)
	assert.Equal(t, []string{"Content-Type"}, cfg.AllowHeaders)
	assert.False(t, cfg.AllowCredentials)
	assert.Equal(t, 1*time.Hour, cfg.MaxAge)

	// Test that middleware can be created with custom config
	router := setupTestRouter()
	router.Use(CORS(cfg))
	router.GET("/test", func(c *gin.Context) {
		c.JSON(http.StatusOK, gin.H{"message": "success"})
	})

	// Test that handler is reachable (CORS behavior tested by gin-contrib/cors)
	req := httptest.NewRequest("GET", "/test", nil)
	w := httptest.NewRecorder()
	router.ServeHTTP(w, req)

	// Handler should be accessible
	assert.Equal(t, http.StatusOK, w.Code)
}

func TestRateLimit(t *testing.T) {
	if testing.Short() {
		t.Skip("Skipping rate limit test in short mode")
	}

	router := setupTestRouter()

	cfg := RateLimitConfig{
		RequestsPerSecond: 2,
		Burst:             2,
	}
	router.Use(RateLimit(cfg))

	router.GET("/test", func(c *gin.Context) {
		c.JSON(http.StatusOK, gin.H{"message": "success"})
	})

	// First 2 requests should succeed (burst capacity)
	for i := 0; i < 2; i++ {
		req := httptest.NewRequest("GET", "/test", nil)
		req.RemoteAddr = "192.168.1.1:1234"
		w := httptest.NewRecorder()
		router.ServeHTTP(w, req)

		assert.Equal(t, http.StatusOK, w.Code, "Request %d should succeed", i+1)
	}

	// Third request should be rate limited
	req := httptest.NewRequest("GET", "/test", nil)
	req.RemoteAddr = "192.168.1.1:1234"
	w := httptest.NewRecorder()
	router.ServeHTTP(w, req)

	assert.Equal(t, http.StatusTooManyRequests, w.Code)
}

func TestRateLimitDifferentClients(t *testing.T) {
	if testing.Short() {
		t.Skip("Skipping rate limit test in short mode")
	}

	router := setupTestRouter()

	cfg := RateLimitConfig{
		RequestsPerSecond: 1,
		Burst:             1,
	}
	router.Use(RateLimit(cfg))

	router.GET("/test", func(c *gin.Context) {
		c.JSON(http.StatusOK, gin.H{"message": "success"})
	})

	// Client 1 - First request should succeed
	req1 := httptest.NewRequest("GET", "/test", nil)
	req1.RemoteAddr = "192.168.1.1:1234"
	w1 := httptest.NewRecorder()
	router.ServeHTTP(w1, req1)
	assert.Equal(t, http.StatusOK, w1.Code)

	// Client 2 - First request should also succeed (different IP)
	req2 := httptest.NewRequest("GET", "/test", nil)
	req2.RemoteAddr = "192.168.1.2:1234"
	w2 := httptest.NewRecorder()
	router.ServeHTTP(w2, req2)
	assert.Equal(t, http.StatusOK, w2.Code)

	// Client 1 - Second immediate request should be rate limited
	req3 := httptest.NewRequest("GET", "/test", nil)
	req3.RemoteAddr = "192.168.1.1:1234"
	w3 := httptest.NewRecorder()
	router.ServeHTTP(w3, req3)
	assert.Equal(t, http.StatusTooManyRequests, w3.Code)
}

func TestGlobalRateLimit(t *testing.T) {
	if testing.Short() {
		t.Skip("Skipping rate limit test in short mode")
	}

	router := setupTestRouter()

	cfg := RateLimitConfig{
		RequestsPerSecond: 2,
		Burst:             2,
	}
	router.Use(GlobalRateLimit(cfg))

	router.GET("/test", func(c *gin.Context) {
		c.JSON(http.StatusOK, gin.H{"message": "success"})
	})

	// First 2 requests from any client should succeed
	for i := 0; i < 2; i++ {
		req := httptest.NewRequest("GET", "/test", nil)
		req.RemoteAddr = "192.168.1." + string(rune(i)) + ":1234"
		w := httptest.NewRecorder()
		router.ServeHTTP(w, req)

		assert.Equal(t, http.StatusOK, w.Code, "Request %d should succeed", i+1)
	}

	// Third request from any client should be rate limited
	req := httptest.NewRequest("GET", "/test", nil)
	req.RemoteAddr = "192.168.1.3:1234"
	w := httptest.NewRecorder()
	router.ServeHTTP(w, req)

	assert.Equal(t, http.StatusTooManyRequests, w.Code)
}

func TestDefaultCORSConfig(t *testing.T) {
	cfg := DefaultCORSConfig()

	assert.NotEmpty(t, cfg.AllowOrigins)
	assert.Contains(t, cfg.AllowOrigins, "*")
	assert.NotEmpty(t, cfg.AllowMethods)
	assert.Contains(t, cfg.AllowMethods, "GET")
	assert.Contains(t, cfg.AllowMethods, "POST")
	assert.NotEmpty(t, cfg.AllowHeaders)
	assert.Equal(t, 12*time.Hour, cfg.MaxAge)
}

func TestDefaultRateLimitConfig(t *testing.T) {
	cfg := DefaultRateLimitConfig()

	assert.Equal(t, 100, cfg.RequestsPerSecond)
	assert.Equal(t, 200, cfg.Burst)
}

func BenchmarkCORS(b *testing.B) {
	router := setupTestRouter()
	router.Use(CORS(DefaultCORSConfig()))
	router.GET("/test", func(c *gin.Context) {
		c.Status(http.StatusOK)
	})

	req := httptest.NewRequest("GET", "/test", nil)
	req.Header.Set("Origin", "http://localhost:3000")

	b.ResetTimer()
	for i := 0; i < b.N; i++ {
		w := httptest.NewRecorder()
		router.ServeHTTP(w, req)
	}
}

func BenchmarkRateLimit(b *testing.B) {
	router := setupTestRouter()
	router.Use(RateLimit(DefaultRateLimitConfig()))
	router.GET("/test", func(c *gin.Context) {
		c.Status(http.StatusOK)
	})

	req := httptest.NewRequest("GET", "/test", nil)
	req.RemoteAddr = "192.168.1.1:1234"

	b.ResetTimer()
	for i := 0; i < b.N; i++ {
		w := httptest.NewRecorder()
		router.ServeHTTP(w, req)
	}
}
