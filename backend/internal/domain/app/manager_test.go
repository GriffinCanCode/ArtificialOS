package app

import (
	"context"
	"testing"

	"github.com/GriffinCanCode/AgentOS/backend/internal/grpc/kernel"
	"github.com/GriffinCanCode/AgentOS/backend/internal/shared/types"
)

type mockKernel struct{}

func (m *mockKernel) CreateProcess(ctx context.Context, name string, priority uint32, sandboxLevel string, opts *kernel.CreateProcessOptions) (*uint32, *uint32, error) {
	pid := uint32(123)
	osPid := uint32(456)
	return &pid, &osPid, nil
}

func TestSpawn(t *testing.T) {
	m := NewManager(&mockKernel{})

	uiSpec := map[string]interface{}{
		"title":    "Test App",
		"services": []string{"storage"},
	}

	ctx := context.Background()
	app, err := m.Spawn(ctx, "create test app", uiSpec, nil)
	if err != nil {
		t.Fatalf("Spawn failed: %v", err)
	}

	if app.Title != "Test App" {
		t.Errorf("Expected title 'Test App', got '%s'", app.Title)
	}

	if app.State != types.StateActive {
		t.Errorf("Expected state Active, got %s", app.State)
	}

	if app.SandboxPID == nil {
		t.Error("Expected sandbox PID to be set")
	}
}

func TestFocus(t *testing.T) {
	m := NewManager(&mockKernel{})

	ctx := context.Background()
	app1, _ := m.Spawn(ctx, "app1", map[string]interface{}{"title": "App 1"}, nil)
	app2, _ := m.Spawn(ctx, "app2", map[string]interface{}{"title": "App 2"}, nil)

	// Focus should move app1 to background
	if !m.Focus(app2.ID) {
		t.Fatal("Focus failed")
	}

	updated, _ := m.Get(app1.ID)
	if updated.State != types.StateBackground {
		t.Error("Expected first app to be in background")
	}
}

func TestClose(t *testing.T) {
	m := NewManager(&mockKernel{})

	ctx := context.Background()
	parent, _ := m.Spawn(ctx, "parent", map[string]interface{}{"title": "Parent"}, nil)
	child, _ := m.Spawn(ctx, "child", map[string]interface{}{"title": "Child"}, &parent.ID)

	// Closing parent should close child
	if !m.Close(parent.ID) {
		t.Fatal("Close failed")
	}

	if _, ok := m.Get(parent.ID); ok {
		t.Error("Parent should be deleted")
	}

	if _, ok := m.Get(child.ID); ok {
		t.Error("Child should be deleted")
	}
}

func TestStats(t *testing.T) {
	m := NewManager(&mockKernel{})

	ctx := context.Background()
	m.Spawn(ctx, "app1", map[string]interface{}{"title": "App 1"}, nil)
	m.Spawn(ctx, "app2", map[string]interface{}{"title": "App 2"}, nil)

	stats := m.Stats()
	if stats.TotalApps != 2 {
		t.Errorf("Expected 2 total apps, got %d", stats.TotalApps)
	}

	if stats.ActiveApps != 1 {
		t.Errorf("Expected 1 active app, got %d", stats.ActiveApps)
	}

	if stats.BackgroundApps != 1 {
		t.Errorf("Expected 1 background app, got %d", stats.BackgroundApps)
	}
}
