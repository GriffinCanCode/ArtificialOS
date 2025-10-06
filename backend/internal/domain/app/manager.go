package app

import (
	"context"
	"sync"
	"time"

	"github.com/GriffinCanCode/AgentOS/backend/internal/grpc/kernel"
	"github.com/GriffinCanCode/AgentOS/backend/internal/infrastructure/monitoring"
	"github.com/GriffinCanCode/AgentOS/backend/internal/shared/types"
	"github.com/GriffinCanCode/AgentOS/backend/internal/shared/utils"
	"github.com/google/uuid"
)

// Manager orchestrates app lifecycle
type Manager struct {
	mu            sync.RWMutex
	apps          map[string]*types.App // Protected by mu
	focusedID     *string               // Protected by mu
	focusedHash   *string               // Protected by mu
	kernelGRPC    KernelClient
	appIdentifier *utils.AppIdentifier
	metrics       *monitoring.Metrics
}

// KernelClient interface for dependency injection
type KernelClient interface {
	CreateProcess(ctx context.Context, name string, priority uint32, sandboxLevel string, opts *kernel.CreateProcessOptions) (*uint32, *uint32, error)
}

// NewManager creates a new app manager
func NewManager(kernelClient KernelClient) *Manager {
	return &Manager{
		apps:          make(map[string]*types.App),
		kernelGRPC:    kernelClient,
		appIdentifier: utils.NewAppIdentifier(utils.DefaultHasher()),
	}
}

// WithMetrics adds metrics tracking to the manager
func (m *Manager) WithMetrics(metrics *monitoring.Metrics) *Manager {
	m.metrics = metrics
	return m
}

// Spawn creates a new app instance
func (m *Manager) Spawn(ctx context.Context, request string, blueprint map[string]interface{}, parentID *string) (*types.App, error) {
	title, _ := blueprint["title"].(string)
	if title == "" {
		title = "Untitled App"
	}

	services, _ := blueprint["services"].([]string)
	if services == nil {
		services = []string{}
	}

	// Create metadata
	metadata := map[string]interface{}{"request": request}

	// Generate deterministic hash for app identification
	hash := m.appIdentifier.GenerateHash(title, parentID, metadata)

	// Create sandboxed process if kernel available
	var sandboxPID, osPID *uint32
	if m.kernelGRPC != nil && len(services) > 0 {
		// Check if app metadata requests OS execution
		enableOS, _ := metadata["enable_os_execution"].(bool)
		command, _ := metadata["os_command"].(string)

		var opts *kernel.CreateProcessOptions
		if enableOS && command != "" {
			// Spawn actual OS process for this app
			opts = &kernel.CreateProcessOptions{
				Command: command,
				Args:    []string{},
				EnvVars: []string{},
			}
		}

		// Create sandbox process (with or without OS execution)
		if pid, os_pid, err := m.kernelGRPC.CreateProcess(ctx, title, 5, "STANDARD", opts); err == nil {
			sandboxPID = pid
			osPID = os_pid
		}
	}

	app := &types.App{
		ID:         uuid.New().String(),
		Hash:       hash,
		Title:      title,
		Blueprint:  blueprint,
		State:      types.StateActive,
		ParentID:   parentID,
		CreatedAt:  time.Now(),
		Metadata:   metadata,
		Services:   services,
		SandboxPID: sandboxPID,
		OSPID:      osPID,
	}

	m.mu.Lock()
	// Unfocus current app
	if m.focusedID != nil {
		if currentApp, exists := m.apps[*m.focusedID]; exists && currentApp.State == types.StateActive {
			currentApp.State = types.StateBackground
		}
	}

	m.apps[app.ID] = app
	m.focusedID = &app.ID
	m.focusedHash = &hash
	m.mu.Unlock()

	return app, nil
}

// Get retrieves an app by ID
func (m *Manager) Get(id string) (*types.App, bool) {
	m.mu.RLock()
	defer m.mu.RUnlock()

	app, ok := m.apps[id]
	if !ok {
		return nil, false
	}

	// Return a copy to prevent external modifications
	appCopy := *app
	return &appCopy, true
}

// List returns all apps, optionally filtered by state
func (m *Manager) List(state *types.State) []*types.App {
	m.mu.RLock()
	defer m.mu.RUnlock()

	apps := make([]*types.App, 0, len(m.apps))
	for _, app := range m.apps {
		if state == nil || app.State == *state {
			// Return copies to prevent external modifications
			appCopy := *app
			apps = append(apps, &appCopy)
		}
	}
	return apps
}

// Focus brings an app to foreground
func (m *Manager) Focus(id string) bool {
	m.mu.Lock()
	defer m.mu.Unlock()

	app, ok := m.apps[id]
	if !ok {
		return false
	}

	// Unfocus current app
	if m.focusedID != nil && *m.focusedID != id {
		if currentApp, exists := m.apps[*m.focusedID]; exists && currentApp.State == types.StateActive {
			currentApp.State = types.StateBackground
		}
	}

	// Focus new app
	app.State = types.StateActive
	m.focusedID = &id
	m.focusedHash = &app.Hash

	return true
}

// Close destroys an app and its children
func (m *Manager) Close(id string) bool {
	m.mu.Lock()
	defer m.mu.Unlock()

	if _, ok := m.apps[id]; !ok {
		return false
	}

	// Collect children IDs first (to avoid recursive locking)
	var childIDs []string
	for _, child := range m.apps {
		if child.ParentID != nil && *child.ParentID == id {
			childIDs = append(childIDs, child.ID)
		}
	}

	// Close children (within same lock)
	for _, childID := range childIDs {
		m.closeApp(childID)
	}

	// Close this app
	m.closeApp(id)

	// Update focus if this was the focused app
	if m.focusedID != nil && *m.focusedID == id {
		m.focusedID = nil
		m.focusedHash = nil

		// Auto-focus another active app
		for _, app := range m.apps {
			if app.State == types.StateActive || app.State == types.StateBackground {
				m.focusedID = &app.ID
				m.focusedHash = &app.Hash
				app.State = types.StateActive
				break
			}
		}
	}

	return true
}

// closeApp removes an app without handling focus (internal, must hold lock)
func (m *Manager) closeApp(id string) {
	if app, ok := m.apps[id]; ok {
		app.State = types.StateDestroyed
		delete(m.apps, id)
	}
}

// Stats returns manager statistics
func (m *Manager) Stats() types.Stats {
	m.mu.RLock()
	defer m.mu.RUnlock()

	var total, active, background int
	for _, app := range m.apps {
		total++
		if app.State == types.StateActive {
			active++
		} else if app.State == types.StateBackground {
			background++
		}
	}

	// Copy pointers to avoid race
	var focusedID, focusedHash *string
	if m.focusedID != nil {
		id := *m.focusedID
		focusedID = &id
	}
	if m.focusedHash != nil {
		hash := *m.focusedHash
		focusedHash = &hash
	}

	return types.Stats{
		TotalApps:      total,
		ActiveApps:     active,
		BackgroundApps: background,
		FocusedAppID:   focusedID,
		FocusedAppHash: focusedHash,
	}
}

// FindByHash finds an app by its hash (for restoration)
func (m *Manager) FindByHash(hash string) (*types.App, bool) {
	m.mu.RLock()
	defer m.mu.RUnlock()

	for _, app := range m.apps {
		if app.Hash == hash {
			// Return copy to prevent external modifications
			appCopy := *app
			return &appCopy, true
		}
	}
	return nil, false
}

// UpdateWindow updates window state for an app
func (m *Manager) UpdateWindow(id string, windowID string, pos *types.WindowPosition, size *types.WindowSize) bool {
	m.mu.Lock()
	defer m.mu.Unlock()

	app, ok := m.apps[id]
	if !ok {
		return false
	}

	app.WindowID = &windowID
	app.WindowPos = pos
	app.WindowSize = size

	return true
}
