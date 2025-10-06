package kernel

import (
	pb "github.com/GriffinCanCode/AgentOS/backend/proto/kernel"
)

// buildMemorySyscall builds memory syscall requests
func (k *KernelClient) buildMemorySyscall(req *pb.SyscallRequest, syscallType string, params map[string]interface{}) {
	switch syscallType {
	case "get_memory_stats":
		req.Syscall = &pb.SyscallRequest_GetMemoryStats{
			GetMemoryStats: &pb.GetMemoryStatsCall{},
		}
	case "get_process_memory_stats":
		targetPid, _ := params["target_pid"].(uint32)
		req.Syscall = &pb.SyscallRequest_GetProcessMemoryStats{
			GetProcessMemoryStats: &pb.GetProcessMemoryStatsCall{TargetPid: targetPid},
		}
	case "trigger_gc":
		targetPid, hasPid := params["target_pid"].(uint32)
		gcCall := &pb.TriggerGCCall{}
		if hasPid {
			gcCall.TargetPid = &targetPid
		}
		req.Syscall = &pb.SyscallRequest_TriggerGc{
			TriggerGc: gcCall,
		}
	}
}
