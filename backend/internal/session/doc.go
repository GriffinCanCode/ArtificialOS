// Package session provides workspace session management for AI-OS.
//
// Sessions capture and restore the complete state of the desktop environment
// including all running apps, window positions, chat history, and UI state.
//
// Components:
//   - Manager: Session persistence and restoration
//   - Workspace snapshots with app hierarchy
//   - Chat and thought history
//   - Window state tracking
//
// Session Structure:
//   - Apps: Complete app snapshots with state
//   - FocusedID/Hash: Currently focused app
//   - ChatState: Message and thought history
//   - UIState: Generation preview and loading state
//
// Restoration Process:
//  1. Load session JSON from storage
//  2. Close all current apps
//  3. Restore apps in dependency order (parents first)
//  4. Map old IDs to new IDs
//  5. Restore focus state
//
// Example Usage:
//
//	manager := session.NewManager(appMgr, kernel, storagePID, storagePath)
//	session, err := manager.Save(ctx, "My Workspace", "description")
//	err = manager.Restore(ctx, session.ID)
package session
