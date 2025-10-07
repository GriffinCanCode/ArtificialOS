/*
Package resilience provides circuit breaker implementation for graceful degradation.

# Overview

This package implements the circuit breaker pattern to prevent cascading failures
and provide graceful degradation when services become unavailable or slow.

# Features

- Three-state circuit breaker (Closed, Open, Half-Open)
- Configurable failure thresholds and timeouts
- Automatic state transitions
- Concurrent request handling
- State change callbacks for monitoring
- Thread-safe operations

# Usage

	// Create a circuit breaker
	breaker := resilience.New("service", resilience.Settings{
		MaxRequests: 3,
		Interval:    60 * time.Second,
		Timeout:     30 * time.Second,
		ReadyToTrip: func(counts resilience.Counts) bool {
			return counts.ConsecutiveFailures >= 5
		},
		OnStateChange: func(name string, from, to resilience.State) {
			log.Printf("Circuit breaker %s: %s -> %s", name, from, to)
		},
	})

	// Execute request through breaker
	result, err := breaker.Execute(func() (interface{}, error) {
		return client.Call()
	})

# States

- Closed: Normal operation, requests pass through
- Open: Service unavailable, requests fail immediately
- Half-Open: Testing if service recovered, limited requests allowed

# Pattern

The circuit breaker transitions between states based on success/failure rates:

	Closed --[failures]-> Open --[timeout]-> Half-Open --[successes]-> Closed
	                                           |
	                                    [failure]
	                                           |
	                                           v
	                                         Open
*/
package resilience
