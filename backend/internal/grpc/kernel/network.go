package kernel

import (
	pb "github.com/GriffinCanCode/AgentOS/backend/proto/kernel"
)

// buildNetworkSyscall builds network syscall requests
func (k *KernelClient) buildNetworkSyscall(req *pb.SyscallRequest, syscallType string, params map[string]interface{}) {
	switch syscallType {
	case "network_request":
		url, _ := params["url"].(string)
		req.Syscall = &pb.SyscallRequest_NetworkRequest{
			NetworkRequest: &pb.NetworkRequestCall{Url: url},
		}
	}
}
