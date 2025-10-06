package kernel

import (
	pb "github.com/GriffinCanCode/AgentOS/backend/proto/kernel"
)

// buildFilesystemSyscall builds filesystem syscall requests
func (k *KernelClient) buildFilesystemSyscall(req *pb.SyscallRequest, syscallType string, params map[string]interface{}) {
	switch syscallType {
	case "read_file":
		path, _ := params["path"].(string)
		req.Syscall = &pb.SyscallRequest_ReadFile{
			ReadFile: &pb.ReadFileCall{Path: path},
		}
	case "write_file":
		path, _ := params["path"].(string)
		data, _ := params["data"].([]byte)
		req.Syscall = &pb.SyscallRequest_WriteFile{
			WriteFile: &pb.WriteFileCall{Path: path, Data: data},
		}
	case "create_file":
		path, _ := params["path"].(string)
		req.Syscall = &pb.SyscallRequest_CreateFile{
			CreateFile: &pb.CreateFileCall{Path: path},
		}
	case "delete_file":
		path, _ := params["path"].(string)
		req.Syscall = &pb.SyscallRequest_DeleteFile{
			DeleteFile: &pb.DeleteFileCall{Path: path},
		}
	case "list_directory":
		path, _ := params["path"].(string)
		req.Syscall = &pb.SyscallRequest_ListDirectory{
			ListDirectory: &pb.ListDirectoryCall{Path: path},
		}
	case "file_exists":
		path, _ := params["path"].(string)
		req.Syscall = &pb.SyscallRequest_FileExists{
			FileExists: &pb.FileExistsCall{Path: path},
		}
	case "file_stat":
		path, _ := params["path"].(string)
		req.Syscall = &pb.SyscallRequest_FileStat{
			FileStat: &pb.FileStatCall{Path: path},
		}
	case "move_file":
		source, _ := params["source"].(string)
		destination, _ := params["destination"].(string)
		req.Syscall = &pb.SyscallRequest_MoveFile{
			MoveFile: &pb.MoveFileCall{Source: source, Destination: destination},
		}
	case "copy_file":
		source, _ := params["source"].(string)
		destination, _ := params["destination"].(string)
		req.Syscall = &pb.SyscallRequest_CopyFile{
			CopyFile: &pb.CopyFileCall{Source: source, Destination: destination},
		}
	case "create_directory":
		path, _ := params["path"].(string)
		req.Syscall = &pb.SyscallRequest_CreateDirectory{
			CreateDirectory: &pb.CreateDirectoryCall{Path: path},
		}
	case "remove_directory":
		path, _ := params["path"].(string)
		req.Syscall = &pb.SyscallRequest_RemoveDirectory{
			RemoveDirectory: &pb.RemoveDirectoryCall{Path: path},
		}
	case "get_working_directory":
		req.Syscall = &pb.SyscallRequest_GetWorkingDirectory{
			GetWorkingDirectory: &pb.GetWorkingDirectoryCall{},
		}
	case "set_working_directory":
		path, _ := params["path"].(string)
		req.Syscall = &pb.SyscallRequest_SetWorkingDirectory{
			SetWorkingDirectory: &pb.SetWorkingDirectoryCall{Path: path},
		}
	case "truncate_file":
		path, _ := params["path"].(string)
		size, _ := params["size"].(uint64)
		req.Syscall = &pb.SyscallRequest_TruncateFile{
			TruncateFile: &pb.TruncateFileCall{Path: path, Size: size},
		}
	}
}
