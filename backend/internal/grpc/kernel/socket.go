package kernel

import (
	pb "github.com/GriffinCanCode/AgentOS/backend/proto/kernel"
)

// buildSocketSyscall builds socket syscall requests
func (k *KernelClient) buildSocketSyscall(req *pb.SyscallRequest, syscallType string, params map[string]interface{}) {
	switch syscallType {
	case "socket":
		domain, _ := params["domain"].(uint32)
		socketType, _ := params["socket_type"].(uint32)
		protocol, _ := params["protocol"].(uint32)
		req.Syscall = &pb.SyscallRequest_Socket{
			Socket: &pb.SocketCall{
				Domain:     domain,
				SocketType: socketType,
				Protocol:   protocol,
			},
		}
	case "bind":
		sockfd, _ := params["sockfd"].(uint32)
		address, _ := params["address"].(string)
		req.Syscall = &pb.SyscallRequest_Bind{
			Bind: &pb.BindCall{
				Sockfd:  sockfd,
				Address: address,
			},
		}
	case "listen":
		sockfd, _ := params["sockfd"].(uint32)
		backlog, _ := params["backlog"].(uint32)
		req.Syscall = &pb.SyscallRequest_Listen{
			Listen: &pb.ListenCall{
				Sockfd:  sockfd,
				Backlog: backlog,
			},
		}
	case "accept":
		sockfd, _ := params["sockfd"].(uint32)
		req.Syscall = &pb.SyscallRequest_Accept{
			Accept: &pb.AcceptCall{
				Sockfd: sockfd,
			},
		}
	case "connect":
		sockfd, _ := params["sockfd"].(uint32)
		address, _ := params["address"].(string)
		req.Syscall = &pb.SyscallRequest_Connect{
			Connect: &pb.ConnectCall{
				Sockfd:  sockfd,
				Address: address,
			},
		}
	case "send":
		sockfd, _ := params["sockfd"].(uint32)
		data, _ := params["data"].([]byte)
		flags, _ := params["flags"].(uint32)
		req.Syscall = &pb.SyscallRequest_Send{
			Send: &pb.SendCall{
				Sockfd: sockfd,
				Data:   data,
				Flags:  flags,
			},
		}
	case "recv":
		sockfd, _ := params["sockfd"].(uint32)
		size, _ := params["size"].(uint32)
		flags, _ := params["flags"].(uint32)
		req.Syscall = &pb.SyscallRequest_Recv{
			Recv: &pb.RecvCall{
				Sockfd: sockfd,
				Size:   size,
				Flags:  flags,
			},
		}
	case "send_to":
		sockfd, _ := params["sockfd"].(uint32)
		data, _ := params["data"].([]byte)
		address, _ := params["address"].(string)
		flags, _ := params["flags"].(uint32)
		req.Syscall = &pb.SyscallRequest_SendTo{
			SendTo: &pb.SendToCall{
				Sockfd:  sockfd,
				Data:    data,
				Address: address,
				Flags:   flags,
			},
		}
	case "recv_from":
		sockfd, _ := params["sockfd"].(uint32)
		size, _ := params["size"].(uint32)
		flags, _ := params["flags"].(uint32)
		req.Syscall = &pb.SyscallRequest_RecvFrom{
			RecvFrom: &pb.RecvFromCall{
				Sockfd: sockfd,
				Size:   size,
				Flags:  flags,
			},
		}
	case "close_socket":
		sockfd, _ := params["sockfd"].(uint32)
		req.Syscall = &pb.SyscallRequest_CloseSocket{
			CloseSocket: &pb.CloseSocketCall{
				Sockfd: sockfd,
			},
		}
	case "set_sock_opt":
		sockfd, _ := params["sockfd"].(uint32)
		level, _ := params["level"].(uint32)
		optname, _ := params["optname"].(uint32)
		optval, _ := params["optval"].([]byte)
		req.Syscall = &pb.SyscallRequest_SetSockOpt{
			SetSockOpt: &pb.SetSockOptCall{
				Sockfd:  sockfd,
				Level:   level,
				Optname: optname,
				Optval:  optval,
			},
		}
	case "get_sock_opt":
		sockfd, _ := params["sockfd"].(uint32)
		level, _ := params["level"].(uint32)
		optname, _ := params["optname"].(uint32)
		req.Syscall = &pb.SyscallRequest_GetSockOpt{
			GetSockOpt: &pb.GetSockOptCall{
				Sockfd:  sockfd,
				Level:   level,
				Optname: optname,
			},
		}
	}
}
