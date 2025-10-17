package terminal

import (
	"fmt"
	"io"
	"os"
	"os/exec"
	"sync"
	"time"

	"github.com/GriffinCanCode/AgentOS/backend/internal/shared/id"
	"github.com/creack/pty"
)

// Manager manages terminal sessions
type Manager struct {
	sessions sync.Map // map[string]*Session
}

// NewManager creates a new session manager
func NewManager() *Manager {
	return &Manager{}
}

// CreateSession creates a new terminal session with PTY
func (m *Manager) CreateSession(shell, workingDir string, cols, rows int, env map[string]string) (*SessionInfo, error) {
	// Default shell
	if shell == "" {
		shell = os.Getenv("SHELL")
		if shell == "" {
			shell = "/bin/bash"
		}
	}

	// Default working directory
	if workingDir == "" {
		workingDir = os.Getenv("HOME")
		if workingDir == "" {
			workingDir = "/tmp"
		}
	}

	// Default dimensions
	if cols <= 0 {
		cols = 80
	}
	if rows <= 0 {
		rows = 24
	}

	// Generate session ID
	sessionID := string(id.NewSessionID())

	// Create command
	cmd := exec.Command(shell)
	cmd.Dir = workingDir

	// Set environment variables
	cmd.Env = os.Environ()
	cmd.Env = append(cmd.Env, "TERM=xterm-256color")
	for key, value := range env {
		cmd.Env = append(cmd.Env, fmt.Sprintf("%s=%s", key, value))
	}

	// Start PTY
	ptmx, err := pty.StartWithSize(cmd, &pty.Winsize{
		Rows: uint16(rows),
		Cols: uint16(cols),
	})
	if err != nil {
		return nil, fmt.Errorf("failed to start PTY: %w", err)
	}

	// Create session
	session := &Session{
		ID:         sessionID,
		Shell:      shell,
		WorkingDir: workingDir,
		Cols:       cols,
		Rows:       rows,
		StartedAt:  time.Now(),
		cmd:        cmd,
		ptmx:       ptmx,
		outputBuf:  NewBuffer(1024 * 1024), // 1MB buffer
		closed:     false,
	}

	// Store session
	m.sessions.Store(sessionID, session)

	// Start output reader
	go m.readOutput(session)

	// Monitor process
	go m.monitorProcess(session)

	return &SessionInfo{
		ID:         session.ID,
		Shell:      session.Shell,
		WorkingDir: session.WorkingDir,
		Cols:       session.Cols,
		Rows:       session.Rows,
		StartedAt:  session.StartedAt,
		Active:     true,
	}, nil
}

// readOutput continuously reads from PTY and buffers output
func (m *Manager) readOutput(session *Session) {
	buf := make([]byte, 4096)
	for {
		n, err := session.ptmx.Read(buf)
		if err != nil {
			if err != io.EOF {
				// Log error but continue
			}
			break
		}

		if n > 0 {
			session.outputBuf.Write(buf[:n])
		}
	}
}

// monitorProcess waits for process to exit and cleans up
func (m *Manager) monitorProcess(session *Session) {
	session.cmd.Wait()

	session.mu.Lock()
	session.closed = true
	session.mu.Unlock()

	session.ptmx.Close()
}

// Write sends input to a session
func (m *Manager) Write(sessionID string, input []byte) error {
	value, ok := m.sessions.Load(sessionID)
	if !ok {
		return fmt.Errorf("session not found: %s", sessionID)
	}

	session := value.(*Session)

	session.mu.RLock()
	closed := session.closed
	session.mu.RUnlock()

	if closed {
		return fmt.Errorf("session is closed: %s", sessionID)
	}

	_, err := session.ptmx.Write(input)
	return err
}

// Read retrieves buffered output from a session
func (m *Manager) Read(sessionID string) ([]byte, error) {
	value, ok := m.sessions.Load(sessionID)
	if !ok {
		return nil, fmt.Errorf("session not found: %s", sessionID)
	}

	session := value.(*Session)
	return session.outputBuf.ReadAll(), nil
}

// Resize changes terminal dimensions
func (m *Manager) Resize(sessionID string, cols, rows int) error {
	value, ok := m.sessions.Load(sessionID)
	if !ok {
		return fmt.Errorf("session not found: %s", sessionID)
	}

	session := value.(*Session)

	session.mu.Lock()
	defer session.mu.Unlock()

	if session.closed {
		return fmt.Errorf("session is closed: %s", sessionID)
	}

	session.Cols = cols
	session.Rows = rows

	return pty.Setsize(session.ptmx, &pty.Winsize{
		Rows: uint16(rows),
		Cols: uint16(cols),
	})
}

// Kill terminates a session
func (m *Manager) Kill(sessionID string) error {
	value, ok := m.sessions.Load(sessionID)
	if !ok {
		return fmt.Errorf("session not found: %s", sessionID)
	}

	session := value.(*Session)

	session.mu.Lock()
	defer session.mu.Unlock()

	if session.closed {
		return nil // Already closed
	}

	session.closed = true

	// Kill process
	if session.cmd.Process != nil {
		session.cmd.Process.Kill()
	}

	// Close PTY
	session.ptmx.Close()

	// Remove from sessions
	m.sessions.Delete(sessionID)

	return nil
}

// ListSessions returns all active sessions
func (m *Manager) ListSessions() []SessionInfo {
	var sessions []SessionInfo

	m.sessions.Range(func(key, value interface{}) bool {
		session := value.(*Session)

		session.mu.RLock()
		active := !session.closed
		session.mu.RUnlock()

		sessions = append(sessions, SessionInfo{
			ID:         session.ID,
			Shell:      session.Shell,
			WorkingDir: session.WorkingDir,
			Cols:       session.Cols,
			Rows:       session.Rows,
			StartedAt:  session.StartedAt,
			Active:     active,
		})
		return true
	})

	return sessions
}

// GetSession retrieves session info
func (m *Manager) GetSession(sessionID string) (*SessionInfo, error) {
	value, ok := m.sessions.Load(sessionID)
	if !ok {
		return nil, fmt.Errorf("session not found: %s", sessionID)
	}

	session := value.(*Session)

	session.mu.RLock()
	active := !session.closed
	session.mu.RUnlock()

	return &SessionInfo{
		ID:         session.ID,
		Shell:      session.Shell,
		WorkingDir: session.WorkingDir,
		Cols:       session.Cols,
		Rows:       session.Rows,
		StartedAt:  session.StartedAt,
		Active:     active,
	}, nil
}
