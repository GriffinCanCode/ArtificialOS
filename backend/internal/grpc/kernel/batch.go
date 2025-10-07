package kernel

import (
	"context"
	"fmt"
	"time"

	pb "github.com/GriffinCanCode/AgentOS/backend/proto/kernel"
)

// BatchRequest represents a single syscall in a batch
type BatchRequest struct {
	PID         uint32
	SyscallType string
	Params      map[string]interface{}
}

// BatchResult contains batch execution results
type BatchResult struct {
	Responses    []*pb.SyscallResponse
	SuccessCount uint32
	FailureCount uint32
}

// ExecuteBatch executes multiple syscalls in a batch
func (k *KernelClient) ExecuteBatch(ctx context.Context, requests []BatchRequest, parallel bool) (*BatchResult, error) {
	ctx, cancel := context.WithTimeout(ctx, 30*time.Second)
	defer cancel()

	// Build protobuf requests
	protoRequests := make([]*pb.SyscallRequest, 0, len(requests))
	for _, req := range requests {
		pbReq := &pb.SyscallRequest{Pid: req.PID}
		k.buildSyscallRequest(pbReq, req.SyscallType, req.Params)
		protoRequests = append(protoRequests, pbReq)
	}

	batchReq := &pb.BatchSyscallRequest{
		Requests: protoRequests,
		Parallel: parallel,
	}

	resp, err := k.client.ExecuteSyscallBatch(ctx, batchReq)
	if err != nil {
		return nil, fmt.Errorf("batch execution failed: %w", err)
	}

	return &BatchResult{
		Responses:    resp.Responses,
		SuccessCount: resp.SuccessCount,
		FailureCount: resp.FailureCount,
	}, nil
}

// ExecuteBatchSimple is a convenience wrapper for simple batch operations
func (k *KernelClient) ExecuteBatchSimple(ctx context.Context, pid uint32, syscalls []struct {
	Type   string
	Params map[string]interface{}
}) (*BatchResult, error) {
	requests := make([]BatchRequest, len(syscalls))
	for i, sc := range syscalls {
		requests[i] = BatchRequest{
			PID:         pid,
			SyscallType: sc.Type,
			Params:      sc.Params,
		}
	}
	return k.ExecuteBatch(ctx, requests, false)
}
