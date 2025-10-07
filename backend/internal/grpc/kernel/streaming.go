package kernel

import (
	"context"
	"fmt"
	"io"

	pb "github.com/GriffinCanCode/AgentOS/backend/proto/kernel"
)

// StreamFileRead streams a file read operation in chunks
func (k *KernelClient) StreamFileRead(ctx context.Context, pid uint32, path string, chunkSize uint32) (<-chan []byte, <-chan error) {
	dataChan := make(chan []byte, 10)
	errChan := make(chan error, 1)

	go func() {
		defer close(dataChan)
		defer close(errChan)

		stream, err := k.client.StreamSyscall(ctx)
		if err != nil {
			errChan <- fmt.Errorf("failed to create stream: %w", err)
			return
		}

		// Send read request
		req := &pb.StreamSyscallRequest{
			Pid: pid,
			Request: &pb.StreamSyscallRequest_Read{
				Read: &pb.StreamFileRead{
					Path:      path,
					ChunkSize: chunkSize,
				},
			},
		}

		if err := stream.Send(req); err != nil {
			errChan <- fmt.Errorf("failed to send request: %w", err)
			return
		}

		if err := stream.CloseSend(); err != nil {
			errChan <- fmt.Errorf("failed to close send: %w", err)
			return
		}

		// Receive chunks
		for {
			chunk, err := stream.Recv()
			if err == io.EOF {
				break
			}
			if err != nil {
				errChan <- fmt.Errorf("stream error: %w", err)
				return
			}

			switch c := chunk.Chunk.(type) {
			case *pb.StreamSyscallChunk_Data:
				dataChan <- c.Data
			case *pb.StreamSyscallChunk_Error:
				errChan <- fmt.Errorf("server error: %s", c.Error)
				return
			case *pb.StreamSyscallChunk_Complete:
				return
			}
		}
	}()

	return dataChan, errChan
}
