package grpc

import (
	"testing"

	pb "github.com/GriffinCanCode/AgentOS/backend/proto/kernel"
)

// TestSyscallRequestBuilding verifies syscall request construction
func TestSyscallRequestBuilding(t *testing.T) {
	tests := []struct {
		name        string
		syscallType string
		params      map[string]interface{}
		validate    func(*pb.SyscallRequest) error
	}{
		{
			name:        "read_file",
			syscallType: "read_file",
			params:      map[string]interface{}{"path": "/test/file.txt"},
			validate: func(req *pb.SyscallRequest) error {
				if req.Syscall == nil {
					t.Error("Syscall is nil")
				}
				return nil
			},
		},
		{
			name:        "write_file",
			syscallType: "write_file",
			params: map[string]interface{}{
				"path": "/test/file.txt",
				"data": []byte("test data"),
			},
			validate: func(req *pb.SyscallRequest) error {
				if req.Syscall == nil {
					t.Error("Syscall is nil")
				}
				return nil
			},
		},
		{
			name:        "delete_file",
			syscallType: "delete_file",
			params:      map[string]interface{}{"path": "/test/file.txt"},
			validate: func(req *pb.SyscallRequest) error {
				if req.Syscall == nil {
					t.Error("Syscall is nil")
				}
				return nil
			},
		},
		{
			name:        "file_exists",
			syscallType: "file_exists",
			params:      map[string]interface{}{"path": "/test/file.txt"},
			validate: func(req *pb.SyscallRequest) error {
				if req.Syscall == nil {
					t.Error("Syscall is nil")
				}
				return nil
			},
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			req := &pb.SyscallRequest{Pid: 1}

			switch tt.syscallType {
			case "read_file":
				path, _ := tt.params["path"].(string)
				req.Syscall = &pb.SyscallRequest_ReadFile{
					ReadFile: &pb.ReadFileCall{Path: path},
				}
			case "write_file":
				path, _ := tt.params["path"].(string)
				data, _ := tt.params["data"].([]byte)
				req.Syscall = &pb.SyscallRequest_WriteFile{
					WriteFile: &pb.WriteFileCall{Path: path, Data: data},
				}
			case "delete_file":
				path, _ := tt.params["path"].(string)
				req.Syscall = &pb.SyscallRequest_DeleteFile{
					DeleteFile: &pb.DeleteFileCall{Path: path},
				}
			case "file_exists":
				path, _ := tt.params["path"].(string)
				req.Syscall = &pb.SyscallRequest_FileExists{
					FileExists: &pb.FileExistsCall{Path: path},
				}
			}

			if err := tt.validate(req); err != nil {
				t.Errorf("Validation failed: %v", err)
			}
		})
	}
}
