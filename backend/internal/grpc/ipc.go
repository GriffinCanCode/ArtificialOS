package grpc

import (
	"context"
	"encoding/json"
	"fmt"
	"time"

	"github.com/GriffinCanCode/AgentOS/backend/internal/grpc/kernel"
	pb "github.com/GriffinCanCode/AgentOS/backend/proto/kernel"
)

// IPCClient provides IPC operations through the kernel
type IPCClient struct {
	client *kernel.KernelClient
}

// NewIPCClient creates a new IPC client
func NewIPCClient(kernelClient *kernel.KernelClient) *IPCClient {
	return &IPCClient{client: kernelClient}
}

// ========================================================================
// Pipe Operations
// ========================================================================

// CreatePipe creates a bidirectional pipe between two processes
func (c *IPCClient) CreatePipe(ctx context.Context, readerPID, writerPID uint32, capacity *uint32) (uint32, error) {
	ctx, cancel := context.WithTimeout(ctx, 5*time.Second)
	defer cancel()

	req := &pb.SyscallRequest{
		Pid: readerPID,
		Syscall: &pb.SyscallRequest_CreatePipe{
			CreatePipe: &pb.CreatePipeCall{
				ReaderPid: readerPID,
				WriterPid: writerPID,
				Capacity:  capacity,
			},
		},
	}

	resp, err := c.client.ExecuteSyscallRaw(ctx, req)
	if err != nil {
		return 0, fmt.Errorf("create pipe failed: %w", err)
	}

	switch result := resp.Result.(type) {
	case *pb.SyscallResponse_Success:
		// Decode pipe ID from response data
		if len(result.Success.Data) < 4 {
			return 0, fmt.Errorf("invalid pipe ID response")
		}
		// Parse JSON response with pipe ID
		var pipeID uint32
		// Response is JSON-encoded, parse it
		if err := parseJSONUint32(result.Success.Data, &pipeID); err != nil {
			return 0, fmt.Errorf("failed to parse pipe ID: %w", err)
		}
		return pipeID, nil
	case *pb.SyscallResponse_Error:
		return 0, fmt.Errorf("pipe creation error: %s", result.Error.Message)
	case *pb.SyscallResponse_PermissionDenied:
		return 0, fmt.Errorf("permission denied: %s", result.PermissionDenied.Reason)
	default:
		return 0, fmt.Errorf("unexpected response type")
	}
}

// WritePipe writes data to a pipe
func (c *IPCClient) WritePipe(ctx context.Context, pid, pipeID uint32, data []byte) (uint32, error) {
	ctx, cancel := context.WithTimeout(ctx, 5*time.Second)
	defer cancel()

	req := &pb.SyscallRequest{
		Pid: pid,
		Syscall: &pb.SyscallRequest_WritePipe{
			WritePipe: &pb.WritePipeCall{
				PipeId: pipeID,
				Data:   data,
			},
		},
	}

	resp, err := c.client.ExecuteSyscallRaw(ctx, req)
	if err != nil {
		return 0, fmt.Errorf("write pipe failed: %w", err)
	}

	switch result := resp.Result.(type) {
	case *pb.SyscallResponse_Success:
		// Decode bytes written from response
		var written uint32
		if err := parseJSONUint32(result.Success.Data, &written); err != nil {
			return 0, fmt.Errorf("failed to parse bytes written: %w", err)
		}
		return written, nil
	case *pb.SyscallResponse_Error:
		return 0, fmt.Errorf("pipe write error: %s", result.Error.Message)
	case *pb.SyscallResponse_PermissionDenied:
		return 0, fmt.Errorf("permission denied: %s", result.PermissionDenied.Reason)
	default:
		return 0, fmt.Errorf("unexpected response type")
	}
}

// ReadPipe reads data from a pipe
func (c *IPCClient) ReadPipe(ctx context.Context, pid, pipeID, size uint32) ([]byte, error) {
	ctx, cancel := context.WithTimeout(ctx, 5*time.Second)
	defer cancel()

	req := &pb.SyscallRequest{
		Pid: pid,
		Syscall: &pb.SyscallRequest_ReadPipe{
			ReadPipe: &pb.ReadPipeCall{
				PipeId: pipeID,
				Size:   size,
			},
		},
	}

	resp, err := c.client.ExecuteSyscallRaw(ctx, req)
	if err != nil {
		return nil, fmt.Errorf("read pipe failed: %w", err)
	}

	switch result := resp.Result.(type) {
	case *pb.SyscallResponse_Success:
		return result.Success.Data, nil
	case *pb.SyscallResponse_Error:
		return nil, fmt.Errorf("pipe read error: %s", result.Error.Message)
	case *pb.SyscallResponse_PermissionDenied:
		return nil, fmt.Errorf("permission denied: %s", result.PermissionDenied.Reason)
	default:
		return nil, fmt.Errorf("unexpected response type")
	}
}

// ClosePipe closes a pipe
func (c *IPCClient) ClosePipe(ctx context.Context, pid, pipeID uint32) error {
	ctx, cancel := context.WithTimeout(ctx, 5*time.Second)
	defer cancel()

	req := &pb.SyscallRequest{
		Pid: pid,
		Syscall: &pb.SyscallRequest_ClosePipe{
			ClosePipe: &pb.ClosePipeCall{
				PipeId: pipeID,
			},
		},
	}

	resp, err := c.client.ExecuteSyscallRaw(ctx, req)
	if err != nil {
		return fmt.Errorf("close pipe failed: %w", err)
	}

	switch result := resp.Result.(type) {
	case *pb.SyscallResponse_Success:
		return nil
	case *pb.SyscallResponse_Error:
		return fmt.Errorf("pipe close error: %s", result.Error.Message)
	case *pb.SyscallResponse_PermissionDenied:
		return fmt.Errorf("permission denied: %s", result.PermissionDenied.Reason)
	default:
		return fmt.Errorf("unexpected response type")
	}
}

// ========================================================================
// Shared Memory Operations
// ========================================================================

// CreateShm creates a shared memory segment
func (c *IPCClient) CreateShm(ctx context.Context, pid, size uint32) (uint32, error) {
	ctx, cancel := context.WithTimeout(ctx, 5*time.Second)
	defer cancel()

	req := &pb.SyscallRequest{
		Pid: pid,
		Syscall: &pb.SyscallRequest_CreateShm{
			CreateShm: &pb.CreateShmCall{
				Size: size,
			},
		},
	}

	resp, err := c.client.ExecuteSyscallRaw(ctx, req)
	if err != nil {
		return 0, fmt.Errorf("create shm failed: %w", err)
	}

	switch result := resp.Result.(type) {
	case *pb.SyscallResponse_Success:
		var segmentID uint32
		if err := parseJSONUint32(result.Success.Data, &segmentID); err != nil {
			return 0, fmt.Errorf("failed to parse segment ID: %w", err)
		}
		return segmentID, nil
	case *pb.SyscallResponse_Error:
		return 0, fmt.Errorf("shm creation error: %s", result.Error.Message)
	case *pb.SyscallResponse_PermissionDenied:
		return 0, fmt.Errorf("permission denied: %s", result.PermissionDenied.Reason)
	default:
		return 0, fmt.Errorf("unexpected response type")
	}
}

// AttachShm attaches to a shared memory segment
func (c *IPCClient) AttachShm(ctx context.Context, pid, segmentID uint32, readOnly bool) error {
	ctx, cancel := context.WithTimeout(ctx, 5*time.Second)
	defer cancel()

	req := &pb.SyscallRequest{
		Pid: pid,
		Syscall: &pb.SyscallRequest_AttachShm{
			AttachShm: &pb.AttachShmCall{
				SegmentId: segmentID,
				ReadOnly:  readOnly,
			},
		},
	}

	resp, err := c.client.ExecuteSyscallRaw(ctx, req)
	if err != nil {
		return fmt.Errorf("attach shm failed: %w", err)
	}

	switch result := resp.Result.(type) {
	case *pb.SyscallResponse_Success:
		return nil
	case *pb.SyscallResponse_Error:
		return fmt.Errorf("shm attach error: %s", result.Error.Message)
	case *pb.SyscallResponse_PermissionDenied:
		return fmt.Errorf("permission denied: %s", result.PermissionDenied.Reason)
	default:
		return fmt.Errorf("unexpected response type")
	}
}

// WriteShm writes data to shared memory
func (c *IPCClient) WriteShm(ctx context.Context, pid, segmentID, offset uint32, data []byte) error {
	ctx, cancel := context.WithTimeout(ctx, 5*time.Second)
	defer cancel()

	req := &pb.SyscallRequest{
		Pid: pid,
		Syscall: &pb.SyscallRequest_WriteShm{
			WriteShm: &pb.WriteShmCall{
				SegmentId: segmentID,
				Offset:    offset,
				Data:      data,
			},
		},
	}

	resp, err := c.client.ExecuteSyscallRaw(ctx, req)
	if err != nil {
		return fmt.Errorf("write shm failed: %w", err)
	}

	switch result := resp.Result.(type) {
	case *pb.SyscallResponse_Success:
		return nil
	case *pb.SyscallResponse_Error:
		return fmt.Errorf("shm write error: %s", result.Error.Message)
	case *pb.SyscallResponse_PermissionDenied:
		return fmt.Errorf("permission denied: %s", result.PermissionDenied.Reason)
	default:
		return fmt.Errorf("unexpected response type")
	}
}

// ReadShm reads data from shared memory
func (c *IPCClient) ReadShm(ctx context.Context, pid, segmentID, offset, size uint32) ([]byte, error) {
	ctx, cancel := context.WithTimeout(ctx, 5*time.Second)
	defer cancel()

	req := &pb.SyscallRequest{
		Pid: pid,
		Syscall: &pb.SyscallRequest_ReadShm{
			ReadShm: &pb.ReadShmCall{
				SegmentId: segmentID,
				Offset:    offset,
				Size:      size,
			},
		},
	}

	resp, err := c.client.ExecuteSyscallRaw(ctx, req)
	if err != nil {
		return nil, fmt.Errorf("read shm failed: %w", err)
	}

	switch result := resp.Result.(type) {
	case *pb.SyscallResponse_Success:
		return result.Success.Data, nil
	case *pb.SyscallResponse_Error:
		return nil, fmt.Errorf("shm read error: %s", result.Error.Message)
	case *pb.SyscallResponse_PermissionDenied:
		return nil, fmt.Errorf("permission denied: %s", result.PermissionDenied.Reason)
	default:
		return nil, fmt.Errorf("unexpected response type")
	}
}

// Helper function to parse JSON-encoded uint32
func parseJSONUint32(data []byte, out *uint32) error {
	if len(data) == 0 {
		return fmt.Errorf("empty data")
	}

	// Unmarshal JSON number
	return json.Unmarshal(data, out)
}
