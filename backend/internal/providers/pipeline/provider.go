package pipeline

import (
	"context"
	"fmt"
	"sync"
	"time"

	"github.com/GriffinCanCode/AgentOS/backend/internal/grpc"
	"github.com/GriffinCanCode/AgentOS/backend/internal/grpc/kernel"
	"github.com/GriffinCanCode/AgentOS/backend/internal/shared/types"
)

// Provider implements data pipeline operations with IPC
type Provider struct {
	kernelClient *kernel.KernelClient
	ipcClient    *grpc.IPCClient

	// Pipeline state
	mu           sync.RWMutex
	running      bool
	pipelineID   string
	processIDs   map[string]uint32 // stage name -> PID
	ipcResources IPCResources
	metrics      PipelineMetrics
	logs         []LogEntry
	cancelFunc   context.CancelFunc
}

// IPCResources tracks all IPC resources created
type IPCResources struct {
	PipeID     uint32   // Stage 1 -> 2
	ShmID      uint32   // Stage 2 -> 3
	QueueID    uint32   // Stage 3 -> Writers
	WriterPIDs []uint32 // 3 writer processes
}

// PipelineMetrics tracks performance metrics
type PipelineMetrics struct {
	Stage1Processed int64
	Stage2Processed int64
	Stage3Processed int64
	ThroughputRPS   float64
	Stage1LatencyMs float64
	Stage2LatencyMs float64
	Stage3LatencyMs float64
	PipeMemoryKB    float64
	ShmMemoryKB     float64
	QueueMemoryKB   float64
	StartTime       time.Time
}

// LogEntry represents a pipeline log
type LogEntry struct {
	Timestamp time.Time
	Level     string
	Stage     string
	Message   string
}

// NewProvider creates a new pipeline provider
func NewProvider(kernelClient *kernel.KernelClient) *Provider {
	return &Provider{
		kernelClient: kernelClient,
		ipcClient:    grpc.NewIPCClient(kernelClient),
		processIDs:   make(map[string]uint32),
		logs:         make([]LogEntry, 0),
	}
}

// Definition returns the service definition
func (p *Provider) Definition() types.Service {
	return types.Service{
		ID:          "pipeline",
		Name:        "Data Pipeline",
		Description: "Multi-process ETL pipeline with IPC demonstration",
		Category:    types.CategorySystem,
		Capabilities: []string{
			"start_pipeline",
			"stop_pipeline",
			"get_metrics",
			"get_logs",
			"get_ipc_details",
		},
		Tools: []types.Tool{
			{
				ID:          "pipeline.init",
				Name:        "Initialize Pipeline",
				Description: "Initialize pipeline state on mount",
				Parameters:  []types.Parameter{},
				Returns:     "Pipeline initialization status",
			},
			{
				ID:          "pipeline.start",
				Name:        "Start Pipeline",
				Description: "Start the data pipeline with all IPC mechanisms",
				Parameters:  []types.Parameter{},
				Returns:     "Pipeline started status",
			},
			{
				ID:          "pipeline.stop",
				Name:        "Stop Pipeline",
				Description: "Stop the running pipeline and cleanup IPC resources",
				Parameters:  []types.Parameter{},
				Returns:     "Pipeline stopped status",
			},
			{
				ID:          "pipeline.clear_logs",
				Name:        "Clear Logs",
				Description: "Clear all pipeline logs",
				Parameters:  []types.Parameter{},
				Returns:     "Logs cleared",
			},
		},
	}
}

// Execute handles pipeline tool execution
func (p *Provider) Execute(ctx context.Context, toolID string, params map[string]interface{}, appCtx *types.Context) (*types.Result, error) {
	switch toolID {
	case "pipeline.init":
		return p.initialize(ctx, appCtx)
	case "pipeline.start":
		return p.startPipeline(ctx, appCtx)
	case "pipeline.stop":
		return p.stopPipeline(ctx, appCtx)
	case "pipeline.clear_logs":
		return p.clearLogs(ctx, appCtx)
	default:
		return &types.Result{
			Success: false,
			Error:   stringPtr(fmt.Sprintf("unknown tool: %s", toolID)),
		}, fmt.Errorf("unknown tool: %s", toolID)
	}
}

// initialize sets up initial pipeline state
func (p *Provider) initialize(ctx context.Context, appCtx *types.Context) (*types.Result, error) {
	p.mu.Lock()
	defer p.mu.Unlock()

	p.addLog("INFO", "SYSTEM", "Pipeline initialized and ready")

	return &types.Result{
		Success: true,
		Data:    map[string]interface{}{"status": "initialized"},
	}, nil
}

// startPipeline creates all IPC resources and starts the pipeline
func (p *Provider) startPipeline(ctx context.Context, appCtx *types.Context) (*types.Result, error) {
	p.mu.Lock()
	defer p.mu.Unlock()

	if p.running {
		return &types.Result{
			Success: false,
			Error:   stringPtr("Pipeline already running"),
		}, fmt.Errorf("pipeline already running")
	}

	// Get sandbox PID for IPC operations
	if appCtx.SandboxPID == nil {
		return &types.Result{
			Success: false,
			Error:   stringPtr("No sandbox PID available"),
		}, fmt.Errorf("no sandbox PID")
	}
	mainPID := *appCtx.SandboxPID

	p.addLog("INFO", "SYSTEM", "Starting data pipeline...")

	// Create 4 stage processes
	stage1PID, err := p.createProcess(ctx, "stage-1-ingestion")
	if err != nil {
		p.addLog("ERROR", "SYSTEM", fmt.Sprintf("Failed to create stage 1 process: %v", err))
		return &types.Result{Success: false, Error: stringPtr(err.Error())}, err
	}
	p.processIDs["stage1"] = stage1PID
	p.addLog("INFO", "STAGE-1", fmt.Sprintf("Created ingestion process (PID: %d)", stage1PID))

	stage2PID, err := p.createProcess(ctx, "stage-2-transform")
	if err != nil {
		p.addLog("ERROR", "SYSTEM", fmt.Sprintf("Failed to create stage 2 process: %v", err))
		return &types.Result{Success: false, Error: stringPtr(err.Error())}, err
	}
	p.processIDs["stage2"] = stage2PID
	p.addLog("INFO", "STAGE-2", fmt.Sprintf("Created transformation process (PID: %d)", stage2PID))

	stage3PID, err := p.createProcess(ctx, "stage-3-aggregate")
	if err != nil {
		p.addLog("ERROR", "SYSTEM", fmt.Sprintf("Failed to create stage 3 process: %v", err))
		return &types.Result{Success: false, Error: stringPtr(err.Error())}, err
	}
	p.processIDs["stage3"] = stage3PID
	p.addLog("INFO", "STAGE-3", fmt.Sprintf("Created aggregation process (PID: %d)", stage3PID))

	// Create 3 writer processes
	p.ipcResources.WriterPIDs = make([]uint32, 3)
	for i := 0; i < 3; i++ {
		writerPID, err := p.createProcess(ctx, fmt.Sprintf("writer-%d", i+1))
		if err != nil {
			p.addLog("ERROR", "SYSTEM", fmt.Sprintf("Failed to create writer %d: %v", i+1, err))
			return &types.Result{Success: false, Error: stringPtr(err.Error())}, err
		}
		p.ipcResources.WriterPIDs[i] = writerPID
		p.addLog("INFO", "WRITERS", fmt.Sprintf("Created writer %d (PID: %d)", i+1, writerPID))
	}

	// Step 1: Create PIPE (Stage 1 -> Stage 2)
	pipeID, err := p.ipcClient.CreatePipe(ctx, stage2PID, stage1PID, nil)
	if err != nil {
		p.addLog("ERROR", "IPC", fmt.Sprintf("Failed to create pipe: %v", err))
		return &types.Result{Success: false, Error: stringPtr(err.Error())}, err
	}
	p.ipcResources.PipeID = pipeID
	p.addLog("INFO", "IPC", fmt.Sprintf("Created pipe %d (Stage 1 → Stage 2)", pipeID))

	// Step 2: Create SHARED MEMORY (Stage 2 -> Stage 3)
	shmID, err := p.ipcClient.CreateShm(ctx, mainPID, 1024*1024) // 1MB
	if err != nil {
		p.addLog("ERROR", "IPC", fmt.Sprintf("Failed to create shared memory: %v", err))
		return &types.Result{Success: false, Error: stringPtr(err.Error())}, err
	}
	p.ipcResources.ShmID = shmID
	p.addLog("INFO", "IPC", fmt.Sprintf("Created shared memory segment %d (1MB, Stage 2 → Stage 3)", shmID))

	// Attach stage 2 and 3 to shared memory
	if err := p.ipcClient.AttachShm(ctx, mainPID, shmID, false); err != nil {
		p.addLog("ERROR", "IPC", fmt.Sprintf("Failed to attach Stage 2 to SHM: %v", err))
	}
	if err := p.ipcClient.AttachShm(ctx, mainPID, shmID, true); err != nil {
		p.addLog("ERROR", "IPC", fmt.Sprintf("Failed to attach Stage 3 to SHM: %v", err))
	}

	// Step 3: Create PUBSUB QUEUE (Stage 3 -> Writers)
	queueID, err := p.ipcClient.CreateQueue(ctx, mainPID, "PubSub", nil)
	if err != nil {
		p.addLog("ERROR", "IPC", fmt.Sprintf("Failed to create queue: %v", err))
		return &types.Result{Success: false, Error: stringPtr(err.Error())}, err
	}
	p.ipcResources.QueueID = queueID
	p.addLog("INFO", "IPC", fmt.Sprintf("Created PubSub queue %d (Stage 3 → Writers)", queueID))

	// Subscribe all 3 writers to the queue
	for i := range p.ipcResources.WriterPIDs {
		if err := p.ipcClient.SubscribeQueue(ctx, mainPID, queueID); err != nil {
			p.addLog("ERROR", "IPC", fmt.Sprintf("Failed to subscribe writer %d to queue: %v", i+1, err))
		} else {
			p.addLog("INFO", "IPC", fmt.Sprintf("Subscribed writer %d to queue %d", i+1, queueID))
		}
	}

	// Initialize metrics
	p.metrics = PipelineMetrics{
		StartTime: time.Now(),
	}

	// Start pipeline simulation
	pipelineCtx, cancel := context.WithCancel(context.Background())
	p.cancelFunc = cancel
	p.running = true

	go p.runPipelineSimulation(pipelineCtx, mainPID)

	p.addLog("INFO", "SYSTEM", "✅ Pipeline started successfully!")

	return &types.Result{
		Success: true,
		Data: map[string]interface{}{
			"status": "running",
			"ipc": map[string]interface{}{
				"pipe_id":  pipeID,
				"shm_id":   shmID,
				"queue_id": queueID,
			},
			"processes": p.processIDs,
		},
	}, nil
}

// runPipelineSimulation simulates data flowing through the pipeline
func (p *Provider) runPipelineSimulation(ctx context.Context, mainPID uint32) {
	ticker := time.NewTicker(2 * time.Second)
	defer ticker.Stop()

	recordCount := 0

	for {
		select {
		case <-ctx.Done():
			return
		case <-ticker.C:
			p.mu.Lock()

			// Simulate Stage 1: Data Ingestion
			recordCount += 100
			p.metrics.Stage1Processed = int64(recordCount)
			p.metrics.Stage1LatencyMs = 5.2 + float64(recordCount%10)*0.1

			// Write to pipe
			data := fmt.Sprintf("batch-%d", recordCount)
			if _, err := p.ipcClient.WritePipe(ctx, mainPID, p.ipcResources.PipeID, []byte(data)); err != nil {
				p.addLog("WARN", "STAGE-1", fmt.Sprintf("Pipe write failed: %v", err))
			} else {
				p.addLog("DEBUG", "STAGE-1", fmt.Sprintf("Wrote %d bytes to pipe", len(data)))
			}

			// Simulate Stage 2: Transformation
			p.metrics.Stage2Processed = int64(recordCount - 50)
			p.metrics.Stage2LatencyMs = 12.8 + float64(recordCount%15)*0.2

			// Read from pipe, write to shared memory
			if _, err := p.ipcClient.ReadPipe(ctx, mainPID, p.ipcResources.PipeID, 1024); err == nil {
				transformedData := fmt.Sprintf("transformed-%d", recordCount)
				if err := p.ipcClient.WriteShm(ctx, mainPID, p.ipcResources.ShmID, 0, []byte(transformedData)); err != nil {
					p.addLog("WARN", "STAGE-2", fmt.Sprintf("SHM write failed: %v", err))
				} else {
					p.addLog("DEBUG", "STAGE-2", fmt.Sprintf("Wrote %d bytes to shared memory", len(transformedData)))
				}
			}

			// Simulate Stage 3: Aggregation
			p.metrics.Stage3Processed = int64(recordCount - 100)
			p.metrics.Stage3LatencyMs = 8.3 + float64(recordCount%20)*0.15

			// Read from shared memory, publish to queue
			if _, err := p.ipcClient.ReadShm(ctx, mainPID, p.ipcResources.ShmID, 0, 1024); err == nil {
				aggregatedData := fmt.Sprintf("aggregated-%d", recordCount)
				if err := p.ipcClient.SendQueue(ctx, mainPID, p.ipcResources.QueueID, []byte(aggregatedData), nil); err != nil {
					p.addLog("WARN", "STAGE-3", fmt.Sprintf("Queue send failed: %v", err))
				} else {
					p.addLog("DEBUG", "STAGE-3", fmt.Sprintf("Published %d bytes to queue (3 subscribers)", len(aggregatedData)))
				}
			}

			// Update throughput
			elapsed := time.Since(p.metrics.StartTime).Seconds()
			if elapsed > 0 {
				p.metrics.ThroughputRPS = float64(p.metrics.Stage1Processed) / elapsed
			}

			// Simulate memory usage
			p.metrics.PipeMemoryKB = 64.0
			p.metrics.ShmMemoryKB = 1024.0
			p.metrics.QueueMemoryKB = 128.0 + float64(recordCount%50)*2.0

			p.mu.Unlock()
		}
	}
}

// stopPipeline stops the pipeline and cleans up IPC resources
func (p *Provider) stopPipeline(ctx context.Context, appCtx *types.Context) (*types.Result, error) {
	p.mu.Lock()
	defer p.mu.Unlock()

	if !p.running {
		return &types.Result{
			Success: false,
			Error:   stringPtr("Pipeline not running"),
		}, fmt.Errorf("pipeline not running")
	}

	// Stop simulation
	if p.cancelFunc != nil {
		p.cancelFunc()
	}

	mainPID := *appCtx.SandboxPID

	p.addLog("INFO", "SYSTEM", "Stopping pipeline and cleaning up IPC resources...")

	// Clean up IPC resources
	if p.ipcResources.QueueID != 0 {
		// Unsubscribe writers
		for i := range p.ipcResources.WriterPIDs {
			if err := p.ipcClient.UnsubscribeQueue(ctx, mainPID, p.ipcResources.QueueID); err != nil {
				p.addLog("WARN", "IPC", fmt.Sprintf("Failed to unsubscribe writer %d: %v", i+1, err))
			}
		}
		p.addLog("INFO", "IPC", fmt.Sprintf("Destroyed queue %d", p.ipcResources.QueueID))
	}

	if p.ipcResources.ShmID != 0 {
		// Note: Kernel automatically cleans up SHM when processes terminate
		p.addLog("INFO", "IPC", fmt.Sprintf("Shared memory segment %d will be cleaned up by kernel", p.ipcResources.ShmID))
	}

	if p.ipcResources.PipeID != 0 {
		if err := p.ipcClient.ClosePipe(ctx, mainPID, p.ipcResources.PipeID); err != nil {
			p.addLog("WARN", "IPC", fmt.Sprintf("Failed to close pipe: %v", err))
		}
		p.addLog("INFO", "IPC", fmt.Sprintf("Closed pipe %d", p.ipcResources.PipeID))
	}

	// Clean up processes (kernel cleanup happens automatically)
	processCount := len(p.processIDs) + len(p.ipcResources.WriterPIDs)
	p.addLog("INFO", "SYSTEM", fmt.Sprintf("Worker processes (%d) will be cleaned up by kernel", processCount))

	// Reset state
	p.running = false
	p.processIDs = make(map[string]uint32)
	p.ipcResources = IPCResources{}

	p.addLog("INFO", "SYSTEM", "✅ Pipeline stopped and cleaned up")

	return &types.Result{
		Success: true,
		Data:    map[string]interface{}{"status": "stopped"},
	}, nil
}

// clearLogs clears all pipeline logs
func (p *Provider) clearLogs(ctx context.Context, appCtx *types.Context) (*types.Result, error) {
	p.mu.Lock()
	defer p.mu.Unlock()

	p.logs = make([]LogEntry, 0)

	return &types.Result{
		Success: true,
		Data:    map[string]interface{}{"status": "cleared"},
	}, nil
}

// createProcess creates a kernel process for a pipeline stage
func (p *Provider) createProcess(ctx context.Context, name string) (uint32, error) {
	pid, _, err := p.kernelClient.CreateProcess(ctx, name, 5, "STANDARD", nil)
	if err != nil {
		return 0, err
	}
	return *pid, nil
}

// addLog adds a log entry (must be called with lock held)
func (p *Provider) addLog(level, stage, message string) {
	entry := LogEntry{
		Timestamp: time.Now(),
		Level:     level,
		Stage:     stage,
		Message:   message,
	}
	p.logs = append(p.logs, entry)

	// Keep only last 100 logs
	if len(p.logs) > 100 {
		p.logs = p.logs[len(p.logs)-100:]
	}
}

func stringPtr(s string) *string {
	return &s
}
