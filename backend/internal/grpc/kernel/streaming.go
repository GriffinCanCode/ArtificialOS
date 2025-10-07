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

// StreamFileWriteResult holds the result of a streaming write operation
type StreamFileWriteResult struct {
	TotalBytes uint64
	Error      error
}

// StreamFileWrite streams a file write operation in chunks
// Returns channels for sending data and receiving completion/errors
func (k *KernelClient) StreamFileWrite(ctx context.Context, pid uint32, path string) (chan<- []byte, <-chan StreamFileWriteResult) {
	dataChan := make(chan []byte, 10)
	resultChan := make(chan StreamFileWriteResult, 1)

	go func() {
		defer close(resultChan)

		stream, err := k.client.StreamSyscall(ctx)
		if err != nil {
			resultChan <- StreamFileWriteResult{
				Error: fmt.Errorf("failed to create stream: %w", err),
			}
			return
		}

		var totalBytes uint64

		// Send chunks from dataChan
		for chunk := range dataChan {
			req := &pb.StreamSyscallRequest{
				Pid: pid,
				Request: &pb.StreamSyscallRequest_Write{
					Write: &pb.StreamFileWrite{
						Path:   path,
						Chunk:  chunk,
						Finish: false,
					},
				},
			}

			if err := stream.Send(req); err != nil {
				resultChan <- StreamFileWriteResult{
					TotalBytes: totalBytes,
					Error:      fmt.Errorf("failed to send chunk: %w", err),
				}
				return
			}

			totalBytes += uint64(len(chunk))
		}

		// Send final chunk to signal completion
		req := &pb.StreamSyscallRequest{
			Pid: pid,
			Request: &pb.StreamSyscallRequest_Write{
				Write: &pb.StreamFileWrite{
					Path:   path,
					Chunk:  nil,
					Finish: true,
				},
			},
		}

		if err := stream.Send(req); err != nil {
			resultChan <- StreamFileWriteResult{
				TotalBytes: totalBytes,
				Error:      fmt.Errorf("failed to send final chunk: %w", err),
			}
			return
		}

		if err := stream.CloseSend(); err != nil {
			resultChan <- StreamFileWriteResult{
				TotalBytes: totalBytes,
				Error:      fmt.Errorf("failed to close send: %w", err),
			}
			return
		}

		// Wait for server confirmation
		for {
			chunk, err := stream.Recv()
			if err == io.EOF {
				break
			}
			if err != nil {
				resultChan <- StreamFileWriteResult{
					TotalBytes: totalBytes,
					Error:      fmt.Errorf("stream error: %w", err),
				}
				return
			}

			switch c := chunk.Chunk.(type) {
			case *pb.StreamSyscallChunk_Error:
				resultChan <- StreamFileWriteResult{
					TotalBytes: totalBytes,
					Error:      fmt.Errorf("server error: %s", c.Error),
				}
				return
			case *pb.StreamSyscallChunk_Complete:
				resultChan <- StreamFileWriteResult{
					TotalBytes: c.Complete.TotalBytes,
					Error:      nil,
				}
				return
			}
		}

		// Success
		resultChan <- StreamFileWriteResult{
			TotalBytes: totalBytes,
			Error:      nil,
		}
	}()

	return dataChan, resultChan
}

// StreamFileWriteFrom is a convenience function that writes from an io.Reader
func (k *KernelClient) StreamFileWriteFrom(ctx context.Context, pid uint32, path string, reader io.Reader, chunkSize int) error {
	if chunkSize <= 0 {
		chunkSize = 64 * 1024 // Default 64KB chunks
	}

	dataChan, resultChan := k.StreamFileWrite(ctx, pid, path)

	// Send data from reader in chunks
	go func() {
		defer close(dataChan)

		buffer := make([]byte, chunkSize)
		for {
			n, err := reader.Read(buffer)
			if err == io.EOF {
				break
			}
			if err != nil {
				// Can't send error through data channel, just stop
				return
			}

			if n > 0 {
				// Make a copy since buffer will be reused
				chunk := make([]byte, n)
				copy(chunk, buffer[:n])
				dataChan <- chunk
			}
		}
	}()

	// Wait for result
	result := <-resultChan
	return result.Error
}
