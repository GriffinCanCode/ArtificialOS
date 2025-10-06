package kernel

import (
	pb "github.com/GriffinCanCode/AgentOS/backend/proto/kernel"
)

// buildIPCSyscall builds IPC syscall requests (pipes and shared memory)
func (k *KernelClient) buildIPCSyscall(req *pb.SyscallRequest, syscallType string, params map[string]interface{}) {
	switch syscallType {
	// IPC - Pipes
	case "create_pipe":
		readerPid, _ := params["reader_pid"].(uint32)
		writerPid, _ := params["writer_pid"].(uint32)
		createPipeCall := &pb.CreatePipeCall{
			ReaderPid: readerPid,
			WriterPid: writerPid,
		}
		if capacity, hasCapacity := params["capacity"].(uint32); hasCapacity {
			createPipeCall.Capacity = &capacity
		}
		req.Syscall = &pb.SyscallRequest_CreatePipe{
			CreatePipe: createPipeCall,
		}
	case "write_pipe":
		pipeId, _ := params["pipe_id"].(uint32)
		data, _ := params["data"].([]byte)
		req.Syscall = &pb.SyscallRequest_WritePipe{
			WritePipe: &pb.WritePipeCall{PipeId: pipeId, Data: data},
		}
	case "read_pipe":
		pipeId, _ := params["pipe_id"].(uint32)
		size, _ := params["size"].(uint32)
		req.Syscall = &pb.SyscallRequest_ReadPipe{
			ReadPipe: &pb.ReadPipeCall{PipeId: pipeId, Size: size},
		}
	case "close_pipe":
		pipeId, _ := params["pipe_id"].(uint32)
		req.Syscall = &pb.SyscallRequest_ClosePipe{
			ClosePipe: &pb.ClosePipeCall{PipeId: pipeId},
		}
	case "destroy_pipe":
		pipeId, _ := params["pipe_id"].(uint32)
		req.Syscall = &pb.SyscallRequest_DestroyPipe{
			DestroyPipe: &pb.DestroyPipeCall{PipeId: pipeId},
		}
	case "pipe_stats":
		pipeId, _ := params["pipe_id"].(uint32)
		req.Syscall = &pb.SyscallRequest_PipeStats{
			PipeStats: &pb.PipeStatsCall{PipeId: pipeId},
		}
	// IPC - Shared Memory
	case "create_shm":
		size, _ := params["size"].(uint32)
		req.Syscall = &pb.SyscallRequest_CreateShm{
			CreateShm: &pb.CreateShmCall{Size: size},
		}
	case "attach_shm":
		segmentId, _ := params["segment_id"].(uint32)
		readOnly, _ := params["read_only"].(bool)
		req.Syscall = &pb.SyscallRequest_AttachShm{
			AttachShm: &pb.AttachShmCall{SegmentId: segmentId, ReadOnly: readOnly},
		}
	case "detach_shm":
		segmentId, _ := params["segment_id"].(uint32)
		req.Syscall = &pb.SyscallRequest_DetachShm{
			DetachShm: &pb.DetachShmCall{SegmentId: segmentId},
		}
	case "write_shm":
		segmentId, _ := params["segment_id"].(uint32)
		offset, _ := params["offset"].(uint32)
		data, _ := params["data"].([]byte)
		req.Syscall = &pb.SyscallRequest_WriteShm{
			WriteShm: &pb.WriteShmCall{SegmentId: segmentId, Offset: offset, Data: data},
		}
	case "read_shm":
		segmentId, _ := params["segment_id"].(uint32)
		offset, _ := params["offset"].(uint32)
		size, _ := params["size"].(uint32)
		req.Syscall = &pb.SyscallRequest_ReadShm{
			ReadShm: &pb.ReadShmCall{SegmentId: segmentId, Offset: offset, Size: size},
		}
	case "destroy_shm":
		segmentId, _ := params["segment_id"].(uint32)
		req.Syscall = &pb.SyscallRequest_DestroyShm{
			DestroyShm: &pb.DestroyShmCall{SegmentId: segmentId},
		}
	case "shm_stats":
		segmentId, _ := params["segment_id"].(uint32)
		req.Syscall = &pb.SyscallRequest_ShmStats{
			ShmStats: &pb.ShmStatsCall{SegmentId: segmentId},
		}
	}
}
