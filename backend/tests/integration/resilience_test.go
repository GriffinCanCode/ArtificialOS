//go:build integration
// +build integration

package integration

import (
	"errors"
	"testing"
	"time"

	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"

	"github.com/GriffinCanCode/AgentOS/backend/internal/infrastructure/resilience"
)

func TestCircuitBreakerIntegration(t *testing.T) {
	if testing.Short() {
		t.Skip("Skipping circuit breaker integration test")
	}

	t.Run("Circuit breaker prevents cascading failures", func(t *testing.T) {
		failureCount := 0
		maxFailures := 3

		breaker := resilience.New("test-service", resilience.Settings{
			MaxRequests: 1,
			Interval:    time.Second,
			Timeout:     100 * time.Millisecond,
			ReadyToTrip: func(counts resilience.Counts) bool {
				return counts.ConsecutiveFailures >= uint32(maxFailures)
			},
		})

		// Simulate failing service
		callService := func() (interface{}, error) {
			if failureCount < maxFailures {
				failureCount++
				return nil, errors.New("service unavailable")
			}
			return "success", nil
		}

		// Execute requests until circuit opens
		for i := 0; i < maxFailures+1; i++ {
			_, _ = breaker.Execute(callService)
		}

		assert.Equal(t, resilience.StateOpen, breaker.State())

		// Verify requests fail fast while circuit is open
		_, err := breaker.Execute(callService)
		assert.Equal(t, resilience.ErrCircuitOpen, err)

		// Wait for circuit to transition to half-open
		time.Sleep(150 * time.Millisecond)

		assert.Equal(t, resilience.StateHalfOpen, breaker.State())

		// Service recovers
		_, err = breaker.Execute(callService)
		require.NoError(t, err)

		assert.Equal(t, resilience.StateClosed, breaker.State())
	})

	t.Run("Circuit breaker tracks metrics", func(t *testing.T) {
		breaker := resilience.New("metrics-test", resilience.Settings{
			MaxRequests: 1,
			Interval:    time.Minute,
			Timeout:     time.Minute,
		})

		// Execute mix of successful and failed requests
		for i := 0; i < 5; i++ {
			_, _ = breaker.Execute(func() (interface{}, error) {
				if i%2 == 0 {
					return "ok", nil
				}
				return nil, errors.New("failed")
			})
		}

		counts := breaker.Counts()
		assert.Equal(t, uint32(5), counts.Requests)
		assert.True(t, counts.TotalSuccesses > 0)
		assert.True(t, counts.TotalFailures > 0)
	})

	t.Run("Multiple concurrent circuits", func(t *testing.T) {
		services := []string{"service-a", "service-b", "service-c"}
		breakers := make(map[string]*resilience.Breaker)

		for _, svc := range services {
			breakers[svc] = resilience.New(svc, resilience.Settings{
				MaxRequests: 1,
				Interval:    time.Minute,
				Timeout:     time.Minute,
				ReadyToTrip: func(counts resilience.Counts) bool {
					return counts.ConsecutiveFailures >= 2
				},
			})
		}

		// Service A fails
		for i := 0; i < 2; i++ {
			_, _ = breakers["service-a"].Execute(func() (interface{}, error) {
				return nil, errors.New("failed")
			})
		}

		// Service B succeeds
		_, err := breakers["service-b"].Execute(func() (interface{}, error) {
			return "ok", nil
		})
		require.NoError(t, err)

		// Verify independent circuit states
		assert.Equal(t, resilience.StateOpen, breakers["service-a"].State())
		assert.Equal(t, resilience.StateClosed, breakers["service-b"].State())
		assert.Equal(t, resilience.StateClosed, breakers["service-c"].State())
	})
}
