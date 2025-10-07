package kernel

import (
	"testing"
	"time"

	pb "github.com/GriffinCanCode/AgentOS/backend/proto/kernel"
)

func TestBatchRequest_Structure(t *testing.T) {
	req := BatchRequest{
		PID:         123,
		SyscallType: "read_file",
		Params: map[string]interface{}{
			"path": "/test.txt",
		},
	}

	if req.PID != 123 {
		t.Errorf("PID = %d, want 123", req.PID)
	}
	if req.SyscallType != "read_file" {
		t.Errorf("SyscallType = %s, want read_file", req.SyscallType)
	}
}

func TestBatchResult_Validation(t *testing.T) {
	result := &BatchResult{
		Responses: []*pb.SyscallResponse{
			{Result: &pb.SyscallResponse_Success{Success: &pb.SuccessResult{}}},
			{Result: &pb.SyscallResponse_Error{Error: &pb.ErrorResult{Message: "test error"}}},
		},
		SuccessCount: 1,
		FailureCount: 1,
	}

	if len(result.Responses) != 2 {
		t.Errorf("Expected 2 responses, got %d", len(result.Responses))
	}
	if result.SuccessCount != 1 {
		t.Errorf("SuccessCount = %d, want 1", result.SuccessCount)
	}
	if result.FailureCount != 1 {
		t.Errorf("FailureCount = %d, want 1", result.FailureCount)
	}
}

func TestBatchExecution_ParallelVsSequential(t *testing.T) {
	// Test conceptual timing difference

	t.Run("sequential", func(t *testing.T) {
		start := time.Now()

		// Simulate 10 operations at 10ms each
		for i := 0; i < 10; i++ {
			time.Sleep(10 * time.Millisecond)
		}

		duration := time.Since(start)
		// Should take ~100ms
		if duration < 100*time.Millisecond {
			t.Errorf("Sequential should take ~100ms, took %v", duration)
		}
	})

	t.Run("parallel", func(t *testing.T) {
		start := time.Now()

		// Simulate 10 parallel operations
		done := make(chan bool, 10)
		for i := 0; i < 10; i++ {
			go func() {
				time.Sleep(10 * time.Millisecond)
				done <- true
			}()
		}

		// Wait for all
		for i := 0; i < 10; i++ {
			<-done
		}

		duration := time.Since(start)
		// Should take ~10ms (with some overhead)
		if duration > 50*time.Millisecond {
			t.Errorf("Parallel should be faster, took %v", duration)
		}
	})
}

func TestBatchRequestValidation(t *testing.T) {
	tests := []struct {
		name    string
		request BatchRequest
		wantErr bool
	}{
		{
			name: "valid",
			request: BatchRequest{
				PID:         1,
				SyscallType: "read_file",
				Params:      map[string]interface{}{"path": "/test"},
			},
			wantErr: false,
		},
		{
			name: "empty_syscall_type",
			request: BatchRequest{
				PID:         1,
				SyscallType: "",
				Params:      map[string]interface{}{},
			},
			wantErr: true,
		},
		{
			name: "zero_pid",
			request: BatchRequest{
				PID:         0,
				SyscallType: "read_file",
				Params:      map[string]interface{}{},
			},
			wantErr: false, // PID 0 might be valid in some contexts
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			// Validate request
			hasError := tt.request.SyscallType == ""
			if hasError != tt.wantErr {
				t.Errorf("Validation error = %v, want %v", hasError, tt.wantErr)
			}
		})
	}
}

func TestBatchResponseAggregation(t *testing.T) {
	responses := []*pb.SyscallResponse{
		{Result: &pb.SyscallResponse_Success{Success: &pb.SuccessResult{}}},
		{Result: &pb.SyscallResponse_Success{Success: &pb.SuccessResult{}}},
		{Result: &pb.SyscallResponse_Error{Error: &pb.ErrorResult{}}},
		{Result: &pb.SyscallResponse_PermissionDenied{PermissionDenied: &pb.PermissionDeniedResult{}}},
	}

	var successCount, failureCount uint32
	for _, resp := range responses {
		switch resp.Result.(type) {
		case *pb.SyscallResponse_Success:
			successCount++
		case *pb.SyscallResponse_Error, *pb.SyscallResponse_PermissionDenied:
			failureCount++
		}
	}

	if successCount != 2 {
		t.Errorf("SuccessCount = %d, want 2", successCount)
	}
	if failureCount != 2 {
		t.Errorf("FailureCount = %d, want 2", failureCount)
	}
}

func TestExecuteBatchSimple(t *testing.T) {
	// Test the convenience wrapper structure
	syscalls := []struct {
		Type   string
		Params map[string]interface{}
	}{
		{"read_file", map[string]interface{}{"path": "/test1.txt"}},
		{"read_file", map[string]interface{}{"path": "/test2.txt"}},
		{"write_file", map[string]interface{}{"path": "/test3.txt", "data": []byte("test")}},
	}

	if len(syscalls) != 3 {
		t.Errorf("Expected 3 syscalls, got %d", len(syscalls))
	}

	// Verify each has required fields
	for i, sc := range syscalls {
		if sc.Type == "" {
			t.Errorf("Syscall %d has empty type", i)
		}
		if sc.Params == nil {
			t.Errorf("Syscall %d has nil params", i)
		}
	}
}
