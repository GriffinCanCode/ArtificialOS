package terminal

import (
	"os"
	"os/exec"
	"sync"
	"time"
)

// Session represents an active terminal session
type Session struct {
	ID         string
	Shell      string
	WorkingDir string
	Cols       int
	Rows       int
	StartedAt  time.Time

	// Process management
	cmd  *exec.Cmd
	ptmx *os.File

	// Output buffering
	outputBuf *Buffer

	// Lifecycle
	mu     sync.RWMutex
	closed bool
}

// Buffer is a thread-safe circular buffer for terminal output
type Buffer struct {
	data []byte
	size int
	head int
	tail int
	mu   sync.RWMutex
}

// NewBuffer creates a new circular buffer
func NewBuffer(size int) *Buffer {
	return &Buffer{
		data: make([]byte, size),
		size: size,
	}
}

// Write writes data to the buffer
func (b *Buffer) Write(p []byte) (n int, err error) {
	b.mu.Lock()
	defer b.mu.Unlock()

	for _, c := range p {
		b.data[b.tail] = c
		b.tail = (b.tail + 1) % b.size

		// If buffer is full, move head forward
		if b.tail == b.head {
			b.head = (b.head + 1) % b.size
		}
	}

	return len(p), nil
}

// ReadAll reads all available data from the buffer
func (b *Buffer) ReadAll() []byte {
	b.mu.RLock()
	defer b.mu.RUnlock()

	if b.head == b.tail {
		return []byte{}
	}

	var result []byte
	if b.tail > b.head {
		result = make([]byte, b.tail-b.head)
		copy(result, b.data[b.head:b.tail])
	} else {
		// Buffer wrapped around
		firstPart := b.data[b.head:]
		secondPart := b.data[:b.tail]
		result = make([]byte, len(firstPart)+len(secondPart))
		copy(result, firstPart)
		copy(result[len(firstPart):], secondPart)
	}

	// Clear buffer after reading
	b.head = b.tail

	return result
}

// SessionInfo is the public representation of a session
type SessionInfo struct {
	ID         string    `json:"id"`
	Shell      string    `json:"shell"`
	WorkingDir string    `json:"working_dir"`
	Cols       int       `json:"cols"`
	Rows       int       `json:"rows"`
	StartedAt  time.Time `json:"started_at"`
	Active     bool      `json:"active"`
}
