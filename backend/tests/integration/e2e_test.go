//go:build integration
// +build integration

package integration

import (
	"context"
	"testing"
	"time"

	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
	"google.golang.org/grpc"
	"google.golang.org/grpc/credentials/insecure"

	aiPb "github.com/GriffinCanCode/AgentOS/backend/proto/ai"
	kernelPb "github.com/GriffinCanCode/AgentOS/backend/proto/kernel"
)

// TestEndToEndWorkflow tests the complete flow:
// Backend -> AI Service -> Backend -> Kernel
func TestEndToEndWorkflow(t *testing.T) {
	if testing.Short() {
		t.Skip("Skipping end-to-end test in short mode")
	}

	ctx, cancel := context.WithTimeout(context.Background(), 30*time.Second)
	defer cancel()

	// Setup kernel connection
	kernelConn, err := grpc.DialContext(
		ctx,
		"localhost:50051",
		grpc.WithTransportCredentials(insecure.NewCredentials()),
		grpc.WithBlock(),
	)
	require.NoError(t, err, "Failed to connect to kernel")
	defer kernelConn.Close()

	kernelClient := kernelPb.NewKernelServiceClient(kernelConn)

	// Setup AI service connection
	aiConn, err := grpc.DialContext(
		ctx,
		"localhost:50052",
		grpc.WithTransportCredentials(insecure.NewCredentials()),
		grpc.WithBlock(),
	)
	require.NoError(t, err, "Failed to connect to AI service")
	defer aiConn.Close()

	aiClient := aiPb.NewAIServiceClient(aiConn)

	t.Run("Kernel Process Creation", func(t *testing.T) {
		req := &kernelPb.CreateProcessRequest{
			Command: "test_process",
		}

		resp, err := kernelClient.CreateProcess(ctx, req)
		require.NoError(t, err)
		assert.NotNil(t, resp)
		assert.True(t, resp.Pid > 0, "Process ID should be positive")
	})

	t.Run("AI UI Generation", func(t *testing.T) {
		req := &aiPb.UIRequest{
			Message: "Create a simple counter app",
			Context: &aiPb.Context{
				SessionId: "test-session",
			},
		}

		resp, err := aiClient.GenerateUI(ctx, req)
		require.NoError(t, err)
		assert.NotNil(t, resp)
		assert.NotEmpty(t, resp.Blueprint, "Blueprint should not be empty")
	})

	t.Run("AI Chat Streaming", func(t *testing.T) {
		req := &aiPb.ChatRequest{
			Message: "What can you do?",
			Context: &aiPb.Context{
				SessionId: "test-session",
			},
		}

		stream, err := aiClient.StreamChat(ctx, req)
		require.NoError(t, err)

		tokenCount := 0
		for {
			token, err := stream.Recv()
			if err != nil {
				break
			}
			assert.NotNil(t, token)
			tokenCount++
		}

		assert.True(t, tokenCount > 0, "Should receive at least one token")
	})
}

// TestServiceResilience tests service behavior under failure conditions
func TestServiceResilience(t *testing.T) {
	if testing.Short() {
		t.Skip("Skipping resilience test in short mode")
	}

	ctx := context.Background()

	t.Run("Kernel Connection Timeout", func(t *testing.T) {
		timeoutCtx, cancel := context.WithTimeout(ctx, 100*time.Millisecond)
		defer cancel()

		conn, err := grpc.DialContext(
			timeoutCtx,
			"localhost:50051",
			grpc.WithTransportCredentials(insecure.NewCredentials()),
			grpc.WithBlock(),
		)

		if err != nil {
			// Expected if kernel is not running
			t.Logf("Kernel not available (expected): %v", err)
			return
		}
		defer conn.Close()
	})

	t.Run("AI Service Connection Timeout", func(t *testing.T) {
		timeoutCtx, cancel := context.WithTimeout(ctx, 100*time.Millisecond)
		defer cancel()

		conn, err := grpc.DialContext(
			timeoutCtx,
			"localhost:50052",
			grpc.WithTransportCredentials(insecure.NewCredentials()),
			grpc.WithBlock(),
		)

		if err != nil {
			// Expected if AI service is not running
			t.Logf("AI service not available (expected): %v", err)
			return
		}
		defer conn.Close()
	})
}

// TestConcurrentRequests tests system behavior under concurrent load
func TestConcurrentRequests(t *testing.T) {
	if testing.Short() {
		t.Skip("Skipping concurrent test in short mode")
	}

	ctx, cancel := context.WithTimeout(context.Background(), 60*time.Second)
	defer cancel()

	// Setup kernel connection
	kernelConn, err := grpc.DialContext(
		ctx,
		"localhost:50051",
		grpc.WithTransportCredentials(insecure.NewCredentials()),
		grpc.WithBlock(),
	)
	if err != nil {
		t.Skipf("Kernel not available: %v", err)
		return
	}
	defer kernelConn.Close()

	kernelClient := kernelPb.NewKernelServiceClient(kernelConn)

	// Run concurrent process creations
	const concurrentRequests = 10

	type result struct {
		pid uint32
		err error
	}

	results := make(chan result, concurrentRequests)

	for i := 0; i < concurrentRequests; i++ {
		go func(id int) {
			req := &kernelPb.CreateProcessRequest{
				Command: "concurrent_test",
			}

			resp, err := kernelClient.CreateProcess(ctx, req)
			if err != nil {
				results <- result{err: err}
				return
			}

			results <- result{pid: resp.Pid}
		}(i)
	}

	// Collect results
	successCount := 0
	for i := 0; i < concurrentRequests; i++ {
		r := <-results
		if r.err == nil {
			successCount++
			assert.True(t, r.pid > 0)
		}
	}

	assert.True(t, successCount > 0, "At least some concurrent requests should succeed")
	t.Logf("Concurrent requests: %d/%d succeeded", successCount, concurrentRequests)
}
