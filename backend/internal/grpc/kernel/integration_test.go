//go:build integration
// +build integration

package kernel

import (
	"context"
	"testing"
	"time"

	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

// TestSignalIntegration tests the complete signal system integration
func TestSignalIntegration(t *testing.T) {
	if testing.Short() {
		t.Skip("Skipping integration test in short mode")
	}

	// Connect to kernel
	client, err := New("127.0.0.1:50051")
	require.NoError(t, err, "Failed to create kernel client")
	defer client.Close()

	ctx := context.Background()

	// Create two test processes
	pid1, _, err := client.CreateProcess(ctx, "signal-test-1", 5, "STANDARD", nil)
	require.NoError(t, err, "Failed to create process 1")
	require.NotNil(t, pid1)

	pid2, _, err := client.CreateProcess(ctx, "signal-test-2", 5, "STANDARD", nil)
	require.NoError(t, err, "Failed to create process 2")
	require.NotNil(t, pid2)

	t.Run("RegisterAndSendSignal", func(t *testing.T) {
		// Register a signal handler for SIGUSR1
		err := client.RegisterSignalHandler(ctx, *pid1, SIGUSR1, 1)
		assert.NoError(t, err, "Failed to register signal handler")

		// Send SIGUSR1 from pid2 to pid1
		err = client.SendSignal(ctx, *pid2, *pid1, SIGUSR1)
		assert.NoError(t, err, "Failed to send signal")

		// Check pending signals
		pending, err := client.GetPendingSignals(ctx, *pid1)
		assert.NoError(t, err, "Failed to get pending signals")
		assert.Contains(t, pending, uint32(SIGUSR1), "SIGUSR1 should be pending")
	})

	t.Run("BlockAndUnblockSignal", func(t *testing.T) {
		// Block SIGUSR2
		err := client.BlockSignal(ctx, *pid1, SIGUSR2)
		assert.NoError(t, err, "Failed to block signal")

		// Send blocked signal
		err = client.SendSignal(ctx, *pid2, *pid1, SIGUSR2)
		// Should get error since signal is blocked
		assert.Error(t, err, "Blocked signal should fail")

		// Unblock SIGUSR2
		err = client.UnblockSignal(ctx, *pid1, SIGUSR2)
		assert.NoError(t, err, "Failed to unblock signal")

		// Now sending should work
		err = client.SendSignal(ctx, *pid2, *pid1, SIGUSR2)
		assert.NoError(t, err, "Failed to send unblocked signal")
	})

	t.Run("GetSignalStats", func(t *testing.T) {
		stats, err := client.GetSignalStats(ctx, *pid1)
		assert.NoError(t, err, "Failed to get signal stats")
		assert.NotNil(t, stats)
		assert.Greater(t, stats.TotalSignalsSent, uint64(0), "Should have sent signals")
	})

	t.Run("GetSignalState", func(t *testing.T) {
		state, err := client.GetSignalState(ctx, *pid1, pid1)
		assert.NoError(t, err, "Failed to get signal state")
		assert.NotNil(t, state)
		assert.Equal(t, *pid1, state.Pid, "PID should match")
	})

	t.Run("WaitForSignal", func(t *testing.T) {
		// Send a signal
		err := client.SendSignal(ctx, *pid2, *pid1, SIGTERM)
		require.NoError(t, err, "Failed to send SIGTERM")

		// Wait for it
		timeout := 5 * time.Second
		signal, err := client.WaitForSignal(ctx, *pid1, []uint32{SIGTERM, SIGKILL}, &timeout)
		assert.NoError(t, err, "Failed to wait for signal")
		assert.Equal(t, uint32(SIGTERM), signal, "Should receive SIGTERM")
	})
}

// TestSchedulerIntegration tests advanced scheduler operations
func TestSchedulerIntegration(t *testing.T) {
	if testing.Short() {
		t.Skip("Skipping integration test in short mode")
	}

	client, err := New("127.0.0.1:50051")
	require.NoError(t, err, "Failed to create kernel client")
	defer client.Close()

	ctx := context.Background()

	// Create test process
	pid, _, err := client.CreateProcess(ctx, "scheduler-test", 5, "STANDARD", nil)
	require.NoError(t, err, "Failed to create process")
	require.NotNil(t, pid)

	t.Run("YieldProcess", func(t *testing.T) {
		err := client.YieldProcess(ctx, *pid)
		assert.NoError(t, err, "Failed to yield process")
	})

	t.Run("GetCurrentScheduled", func(t *testing.T) {
		currentPid, err := client.GetCurrentScheduled(ctx, *pid)
		assert.NoError(t, err, "Failed to get current scheduled")
		assert.Greater(t, currentPid, uint32(0), "Should have a scheduled process")
	})

	t.Run("SetAndGetTimeQuantum", func(t *testing.T) {
		// Set quantum to 10ms (10000 microseconds)
		err := client.SetTimeQuantum(ctx, *pid, 10000)
		assert.NoError(t, err, "Failed to set time quantum")

		// Get and verify
		quantum, err := client.GetTimeQuantum(ctx, *pid)
		assert.NoError(t, err, "Failed to get time quantum")
		assert.Equal(t, uint64(10000), quantum, "Time quantum should match")
	})

	t.Run("GetProcessSchedulerStats", func(t *testing.T) {
		stats, err := client.GetProcessSchedulerStats(ctx, *pid, *pid)
		assert.NoError(t, err, "Failed to get process scheduler stats")
		assert.NotNil(t, stats)
		assert.Equal(t, *pid, stats.Pid, "PID should match")
	})

	t.Run("GetAllProcessSchedulerStats", func(t *testing.T) {
		allStats, err := client.GetAllProcessSchedulerStats(ctx, *pid)
		assert.NoError(t, err, "Failed to get all process scheduler stats")
		assert.NotEmpty(t, allStats, "Should have stats for at least one process")
	})

	t.Run("BoostAndLowerPriority", func(t *testing.T) {
		// Boost priority
		err := client.BoostPriority(ctx, *pid, *pid)
		assert.NoError(t, err, "Failed to boost priority")

		// Lower priority
		err = client.LowerPriority(ctx, *pid, *pid)
		assert.NoError(t, err, "Failed to lower priority")
	})

	t.Run("SetAndGetSchedulingPolicy", func(t *testing.T) {
		// Set to priority-based scheduling
		err := client.SetSchedulingPolicy(ctx, "priority")
		assert.NoError(t, err, "Failed to set scheduling policy")

		// Get and verify
		policy, err := client.GetSchedulingPolicy(ctx, *pid)
		assert.NoError(t, err, "Failed to get scheduling policy")
		assert.Equal(t, "priority", policy, "Policy should match")
	})
}

// TestStreamingWrite tests streaming file write operations
func TestStreamingWrite(t *testing.T) {
	if testing.Short() {
		t.Skip("Skipping integration test in short mode")
	}

	client, err := New("127.0.0.1:50051")
	require.NoError(t, err, "Failed to create kernel client")
	defer client.Close()

	ctx := context.Background()

	// Create test process
	pid, _, err := client.CreateProcess(ctx, "streaming-test", 5, "STANDARD", nil)
	require.NoError(t, err, "Failed to create process")
	require.NotNil(t, pid)

	t.Run("StreamFileWrite", func(t *testing.T) {
		dataChan, resultChan := client.StreamFileWrite(ctx, *pid, "/tmp/streamed_file.txt")

		// Send data in chunks
		testData := [][]byte{
			[]byte("Hello, "),
			[]byte("this is "),
			[]byte("a streaming "),
			[]byte("write test!"),
		}

		go func() {
			for _, chunk := range testData {
				dataChan <- chunk
			}
			close(dataChan)
		}()

		// Wait for result
		result := <-resultChan
		assert.NoError(t, result.Error, "Streaming write should succeed")
		assert.Greater(t, result.TotalBytes, uint64(0), "Should have written bytes")
	})
}
