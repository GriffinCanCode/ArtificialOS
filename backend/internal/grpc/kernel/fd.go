package kernel

import (
	pb "github.com/GriffinCanCode/AgentOS/backend/proto/kernel"
)

// buildFdSyscall builds file descriptor syscall requests
func (k *KernelClient) buildFdSyscall(req *pb.SyscallRequest, syscallType string, params map[string]interface{}) {
	switch syscallType {
	case "open":
		path, _ := params["path"].(string)
		flags, _ := params["flags"].(uint32)
		mode, _ := params["mode"].(uint32)
		req.Syscall = &pb.SyscallRequest_Open{
			Open: &pb.OpenCall{
				Path:  path,
				Flags: flags,
				Mode:  mode,
			},
		}
	case "close":
		fd, _ := params["fd"].(uint32)
		req.Syscall = &pb.SyscallRequest_Close{
			Close: &pb.CloseCall{
				Fd: fd,
			},
		}
	case "dup":
		fd, _ := params["fd"].(uint32)
		req.Syscall = &pb.SyscallRequest_Dup{
			Dup: &pb.DupCall{
				Fd: fd,
			},
		}
	case "dup2":
		oldfd, _ := params["oldfd"].(uint32)
		newfd, _ := params["newfd"].(uint32)
		req.Syscall = &pb.SyscallRequest_Dup2{
			Dup2: &pb.Dup2Call{
				Oldfd: oldfd,
				Newfd: newfd,
			},
		}
	case "lseek":
		fd, _ := params["fd"].(uint32)
		offset, _ := params["offset"].(int64)
		whence, _ := params["whence"].(uint32)
		req.Syscall = &pb.SyscallRequest_Lseek{
			Lseek: &pb.LseekCall{
				Fd:     fd,
				Offset: offset,
				Whence: whence,
			},
		}
	case "fcntl":
		fd, _ := params["fd"].(uint32)
		cmd, _ := params["cmd"].(uint32)
		arg, _ := params["arg"].(uint32)
		req.Syscall = &pb.SyscallRequest_Fcntl{
			Fcntl: &pb.FcntlCall{
				Fd:  fd,
				Cmd: cmd,
				Arg: arg,
			},
		}
	}
}
