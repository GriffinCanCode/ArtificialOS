package resilience

import (
	"errors"
	"sync"
	"time"
)

var (
	ErrCircuitOpen     = errors.New("circuit breaker is open")
	ErrTooManyRequests = errors.New("too many requests")
)

// State represents the circuit breaker state
type State int

const (
	StateClosed State = iota
	StateHalfOpen
	StateOpen
)

// String returns the string representation of the state
func (s State) String() string {
	switch s {
	case StateClosed:
		return "closed"
	case StateHalfOpen:
		return "half-open"
	case StateOpen:
		return "open"
	default:
		return "unknown"
	}
}

// Settings configures the circuit breaker behavior
type Settings struct {
	// MaxRequests is the maximum number of requests allowed in half-open state
	MaxRequests uint32
	// Interval is the cyclic period of the closed state to clear internal counts
	Interval time.Duration
	// Timeout is the period of the open state until transitioning to half-open
	Timeout time.Duration
	// ReadyToTrip is called with counts when a request fails in closed state
	ReadyToTrip func(counts Counts) bool
	// OnStateChange is called whenever the state changes
	OnStateChange func(name string, from State, to State)
}

// Counts holds the statistics for the circuit breaker
type Counts struct {
	Requests             uint32
	TotalSuccesses       uint32
	TotalFailures        uint32
	ConsecutiveSuccesses uint32
	ConsecutiveFailures  uint32
}

// Breaker implements the circuit breaker pattern
type Breaker struct {
	name     string
	settings Settings

	mu     sync.Mutex
	state  State
	counts Counts
	expiry time.Time
}

// New creates a new circuit breaker with the given settings
func New(name string, settings Settings) *Breaker {
	// Set default values
	if settings.MaxRequests == 0 {
		settings.MaxRequests = 1
	}
	if settings.Interval == 0 {
		settings.Interval = 60 * time.Second
	}
	if settings.Timeout == 0 {
		settings.Timeout = 60 * time.Second
	}
	if settings.ReadyToTrip == nil {
		settings.ReadyToTrip = func(counts Counts) bool {
			return counts.ConsecutiveFailures > 5
		}
	}

	return &Breaker{
		name:     name,
		settings: settings,
		state:    StateClosed,
		expiry:   time.Now().Add(settings.Interval),
	}
}

// Name returns the name of the circuit breaker
func (b *Breaker) Name() string {
	return b.name
}

// State returns the current state of the circuit breaker
func (b *Breaker) State() State {
	b.mu.Lock()
	defer b.mu.Unlock()

	now := time.Now()
	state, _ := b.currentState(now)
	return state
}

// Counts returns a copy of the internal counts
func (b *Breaker) Counts() Counts {
	b.mu.Lock()
	defer b.mu.Unlock()

	return b.counts
}

// Execute runs the given request if the circuit breaker accepts it
func (b *Breaker) Execute(req func() (interface{}, error)) (interface{}, error) {
	generation, err := b.beforeRequest()
	if err != nil {
		return nil, err
	}

	defer func() {
		e := recover()
		if e != nil {
			b.afterRequest(generation, false)
			panic(e)
		}
	}()

	result, err := req()
	b.afterRequest(generation, err == nil)
	return result, err
}

// beforeRequest is called before a request is executed
func (b *Breaker) beforeRequest() (uint64, error) {
	b.mu.Lock()
	defer b.mu.Unlock()

	now := time.Now()
	state, generation := b.currentState(now)

	if state == StateOpen {
		return generation, ErrCircuitOpen
	}

	if state == StateHalfOpen && b.counts.Requests >= b.settings.MaxRequests {
		return generation, ErrTooManyRequests
	}

	b.counts.Requests++
	return generation, nil
}

// afterRequest is called after a request is executed
func (b *Breaker) afterRequest(before uint64, success bool) {
	b.mu.Lock()
	defer b.mu.Unlock()

	now := time.Now()
	state, generation := b.currentState(now)

	if generation != before {
		return
	}

	if success {
		b.onSuccess(state, now)
	} else {
		b.onFailure(state, now)
	}
}

// onSuccess handles successful requests
func (b *Breaker) onSuccess(state State, now time.Time) {
	switch state {
	case StateClosed:
		b.counts.TotalSuccesses++
		b.counts.ConsecutiveSuccesses++
		b.counts.ConsecutiveFailures = 0
	case StateHalfOpen:
		b.counts.TotalSuccesses++
		b.counts.ConsecutiveSuccesses++
		b.counts.ConsecutiveFailures = 0
		if b.counts.ConsecutiveSuccesses >= b.settings.MaxRequests {
			b.setState(StateClosed, now)
		}
	}
}

// onFailure handles failed requests
func (b *Breaker) onFailure(state State, now time.Time) {
	switch state {
	case StateClosed:
		b.counts.TotalFailures++
		b.counts.ConsecutiveFailures++
		b.counts.ConsecutiveSuccesses = 0
		if b.settings.ReadyToTrip(b.counts) {
			b.setState(StateOpen, now)
		}
	case StateHalfOpen:
		b.setState(StateOpen, now)
	}
}

// currentState returns the current state and generation
func (b *Breaker) currentState(now time.Time) (State, uint64) {
	switch b.state {
	case StateClosed:
		if !b.expiry.IsZero() && b.expiry.Before(now) {
			b.resetCounts()
			b.expiry = now.Add(b.settings.Interval)
		}
	case StateOpen:
		if b.expiry.Before(now) {
			b.setState(StateHalfOpen, now)
		}
	}

	return b.state, uint64(b.expiry.UnixNano())
}

// setState changes the state of the circuit breaker
func (b *Breaker) setState(state State, now time.Time) {
	if b.state == state {
		return
	}

	prev := b.state
	b.state = state

	b.resetCounts()

	switch state {
	case StateClosed:
		b.expiry = now.Add(b.settings.Interval)
	case StateOpen:
		b.expiry = now.Add(b.settings.Timeout)
	case StateHalfOpen:
		b.expiry = time.Time{}
	}

	if b.settings.OnStateChange != nil {
		b.settings.OnStateChange(b.name, prev, state)
	}
}

// resetCounts resets the internal counts
func (b *Breaker) resetCounts() {
	b.counts = Counts{}
}
