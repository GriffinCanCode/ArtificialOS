// Package terminal provides a full-featured terminal emulator service.
//
// This provider enables interactive shell sessions with PTY (pseudo-terminal) support,
// allowing apps to spawn and interact with shell processes like bash, zsh, etc.
//
// Features:
//   - PTY support for full terminal emulation
//   - Multiple concurrent sessions
//   - ANSI escape sequence support
//   - Terminal resizing
//   - Session persistence
//   - Environment variable configuration
//   - Working directory control
//
// Architecture:
//   - Each session spawns a shell process via kernel
//   - PTY provides bidirectional I/O with proper terminal semantics
//   - Output buffering for efficient streaming
//   - Automatic cleanup on session termination
//
// Example Usage:
//
//	// Create a new terminal session
//	session := terminal.create_session(shell: "/bin/zsh", working_dir: "/home/user")
//	// → Returns session_id
//
//	// Write input to terminal
//	terminal.write(session_id: "abc123", input: "ls -la\n")
//
//	// Read output from terminal
//	output := terminal.read(session_id: "abc123")
//	// → Returns buffered output
//
//	// Resize terminal
//	terminal.resize(session_id: "abc123", cols: 80, rows: 24)
//
//	// List active sessions
//	sessions := terminal.list_sessions()
//
//	// Kill session
//	terminal.kill(session_id: "abc123")
//
// Tools:
//   - terminal.create_session: Create new shell session with PTY
//   - terminal.write: Send input to session
//   - terminal.read: Read buffered output from session
//   - terminal.resize: Resize terminal dimensions
//   - terminal.list_sessions: List all active sessions
//   - terminal.kill: Terminate session and cleanup
package terminal
