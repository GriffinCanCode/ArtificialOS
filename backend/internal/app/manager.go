package app

import (
	"sync"
	"time"

	"github.com/GriffinCanCode/AgentOS/backend/internal/types"
	"github.com/GriffinCanCode/AgentOS/backend/internal/utils"
	"github.com/google/uuid"
)

// Manager orchestrates app lifecycle
type Manager struct {
	apps          sync.Map
	focusedID     *string
	focusedHash   *string // Hash of focused app for restoration
	mu            sync.RWMutex
	kernelGRPC    KernelClient
	appIdentifier *utils.AppIdentifier
}

// KernelClient interface for dependency injection
type KernelClient interface {
	CreateProcess(name string, priority uint32, sandboxLevel string) (*uint32, error)
}

// NewManager creates a new app manager
func NewManager(kernelClient KernelClient) *Manager {
	return &Manager{
		kernelGRPC:    kernelClient,
		appIdentifier: utils.NewAppIdentifier(utils.DefaultHasher()),
	}
}

// Spawn creates a new app instance
func (m *Manager) Spawn(request string, uiSpec map[string]interface{}, parentID *string) (*types.App, error) {
	title, _ := uiSpec["title"].(string)
	if title == "" {
		title = "Untitled App"
	}

	services, _ := uiSpec["services"].([]string)
	if services == nil {
		services = []string{}
	}

	// Create metadata
	metadata := map[string]interface{}{"request": request}

	// Generate deterministic hash for app identification
	hash := m.appIdentifier.GenerateHash(title, parentID, metadata)

	// Create sandboxed process if kernel available
	var sandboxPID *uint32
	if m.kernelGRPC != nil && len(services) > 0 {
		if pid, err := m.kernelGRPC.CreateProcess(title, 5, "STANDARD"); err == nil {
			sandboxPID = pid
		}
	}

	app := &types.App{
		ID:         uuid.New().String(),
		Hash:       hash,
		Title:      title,
		UISpec:     uiSpec,
		State:      types.StateActive,
		ParentID:   parentID,
		CreatedAt:  time.Now(),
		Metadata:   metadata,
		Services:   services,
		SandboxPID: sandboxPID,
	}

	m.apps.Store(app.ID, app)
	m.setFocused(app.ID, hash)

	return app, nil
}

// Get retrieves an app by ID
func (m *Manager) Get(id string) (*types.App, bool) {
	val, ok := m.apps.Load(id)
	if !ok {
		return nil, false
	}
	return val.(*types.App), true
}

// List returns all apps, optionally filtered by state
func (m *Manager) List(state *types.State) []*types.App {
	var apps []*types.App
	m.apps.Range(func(_, value interface{}) bool {
		app := value.(*types.App)
		if state == nil || app.State == *state {
			apps = append(apps, app)
		}
		return true
	})
	return apps
}

// Focus brings an app to foreground
func (m *Manager) Focus(id string) bool {
	app, ok := m.Get(id)
	if !ok {
		return false
	}

	// Unfocus current
	m.mu.RLock()
	currentID := m.focusedID
	m.mu.RUnlock()

	if currentID != nil && *currentID != id {
		if current, ok := m.Get(*currentID); ok && current.State == types.StateActive {
			current.State = types.StateBackground
			m.apps.Store(*currentID, current)
		}
	}

	// Focus new
	app.State = types.StateActive
	m.apps.Store(id, app)
	m.setFocused(id, app.Hash)

	return true
}

// Close destroys an app and its children
func (m *Manager) Close(id string) bool {
	app, ok := m.Get(id)
	if !ok {
		return false
	}

	// Close children first
	m.apps.Range(func(_, value interface{}) bool {
		child := value.(*types.App)
		if child.ParentID != nil && *child.ParentID == id {
			m.Close(child.ID)
		}
		return true
	})

	// Mark destroyed and remove
	app.State = types.StateDestroyed
	m.apps.Delete(id)

	// Update focus
	m.mu.Lock()
	if m.focusedID != nil && *m.focusedID == id {
		m.focusedID = nil
		// Auto-focus another app
		active := m.List(nil)
		if len(active) > 0 {
			m.mu.Unlock()
			m.Focus(active[0].ID)
			return true
		}
	}
	m.mu.Unlock()

	return true
}

// Stats returns manager statistics
func (m *Manager) Stats() types.Stats {
	var total, active, background int
	m.apps.Range(func(_, value interface{}) bool {
		app := value.(*types.App)
		total++
		if app.State == types.StateActive {
			active++
		} else if app.State == types.StateBackground {
			background++
		}
		return true
	})

	m.mu.RLock()
	focusedID := m.focusedID
	focusedHash := m.focusedHash
	m.mu.RUnlock()

	return types.Stats{
		TotalApps:      total,
		ActiveApps:     active,
		BackgroundApps: background,
		FocusedAppID:   focusedID,
		FocusedAppHash: focusedHash,
	}
}

func (m *Manager) setFocused(id string, hash string) {
	m.mu.Lock()
	m.focusedID = &id
	m.focusedHash = &hash
	m.mu.Unlock()
}

// FindByHash finds an app by its hash (for restoration)
func (m *Manager) FindByHash(hash string) (*types.App, bool) {
	var found *types.App
	m.apps.Range(func(_, value interface{}) bool {
		app := value.(*types.App)
		if app.Hash == hash {
			found = app
			return false // Stop iteration
		}
		return true
	})
	if found != nil {
		return found, true
	}
	return nil, false
}
