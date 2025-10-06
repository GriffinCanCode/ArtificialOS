package system

import (
	"context"
	"testing"

	"github.com/GriffinCanCode/AgentOS/backend/internal/shared/types"
)

func TestSystemInfo(t *testing.T) {
	sys := NewProvider()
	ctx := context.Background()

	result, err := sys.Execute(ctx, "system.info", nil, nil)

	if err != nil || !result.Success {
		t.Fatalf("System info failed: %v", err)
	}

	if result.Data["go_version"] == nil {
		t.Error("Expected go_version in response")
	}
}

func TestSystemTime(t *testing.T) {
	sys := NewProvider()
	ctx := context.Background()

	result, err := sys.Execute(ctx, "system.time", nil, nil)

	if err != nil || !result.Success {
		t.Fatalf("System time failed: %v", err)
	}

	if result.Data["timestamp"] == nil {
		t.Error("Expected timestamp in response")
	}
}

func TestSystemLog(t *testing.T) {
	sys := NewProvider()
	ctx := context.Background()

	// Log a message
	result, err := sys.Execute(ctx, "system.log", map[string]interface{}{
		"message": "Test log message",
		"level":   "info",
	}, nil)

	if err != nil || !result.Success {
		t.Fatalf("Log failed: %v", err)
	}

	// Get logs
	result, err = sys.Execute(ctx, "system.getLogs", map[string]interface{}{
		"limit": 10.0,
	}, nil)

	if err != nil || !result.Success {
		t.Fatalf("Get logs failed: %v", err)
	}

	logs := result.Data["logs"].([]LogEntry)
	if len(logs) == 0 {
		t.Error("Expected at least one log entry")
	}

	if logs[0].Message != "Test log message" {
		t.Errorf("Expected 'Test log message', got %s", logs[0].Message)
	}
}

func TestSystemLogFilter(t *testing.T) {
	sys := NewProvider()
	ctx := context.Background()

	// Log messages at different levels
	sys.Execute(ctx, "system.log", map[string]interface{}{
		"message": "Info message",
		"level":   "info",
	}, nil)

	sys.Execute(ctx, "system.log", map[string]interface{}{
		"message": "Error message",
		"level":   "error",
	}, nil)

	// Get only error logs
	result, err := sys.Execute(ctx, "system.getLogs", map[string]interface{}{
		"limit": 10.0,
		"level": "error",
	}, nil)

	if err != nil || !result.Success {
		t.Fatalf("Get logs failed: %v", err)
	}

	logs := result.Data["logs"].([]LogEntry)
	if len(logs) == 0 {
		t.Error("Expected at least one error log")
	}

	for _, log := range logs {
		if log.Level != "error" {
			t.Errorf("Expected only error logs, got %s", log.Level)
		}
	}
}

func TestSystemPing(t *testing.T) {
	sys := NewProvider()
	ctx := context.Background()

	result, err := sys.Execute(ctx, "system.ping", nil, nil)

	if err != nil || !result.Success {
		t.Fatalf("Ping failed: %v", err)
	}

	if !result.Data["pong"].(bool) {
		t.Error("Expected pong: true in response")
	}
}

func TestSystemLogRotation(t *testing.T) {
	sys := NewProvider()
	ctx := context.Background()

	// Log more than buffer size to test circular buffer
	for i := 0; i < 1100; i++ {
		sys.Execute(ctx, "system.log", map[string]interface{}{
			"message": "Test message",
			"level":   "info",
		}, nil)
	}

	// Get all logs (should be limited to buffer size)
	result, _ := sys.Execute(ctx, "system.getLogs", map[string]interface{}{
		"limit": 2000.0,
	}, nil)

	logs := result.Data["logs"].([]LogEntry)
	if len(logs) > 1000 {
		t.Errorf("Expected max 1000 logs, got %d", len(logs))
	}
}

func TestSystemLogWithAppContext(t *testing.T) {
	sys := NewProvider()
	ctx := context.Background()
	appID := "test-app"
	appCtx := &types.Context{AppID: &appID}

	// Log with app context
	sys.Execute(ctx, "system.log", map[string]interface{}{
		"message": "App log message",
		"level":   "info",
	}, appCtx)

	// Verify app ID is in log
	result, _ := sys.Execute(ctx, "system.getLogs", map[string]interface{}{
		"limit": 10.0,
	}, nil)

	logs := result.Data["logs"].([]LogEntry)
	if logs[0].AppID != "test-app" {
		t.Errorf("Expected app_id 'test-app', got %s", logs[0].AppID)
	}
}
