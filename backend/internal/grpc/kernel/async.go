package kernel

import (
	"context"
	"fmt"
	"time"

	pb "github.com/GriffinCanCode/AgentOS/backend/proto/kernel"
)

// ExecuteSyscallAsync executes a syscall asynchronously and returns a task ID
func (k *KernelClient) ExecuteSyscallAsync(ctx context.Context, pid uint32, syscallType string, params map[string]interface{}) (string, error) {
	ctx, cancel := context.WithTimeout(ctx, 5*time.Second)
	defer cancel()

	req := &pb.SyscallRequest{Pid: pid}
	k.buildSyscallRequest(req, syscallType, params)

	resp, err := k.client.ExecuteSyscallAsync(ctx, req)
	if err != nil {
		return "", fmt.Errorf("async execution failed: %w", err)
	}

	if !resp.Accepted {
		return "", fmt.Errorf("task not accepted: %s", resp.Error)
	}

	return resp.TaskId, nil
}

// GetAsyncStatus retrieves the status of an async task
func (k *KernelClient) GetAsyncStatus(ctx context.Context, taskID string) (*pb.AsyncStatusResponse, error) {
	ctx, cancel := context.WithTimeout(ctx, 5*time.Second)
	defer cancel()

	resp, err := k.client.GetAsyncStatus(ctx, &pb.AsyncStatusRequest{
		TaskId: taskID,
	})
	if err != nil {
		return nil, fmt.Errorf("failed to get status: %w", err)
	}

	return resp, nil
}

// CancelAsync cancels an async task
func (k *KernelClient) CancelAsync(ctx context.Context, taskID string) error {
	ctx, cancel := context.WithTimeout(ctx, 5*time.Second)
	defer cancel()

	resp, err := k.client.CancelAsync(ctx, &pb.AsyncCancelRequest{
		TaskId: taskID,
	})
	if err != nil {
		return fmt.Errorf("failed to cancel: %w", err)
	}

	if !resp.Cancelled {
		return fmt.Errorf("cancellation failed: %s", resp.Error)
	}

	return nil
}

// WaitForAsyncCompletion polls for task completion
func (k *KernelClient) WaitForAsyncCompletion(ctx context.Context, taskID string, pollInterval time.Duration) (*pb.SyscallResponse, error) {
	ticker := time.NewTicker(pollInterval)
	defer ticker.Stop()

	for {
		select {
		case <-ctx.Done():
			return nil, ctx.Err()
		case <-ticker.C:
			status, err := k.GetAsyncStatus(ctx, taskID)
			if err != nil {
				return nil, err
			}

			switch status.Status {
			case pb.AsyncStatusResponse_COMPLETED:
				return status.Result, nil
			case pb.AsyncStatusResponse_FAILED:
				return status.Result, fmt.Errorf("task failed")
			case pb.AsyncStatusResponse_CANCELLED:
				return nil, fmt.Errorf("task cancelled")
			}
		}
	}
}

// Helper to build syscall request (delegate to existing methods)
func (k *KernelClient) buildSyscallRequest(req *pb.SyscallRequest, syscallType string, params map[string]interface{}) {
	switch syscallType {
	case "read_file", "write_file", "create_file", "delete_file":
		k.buildFilesystemSyscall(req, syscallType, params)
	case "spawn_process", "kill_process":
		k.buildProcessSyscall(req, syscallType, params)
	default:
		// Add more as needed
	}
}
