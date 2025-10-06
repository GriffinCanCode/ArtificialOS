package kernel

import (
	pb "github.com/GriffinCanCode/AgentOS/backend/proto/kernel"
)

// buildSystemSyscall builds system info syscall requests
func (k *KernelClient) buildSystemSyscall(req *pb.SyscallRequest, syscallType string, params map[string]interface{}) {
	switch syscallType {
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
	case "set_env_var":
		key, _ := params["key"].(string)
		value, _ := params["value"].(string)
		req.Syscall = &pb.SyscallRequest_SetEnvVar{
			SetEnvVar: &pb.SetEnvVarCall{Key: key, Value: value},
		}
	}
}
