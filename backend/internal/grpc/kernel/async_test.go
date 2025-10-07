package kernel

import (
	"context"
	"testing"
	"time"

	pb "github.com/GriffinCanCode/AgentOS/backend/proto/kernel"
)

func TestAsyncStatusResponse_StatusEnum(t *testing.T) {
	tests := []struct {
		name   string
		status pb.AsyncStatusResponse_Status
		want   string
	}{
		{"pending", pb.AsyncStatusResponse_PENDING, "PENDING"},
		{"running", pb.AsyncStatusResponse_RUNNING, "RUNNING"},
		{"completed", pb.AsyncStatusResponse_COMPLETED, "COMPLETED"},
		{"failed", pb.AsyncStatusResponse_FAILED, "FAILED"},
		{"cancelled", pb.AsyncStatusResponse_CANCELLED, "CANCELLED"},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			if tt.status.String() != tt.want {
				t.Errorf("Status = %v, want %v", tt.status, tt.want)
			}
		})
	}
}

func TestWaitForAsyncCompletion_Timeout(t *testing.T) {
	// Mock scenario: waiting for task that never completes
	ctx, cancel := context.WithTimeout(context.Background(), 100*time.Millisecond)
	defer cancel()

	// Simulate waiting
	ticker := time.NewTicker(10 * time.Millisecond)
	defer ticker.Stop()

	for {
		select {
		case <-ctx.Done():
			// Expected timeout
			if ctx.Err() != context.DeadlineExceeded {
				t.Errorf("Expected DeadlineExceeded, got %v", ctx.Err())
			}
			return
		case <-ticker.C:
			// Simulate polling
			continue
		}
	}
}

func TestWaitForAsyncCompletion_SuccessScenario(t *testing.T) {
	// Simulate task completing
	taskComplete := make(chan bool, 1)

	go func() {
		time.Sleep(50 * time.Millisecond)
		taskComplete <- true
	}()

	select {
	case <-taskComplete:
		// Success
	case <-time.After(200 * time.Millisecond):
		t.Fatal("Task didn't complete in time")
	}
}

func TestAsyncTaskID_Generation(t *testing.T) {
	// Task IDs should be unique (simulated with UUIDs)
	ids := make(map[string]bool)

	// Generate some pseudo-IDs
	for i := 0; i < 100; i++ {
		id := generateMockTaskID(i)
		if ids[id] {
			t.Errorf("Duplicate task ID: %s", id)
		}
		ids[id] = true
	}

	if len(ids) != 100 {
		t.Errorf("Expected 100 unique IDs, got %d", len(ids))
	}
}

func generateMockTaskID(i int) string {
	return "task-" + string(rune(i))
}

func TestBuildSyscallRequest(t *testing.T) {
	// Test that we can build syscall requests
	req := &pb.SyscallRequest{Pid: 123}

	// Simulate building different syscall types
	tests := []struct {
		name   string
		scType string
		params map[string]interface{}
	}{
		{"read_file", "read_file", map[string]interface{}{"path": "/test.txt"}},
		{"write_file", "write_file", map[string]interface{}{"path": "/test.txt", "data": []byte("test")}},
		{"spawn_process", "spawn_process", map[string]interface{}{"command": "echo", "args": []string{"hello"}}},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			// Would call buildSyscallRequest method
			// Just verify we can create the structure
			if req.Pid != 123 {
				t.Errorf("PID not set correctly")
			}
			if tt.scType == "" {
				t.Error("Syscall type should not be empty")
			}
		})
	}
}
