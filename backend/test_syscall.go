package main

import (
	"context"
	"fmt"
	"time"
	"github.com/GriffinCanCode/AgentOS/backend/internal/grpc"
)

func main() {
	// Connect to kernel
	kernel, err := grpc.NewKernelClient("localhost:50051")
	if err != nil {
		fmt.Printf("Failed to connect: %v\n", err)
		return
	}
	defer kernel.Close()

	// Create storage process
	ctx := context.Background()
	pid, _, err := kernel.CreateProcess(ctx, "test-storage", 10, "PRIVILEGED", nil)
	if err != nil {
		fmt.Printf("Failed to create process: %v\n", err)
		return
	}
	fmt.Printf("Created process with PID: %d\n", *pid)

	// Try a simple file write
	params := map[string]interface{}{
		"path": "/tmp/ai-os-storage/test.txt",
		"data": []byte("hello world"),
	}

	fmt.Println("Attempting file write...")
	start := time.Now()
	_, err = kernel.ExecuteSyscall(ctx, *pid, "write_file", params)
	elapsed := time.Since(start)
	
	if err != nil {
		fmt.Printf("Write failed after %v: %v\n", elapsed, err)
	} else {
		fmt.Printf("Write succeeded in %v\n", elapsed)
	}
}
