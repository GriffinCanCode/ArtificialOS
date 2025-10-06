// Package app provides application lifecycle management for the AI-OS system.
//
// This package handles the creation, management, and termination of application
// instances. Each app runs in a sandboxed environment managed by the kernel.
//
// Key Components:
//   - Manager: Central app lifecycle coordinator
//   - App state management (active, background, focused)
//   - Process creation and sandbox integration
//   - Window state tracking for session restoration
//
// Example Usage:
//
//	manager := app.NewManager(kernelClient)
//	app, err := manager.Spawn(ctx, "calculator", blueprint, nil)
//	if err != nil {
//	    log.Fatal(err)
//	}
//	manager.Focus(app.ID)
package app
