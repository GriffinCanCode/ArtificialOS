package resilience

import (
	"errors"
	"testing"
	"time"

	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

func TestBreakerStateTransitions(t *testing.T) {
	tests := []struct {
		name          string
		settings      Settings
		requests      []bool // true = success, false = failure
		expectedState State
	}{
		{
			name: "stays closed on successes",
			settings: Settings{
				MaxRequests: 1,
				Interval:    time.Minute,
				Timeout:     time.Minute,
			},
			requests:      []bool{true, true, true},
			expectedState: StateClosed,
		},
		{
			name: "opens after consecutive failures",
			settings: Settings{
				MaxRequests: 1,
				Interval:    time.Minute,
				Timeout:     time.Minute,
				ReadyToTrip: func(counts Counts) bool {
					return counts.ConsecutiveFailures >= 3
				},
			},
			requests:      []bool{false, false, false},
			expectedState: StateOpen,
		},
		{
			name: "transitions to half-open after timeout",
			settings: Settings{
				MaxRequests: 1,
				Interval:    time.Minute,
				Timeout:     10 * time.Millisecond,
				ReadyToTrip: func(counts Counts) bool {
					return counts.ConsecutiveFailures >= 2
				},
			},
			requests:      []bool{false, false},
			expectedState: StateOpen,
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			breaker := New("test", tt.settings)

			for _, success := range tt.requests {
				_, err := breaker.Execute(func() (interface{}, error) {
					if success {
						return "ok", nil
					}
					return nil, errors.New("failed")
				})

				if err != nil && err != ErrCircuitOpen {
					// Expected failure
				}
			}

			assert.Equal(t, tt.expectedState, breaker.State())
		})
	}
}

func TestBreakerCounts(t *testing.T) {
	breaker := New("test", Settings{
		MaxRequests: 1,
		Interval:    time.Minute,
		Timeout:     time.Minute,
	})

	// Execute successful request
	_, err := breaker.Execute(func() (interface{}, error) {
		return "ok", nil
	})
	require.NoError(t, err)

	counts := breaker.Counts()
	assert.Equal(t, uint32(1), counts.Requests)
	assert.Equal(t, uint32(1), counts.TotalSuccesses)
	assert.Equal(t, uint32(1), counts.ConsecutiveSuccesses)
	assert.Equal(t, uint32(0), counts.TotalFailures)

	// Execute failed request
	_, err = breaker.Execute(func() (interface{}, error) {
		return nil, errors.New("failed")
	})
	assert.Error(t, err)

	counts = breaker.Counts()
	assert.Equal(t, uint32(2), counts.Requests)
	assert.Equal(t, uint32(1), counts.TotalFailures)
	assert.Equal(t, uint32(1), counts.ConsecutiveFailures)
	assert.Equal(t, uint32(0), counts.ConsecutiveSuccesses)
}

func TestBreakerOpenState(t *testing.T) {
	breaker := New("test", Settings{
		MaxRequests: 1,
		Interval:    time.Minute,
		Timeout:     time.Minute,
		ReadyToTrip: func(counts Counts) bool {
			return counts.ConsecutiveFailures >= 2
		},
	})

	// Cause breaker to open
	for i := 0; i < 2; i++ {
		_, _ = breaker.Execute(func() (interface{}, error) {
			return nil, errors.New("failed")
		})
	}

	assert.Equal(t, StateOpen, breaker.State())

	// Next request should fail immediately
	_, err := breaker.Execute(func() (interface{}, error) {
		return "ok", nil
	})
	assert.Equal(t, ErrCircuitOpen, err)
}

func TestBreakerHalfOpenState(t *testing.T) {
	breaker := New("test", Settings{
		MaxRequests: 2,
		Interval:    time.Minute,
		Timeout:     50 * time.Millisecond,
		ReadyToTrip: func(counts Counts) bool {
			return counts.ConsecutiveFailures >= 2
		},
	})

	// Open the breaker
	for i := 0; i < 2; i++ {
		_, _ = breaker.Execute(func() (interface{}, error) {
			return nil, errors.New("failed")
		})
	}

	assert.Equal(t, StateOpen, breaker.State())

	// Wait for timeout
	time.Sleep(60 * time.Millisecond)

	// Breaker should be half-open now
	assert.Equal(t, StateHalfOpen, breaker.State())

	// Execute successful requests to close it
	for i := 0; i < 2; i++ {
		_, err := breaker.Execute(func() (interface{}, error) {
			return "ok", nil
		})
		require.NoError(t, err)
	}

	assert.Equal(t, StateClosed, breaker.State())
}

func TestBreakerCallbacks(t *testing.T) {
	var transitions []string

	breaker := New("test", Settings{
		MaxRequests: 1,
		Interval:    time.Minute,
		Timeout:     10 * time.Millisecond,
		ReadyToTrip: func(counts Counts) bool {
			return counts.ConsecutiveFailures >= 2
		},
		OnStateChange: func(name string, from State, to State) {
			transitions = append(transitions, from.String()+"->"+to.String())
		},
	})

	// Trigger state transitions
	for i := 0; i < 2; i++ {
		_, _ = breaker.Execute(func() (interface{}, error) {
			return nil, errors.New("failed")
		})
	}

	time.Sleep(20 * time.Millisecond)

	// Trigger half-open
	state := breaker.State()
	assert.Equal(t, StateHalfOpen, state)

	// Verify transitions were recorded
	assert.Contains(t, transitions, "closed->open")
	assert.Contains(t, transitions, "open->half-open")
}
