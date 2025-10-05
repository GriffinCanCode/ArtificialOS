package grpc

import (
	"context"
	"fmt"
	"time"

	"google.golang.org/grpc"
	"google.golang.org/grpc/credentials/insecure"
	"google.golang.org/grpc/keepalive"

	pb "github.com/GriffinCanCode/AgentOS/backend/proto/kernel"
)

// KernelClient wraps gRPC client for kernel communication
type KernelClient struct {
	conn   *grpc.ClientConn
	client pb.KernelServiceClient
	addr   string
}

// NewKernelClient creates a new kernel client with proper connection management
func NewKernelClient(addr string) (*KernelClient, error) {
	// Configure connection options for production use
	opts := []grpc.DialOption{
		grpc.WithTransportCredentials(insecure.NewCredentials()),
		// Configure keepalive to detect broken connections (reduced frequency to avoid "too_many_pings")
		grpc.WithKeepaliveParams(keepalive.ClientParameters{
			Time:                60 * time.Second, // Send pings every 60 seconds
			Timeout:             20 * time.Second, // Wait 20 seconds for ping ack
			PermitWithoutStream: false,            // Only send pings when streams are active
		}),
		// Set reasonable message size limits
		grpc.WithDefaultCallOptions(
			grpc.MaxCallRecvMsgSize(10*1024*1024), // 10MB receive limit
			grpc.MaxCallSendMsgSize(10*1024*1024), // 10MB send limit
		),
	}

	// Dial without WithBlock() - it's deprecated and problematic
	conn, err := grpc.Dial(addr, opts...)
	if err != nil {
		return nil, fmt.Errorf("failed to dial kernel: %w", err)
	}

	return &KernelClient{
		conn:   conn,
		client: pb.NewKernelServiceClient(conn),
		addr:   addr,
	}, nil
}

// Close closes the connection
func (k *KernelClient) Close() error {
	if k.conn != nil {
		return k.conn.Close()
	}
	return nil
}

// CreateProcessOptions optional parameters for process creation
type CreateProcessOptions struct {
	Command string
	Args    []string
	EnvVars []string
}

// CreateProcess creates a new sandboxed process
func (k *KernelClient) CreateProcess(
	ctx context.Context,
	name string,
	priority uint32,
	sandboxLevel string,
	opts *CreateProcessOptions,
) (*uint32, *uint32, error) {
	levelMap := map[string]pb.SandboxLevel{
		"MINIMAL":    pb.SandboxLevel_MINIMAL,
		"STANDARD":   pb.SandboxLevel_STANDARD,
		"PRIVILEGED": pb.SandboxLevel_PRIVILEGED,
	}

	level, ok := levelMap[sandboxLevel]
	if !ok {
		level = pb.SandboxLevel_STANDARD
	}

	// Build request
	req := &pb.CreateProcessRequest{
		Name:         name,
		Priority:     priority,
		SandboxLevel: level,
	}

	// Add optional execution parameters
	if opts != nil {
		if opts.Command != "" {
			req.Command = &opts.Command
			req.Args = opts.Args
			req.EnvVars = opts.EnvVars
		}
	}

	// Use provided context with timeout
	ctx, cancel := context.WithTimeout(ctx, 5*time.Second)
	defer cancel()

	resp, err := k.client.CreateProcess(ctx, req)
	if err != nil {
		return nil, nil, fmt.Errorf("create process failed: %w", err)
	}

	if !resp.Success {
		errMsg := "unknown error"
		if resp.Error != "" {
			errMsg = resp.Error
		}
		return nil, nil, fmt.Errorf("create process failed: %s", errMsg)
	}

	return &resp.Pid, resp.OsPid, nil
}

// ExecuteSyscall executes a system call
func (k *KernelClient) ExecuteSyscall(ctx context.Context, pid uint32, syscallType string, params map[string]interface{}) ([]byte, error) {
	// Use provided context with timeout
	ctx, cancel := context.WithTimeout(ctx, 30*time.Second)
	defer cancel()

	req := &pb.SyscallRequest{Pid: pid}

	// Build syscall based on type
	switch syscallType {
	// File system operations
	case "read_file":
		path, _ := params["path"].(string)
		req.Syscall = &pb.SyscallRequest_ReadFile{
			ReadFile: &pb.ReadFileCall{Path: path},
		}
	case "write_file":
		path, _ := params["path"].(string)
		data, _ := params["data"].([]byte)
		req.Syscall = &pb.SyscallRequest_WriteFile{
			WriteFile: &pb.WriteFileCall{Path: path, Data: data},
		}
	case "create_file":
		path, _ := params["path"].(string)
		req.Syscall = &pb.SyscallRequest_CreateFile{
			CreateFile: &pb.CreateFileCall{Path: path},
		}
	case "delete_file":
		path, _ := params["path"].(string)
		req.Syscall = &pb.SyscallRequest_DeleteFile{
			DeleteFile: &pb.DeleteFileCall{Path: path},
		}
	case "list_directory":
		path, _ := params["path"].(string)
		req.Syscall = &pb.SyscallRequest_ListDirectory{
			ListDirectory: &pb.ListDirectoryCall{Path: path},
		}
	case "file_exists":
		path, _ := params["path"].(string)
		req.Syscall = &pb.SyscallRequest_FileExists{
			FileExists: &pb.FileExistsCall{Path: path},
		}
	// System info operations
	case "get_system_info":
		req.Syscall = &pb.SyscallRequest_GetSystemInfo{
			GetSystemInfo: &pb.GetSystemInfoCall{},
		}
	case "get_current_time":
		req.Syscall = &pb.SyscallRequest_GetCurrentTime{
			GetCurrentTime: &pb.GetCurrentTimeCall{},
		}
	case "get_env_var":
		key, _ := params["key"].(string)
		req.Syscall = &pb.SyscallRequest_GetEnvVar{
			GetEnvVar: &pb.GetEnvVarCall{Key: key},
		}
	default:
		return nil, fmt.Errorf("unsupported syscall type: %s", syscallType)
	}

	resp, err := k.client.ExecuteSyscall(ctx, req)
	if err != nil {
		return nil, fmt.Errorf("syscall failed: %w", err)
	}

	switch result := resp.Result.(type) {
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
