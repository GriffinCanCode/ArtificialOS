package kernel

import (
	pb "github.com/GriffinCanCode/AgentOS/backend/proto/kernel"
)

// buildSignalSyscall builds signal syscall requests
func (k *KernelClient) buildSignalSyscall(req *pb.SyscallRequest, syscallType string, params map[string]interface{}) {
	switch syscallType {
	case "send_signal":
		targetPid, _ := params["target_pid"].(uint32)
		signal, _ := params["signal"].(uint32)
		req.Syscall = &pb.SyscallRequest_SendSignal{
			SendSignal: &pb.SendSignalCall{TargetPid: targetPid, Signal: signal},
		}
	}
}
