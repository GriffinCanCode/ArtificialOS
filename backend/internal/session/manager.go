package session

import (
	"encoding/json"
	"fmt"
	"path/filepath"
	"sync"
	"time"

	"github.com/GriffinCanCode/AgentOS/backend/internal/types"
)

// AppManager interface for getting app state
type AppManager interface {
	List(state *types.State) []*types.App
	Get(id string) (*types.App, bool)
	Spawn(request string, uiSpec map[string]interface{}, parentID *string) (*types.App, error)
	Close(id string) bool
	Focus(id string) bool
	Stats() types.Stats
}

// KernelClient interface for file operations
type KernelClient interface {
	ExecuteSyscall(pid uint32, syscallType string, params map[string]interface{}) ([]byte, error)
}

// Manager handles session persistence
type Manager struct {
	sessions     sync.Map
	appManager   AppManager
	kernel       KernelClient
	storagePID   uint32
	storagePath  string
	mu           sync.RWMutex
	lastSaved    *time.Time
	lastRestored *time.Time
}

// NewManager creates a new session manager
func NewManager(appManager AppManager, kernel KernelClient, storagePID uint32, storagePath string) *Manager {
	return &Manager{
		appManager:  appManager,
		kernel:      kernel,
		storagePID:  storagePID,
		storagePath: storagePath,
	}
}

// Save captures current workspace and saves to disk
func (m *Manager) Save(name string, description string) (*types.Session, error) {
	m.mu.Lock()
	defer m.mu.Unlock()

	// Capture workspace state
	workspace, err := m.captureWorkspace()
	if err != nil {
		return nil, fmt.Errorf("failed to capture workspace: %w", err)
	}

	// Create session
	now := time.Now()
	session := &types.Session{
		ID:          fmt.Sprintf("session-%s", now.Format("20060102-150405")),
		Name:        name,
		Description: description,
		CreatedAt:   now,
		UpdatedAt:   now,
		Workspace:   *workspace,
		Metadata:    map[string]interface{}{},
	}

	// Marshal to JSON
	data, err := json.Marshal(session)
	if err != nil {
		return nil, fmt.Errorf("failed to marshal session: %w", err)
	}

	// Write to filesystem via kernel
	path := m.sessionPath(session.ID)
	params := map[string]interface{}{
		"path": path,
		"data": data,
	}

	if m.kernel != nil {
		_, err = m.kernel.ExecuteSyscall(m.storagePID, "write_file", params)
		if err != nil {
			return nil, fmt.Errorf("failed to write session: %w", err)
		}
	}

	// Cache in memory
	m.sessions.Store(session.ID, session)

	// Update last saved time
	m.lastSaved = &now

	return session, nil
}

// SaveDefault saves session with default name
func (m *Manager) SaveDefault() (*types.Session, error) {
	return m.Save("default", "Auto-saved session")
}

// Load restores a session from disk
func (m *Manager) Load(id string) (*types.Session, error) {
	// Check cache first
	if cached, ok := m.sessions.Load(id); ok {
		return cached.(*types.Session), nil
	}

	// Load from filesystem
	path := m.sessionPath(id)
	params := map[string]interface{}{
		"path": path,
	}

	data, err := m.kernel.ExecuteSyscall(m.storagePID, "read_file", params)
	if err != nil {
		return nil, fmt.Errorf("failed to read session: %w", err)
	}

	// Unmarshal
	var session types.Session
	if err := json.Unmarshal(data, &session); err != nil {
		return nil, fmt.Errorf("failed to unmarshal session: %w", err)
	}

	// Cache
	m.sessions.Store(id, &session)

	return &session, nil
}

// Restore applies a saved session to the workspace
func (m *Manager) Restore(id string) error {
	m.mu.Lock()
	defer m.mu.Unlock()

	// Load session
	session, err := m.Load(id)
	if err != nil {
		return fmt.Errorf("failed to load session: %w", err)
	}

	// Clear current workspace (close all apps)
	currentApps := m.appManager.List(nil)
	for _, app := range currentApps {
		// Only close top-level apps (children will be closed automatically)
		if app.ParentID == nil {
			m.appManager.Close(app.ID)
		}
	}

	// Restore apps in order (parents before children)
	appMap := make(map[string]string) // old ID -> new ID

	// First pass: restore apps without parents
	for _, snapshot := range session.Workspace.Apps {
		if snapshot.ParentID == nil {
			newApp, err := m.restoreApp(&snapshot, nil, appMap)
			if err != nil {
				return fmt.Errorf("failed to restore app %s: %w", snapshot.ID, err)
			}
			appMap[snapshot.ID] = newApp.ID
		}
	}

	// Second pass: restore child apps
	for _, snapshot := range session.Workspace.Apps {
		if snapshot.ParentID != nil {
			newParentID, ok := appMap[*snapshot.ParentID]
			if !ok {
				continue // Parent wasn't restored, skip child
			}
			newApp, err := m.restoreApp(&snapshot, &newParentID, appMap)
			if err != nil {
				return fmt.Errorf("failed to restore child app %s: %w", snapshot.ID, err)
			}
			appMap[snapshot.ID] = newApp.ID
		}
	}

	// Restore focus
	if session.Workspace.FocusedID != nil {
		if newID, ok := appMap[*session.Workspace.FocusedID]; ok {
			m.appManager.Focus(newID)
		}
	}

	// Update last restored time
	now := time.Now()
	m.lastRestored = &now

	return nil
}

// List returns all saved sessions
func (m *Manager) List() ([]types.SessionMetadata, error) {
	var metadata []types.SessionMetadata

	m.sessions.Range(func(_, value interface{}) bool {
		session := value.(*types.Session)
		metadata = append(metadata, session.ToMetadata())
		return true
	})

	return metadata, nil
}

// Delete removes a session
func (m *Manager) Delete(id string) error {
	// Delete from filesystem
	path := m.sessionPath(id)
	params := map[string]interface{}{
		"path": path,
	}

	if m.kernel != nil {
		_, err := m.kernel.ExecuteSyscall(m.storagePID, "delete_file", params)
		if err != nil {
			return fmt.Errorf("failed to delete session: %w", err)
		}
	}

	// Remove from cache
	m.sessions.Delete(id)

	return nil
}

// Stats returns session manager statistics
func (m *Manager) Stats() types.SessionStats {
	var total int
	m.sessions.Range(func(_, _ interface{}) bool {
		total++
		return true
	})

	return types.SessionStats{
		TotalSessions: total,
		LastSaved:     m.lastSaved,
		LastRestored:  m.lastRestored,
	}
}

// captureWorkspace captures current workspace state
func (m *Manager) captureWorkspace() (*types.Workspace, error) {
	// Get all apps
	apps := m.appManager.List(nil)

	// Convert to snapshots
	snapshots := make([]types.AppSnapshot, len(apps))
	for i, app := range apps {
		snapshots[i] = types.AppSnapshot{
			ID:             app.ID,
			Title:          app.Title,
			UISpec:         app.UISpec,
			State:          app.State,
			ParentID:       app.ParentID,
			CreatedAt:      app.CreatedAt,
			Metadata:       app.Metadata,
			Services:       app.Services,
			ComponentState: map[string]interface{}{}, // TODO: Capture component state from frontend
		}
	}

	// Get focused app
	stats := m.appManager.Stats()
	var focusedID *string
	if stats.FocusedApp != nil {
		// Find app ID by title
		for _, app := range apps {
			if app.Title == *stats.FocusedApp {
				focusedID = &app.ID
				break
			}
		}
	}

	workspace := &types.Workspace{
		Apps:      snapshots,
		FocusedID: focusedID,
		ChatState: nil, // TODO: Capture from frontend
		UIState:   nil, // TODO: Capture from frontend
	}

	return workspace, nil
}

// restoreApp restores a single app from snapshot
func (m *Manager) restoreApp(snapshot *types.AppSnapshot, parentID *string, appMap map[string]string) (*types.App, error) {
	// Get original request from metadata
	request := "Restore app"
	if req, ok := snapshot.Metadata["request"].(string); ok {
		request = req
	}

	// Spawn app with saved UISpec
	app, err := m.appManager.Spawn(request, snapshot.UISpec, parentID)
	if err != nil {
		return nil, err
	}

	// Restore state
	app.State = snapshot.State

	// Restore metadata (merge with new metadata)
	for k, v := range snapshot.Metadata {
		app.Metadata[k] = v
	}
	app.Metadata["restored_from_session"] = true
	app.Metadata["original_id"] = snapshot.ID

	return app, nil
}

// sessionPath generates the filesystem path for a session
func (m *Manager) sessionPath(id string) string {
	return filepath.Join(m.storagePath, "sessions", id+".session")
}
