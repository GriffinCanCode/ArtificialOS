package client

import (
	"context"
	"errors"
	"testing"
	"time"

	"github.com/GriffinCanCode/AgentOS/backend/internal/infrastructure/resilience"
	"github.com/go-resty/resty/v2"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

func TestClientCircuitBreakerIntegration(t *testing.T) {
	t.Run("circuit breaker protects against failures", func(t *testing.T) {
		client := NewClient()

		// Verify breaker is initialized
		require.NotNil(t, client.Breaker)
		assert.Equal(t, resilience.StateClosed, client.Breaker.State())

		// Check breaker state is accessible
		state := client.BreakerState()
		assert.Equal(t, resilience.StateClosed, state)

		// Check counts are accessible
		counts := client.BreakerCounts()
		assert.Equal(t, uint32(0), counts.Requests)
	})

	t.Run("request checks circuit breaker state", func(t *testing.T) {
		client := NewClient()
		ctx := context.Background()

		// Should succeed when circuit is closed
		req, err := client.Request(ctx)
		require.NoError(t, err)
		assert.NotNil(t, req)

		// Manually trip the circuit
		for i := 0; i < 10; i++ {
			client.Breaker.Execute(func() (interface{}, error) {
				return nil, errors.New("simulated failure")
			})
		}

		// Wait a moment for circuit to open
		time.Sleep(10 * time.Millisecond)

		// Request should fail when circuit is open
		if client.Breaker.State() == resilience.StateOpen {
			req, err = client.Request(ctx)
			assert.Error(t, err)
			assert.Equal(t, resilience.ErrCircuitOpen, err)
			assert.Nil(t, req)
		}
	})

	t.Run("execute with breaker handles failures", func(t *testing.T) {
		client := NewClient()

		// Test successful execution
		resp, err := client.ExecuteWithBreaker(func() (*resty.Response, error) {
			return nil, nil
		})
		require.NoError(t, err)
		assert.Nil(t, resp) // Our mock returns nil

		// Test failed execution
		testErr := errors.New("test error")
		resp, err = client.ExecuteWithBreaker(func() (*resty.Response, error) {
			return nil, testErr
		})
		assert.Error(t, err)
		assert.Equal(t, testErr, err)
		assert.Nil(t, resp)
	})

	t.Run("circuit breaker configuration", func(t *testing.T) {
		client := NewClient()

		// Verify breaker settings
		assert.Equal(t, "http-external", client.Breaker.Name())

		// Verify breaker tracks statistics
		counts := client.BreakerCounts()
		assert.Equal(t, uint32(0), counts.TotalSuccesses)
		assert.Equal(t, uint32(0), counts.TotalFailures)

		// Execute some successful requests
		for i := 0; i < 5; i++ {
			client.Breaker.Execute(func() (interface{}, error) {
				return "ok", nil
			})
		}

		counts = client.BreakerCounts()
		assert.Equal(t, uint32(5), counts.TotalSuccesses)
		assert.Equal(t, uint32(0), counts.TotalFailures)
	})

	t.Run("circuit breaker opens after consecutive failures", func(t *testing.T) {
		client := NewClient()

		// Trigger 10 consecutive failures (circuit should open)
		for i := 0; i < 10; i++ {
			client.Breaker.Execute(func() (interface{}, error) {
				return nil, errors.New("failure")
			})
		}

		// Circuit should be open now
		assert.Equal(t, resilience.StateOpen, client.BreakerState())

		// Next execution should fail fast
		_, err := client.ExecuteWithBreaker(func() (*resty.Response, error) {
			return nil, nil
		})
		assert.Error(t, err)
		assert.Contains(t, err.Error(), "circuit breaker open")
	})
}

func TestClientRateLimiting(t *testing.T) {
	t.Run("rate limiter works with circuit breaker", func(t *testing.T) {
		client := NewClient()

		// Set a rate limit
		client.SetRateLimit(10) // 10 requests per second

		ctx := context.Background()

		// Should be able to make requests
		req, err := client.Request(ctx)
		require.NoError(t, err)
		assert.NotNil(t, req)
	})

	t.Run("context cancellation prevents request", func(t *testing.T) {
		client := NewClient()
		client.SetRateLimit(1) // Very low rate

		ctx, cancel := context.WithCancel(context.Background())
		cancel() // Cancel immediately

		// Should fail due to context cancellation
		req, err := client.Request(ctx)
		assert.Error(t, err)
		assert.Nil(t, req)
	})
}

func TestClientConfiguration(t *testing.T) {
	t.Run("client has sane defaults", func(t *testing.T) {
		client := NewClient()

		require.NotNil(t, client.Resty)
		require.NotNil(t, client.Limiter)
		require.NotNil(t, client.Breaker)

		// Check Resty configuration
		assert.NotNil(t, client.Resty)

		// Check circuit breaker is in closed state
		assert.Equal(t, resilience.StateClosed, client.Breaker.State())
	})

	t.Run("client timeout can be configured", func(t *testing.T) {
		client := NewClient()

		// Set timeout
		client.SetTimeout(10 * time.Second)

		// Verify client still works
		ctx := context.Background()
		req, err := client.Request(ctx)
		require.NoError(t, err)
		assert.NotNil(t, req)
	})

	t.Run("client retry can be configured", func(t *testing.T) {
		client := NewClient()

		// Set retry
		client.SetRetry(5, time.Second, 10*time.Second)

		// Verify client still works
		ctx := context.Background()
		req, err := client.Request(ctx)
		require.NoError(t, err)
		assert.NotNil(t, req)
	})
}
