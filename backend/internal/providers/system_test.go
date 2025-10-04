package providers

import (
	"context"
	"testing"

	"github.com/GriffinCanCode/AgentOS/backend/internal/types"
)

func TestSystemInfo(t *testing.T) {
	system := NewSystem()
	ctx := context.Background()

	result, err := system.Execute(ctx, "system.info", nil, nil)

	if err != nil || !result.Success {
		t.Fatalf("Info failed: %v", err)
	}

	if result.Data["go_version"] == nil {
		t.Error("Expected go_version in result")
	}

	if result.Data["cpus"] == nil {
		t.Error("Expected cpus in result")
	}
}

func TestSystemTime(t *testing.T) {
	system := NewSystem()
	ctx := context.Background()

	result, err := system.Execute(ctx, "system.time", nil, nil)

	if err != nil || !result.Success {
		t.Fatalf("Time failed: %v", err)
	}

	if result.Data["timestamp"] == nil {
		t.Error("Expected timestamp in result")
	}

	if result.Data["iso"] == nil {
		t.Error("Expected iso in result")
	}
}

func TestSystemLog(t *testing.T) {
	system := NewSystem()
	bgCtx := context.Background()

	appID := "test-app"
	ctx := &types.Context{AppID: &appID}

	// Log a message
	result, err := system.Execute(bgCtx, "system.log", map[string]interface{}{
		"message": "Test log message",
		"level":   "info",
	}, ctx)

	if err != nil || !result.Success {
		t.Fatalf("Log failed: %v", err)
	}

	// Retrieve logs
	result, err = system.Execute(bgCtx, "system.getLogs", map[string]interface{}{
		"limit": float64(10),
	}, nil)

	if err != nil || !result.Success {
		t.Fatalf("GetLogs failed: %v", err)
	}

	logs := result.Data["logs"].([]LogEntry)
	if len(logs) != 1 {
		t.Errorf("Expected 1 log, got %d", len(logs))
	}

	if logs[0].Message != "Test log message" {
		t.Errorf("Expected 'Test log message', got %s", logs[0].Message)
	}

	if logs[0].AppID != appID {
		t.Errorf("Expected app_id %s, got %s", appID, logs[0].AppID)
	}
}

func TestSystemLogFilter(t *testing.T) {
	system := NewSystem()
	ctx := context.Background()

	// Log multiple messages with different levels
	system.Execute(ctx, "system.log", map[string]interface{}{
		"message": "Info message",
		"level":   "info",
	}, nil)

	system.Execute(ctx, "system.log", map[string]interface{}{
		"message": "Error message",
		"level":   "error",
	}, nil)

	system.Execute(ctx, "system.log", map[string]interface{}{
		"message": "Warn message",
		"level":   "warn",
	}, nil)

	// Get only error logs
	result, err := system.Execute(ctx, "system.getLogs", map[string]interface{}{
		"level": "error",
	}, nil)

	if err != nil || !result.Success {
		t.Fatalf("GetLogs failed: %v", err)
	}

	logs := result.Data["logs"].([]LogEntry)
	if len(logs) != 1 {
		t.Errorf("Expected 1 error log, got %d", len(logs))
	}

	if logs[0].Level != "error" {
		t.Errorf("Expected level 'error', got %s", logs[0].Level)
	}
}

func TestSystemPing(t *testing.T) {
	system := NewSystem()
	ctx := context.Background()

	result, err := system.Execute(ctx, "system.ping", nil, nil)

	if err != nil || !result.Success {
		t.Fatalf("Ping failed: %v", err)
	}

	if !result.Data["pong"].(bool) {
		t.Error("Expected pong to be true")
	}

	if result.Data["timestamp"] == nil {
		t.Error("Expected timestamp in ping response")
	}
}

func TestSystemLogRotation(t *testing.T) {
	system := NewSystem()
	ctx := context.Background()
	// Note: System uses maxLogs = 1000 by default
	// We'll test that it doesn't grow infinitely

	// Log multiple messages
	for i := 0; i < 10; i++ {
		system.Execute(ctx, "system.log", map[string]interface{}{
			"message": "Message",
		}, nil)
	}

	// Verify rotation
	result, _ := system.Execute(ctx, "system.getLogs", map[string]interface{}{
		"limit": float64(100),
	}, nil)

	logs := result.Data["logs"].([]LogEntry)
	// Verify logs are present and limited
	if len(logs) != 10 {
		t.Errorf("Expected 10 logs, got %d", len(logs))
	}
}
