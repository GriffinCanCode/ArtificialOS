package kernel

import (
	"context"
	"fmt"
	"time"

	pb "github.com/GriffinCanCode/AgentOS/backend/proto/kernel"
)

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
