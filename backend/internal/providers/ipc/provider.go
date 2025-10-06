package ipc

import (
	"context"
	"fmt"

	"github.com/GriffinCanCode/AgentOS/backend/internal/grpc"
	"github.com/GriffinCanCode/AgentOS/backend/internal/grpc/kernel"
	"github.com/GriffinCanCode/AgentOS/backend/internal/shared/types"
)

// Provider implements IPC operations through the kernel
type Provider struct {
	ipcClient *grpc.IPCClient
}

// NewProvider creates a new IPC provider
func NewProvider(kernelClient *kernel.KernelClient) *Provider {
	return &Provider{
		ipcClient: grpc.NewIPCClient(kernelClient),
	}
}

// Definition returns the service definition
func (p *Provider) Definition() types.Service {
	return types.Service{
		ID:          "ipc",
		Name:        "Inter-Process Communication",
		Description: "Provides pipes, shared memory, and async queues for efficient data sharing between apps",
		Category:    types.CategorySystem,
		Capabilities: []string{
			"create_pipe",
			"write_pipe",
			"read_pipe",
			"create_shared_memory",
			"write_shared_memory",
			"read_shared_memory",
			"create_queue",
			"send_queue",
			"receive_queue",
			"subscribe_queue",
		},
		Tools: []types.Tool{
			{
				ID:          "ipc.create_pipe",
				Name:        "Create Pipe",
				Description: "Create a unidirectional pipe for streaming data between two processes",
				Parameters: []types.Parameter{
					{
						Name:        "reader_pid",
						Type:        "number",
						Description: "Process ID that will read from the pipe",
						Required:    true,
					},
					{
						Name:        "writer_pid",
						Type:        "number",
						Description: "Process ID that will write to the pipe",
						Required:    true,
					},
					{
						Name:        "capacity",
						Type:        "number",
						Description: "Optional pipe buffer capacity in bytes (default: 64KB)",
						Required:    false,
					},
				},
				Returns: "Pipe ID (number)",
			},
			{
				ID:          "ipc.write_pipe",
				Name:        "Write to Pipe",
				Description: "Write data to a pipe",
				Parameters: []types.Parameter{
					{
						Name:        "pipe_id",
						Type:        "number",
						Description: "ID of the pipe to write to",
						Required:    true,
					},
					{
						Name:        "data",
						Type:        "string",
						Description: "Data to write (will be converted to bytes)",
						Required:    true,
					},
				},
				Returns: "Number of bytes written",
			},
			{
				ID:          "ipc.read_pipe",
				Name:        "Read from Pipe",
				Description: "Read data from a pipe",
				Parameters: []types.Parameter{
					{
						Name:        "pipe_id",
						Type:        "number",
						Description: "ID of the pipe to read from",
						Required:    true,
					},
					{
						Name:        "size",
						Type:        "number",
						Description: "Maximum number of bytes to read",
						Required:    true,
					},
				},
				Returns: "Data read from pipe (string)",
			},
			{
				ID:          "ipc.create_shm",
				Name:        "Create Shared Memory",
				Description: "Create a shared memory segment for zero-copy data sharing",
				Parameters: []types.Parameter{
					{
						Name:        "size",
						Type:        "number",
						Description: "Size of the shared memory segment in bytes (max 100MB)",
						Required:    true,
					},
				},
				Returns: "Segment ID (number)",
			},
			{
				ID:          "ipc.attach_shm",
				Name:        "Attach to Shared Memory",
				Description: "Attach to an existing shared memory segment",
				Parameters: []types.Parameter{
					{
						Name:        "segment_id",
						Type:        "number",
						Description: "ID of the shared memory segment",
						Required:    true,
					},
					{
						Name:        "read_only",
						Type:        "boolean",
						Description: "Whether to attach as read-only",
						Required:    false,
					},
				},
				Returns: "Success confirmation",
			},
			{
				ID:          "ipc.write_shm",
				Name:        "Write to Shared Memory",
				Description: "Write data to a shared memory segment",
				Parameters: []types.Parameter{
					{
						Name:        "segment_id",
						Type:        "number",
						Description: "ID of the shared memory segment",
						Required:    true,
					},
					{
						Name:        "offset",
						Type:        "number",
						Description: "Byte offset to write at",
						Required:    true,
					},
					{
						Name:        "data",
						Type:        "string",
						Description: "Data to write",
						Required:    true,
					},
				},
				Returns: "Success confirmation",
			},
			{
				ID:          "ipc.read_shm",
				Name:        "Read from Shared Memory",
				Description: "Read data from a shared memory segment",
				Parameters: []types.Parameter{
					{
						Name:        "segment_id",
						Type:        "number",
						Description: "ID of the shared memory segment",
						Required:    true,
					},
					{
						Name:        "offset",
						Type:        "number",
						Description: "Byte offset to read from",
						Required:    true,
					},
					{
						Name:        "size",
						Type:        "number",
						Description: "Number of bytes to read",
						Required:    true,
					},
				},
				Returns: "Data read from segment (string)",
			},
			{
				ID:          "ipc.create_queue",
				Name:        "Create Async Queue",
				Description: "Create an async message queue (FIFO, Priority, or PubSub)",
				Parameters: []types.Parameter{
					{
						Name:        "queue_type",
						Type:        "string",
						Description: "Queue type: 'fifo', 'priority', or 'pubsub'",
						Required:    true,
					},
					{
						Name:        "capacity",
						Type:        "number",
						Description: "Maximum queue capacity (default: 1000)",
						Required:    false,
					},
				},
				Returns: "Queue ID",
			},
			{
				ID:          "ipc.send_queue",
				Name:        "Send to Queue",
				Description: "Send a message to an async queue",
				Parameters: []types.Parameter{
					{
						Name:        "queue_id",
						Type:        "number",
						Description: "ID of the queue",
						Required:    true,
					},
					{
						Name:        "data",
						Type:        "string",
						Description: "Data to send",
						Required:    true,
					},
					{
						Name:        "priority",
						Type:        "number",
						Description: "Message priority (0-255, for priority queues)",
						Required:    false,
					},
				},
				Returns: "Success",
			},
			{
				ID:          "ipc.receive_queue",
				Name:        "Receive from Queue",
				Description: "Receive a message from an async queue (non-blocking)",
				Parameters: []types.Parameter{
					{
						Name:        "queue_id",
						Type:        "number",
						Description: "ID of the queue",
						Required:    true,
					},
				},
				Returns: "Message data (or null if empty)",
			},
			{
				ID:          "ipc.subscribe_queue",
				Name:        "Subscribe to Queue",
				Description: "Subscribe to a PubSub queue to receive broadcasts",
				Parameters: []types.Parameter{
					{
						Name:        "queue_id",
						Type:        "number",
						Description: "ID of the PubSub queue",
						Required:    true,
					},
				},
				Returns: "Success",
			},
			{
				ID:          "ipc.unsubscribe_queue",
				Name:        "Unsubscribe from Queue",
				Description: "Unsubscribe from a PubSub queue",
				Parameters: []types.Parameter{
					{
						Name:        "queue_id",
						Type:        "number",
						Description: "ID of the PubSub queue",
						Required:    true,
					},
				},
				Returns: "Success",
			},
		},
	}
}

// Execute handles IPC tool execution
func (p *Provider) Execute(ctx context.Context, toolID string, params map[string]interface{}, appCtx *types.Context) (*types.Result, error) {
	// Get PID from app context
	if appCtx.SandboxPID == nil {
		return &types.Result{
			Success: false,
			Error:   stringPtr("app does not have a sandbox PID"),
		}, fmt.Errorf("missing sandbox PID")
	}
	pid := *appCtx.SandboxPID

	switch toolID {
	case "ipc.create_pipe":
		return p.createPipe(ctx, params, pid)
	case "ipc.write_pipe":
		return p.writePipe(ctx, params, pid)
	case "ipc.read_pipe":
		return p.readPipe(ctx, params, pid)
	case "ipc.create_shm":
		return p.createShm(ctx, params, pid)
	case "ipc.attach_shm":
		return p.attachShm(ctx, params, pid)
	case "ipc.write_shm":
		return p.writeShm(ctx, params, pid)
	case "ipc.read_shm":
		return p.readShm(ctx, params, pid)
	case "ipc.create_queue":
		return p.createQueue(ctx, params, pid)
	case "ipc.send_queue":
		return p.sendQueue(ctx, params, pid)
	case "ipc.receive_queue":
		return p.receiveQueue(ctx, params, pid)
	case "ipc.subscribe_queue":
		return p.subscribeQueue(ctx, params, pid)
	case "ipc.unsubscribe_queue":
		return p.unsubscribeQueue(ctx, params, pid)
	default:
		return &types.Result{
			Success: false,
			Error:   stringPtr(fmt.Sprintf("unknown tool: %s", toolID)),
		}, fmt.Errorf("unknown tool: %s", toolID)
	}
}

func (p *Provider) createPipe(ctx context.Context, params map[string]interface{}, ownerPID uint32) (*types.Result, error) {
	readerPID, ok := params["reader_pid"].(float64)
	if !ok {
		return errorResult("reader_pid is required")
	}

	writerPID, ok := params["writer_pid"].(float64)
	if !ok {
		return errorResult("writer_pid is required")
	}

	var capacity *uint32
	if cap, ok := params["capacity"].(float64); ok {
		c := uint32(cap)
		capacity = &c
	}

	pipeID, err := p.ipcClient.CreatePipe(ctx, uint32(readerPID), uint32(writerPID), capacity)
	if err != nil {
		return errorResult(err.Error())
	}

	return &types.Result{
		Success: true,
		Data: map[string]interface{}{
			"pipe_id":    pipeID,
			"reader_pid": uint32(readerPID),
			"writer_pid": uint32(writerPID),
		},
	}, nil
}

func (p *Provider) writePipe(ctx context.Context, params map[string]interface{}, pid uint32) (*types.Result, error) {
	pipeID, ok := params["pipe_id"].(float64)
	if !ok {
		return errorResult("pipe_id is required")
	}

	data, ok := params["data"].(string)
	if !ok {
		return errorResult("data is required")
	}

	written, err := p.ipcClient.WritePipe(ctx, pid, uint32(pipeID), []byte(data))
	if err != nil {
		return errorResult(err.Error())
	}

	return &types.Result{
		Success: true,
		Data: map[string]interface{}{
			"bytes_written": written,
		},
	}, nil
}

func (p *Provider) readPipe(ctx context.Context, params map[string]interface{}, pid uint32) (*types.Result, error) {
	pipeID, ok := params["pipe_id"].(float64)
	if !ok {
		return errorResult("pipe_id is required")
	}

	size, ok := params["size"].(float64)
	if !ok {
		return errorResult("size is required")
	}

	data, err := p.ipcClient.ReadPipe(ctx, pid, uint32(pipeID), uint32(size))
	if err != nil {
		return errorResult(err.Error())
	}

	return &types.Result{
		Success: true,
		Data: map[string]interface{}{
			"data":  string(data),
			"bytes": len(data),
		},
	}, nil
}

func (p *Provider) createShm(ctx context.Context, params map[string]interface{}, pid uint32) (*types.Result, error) {
	size, ok := params["size"].(float64)
	if !ok {
		return errorResult("size is required")
	}

	segmentID, err := p.ipcClient.CreateShm(ctx, pid, uint32(size))
	if err != nil {
		return errorResult(err.Error())
	}

	return &types.Result{
		Success: true,
		Data: map[string]interface{}{
			"segment_id": segmentID,
			"size":       uint32(size),
			"owner_pid":  pid,
		},
	}, nil
}

func (p *Provider) attachShm(ctx context.Context, params map[string]interface{}, pid uint32) (*types.Result, error) {
	segmentID, ok := params["segment_id"].(float64)
	if !ok {
		return errorResult("segment_id is required")
	}

	readOnly := false
	if ro, ok := params["read_only"].(bool); ok {
		readOnly = ro
	}

	err := p.ipcClient.AttachShm(ctx, pid, uint32(segmentID), readOnly)
	if err != nil {
		return errorResult(err.Error())
	}

	return &types.Result{
		Success: true,
		Data: map[string]interface{}{
			"segment_id": uint32(segmentID),
			"attached":   true,
			"read_only":  readOnly,
		},
	}, nil
}

func (p *Provider) writeShm(ctx context.Context, params map[string]interface{}, pid uint32) (*types.Result, error) {
	segmentID, ok := params["segment_id"].(float64)
	if !ok {
		return errorResult("segment_id is required")
	}

	offset, ok := params["offset"].(float64)
	if !ok {
		return errorResult("offset is required")
	}

	data, ok := params["data"].(string)
	if !ok {
		return errorResult("data is required")
	}

	err := p.ipcClient.WriteShm(ctx, pid, uint32(segmentID), uint32(offset), []byte(data))
	if err != nil {
		return errorResult(err.Error())
	}

	return &types.Result{
		Success: true,
		Data: map[string]interface{}{
			"bytes_written": len(data),
		},
	}, nil
}

func (p *Provider) readShm(ctx context.Context, params map[string]interface{}, pid uint32) (*types.Result, error) {
	segmentID, ok := params["segment_id"].(float64)
	if !ok {
		return errorResult("segment_id is required")
	}

	offset, ok := params["offset"].(float64)
	if !ok {
		return errorResult("offset is required")
	}

	size, ok := params["size"].(float64)
	if !ok {
		return errorResult("size is required")
	}

	data, err := p.ipcClient.ReadShm(ctx, pid, uint32(segmentID), uint32(offset), uint32(size))
	if err != nil {
		return errorResult(err.Error())
	}

	return &types.Result{
		Success: true,
		Data: map[string]interface{}{
			"data":  string(data),
			"bytes": len(data),
		},
	}, nil
}

func (p *Provider) createQueue(ctx context.Context, params map[string]interface{}, pid uint32) (*types.Result, error) {
	queueType, ok := params["queue_type"].(string)
	if !ok {
		return errorResult("queue_type is required")
	}

	var capacity *uint32
	if cap, ok := params["capacity"].(float64); ok {
		c := uint32(cap)
		capacity = &c
	}

	queueID, err := p.ipcClient.CreateQueue(ctx, pid, queueType, capacity)
	if err != nil {
		return errorResult(err.Error())
	}

	return &types.Result{
		Success: true,
		Data: map[string]interface{}{
			"queue_id":   queueID,
			"queue_type": queueType,
			"owner_pid":  pid,
		},
	}, nil
}

func (p *Provider) sendQueue(ctx context.Context, params map[string]interface{}, pid uint32) (*types.Result, error) {
	queueID, ok := params["queue_id"].(float64)
	if !ok {
		return errorResult("queue_id is required")
	}

	data, ok := params["data"].(string)
	if !ok {
		return errorResult("data is required")
	}

	var priority *uint32
	if pri, ok := params["priority"].(float64); ok {
		p := uint32(pri)
		priority = &p
	}

	err := p.ipcClient.SendQueue(ctx, pid, uint32(queueID), []byte(data), priority)
	if err != nil {
		return errorResult(err.Error())
	}

	return &types.Result{
		Success: true,
		Data: map[string]interface{}{
			"queue_id": uint32(queueID),
			"sent":     true,
		},
	}, nil
}

func (p *Provider) receiveQueue(ctx context.Context, params map[string]interface{}, pid uint32) (*types.Result, error) {
	queueID, ok := params["queue_id"].(float64)
	if !ok {
		return errorResult("queue_id is required")
	}

	data, err := p.ipcClient.ReceiveQueue(ctx, pid, uint32(queueID))
	if err != nil {
		return errorResult(err.Error())
	}

	return &types.Result{
		Success: true,
		Data: map[string]interface{}{
			"queue_id": uint32(queueID),
			"data":     string(data),
			"bytes":    len(data),
		},
	}, nil
}

func (p *Provider) subscribeQueue(ctx context.Context, params map[string]interface{}, pid uint32) (*types.Result, error) {
	queueID, ok := params["queue_id"].(float64)
	if !ok {
		return errorResult("queue_id is required")
	}

	err := p.ipcClient.SubscribeQueue(ctx, pid, uint32(queueID))
	if err != nil {
		return errorResult(err.Error())
	}

	return &types.Result{
		Success: true,
		Data: map[string]interface{}{
			"queue_id":   uint32(queueID),
			"subscribed": true,
		},
	}, nil
}

func (p *Provider) unsubscribeQueue(ctx context.Context, params map[string]interface{}, pid uint32) (*types.Result, error) {
	queueID, ok := params["queue_id"].(float64)
	if !ok {
		return errorResult("queue_id is required")
	}

	err := p.ipcClient.UnsubscribeQueue(ctx, pid, uint32(queueID))
	if err != nil {
		return errorResult(err.Error())
	}

	return &types.Result{
		Success: true,
		Data: map[string]interface{}{
			"queue_id":     uint32(queueID),
			"unsubscribed": true,
		},
	}, nil
}

func errorResult(message string) (*types.Result, error) {
	return &types.Result{
		Success: false,
		Error:   stringPtr(message),
	}, fmt.Errorf("%s", message)
}

func stringPtr(s string) *string {
	return &s
}
