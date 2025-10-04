package grpc

import (
	"context"
	"fmt"
	"time"

	"google.golang.org/grpc"
	"google.golang.org/grpc/credentials/insecure"

	pb "github.com/GriffinCanCode/AgentOS/backend/proto/kernel"
)

// KernelClient wraps gRPC client for kernel communication
type KernelClient struct {
	conn   *grpc.ClientConn
	client pb.KernelServiceClient
	addr   string
}

// NewKernelClient creates a new kernel client
func NewKernelClient(addr string) (*KernelClient, error) {
	ctx, cancel := context.WithTimeout(context.Background(), 5*time.Second)
	defer cancel()

	conn, err := grpc.DialContext(ctx, addr,
		grpc.WithTransportCredentials(insecure.NewCredentials()),
		grpc.WithBlock(),
	)
	if err != nil {
		return nil, fmt.Errorf("failed to connect to kernel: %w", err)
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

// CreateProcess creates a new sandboxed process
func (k *KernelClient) CreateProcess(name string, priority uint32, sandboxLevel string) (*uint32, error) {
	levelMap := map[string]pb.SandboxLevel{
		"MINIMAL":    pb.SandboxLevel_MINIMAL,
		"STANDARD":   pb.SandboxLevel_STANDARD,
		"PRIVILEGED": pb.SandboxLevel_PRIVILEGED,
	}

	level, ok := levelMap[sandboxLevel]
	if !ok {
		level = pb.SandboxLevel_STANDARD
	}

	ctx, cancel := context.WithTimeout(context.Background(), 5*time.Second)
	defer cancel()

	resp, err := k.client.CreateProcess(ctx, &pb.CreateProcessRequest{
		Name:         name,
		Priority:     priority,
		SandboxLevel: level,
	})
	if err != nil {
		return nil, fmt.Errorf("create process failed: %w", err)
	}

	if !resp.Success {
		return nil, fmt.Errorf("create process failed: %s", resp.Error)
	}

	return &resp.Pid, nil
}

// ExecuteSyscall executes a system call
func (k *KernelClient) ExecuteSyscall(pid uint32, syscallType string, params map[string]interface{}) ([]byte, error) {
	ctx, cancel := context.WithTimeout(context.Background(), 10*time.Second)
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
		return result.Success.Data, nil
	case *pb.SyscallResponse_Error:
		return nil, fmt.Errorf("syscall error: %s", result.Error.Message)
	case *pb.SyscallResponse_PermissionDenied:
		return nil, fmt.Errorf("permission denied: %s", result.PermissionDenied.Reason)
	default:
		return nil, fmt.Errorf("unknown response type")
	}
}
