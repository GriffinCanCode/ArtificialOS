// Package types provides shared data structures for the AI-OS backend.
//
// This package defines core types used across all backend components,
// ensuring type safety and consistent data structures.
//
// Core Types:
//   - App: Running application instance
//   - Package: Installable application package
//   - Service: Service provider definition
//   - Tool: Service tool specification
//   - Session: Workspace snapshot
//   - Context: Execution context for operations
//   - Result: Standard operation result
//
// Request Types:
//   - ChatRequest, UIRequest: AI interaction
//   - ExecuteRequest: Service tool execution
//   - WSMessage: WebSocket communication
//
// State Management:
//   - State: App state enum (active, background)
//   - WindowPosition, WindowSize: Window geometry
//   - Stats: System statistics
//
// Example Usage:
//
//	app := &types.App{
//	    ID:        string(id.NewAppID()),
//	    Title:     "Calculator",
//	    State:     types.StateActive,
//	    Blueprint: uiSpec,
//	}
package types
