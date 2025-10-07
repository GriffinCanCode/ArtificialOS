package kernel

import (
	"context"
	"fmt"
	"time"

	pb "github.com/GriffinCanCode/AgentOS/backend/proto/kernel"
)

// ExecuteSyscallIouring executes a syscall using io_uring-style async completion
// Returns a task ID that can be used to track the operation
func (k *KernelClient) ExecuteSyscallIouring(ctx context.Context, pid uint32, syscallType string, params map[string]interface{}) (string, error) {
	ctx, cancel := context.WithTimeout(ctx, 5*time.Second)
	defer cancel()

	req := &pb.SyscallRequest{Pid: pid}
	k.buildSyscallRequest(req, syscallType, params)

	resp, err := k.client.ExecuteSyscallIouring(ctx, req)
	if err != nil {
		return "", fmt.Errorf("io_uring execution failed: %w", err)
	}

	if !resp.Accepted {
		return "", fmt.Errorf("io_uring task not accepted: %s", resp.Error)
	}

	return resp.TaskId, nil
}

// ReapCompletions retrieves completed io_uring operations for a process
// maxCompletions: maximum number of completions to reap (0 = all available)
func (k *KernelClient) ReapCompletions(ctx context.Context, pid uint32, maxCompletions uint32) ([]*pb.IoUringCompletion, error) {
	ctx, cancel := context.WithTimeout(ctx, 5*time.Second)
	defer cancel()

	resp, err := k.client.ReapCompletions(ctx, &pb.ReapCompletionsRequest{
		Pid:            pid,
		MaxCompletions: maxCompletions,
	})
	if err != nil {
		return nil, fmt.Errorf("failed to reap completions: %w", err)
	}

	return resp.Completions, nil
}

// SubmitIouringBatch submits multiple syscalls as a batch via io_uring
// Returns the sequence numbers assigned to each operation
func (k *KernelClient) SubmitIouringBatch(ctx context.Context, pid uint32, syscalls []SyscallSpec) ([]uint64, error) {
	ctx, cancel := context.WithTimeout(ctx, 10*time.Second)
	defer cancel()

	// Build batch request
	requests := make([]*pb.SyscallRequest, 0, len(syscalls))
	for _, spec := range syscalls {
		req := &pb.SyscallRequest{Pid: pid}
		k.buildSyscallRequest(req, spec.Type, spec.Params)
		requests = append(requests, req)
	}

	batchReq := &pb.BatchSyscallRequest{
		Requests: requests,
		Parallel: true, // io_uring executes concurrently
	}

	resp, err := k.client.SubmitIouringBatch(ctx, batchReq)
	if err != nil {
		return nil, fmt.Errorf("io_uring batch submission failed: %w", err)
	}

	if !resp.Accepted {
		return nil, fmt.Errorf("io_uring batch not accepted: %s", resp.Error)
	}

	return resp.Sequences, nil
}

// SyscallSpec specifies a syscall to be submitted
type SyscallSpec struct {
	Type   string
	Params map[string]interface{}
}

// WaitForIouringCompletion waits for a specific io_uring operation to complete
// by polling the completion queue
func (k *KernelClient) WaitForIouringCompletion(ctx context.Context, pid uint32, seq uint64, pollInterval time.Duration) (*pb.IoUringCompletion, error) {
	ticker := time.NewTicker(pollInterval)
	defer ticker.Stop()

	for {
		select {
		case <-ctx.Done():
			return nil, ctx.Err()
		case <-ticker.C:
			completions, err := k.ReapCompletions(ctx, pid, 32)
			if err != nil {
				return nil, err
			}

			// Check if our sequence is in the completions
			for _, completion := range completions {
				if completion.Seq == seq {
					return completion, nil
				}
			}
		}
	}
}

// ExecuteSyscallIouringAndWait submits a syscall via io_uring and waits for completion
// This is a convenience method that combines submission and completion
func (k *KernelClient) ExecuteSyscallIouringAndWait(ctx context.Context, pid uint32, syscallType string, params map[string]interface{}, pollInterval time.Duration) ([]byte, error) {
	// Submit via io_uring
	taskID, err := k.ExecuteSyscallIouring(ctx, pid, syscallType, params)
	if err != nil {
		return nil, err
	}

	// Parse sequence number from task ID (format: "iouring_<seq>")
	var seq uint64
	_, err = fmt.Sscanf(taskID, "iouring_%d", &seq)
	if err != nil {
		return nil, fmt.Errorf("failed to parse io_uring task ID: %w", err)
	}

	// Wait for completion
	completion, err := k.WaitForIouringCompletion(ctx, pid, seq, pollInterval)
	if err != nil {
		return nil, err
	}

	// Extract result
	if completion.Result == nil {
		return nil, fmt.Errorf("completion has no result")
	}

	switch result := completion.Result.Result.(type) {
	case *pb.SyscallResponse_Success:
		if result.Success != nil {
			return result.Success.Data, nil
		}
		return nil, fmt.Errorf("syscall success response has nil data")
	case *pb.SyscallResponse_Error:
		if result.Error != nil {
			return nil, fmt.Errorf("syscall error: %s", result.Error.Message)
		}
		return nil, fmt.Errorf("syscall error: unknown error")
	case *pb.SyscallResponse_PermissionDenied:
		if result.PermissionDenied != nil {
			return nil, fmt.Errorf("permission denied: %s", result.PermissionDenied.Reason)
		}
		return nil, fmt.Errorf("permission denied")
	default:
		return nil, fmt.Errorf("unknown response type")
	}
}
