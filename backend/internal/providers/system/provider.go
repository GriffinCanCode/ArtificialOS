package system

import (
	"context"
	"fmt"
	"runtime"
	"sync"
	"time"

	"github.com/GriffinCanCode/AgentOS/backend/internal/shared/types"
)

// Provider implements system information and utilities
type Provider struct {
	startTime time.Time
	logs      *CircularLogBuffer
}

// CircularLogBuffer is a thread-safe circular buffer for log entries
type CircularLogBuffer struct {
	entries []*LogEntry
	head    int
	size    int
	maxSize int
	mu      sync.RWMutex
}

// LogEntry represents a system log entry
type LogEntry struct {
	Timestamp time.Time              `json:"timestamp"`
	Level     string                 `json:"level"`
	Message   string                 `json:"message"`
	AppID     string                 `json:"app_id,omitempty"`
	Context   map[string]interface{} `json:"context,omitempty"`
}

// NewProvider creates a system provider
func NewProvider() *Provider {
	return &Provider{
		startTime: time.Now(),
		logs:      NewCircularLogBuffer(1000),
	}
}

// NewCircularLogBuffer creates a new circular buffer for logs
func NewCircularLogBuffer(maxSize int) *CircularLogBuffer {
	return &CircularLogBuffer{
		entries: make([]*LogEntry, maxSize),
		head:    0,
		size:    0,
		maxSize: maxSize,
	}
}

// Add inserts a log entry into the circular buffer
func (cb *CircularLogBuffer) Add(entry *LogEntry) {
	cb.mu.Lock()
	defer cb.mu.Unlock()

	cb.entries[cb.head] = entry
	cb.head = (cb.head + 1) % cb.maxSize
	if cb.size < cb.maxSize {
		cb.size++
	}
}

// GetRecent retrieves the most recent N entries, optionally filtered by level
func (cb *CircularLogBuffer) GetRecent(limit int, levelFilter string) []LogEntry {
	cb.mu.RLock()
	defer cb.mu.RUnlock()

	if limit > cb.size {
		limit = cb.size
	}

	result := make([]LogEntry, 0, limit)

	// Start from the most recent entry (head - 1) and go backwards
	for i := 0; i < cb.size && len(result) < limit; i++ {
		idx := (cb.head - 1 - i + cb.maxSize) % cb.maxSize
		entry := cb.entries[idx]
		if entry != nil {
			if levelFilter == "" || entry.Level == levelFilter {
				result = append(result, *entry)
			}
		}
	}

	return result
}

// Definition returns service metadata
func (s *Provider) Definition() types.Service {
	return types.Service{
		ID:          "system",
		Name:        "System Service",
		Description: "System information and utilities",
		Category:    types.CategorySystem,
		Capabilities: []string{
			"info",
			"logging",
			"monitoring",
		},
		Tools: []types.Tool{
			{
				ID:          "system.info",
				Name:        "System Info",
				Description: "Get system information",
				Parameters:  []types.Parameter{},
				Returns:     "object",
			},
			{
				ID:          "system.time",
				Name:        "Current Time",
				Description: "Get current server time",
				Parameters:  []types.Parameter{},
				Returns:     "object",
			},
			{
				ID:          "system.log",
				Name:        "Log Message",
				Description: "Log a message to system logs",
				Parameters: []types.Parameter{
					{Name: "message", Type: "string", Description: "Log message", Required: true},
					{Name: "level", Type: "string", Description: "Log level (info/warn/error)", Required: false},
				},
				Returns: "boolean",
			},
			{
				ID:          "system.getLogs",
				Name:        "Get Logs",
				Description: "Retrieve recent system logs",
				Parameters: []types.Parameter{
					{Name: "limit", Type: "number", Description: "Number of logs to retrieve", Required: false},
					{Name: "level", Type: "string", Description: "Filter by log level", Required: false},
				},
				Returns: "array",
			},
			{
				ID:          "system.ping",
				Name:        "Ping",
				Description: "Test service availability",
				Parameters:  []types.Parameter{},
				Returns:     "object",
			},
		},
	}
}

// Execute runs a system operation
func (s *Provider) Execute(ctx context.Context, toolID string, params map[string]interface{}, appCtx *types.Context) (*types.Result, error) {
	switch toolID {
	case "system.info":
		return s.info()
	case "system.time":
		return s.currentTime()
	case "system.log":
		return s.log(params, appCtx)
	case "system.getLogs":
		return s.getLogs(params)
	case "system.ping":
		return s.ping()
	default:
		return failure(fmt.Sprintf("unknown tool: %s", toolID))
	}
}

func (s *Provider) info() (*types.Result, error) {
	var m runtime.MemStats
	runtime.ReadMemStats(&m)

	return success(map[string]interface{}{
		"go_version":     runtime.Version(),
		"os":             runtime.GOOS,
		"arch":           runtime.GOARCH,
		"cpus":           runtime.NumCPU(),
		"goroutines":     runtime.NumGoroutine(),
		"memory_alloc":   m.Alloc / 1024 / 1024,      // MB
		"memory_total":   m.TotalAlloc / 1024 / 1024, // MB
		"memory_sys":     m.Sys / 1024 / 1024,        // MB
		"uptime_seconds": time.Since(s.startTime).Seconds(),
	})
}

func (s *Provider) currentTime() (*types.Result, error) {
	now := time.Now()
	return success(map[string]interface{}{
		"timestamp": now.Unix(),
		"iso":       now.Format(time.RFC3339),
		"unix_ms":   now.UnixMilli(),
	})
}

func (s *Provider) log(params map[string]interface{}, ctx *types.Context) (*types.Result, error) {
	message, ok := params["message"].(string)
	if !ok || message == "" {
		return failure("message required")
	}

	level := "info"
	if l, ok := params["level"].(string); ok && l != "" {
		level = l
	}

	entry := &LogEntry{
		Timestamp: time.Now(),
		Level:     level,
		Message:   message,
	}

	if ctx != nil && ctx.AppID != nil {
		entry.AppID = *ctx.AppID
	}

	// Add to circular buffer (efficient, no reallocation)
	s.logs.Add(entry)

	return success(map[string]interface{}{"logged": true})
}

func (s *Provider) getLogs(params map[string]interface{}) (*types.Result, error) {
	limit := 100
	if l, ok := params["limit"].(float64); ok && l > 0 {
		limit = int(l)
	}

	levelFilter := ""
	if l, ok := params["level"].(string); ok {
		levelFilter = l
	}

	// Use circular buffer's efficient retrieval
	logs := s.logs.GetRecent(limit, levelFilter)

	return success(map[string]interface{}{
		"logs":  logs,
		"count": len(logs),
	})
}

func (s *Provider) ping() (*types.Result, error) {
	return success(map[string]interface{}{
		"pong":      true,
		"timestamp": time.Now().Unix(),
	})
}

func success(data map[string]interface{}) (*types.Result, error) {
	return &types.Result{Success: true, Data: data}, nil
}

func failure(message string) (*types.Result, error) {
	msg := message
	return &types.Result{Success: false, Error: &msg}, nil
}
