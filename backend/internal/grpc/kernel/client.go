package kernel

import (
	"context"
	"fmt"
	"time"

	"google.golang.org/grpc"
	"google.golang.org/grpc/credentials/insecure"
	"google.golang.org/grpc/keepalive"

	"github.com/GriffinCanCode/AgentOS/backend/internal/infrastructure/resilience"
	pb "github.com/GriffinCanCode/AgentOS/backend/proto/kernel"
)

// KernelClient wraps gRPC client for kernel communication with circuit breaker
type KernelClient struct {
	conn    *grpc.ClientConn
	client  pb.KernelServiceClient
	addr    string
	breaker *resilience.Breaker
}

// New creates a new kernel client with proper connection management
func New(addr string) (*KernelClient, error) {
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

	// Create circuit breaker for kernel calls
	breaker := resilience.New("kernel", resilience.Settings{
		MaxRequests: 3,
		Interval:    30 * time.Second,
		Timeout:     10 * time.Second,
		ReadyToTrip: func(counts resilience.Counts) bool {
			// Trip if 5+ consecutive failures or 50% failure rate with 10+ requests
			return counts.ConsecutiveFailures >= 5 ||
				(counts.Requests >= 10 && float64(counts.TotalFailures)/float64(counts.Requests) > 0.5)
		},
	})

	return &KernelClient{
		conn:    conn,
		client:  pb.NewKernelServiceClient(conn),
		addr:    addr,
		breaker: breaker,
	}, nil
}

// Close closes the connection
func (k *KernelClient) Close() error {
	if k.conn != nil {
		return k.conn.Close()
	}
	return nil
}

// ExecuteSyscallRaw executes a raw protobuf syscall request with circuit breaker
func (k *KernelClient) ExecuteSyscallRaw(ctx context.Context, req *pb.SyscallRequest) (*pb.SyscallResponse, error) {
	result, err := k.breaker.Execute(func() (interface{}, error) {
		return k.client.ExecuteSyscall(ctx, req)
	})

	if err == resilience.ErrCircuitOpen {
		return nil, fmt.Errorf("kernel service unavailable: circuit breaker open")
	}

	if err != nil {
		return nil, err
	}

	return result.(*pb.SyscallResponse), nil
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

	// Execute through circuit breaker
	result, err := k.breaker.Execute(func() (interface{}, error) {
		return k.client.CreateProcess(ctx, req)
	})

	if err == resilience.ErrCircuitOpen {
		return nil, nil, fmt.Errorf("kernel service unavailable: circuit breaker open")
	}

	if err != nil {
		return nil, nil, fmt.Errorf("create process failed: %w", err)
	}

	resp := result.(*pb.CreateProcessResponse)

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

	// Build syscall based on type - delegate to specific handlers
	switch syscallType {
	// File system operations
	case "read_file", "write_file", "create_file", "delete_file", "list_directory",
		"file_exists", "file_stat", "move_file", "copy_file", "create_directory",
		"remove_directory", "get_working_directory", "set_working_directory", "truncate_file":
		k.buildFilesystemSyscall(req, syscallType, params)
	// Process operations
	case "spawn_process", "kill_process", "get_process_info", "get_process_list",
		"set_process_priority", "get_process_state", "get_process_stats", "wait_process":
		k.buildProcessSyscall(req, syscallType, params)
	// System info operations
	case "get_system_info", "get_current_time", "get_env_var", "set_env_var":
		k.buildSystemSyscall(req, syscallType, params)
	// Time operations
	case "sleep", "get_uptime":
		k.buildTimeSyscall(req, syscallType, params)
	// Memory operations
	case "get_memory_stats", "get_process_memory_stats", "trigger_gc":
		k.buildMemorySyscall(req, syscallType, params)
	// Signal operations
	case "send_signal":
		k.buildSignalSyscall(req, syscallType, params)
	// Network operations
	case "network_request":
		k.buildNetworkSyscall(req, syscallType, params)
	// Network - Socket operations
	case "socket", "bind", "listen", "accept", "connect", "send", "recv",
		"send_to", "recv_from", "close_socket", "set_sock_opt", "get_sock_opt":
		k.buildSocketSyscall(req, syscallType, params)
	// File Descriptor operations
	case "open", "close", "dup", "dup2", "lseek", "fcntl":
		k.buildFdSyscall(req, syscallType, params)
	// IPC - Pipes
	case "create_pipe", "write_pipe", "read_pipe", "close_pipe", "destroy_pipe", "pipe_stats":
		k.buildIPCSyscall(req, syscallType, params)
	// IPC - Shared Memory
	case "create_shm", "attach_shm", "detach_shm", "write_shm", "read_shm", "destroy_shm", "shm_stats":
		k.buildIPCSyscall(req, syscallType, params)
	// IPC - Memory-Mapped Files
	case "mmap", "mmap_read", "mmap_write", "msync", "munmap", "mmap_stats":
		k.buildIPCSyscall(req, syscallType, params)
	// Scheduler operations
	case "schedule_next", "yield_process", "get_current_scheduled", "get_scheduler_stats":
		k.buildSchedulerSyscall(req, syscallType, params)
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
