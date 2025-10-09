package sandbox

import (
	"context"
	"errors"
	"sync"
	"time"
)

var (
	ErrPoolClosed = errors.New("sandbox pool is closed")
	ErrTimeout    = errors.New("sandbox acquisition timeout")
)

// Pool manages a pool of reusable sandboxes
type Pool struct {
	config    Config
	sandboxes chan *Runtime
	size      int
	mu        sync.RWMutex
	closed    bool
}

// NewPool creates a sandbox pool
func NewPool(config Config, size int) (*Pool, error) {
	if size <= 0 {
		size = 4
	}

	pool := &Pool{
		config:    config,
		sandboxes: make(chan *Runtime, size),
		size:      size,
	}

	// Pre-create sandboxes
	for i := 0; i < size; i++ {
		sandbox, err := New(config)
		if err != nil {
			pool.Close()
			return nil, err
		}
		pool.sandboxes <- sandbox
	}

	return pool, nil
}

// Acquire gets a sandbox from pool with timeout
func (p *Pool) Acquire(ctx context.Context) (*Runtime, error) {
	p.mu.RLock()
	defer p.mu.RUnlock()

	if p.closed {
		return nil, ErrPoolClosed
	}

	select {
	case sandbox := <-p.sandboxes:
		return sandbox, nil
	case <-ctx.Done():
		return nil, ctx.Err()
	case <-time.After(5 * time.Second):
		return nil, ErrTimeout
	}
}

// Release returns sandbox to pool
func (p *Pool) Release(sandbox *Runtime) error {
	p.mu.RLock()
	defer p.mu.RUnlock()

	if p.closed {
		return sandbox.Close()
	}

	// Reset sandbox state
	if err := sandbox.Reset(); err != nil {
		sandbox.Close()
		// Create new sandbox
		if newSandbox, err := New(p.config); err == nil {
			p.sandboxes <- newSandbox
		}
		return err
	}

	select {
	case p.sandboxes <- sandbox:
		return nil
	default:
		// Pool full, close sandbox
		return sandbox.Close()
	}
}

// Execute runs script using pool
func (p *Pool) Execute(ctx context.Context, script string, dom *DOM) (*Result, error) {
	sandbox, err := p.Acquire(ctx)
	if err != nil {
		return nil, err
	}
	defer p.Release(sandbox)

	return sandbox.Execute(ctx, script, dom)
}

// Close closes pool and all sandboxes
func (p *Pool) Close() error {
	p.mu.Lock()
	defer p.mu.Unlock()

	if p.closed {
		return nil
	}

	p.closed = true
	close(p.sandboxes)

	// Close all sandboxes
	for sandbox := range p.sandboxes {
		sandbox.Close()
	}

	return nil
}

// Stats returns pool statistics
func (p *Pool) Stats() map[string]interface{} {
	p.mu.RLock()
	defer p.mu.RUnlock()

	return map[string]interface{}{
		"size":      p.size,
		"available": len(p.sandboxes),
		"in_use":    p.size - len(p.sandboxes),
		"closed":    p.closed,
	}
}
