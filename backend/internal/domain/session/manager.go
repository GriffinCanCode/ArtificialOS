package session

import (
	"context"
	"encoding/json"
	"fmt"
	"path/filepath"
	"sync"
	"time"

	"github.com/GriffinCanCode/AgentOS/backend/internal/shared/types"
)

// AppManager interface for getting app state
type AppManager interface {
	List(state *types.State) []*types.App
	Get(id string) (*types.App, bool)
	Spawn(ctx context.Context, request string, uiSpec map[string]interface{}, parentID *string) (*types.App, error)
	Close(id string) bool
	Focus(id string) bool
	Stats() types.Stats
}

// KernelClient interface for file operations
type KernelClient interface {
	ExecuteSyscall(ctx context.Context, pid uint32, syscallType string, params map[string]interface{}) ([]byte, error)
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

// SaveOptions contains options for saving a session
type SaveOptions struct {
	Name           string
	Description    string
	ChatState      *types.ChatState
	UIState        *types.UIState
	ComponentState map[string]map[string]interface{} // appID -> component state
}

// Save captures current workspace and saves to disk
func (m *Manager) Save(ctx context.Context, name string, description string) (*types.Session, error) {
	return m.SaveWithOptions(ctx, SaveOptions{
		Name:        name,
		Description: description,
	})
}

// SaveWithOptions captures current workspace with additional state and saves to disk
func (m *Manager) SaveWithOptions(ctx context.Context, opts SaveOptions) (*types.Session, error) {
	// Capture workspace state WITHOUT holding lock
	workspace, err := m.captureWorkspaceWithState(opts.ComponentState)
	if err != nil {
		return nil, fmt.Errorf("failed to capture workspace: %w", err)
	}

	// Add chat and UI state if provided
	workspace.ChatState = opts.ChatState
	workspace.UIState = opts.UIState

	// Create session
	now := time.Now()
	session := &types.Session{
		ID:          fmt.Sprintf("session-%s", now.Format("20060102-150405")),
		Name:        opts.Name,
		Description: opts.Description,
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

	// Write to filesystem via kernel (I/O without holding lock)
	path := m.sessionPath(session.ID)
	params := map[string]interface{}{
		"path": path,
		"data": data,
	}

	if m.kernel != nil {
		_, err = m.kernel.ExecuteSyscall(ctx, m.storagePID, "write_file", params)
		if err != nil {
			return nil, fmt.Errorf("failed to write session: %w", err)
		}
	}

	// Cache in memory (sync.Map has its own synchronization)
	m.sessions.Store(session.ID, session)

	// Update last saved time - ONLY acquire lock for this
	m.mu.Lock()
	m.lastSaved = &now
	m.mu.Unlock()

	return session, nil
}

// SaveDefault saves session with default name
func (m *Manager) SaveDefault(ctx context.Context) (*types.Session, error) {
	return m.Save(ctx, "default", "Auto-saved session")
}

// Load restores a session from disk
func (m *Manager) Load(ctx context.Context, id string) (*types.Session, error) {
	// Check cache first
	if cached, ok := m.sessions.Load(id); ok {
		return cached.(*types.Session), nil
	}

	// Load from filesystem
	path := m.sessionPath(id)
	params := map[string]interface{}{
		"path": path,
	}

	data, err := m.kernel.ExecuteSyscall(ctx, m.storagePID, "read_file", params)
	if err != nil {
		return nil, fmt.Errorf("failed to read session: %w", err)
	}

	// Unmarshal and validate
	var session types.Session
	if err := json.Unmarshal(data, &session); err != nil {
		return nil, fmt.Errorf("failed to unmarshal session %s: %w", id, err)
	}

	// Basic validation
	if session.ID == "" {
		return nil, fmt.Errorf("session %s has empty ID field", id)
	}

	// Cache
	m.sessions.Store(id, &session)

	return &session, nil
}

// Restore applies a saved session to the workspace
func (m *Manager) Restore(ctx context.Context, id string) error {
	// Load session WITHOUT holding lock (does I/O)
	session, err := m.Load(ctx, id)
	if err != nil {
		return fmt.Errorf("failed to load session: %w", err)
	}

	// Clear current workspace (close all apps)
	// Use a consistent snapshot to avoid concurrent modification issues
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
	for i := range session.Workspace.Apps {
		snapshot := &session.Workspace.Apps[i]
		if snapshot.ParentID == nil {
			newApp, err := m.restoreApp(ctx, snapshot, nil, appMap)
			if err != nil {
				return fmt.Errorf("failed to restore app %s: %w", snapshot.ID, err)
			}
			appMap[snapshot.ID] = newApp.ID
		}
	}

	// Second pass: restore child apps
	for i := range session.Workspace.Apps {
		snapshot := &session.Workspace.Apps[i]
		if snapshot.ParentID != nil {
			newParentID, ok := appMap[*snapshot.ParentID]
			if !ok {
				continue // Parent wasn't restored, skip child
			}
			newApp, err := m.restoreApp(ctx, snapshot, &newParentID, appMap)
			if err != nil {
				return fmt.Errorf("failed to restore child app %s: %w", snapshot.ID, err)
			}
			appMap[snapshot.ID] = newApp.ID
		}
	}

	// Restore focus using hash (more reliable than ID)
	// Try hash-based matching first, fall back to ID mapping
	if session.Workspace.FocusedHash != nil {
		// Find app with matching hash using type assertion safely
		if finder, ok := m.appManager.(interface {
			FindByHash(string) (*types.App, bool)
		}); ok {
			if app, found := finder.FindByHash(*session.Workspace.FocusedHash); found {
				m.appManager.Focus(app.ID)
			}
		}
	} else if session.Workspace.FocusedID != nil {
		// Fallback to ID mapping (for backwards compatibility)
		if newID, ok := appMap[*session.Workspace.FocusedID]; ok {
			m.appManager.Focus(newID)
		}
	}

	// Update last restored time - ONLY acquire lock for this
	now := time.Now()
	m.mu.Lock()
	m.lastRestored = &now
	m.mu.Unlock()

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
func (m *Manager) Delete(ctx context.Context, id string) error {
	// Delete from filesystem
	path := m.sessionPath(id)
	params := map[string]interface{}{
		"path": path,
	}

	if m.kernel != nil {
		_, err := m.kernel.ExecuteSyscall(ctx, m.storagePID, "delete_file", params)
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

	// Read timestamp pointers under lock to prevent data races
	m.mu.RLock()
	lastSaved := m.lastSaved
	lastRestored := m.lastRestored
	m.mu.RUnlock()

	return types.SessionStats{
		TotalSessions: total,
		LastSaved:     lastSaved,
		LastRestored:  lastRestored,
	}
}

// captureWorkspace captures current workspace state (legacy, for backward compatibility)
func (m *Manager) captureWorkspace() (*types.Workspace, error) {
	return m.captureWorkspaceWithState(nil)
}

// captureWorkspaceWithState captures current workspace state with component state
func (m *Manager) captureWorkspaceWithState(componentStates map[string]map[string]interface{}) (*types.Workspace, error) {
	// Get all apps
	apps := m.appManager.List(nil)

	// Convert to snapshots
	snapshots := make([]types.AppSnapshot, len(apps))
	for i, app := range apps {
		// Get component state for this app if provided
		var compState map[string]interface{}
		if componentStates != nil {
			if state, ok := componentStates[app.ID]; ok {
				compState = state
			}
		}
		if compState == nil {
			compState = map[string]interface{}{}
		}

		snapshots[i] = types.AppSnapshot{
			ID:             app.ID,
			Hash:           app.Hash,
			Title:          app.Title,
			Blueprint:      app.Blueprint,
			State:          app.State,
			ParentID:       app.ParentID,
			CreatedAt:      app.CreatedAt,
			Metadata:       app.Metadata,
			Services:       app.Services,
			ComponentState: compState,
		}
	}

	// Get focused app
	stats := m.appManager.Stats()

	workspace := &types.Workspace{
		Apps:        snapshots,
		FocusedID:   stats.FocusedAppID,
		FocusedHash: stats.FocusedAppHash,
		ChatState:   nil, // Will be set by caller
		UIState:     nil, // Will be set by caller
	}

	return workspace, nil
}

// restoreApp restores a single app from snapshot
func (m *Manager) restoreApp(ctx context.Context, snapshot *types.AppSnapshot, parentID *string, appMap map[string]string) (*types.App, error) {
	// Get original request from metadata
	request := "Restore app"
	if req, ok := snapshot.Metadata["request"].(string); ok {
		request = req
	}

	// Spawn app with saved UISpec
	app, err := m.appManager.Spawn(ctx, request, snapshot.Blueprint, parentID)
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
