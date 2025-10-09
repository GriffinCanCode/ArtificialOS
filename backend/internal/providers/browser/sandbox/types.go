package sandbox

import (
	"context"
	"time"
)

// Config defines sandbox configuration
type Config struct {
	MaxMemoryMB   int64         // Maximum heap size in MB
	Timeout       time.Duration // Execution timeout
	EnableConsole bool          // Allow console.log/warn/error
	EnableDOM     bool          // Enable DOM API
	EnableFetch   bool          // Enable fetch API (proxied)
}

// Result holds execution result
type Result struct {
	Value      interface{}   // Return value
	Console    []LogEntry    // Console output
	DOMChanges []DOMChange   // DOM modifications
	Duration   time.Duration // Execution time
	Error      error         // Execution error
}

// LogEntry represents console output
type LogEntry struct {
	Level   string    // log, warn, error
	Message string    // Log message
	Time    time.Time // Timestamp
}

// DOMChange represents a DOM modification
type DOMChange struct {
	Type     string      // set_attribute, set_text, etc.
	Selector string      // CSS selector
	Property string      // Property name
	Value    interface{} // New value
}

// Bridge defines host communication interface
type Bridge interface {
	Call(ctx context.Context, method string, args ...interface{}) (interface{}, error)
	Emit(ctx context.Context, event string, data interface{}) error
}

// Sandbox defines the JavaScript execution interface
type Sandbox interface {
	Execute(ctx context.Context, script string, dom *DOM) (*Result, error)
	Reset() error
	Close() error
}

// Default configuration
func DefaultConfig() Config {
	return Config{
		MaxMemoryMB:   50,
		Timeout:       5 * time.Second,
		EnableConsole: true,
		EnableDOM:     true,
		EnableFetch:   false,
	}
}
