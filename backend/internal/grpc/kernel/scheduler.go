package kernel

import (
	"context"
	"encoding/json"
	"fmt"
	"time"

	pb "github.com/GriffinCanCode/AgentOS/backend/proto/kernel"
)

// ProcessSchedulerStats represents per-process scheduler statistics
type ProcessSchedulerStats struct {
	Pid            uint32 `json:"pid"`
	ScheduledCount uint64 `json:"scheduled_count"`
	CPUTimeMicros  uint64 `json:"cpu_time_micros"`
	Priority       uint32 `json:"priority"`
	State          string `json:"state"`
}

// buildSchedulerSyscall builds scheduler syscall requests
func (k *KernelClient) buildSchedulerSyscall(req *pb.SyscallRequest, syscallType string, params map[string]interface{}) {
	switch syscallType {
	case "schedule_next":
		req.Syscall = &pb.SyscallRequest_ScheduleNext{
			ScheduleNext: &pb.ScheduleNextCall{},
		}
	case "yield_process":
		req.Syscall = &pb.SyscallRequest_YieldProcess{
			YieldProcess: &pb.YieldProcessCall{},
		}
	case "get_current_scheduled":
		req.Syscall = &pb.SyscallRequest_GetCurrentScheduled{
			GetCurrentScheduled: &pb.GetCurrentScheduledCall{},
		}
	case "get_scheduler_stats":
		req.Syscall = &pb.SyscallRequest_GetSchedulerStats{
			GetSchedulerStats: &pb.GetSchedulerStatsCall{},
		}
	case "set_scheduling_policy":
		policy, _ := params["policy"].(string)
		req.Syscall = &pb.SyscallRequest_SetSchedulingPolicy{
			SetSchedulingPolicy: &pb.SetSchedulingPolicyCall{Policy: policy},
		}
	case "get_scheduling_policy":
		req.Syscall = &pb.SyscallRequest_GetSchedulingPolicy{
			GetSchedulingPolicy: &pb.GetSchedulingPolicyCall{},
		}
	case "set_time_quantum":
		quantumMicros, _ := params["quantum_micros"].(uint64)
		req.Syscall = &pb.SyscallRequest_SetTimeQuantum{
			SetTimeQuantum: &pb.SetTimeQuantumCall{QuantumMicros: quantumMicros},
		}
	case "get_time_quantum":
		req.Syscall = &pb.SyscallRequest_GetTimeQuantum{
			GetTimeQuantum: &pb.GetTimeQuantumCall{},
		}
	case "get_process_scheduler_stats":
		targetPid, _ := params["target_pid"].(uint32)
		req.Syscall = &pb.SyscallRequest_GetProcessSchedulerStats{
			GetProcessSchedulerStats: &pb.GetProcessSchedulerStatsCall{TargetPid: targetPid},
		}
	case "get_all_process_scheduler_stats":
		req.Syscall = &pb.SyscallRequest_GetAllProcessSchedulerStats{
			GetAllProcessSchedulerStats: &pb.GetAllProcessSchedulerStatsCall{},
		}
	case "boost_priority":
		targetPid, _ := params["target_pid"].(uint32)
		req.Syscall = &pb.SyscallRequest_BoostPriority{
			BoostPriority: &pb.BoostPriorityCall{TargetPid: targetPid},
		}
	case "lower_priority":
		targetPid, _ := params["target_pid"].(uint32)
		req.Syscall = &pb.SyscallRequest_LowerPriority{
			LowerPriority: &pb.LowerPriorityCall{TargetPid: targetPid},
		}
	}
}

// ScheduleNext schedules the next process
func (k *KernelClient) ScheduleNext(ctx context.Context) (*uint32, error) {
	ctx, cancel := context.WithTimeout(ctx, 5*time.Second)
	defer cancel()

	resp, err := k.client.ScheduleNext(ctx, &pb.ScheduleNextRequest{})
	if err != nil {
		return nil, fmt.Errorf("schedule next failed: %w", err)
	}

	if !resp.Success {
		return nil, fmt.Errorf("schedule next failed: %s", resp.Error)
	}

	return resp.NextPid, nil
}

// YieldProcess yields the CPU voluntarily
func (k *KernelClient) YieldProcess(ctx context.Context, pid uint32) error {
	req := &pb.SyscallRequest{
		Pid: pid,
		Syscall: &pb.SyscallRequest_YieldProcess{
			YieldProcess: &pb.YieldProcessCall{},
		},
	}

	resp, err := k.ExecuteSyscallRaw(ctx, req)
	if err != nil {
		return fmt.Errorf("failed to yield process: %w", err)
	}

	return extractError(resp)
}

// GetCurrentScheduled gets the currently scheduled process
func (k *KernelClient) GetCurrentScheduled(ctx context.Context, pid uint32) (uint32, error) {
	req := &pb.SyscallRequest{
		Pid: pid,
		Syscall: &pb.SyscallRequest_GetCurrentScheduled{
			GetCurrentScheduled: &pb.GetCurrentScheduledCall{},
		},
	}

	resp, err := k.ExecuteSyscallRaw(ctx, req)
	if err != nil {
		return 0, fmt.Errorf("failed to get current scheduled: %w", err)
	}

	if err := extractError(resp); err != nil {
		return 0, err
	}

	// Parse response data
	var currentPid uint32
	if data := extractData(resp); data != nil {
		if err := json.Unmarshal(data, &currentPid); err != nil {
			return 0, fmt.Errorf("failed to parse current scheduled PID: %w", err)
		}
	}

	return currentPid, nil
}

// GetSchedulerStats retrieves scheduler statistics
func (k *KernelClient) GetSchedulerStats(ctx context.Context) (*pb.SchedulerStats, error) {
	ctx, cancel := context.WithTimeout(ctx, 5*time.Second)
	defer cancel()

	resp, err := k.client.GetSchedulerStats(ctx, &pb.GetSchedulerStatsRequest{})
	if err != nil {
		return nil, fmt.Errorf("get scheduler stats failed: %w", err)
	}

	if !resp.Success {
		return nil, fmt.Errorf("get scheduler stats failed: %s", resp.Error)
	}

	return resp.Stats, nil
}

// SetSchedulingPolicy sets the scheduling policy
func (k *KernelClient) SetSchedulingPolicy(ctx context.Context, policy string) error {
	ctx, cancel := context.WithTimeout(ctx, 5*time.Second)
	defer cancel()

	resp, err := k.client.SetSchedulingPolicy(ctx, &pb.SetSchedulingPolicyRequest{
		Policy: policy,
	})
	if err != nil {
		return fmt.Errorf("set scheduling policy failed: %w", err)
	}

	if !resp.Success {
		return fmt.Errorf("set scheduling policy failed: %s", resp.Error)
	}

	return nil
}

// GetSchedulingPolicy retrieves the current scheduling policy
func (k *KernelClient) GetSchedulingPolicy(ctx context.Context, pid uint32) (string, error) {
	req := &pb.SyscallRequest{
		Pid: pid,
		Syscall: &pb.SyscallRequest_GetSchedulingPolicy{
			GetSchedulingPolicy: &pb.GetSchedulingPolicyCall{},
		},
	}

	resp, err := k.ExecuteSyscallRaw(ctx, req)
	if err != nil {
		return "", fmt.Errorf("failed to get scheduling policy: %w", err)
	}

	if err := extractError(resp); err != nil {
		return "", err
	}

	// Parse response data
	var policy string
	if data := extractData(resp); data != nil {
		if err := json.Unmarshal(data, &policy); err != nil {
			return "", fmt.Errorf("failed to parse scheduling policy: %w", err)
		}
	}

	return policy, nil
}

// SetTimeQuantum sets the scheduler time quantum
func (k *KernelClient) SetTimeQuantum(ctx context.Context, pid uint32, quantumMicros uint64) error {
	req := &pb.SyscallRequest{
		Pid: pid,
		Syscall: &pb.SyscallRequest_SetTimeQuantum{
			SetTimeQuantum: &pb.SetTimeQuantumCall{
				QuantumMicros: quantumMicros,
			},
		},
	}

	resp, err := k.ExecuteSyscallRaw(ctx, req)
	if err != nil {
		return fmt.Errorf("failed to set time quantum: %w", err)
	}

	return extractError(resp)
}

// GetTimeQuantum retrieves the current time quantum
func (k *KernelClient) GetTimeQuantum(ctx context.Context, pid uint32) (uint64, error) {
	req := &pb.SyscallRequest{
		Pid: pid,
		Syscall: &pb.SyscallRequest_GetTimeQuantum{
			GetTimeQuantum: &pb.GetTimeQuantumCall{},
		},
	}

	resp, err := k.ExecuteSyscallRaw(ctx, req)
	if err != nil {
		return 0, fmt.Errorf("failed to get time quantum: %w", err)
	}

	if err := extractError(resp); err != nil {
		return 0, err
	}

	// Parse response data
	var quantumMicros uint64
	if data := extractData(resp); data != nil {
		if err := json.Unmarshal(data, &quantumMicros); err != nil {
			return 0, fmt.Errorf("failed to parse time quantum: %w", err)
		}
	}

	return quantumMicros, nil
}

// GetProcessSchedulerStats retrieves scheduler statistics for a specific process
func (k *KernelClient) GetProcessSchedulerStats(ctx context.Context, pid uint32, targetPid uint32) (*ProcessSchedulerStats, error) {
	req := &pb.SyscallRequest{
		Pid: pid,
		Syscall: &pb.SyscallRequest_GetProcessSchedulerStats{
			GetProcessSchedulerStats: &pb.GetProcessSchedulerStatsCall{
				TargetPid: targetPid,
			},
		},
	}

	resp, err := k.ExecuteSyscallRaw(ctx, req)
	if err != nil {
		return nil, fmt.Errorf("failed to get process scheduler stats: %w", err)
	}

	if err := extractError(resp); err != nil {
		return nil, err
	}

	// Parse response data
	var stats ProcessSchedulerStats
	if data := extractData(resp); data != nil {
		if err := json.Unmarshal(data, &stats); err != nil {
			return nil, fmt.Errorf("failed to parse process scheduler stats: %w", err)
		}
	}

	return &stats, nil
}

// GetAllProcessSchedulerStats retrieves scheduler statistics for all processes
func (k *KernelClient) GetAllProcessSchedulerStats(ctx context.Context, pid uint32) (map[uint32]*ProcessSchedulerStats, error) {
	req := &pb.SyscallRequest{
		Pid: pid,
		Syscall: &pb.SyscallRequest_GetAllProcessSchedulerStats{
			GetAllProcessSchedulerStats: &pb.GetAllProcessSchedulerStatsCall{},
		},
	}

	resp, err := k.ExecuteSyscallRaw(ctx, req)
	if err != nil {
		return nil, fmt.Errorf("failed to get all process scheduler stats: %w", err)
	}

	if err := extractError(resp); err != nil {
		return nil, err
	}

	// Parse response data
	var stats map[uint32]*ProcessSchedulerStats
	if data := extractData(resp); data != nil {
		if err := json.Unmarshal(data, &stats); err != nil {
			return nil, fmt.Errorf("failed to parse all process scheduler stats: %w", err)
		}
	}

	return stats, nil
}

// BoostPriority increases process priority
func (k *KernelClient) BoostPriority(ctx context.Context, pid uint32, targetPid uint32) error {
	req := &pb.SyscallRequest{
		Pid: pid,
		Syscall: &pb.SyscallRequest_BoostPriority{
			BoostPriority: &pb.BoostPriorityCall{
				TargetPid: targetPid,
			},
		},
	}

	resp, err := k.ExecuteSyscallRaw(ctx, req)
	if err != nil {
		return fmt.Errorf("failed to boost priority: %w", err)
	}

	return extractError(resp)
}

// LowerPriority decreases process priority
func (k *KernelClient) LowerPriority(ctx context.Context, pid uint32, targetPid uint32) error {
	req := &pb.SyscallRequest{
		Pid: pid,
		Syscall: &pb.SyscallRequest_LowerPriority{
			LowerPriority: &pb.LowerPriorityCall{
				TargetPid: targetPid,
			},
		},
	}

	resp, err := k.ExecuteSyscallRaw(ctx, req)
	if err != nil {
		return fmt.Errorf("failed to lower priority: %w", err)
	}

	return extractError(resp)
}
