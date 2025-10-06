package kernel

import (
	pb "github.com/GriffinCanCode/AgentOS/backend/proto/kernel"
)

// buildProcessSyscall builds process syscall requests
func (k *KernelClient) buildProcessSyscall(req *pb.SyscallRequest, syscallType string, params map[string]interface{}) {
	switch syscallType {
	case "spawn_process":
		command, _ := params["command"].(string)
		args, _ := params["args"].([]string)
		req.Syscall = &pb.SyscallRequest_SpawnProcess{
			SpawnProcess: &pb.SpawnProcessCall{Command: command, Args: args},
		}
	case "kill_process":
		targetPid, _ := params["target_pid"].(uint32)
		req.Syscall = &pb.SyscallRequest_KillProcess{
			KillProcess: &pb.KillProcessCall{TargetPid: targetPid},
		}
	case "get_process_info":
		targetPid, _ := params["target_pid"].(uint32)
		req.Syscall = &pb.SyscallRequest_GetProcessInfo{
			GetProcessInfo: &pb.GetProcessInfoCall{TargetPid: targetPid},
		}
	case "get_process_list":
		req.Syscall = &pb.SyscallRequest_GetProcessList{
			GetProcessList: &pb.GetProcessListCall{},
		}
	case "set_process_priority":
		targetPid, _ := params["target_pid"].(uint32)
		priority, _ := params["priority"].(uint32)
		req.Syscall = &pb.SyscallRequest_SetProcessPriority{
			SetProcessPriority: &pb.SetProcessPriorityCall{TargetPid: targetPid, Priority: priority},
		}
	case "get_process_state":
		targetPid, _ := params["target_pid"].(uint32)
		req.Syscall = &pb.SyscallRequest_GetProcessState{
			GetProcessState: &pb.GetProcessStateCall{TargetPid: targetPid},
		}
	case "get_process_stats":
		targetPid, _ := params["target_pid"].(uint32)
		req.Syscall = &pb.SyscallRequest_GetProcessStats{
			GetProcessStats: &pb.GetProcessStatsCall{TargetPid: targetPid},
		}
	case "wait_process":
		targetPid, _ := params["target_pid"].(uint32)
		timeoutMs, hasTimeout := params["timeout_ms"].(uint64)
		waitCall := &pb.WaitProcessCall{TargetPid: targetPid}
		if hasTimeout {
			waitCall.TimeoutMs = &timeoutMs
		}
		req.Syscall = &pb.SyscallRequest_WaitProcess{
			WaitProcess: waitCall,
		}
	}
}
