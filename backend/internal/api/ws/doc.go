// Package ws provides WebSocket handling for real-time AI interactions.
//
// This package implements WebSocket communication for streaming chat and
// UI generation, enabling real-time updates from the AI service.
//
// Features:
//   - Chat streaming with thoughts and tokens
//   - UI generation streaming with previews
//   - Automatic connection upgrade from HTTP
//   - Context-based cancellation
//   - Error handling and recovery
//
// Message Types (Client → Server):
//   - chat: Send chat message with context
//   - generate_ui: Request UI generation
//   - ping: Keep-alive ping
//
// Message Types (Server → Client):
//   - generation_start: UI generation started
//   - thought: AI thought stream
//   - token: Chat token stream
//   - ui_generated: Complete UI specification
//   - complete: Operation finished
//   - error: Error occurred
//
// Example Usage:
//
//	handler := ws.NewHandler(appManager, aiClient)
//	router.GET("/stream", handler.HandleConnection)
package ws
