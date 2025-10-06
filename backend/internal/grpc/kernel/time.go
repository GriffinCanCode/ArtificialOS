package kernel

import (
	pb "github.com/GriffinCanCode/AgentOS/backend/proto/kernel"
)

// buildTimeSyscall builds time syscall requests
func (k *KernelClient) buildTimeSyscall(req *pb.SyscallRequest, syscallType string, params map[string]interface{}) {
	switch syscallType {
	case "sleep":
		durationMs, _ := params["duration_ms"].(uint64)
		req.Syscall = &pb.SyscallRequest_Sleep{
			Sleep: &pb.SleepCall{DurationMs: durationMs},
		}
	case "get_uptime":
		req.Syscall = &pb.SyscallRequest_GetUptime{
			GetUptime: &pb.GetUptimeCall{},
		}
	}
}
