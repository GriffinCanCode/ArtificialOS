package providers

import (
	"fmt"
	"runtime"
	"sync"
	"time"

	"github.com/GriffinCanCode/AgentOS/backend/internal/types"
)

// System provides system information and utilities
type System struct {
	startTime time.Time
	logs      []LogEntry
	logsMu    sync.RWMutex
	maxLogs   int
}

// LogEntry represents a system log entry
type LogEntry struct {
	Timestamp time.Time              `json:"timestamp"`
	Level     string                 `json:"level"`
	Message   string                 `json:"message"`
	AppID     string                 `json:"app_id,omitempty"`
	Context   map[string]interface{} `json:"context,omitempty"`
}

// NewSystem creates a system provider
func NewSystem() *System {
	return &System{
		startTime: time.Now(),
		logs:      make([]LogEntry, 0),
		maxLogs:   1000,
	}
}

// Definition returns service metadata
func (s *System) Definition() types.Service {
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
func (s *System) Execute(toolID string, params map[string]interface{}, ctx *types.Context) (*types.Result, error) {
	switch toolID {
	case "system.info":
		return s.info()
	case "system.time":
		return s.currentTime()
	case "system.log":
		return s.log(params, ctx)
	case "system.getLogs":
		return s.getLogs(params)
	case "system.ping":
		return s.ping()
	default:
		return failure(fmt.Sprintf("unknown tool: %s", toolID))
	}
}

func (s *System) info() (*types.Result, error) {
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

func (s *System) currentTime() (*types.Result, error) {
	now := time.Now()
	return success(map[string]interface{}{
		"timestamp": now.Unix(),
		"iso":       now.Format(time.RFC3339),
		"unix_ms":   now.UnixMilli(),
	})
}

func (s *System) log(params map[string]interface{}, ctx *types.Context) (*types.Result, error) {
	message, ok := params["message"].(string)
	if !ok || message == "" {
		return failure("message required")
	}

	level := "info"
	if l, ok := params["level"].(string); ok && l != "" {
		level = l
	}

	entry := LogEntry{
		Timestamp: time.Now(),
		Level:     level,
		Message:   message,
	}

	if ctx != nil && ctx.AppID != nil {
		entry.AppID = *ctx.AppID
	}

	// Add to logs (with rotation)
	s.logsMu.Lock()
	s.logs = append(s.logs, entry)
	if len(s.logs) > s.maxLogs {
		s.logs = s.logs[len(s.logs)-s.maxLogs:]
	}
	s.logsMu.Unlock()

	return success(map[string]interface{}{"logged": true})
}

func (s *System) getLogs(params map[string]interface{}) (*types.Result, error) {
	limit := 100
	if l, ok := params["limit"].(float64); ok && l > 0 {
		limit = int(l)
	}

	levelFilter := ""
	if l, ok := params["level"].(string); ok {
		levelFilter = l
	}

	s.logsMu.RLock()
	defer s.logsMu.RUnlock()

	// Filter and limit
	var filtered []LogEntry
	for i := len(s.logs) - 1; i >= 0 && len(filtered) < limit; i-- {
		entry := s.logs[i]
		if levelFilter == "" || entry.Level == levelFilter {
			filtered = append(filtered, entry)
		}
	}

	return success(map[string]interface{}{
		"logs":  filtered,
		"count": len(filtered),
	})
}

func (s *System) ping() (*types.Result, error) {
	return success(map[string]interface{}{
		"pong":      true,
		"timestamp": time.Now().Unix(),
	})
}
