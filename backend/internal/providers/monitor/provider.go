package monitor

import (
	"context"
	"encoding/json"
	"fmt"
	"runtime"
	"time"

	"github.com/GriffinCanCode/AgentOS/backend/internal/shared/types"
)

// KernelClient interface for syscall operations
type KernelClient interface {
	ExecuteSyscall(ctx context.Context, pid uint32, syscallType string, params map[string]interface{}) ([]byte, error)
}

// Provider implements system monitoring
type Provider struct {
	kernel     KernelClient
	storagePID uint32
}

// SystemStats represents system resource usage
type SystemStats struct {
	Timestamp  int64       `json:"timestamp"`
	Memory     MemoryStats `json:"memory"`
	CPU        CPUStats    `json:"cpu"`
	Processes  int         `json:"processes"`
	Goroutines int         `json:"goroutines"`
	Uptime     float64     `json:"uptime_seconds"`
}

// MemoryStats represents memory usage
type MemoryStats struct {
	Allocated    uint64  `json:"allocated_bytes"`
	Total        uint64  `json:"total_bytes"`
	System       uint64  `json:"system_bytes"`
	GCPauses     uint32  `json:"gc_pauses"`
	NumGC        uint32  `json:"num_gc"`
	UsagePercent float64 `json:"usage_percent"`
}

// CPUStats represents CPU usage
type CPUStats struct {
	Cores   int     `json:"cores"`
	Threads int     `json:"threads"`
	Usage   float64 `json:"usage_percent"`
}

// ProcessInfo represents a process
type ProcessInfo struct {
	PID        uint32  `json:"pid"`
	Name       string  `json:"name"`
	State      string  `json:"state"`
	Memory     uint64  `json:"memory_bytes"`
	CPUPercent float64 `json:"cpu_percent"`
	CreatedAt  int64   `json:"created_at"`
}

// NetworkStats represents network usage
type NetworkStats struct {
	BytesSent     uint64 `json:"bytes_sent"`
	BytesReceived uint64 `json:"bytes_received"`
	PacketsSent   uint64 `json:"packets_sent"`
	PacketsRecv   uint64 `json:"packets_received"`
}

// NewProvider creates a monitor provider
func NewProvider(kernel KernelClient, storagePID uint32) *Provider {
	return &Provider{
		kernel:     kernel,
		storagePID: storagePID,
	}
}

// Definition returns service metadata
func (m *Provider) Definition() types.Service {
	return types.Service{
		ID:          "monitor",
		Name:        "System Monitor",
		Description: "Real-time system monitoring and statistics",
		Category:    types.CategorySystem,
		Capabilities: []string{
			"system_stats",
			"process_list",
			"network_stats",
			"disk_stats",
		},
		Tools: []types.Tool{
			{
				ID:          "monitor.system",
				Name:        "Get System Stats",
				Description: "Get current system resource usage statistics",
				Parameters:  []types.Parameter{},
				Returns:     "SystemStats",
			},
			{
				ID:          "monitor.processes",
				Name:        "List Processes",
				Description: "List all running processes with stats",
				Parameters:  []types.Parameter{},
				Returns:     "array",
			},
			{
				ID:          "monitor.network",
				Name:        "Get Network Stats",
				Description: "Get network usage statistics",
				Parameters:  []types.Parameter{},
				Returns:     "NetworkStats",
			},
			{
				ID:          "monitor.memory",
				Name:        "Get Memory Details",
				Description: "Get detailed memory usage breakdown",
				Parameters:  []types.Parameter{},
				Returns:     "object",
			},
			{
				ID:          "monitor.cpu",
				Name:        "Get CPU Details",
				Description: "Get detailed CPU usage information",
				Parameters:  []types.Parameter{},
				Returns:     "object",
			},
		},
	}
}

// Execute runs a monitor operation
func (m *Provider) Execute(ctx context.Context, toolID string, params map[string]interface{}, appCtx *types.Context) (*types.Result, error) {
	switch toolID {
	case "monitor.system":
		return m.systemStats(ctx)
	case "monitor.processes":
		return m.processList(ctx)
	case "monitor.network":
		return m.networkStats(ctx)
	case "monitor.memory":
		return m.memoryDetails(ctx)
	case "monitor.cpu":
		return m.cpuDetails(ctx)
	default:
		return failure(fmt.Sprintf("unknown tool: %s", toolID))
	}
}

func (m *Provider) systemStats(ctx context.Context) (*types.Result, error) {
	var memStats runtime.MemStats
	runtime.ReadMemStats(&memStats)

	// Get process count from kernel
	processCount := 0
	if m.kernel != nil {
		resp, err := m.kernel.ExecuteSyscall(ctx, m.storagePID, "get_process_list", map[string]interface{}{})
		if err == nil {
			var result map[string]interface{}
			if err := json.Unmarshal(resp, &result); err == nil {
				if processes, ok := result["processes"].([]interface{}); ok {
					processCount = len(processes)
				}
			}
		}
	}

	stats := SystemStats{
		Timestamp: time.Now().Unix(),
		Memory: MemoryStats{
			Allocated:    memStats.Alloc,
			Total:        memStats.TotalAlloc,
			System:       memStats.Sys,
			GCPauses:     memStats.PauseTotalNs / 1000000, // Convert to ms
			NumGC:        memStats.NumGC,
			UsagePercent: float64(memStats.Alloc) / float64(memStats.Sys) * 100,
		},
		CPU: CPUStats{
			Cores:   runtime.NumCPU(),
			Threads: runtime.GOMAXPROCS(0),
			Usage:   0.0, // Would need OS-specific implementation for real CPU usage
		},
		Processes:  processCount,
		Goroutines: runtime.NumGoroutine(),
		Uptime:     0.0, // Would need to track server start time
	}

	return success(map[string]interface{}{
		"timestamp":  stats.Timestamp,
		"memory":     stats.Memory,
		"cpu":        stats.CPU,
		"processes":  stats.Processes,
		"goroutines": stats.Goroutines,
		"uptime":     stats.Uptime,
	})
}

func (m *Provider) processList(ctx context.Context) (*types.Result, error) {
	if m.kernel == nil {
		return success(map[string]interface{}{"processes": []ProcessInfo{}})
	}

	// Get process list from kernel
	resp, err := m.kernel.ExecuteSyscall(ctx, m.storagePID, "get_process_list", map[string]interface{}{})
	if err != nil {
		return failure(fmt.Sprintf("failed to get process list: %v", err))
	}

	var result map[string]interface{}
	if err := json.Unmarshal(resp, &result); err != nil {
		return failure(fmt.Sprintf("failed to parse process list: %v", err))
	}

	// Convert to ProcessInfo structures
	processes := []ProcessInfo{}
	if processList, ok := result["processes"].([]interface{}); ok {
		for _, p := range processList {
			if procMap, ok := p.(map[string]interface{}); ok {
				process := ProcessInfo{
					PID:   uint32(getFloat64(procMap, "pid")),
					Name:  getString(procMap, "name"),
					State: getString(procMap, "state"),
				}
				processes = append(processes, process)
			}
		}
	}

	return success(map[string]interface{}{
		"processes": processes,
		"count":     len(processes),
	})
}

func (m *Provider) networkStats(ctx context.Context) (*types.Result, error) {
	// Placeholder - would integrate with kernel network stats
	stats := NetworkStats{
		BytesSent:     0,
		BytesReceived: 0,
		PacketsSent:   0,
		PacketsRecv:   0,
	}

	return success(map[string]interface{}{
		"bytes_sent":       stats.BytesSent,
		"bytes_received":   stats.BytesReceived,
		"packets_sent":     stats.PacketsSent,
		"packets_received": stats.PacketsRecv,
	})
}

func (m *Provider) memoryDetails(ctx context.Context) (*types.Result, error) {
	var memStats runtime.MemStats
	runtime.ReadMemStats(&memStats)

	return success(map[string]interface{}{
		"alloc":          memStats.Alloc,
		"total_alloc":    memStats.TotalAlloc,
		"sys":            memStats.Sys,
		"lookups":        memStats.Lookups,
		"mallocs":        memStats.Mallocs,
		"frees":          memStats.Frees,
		"heap_alloc":     memStats.HeapAlloc,
		"heap_sys":       memStats.HeapSys,
		"heap_idle":      memStats.HeapIdle,
		"heap_inuse":     memStats.HeapInuse,
		"heap_released":  memStats.HeapReleased,
		"heap_objects":   memStats.HeapObjects,
		"stack_inuse":    memStats.StackInuse,
		"stack_sys":      memStats.StackSys,
		"gc_sys":         memStats.GCSys,
		"num_gc":         memStats.NumGC,
		"pause_total_ns": memStats.PauseTotalNs,
	})
}

func (m *Provider) cpuDetails(ctx context.Context) (*types.Result, error) {
	return success(map[string]interface{}{
		"num_cpu":       runtime.NumCPU(),
		"gomaxprocs":    runtime.GOMAXPROCS(0),
		"num_goroutine": runtime.NumGoroutine(),
		"num_cgo_call":  runtime.NumCgoCall(),
	})
}

// Helper functions
func success(data map[string]interface{}) (*types.Result, error) {
	return &types.Result{Success: true, Data: data}, nil
}

func failure(message string) (*types.Result, error) {
	errMsg := message
	return &types.Result{Success: false, Error: &errMsg}, nil
}

func getString(m map[string]interface{}, key string) string {
	if val, ok := m[key].(string); ok {
		return val
	}
	return ""
}

func getFloat64(m map[string]interface{}, key string) float64 {
	if val, ok := m[key].(float64); ok {
		return val
	}
	return 0
}
