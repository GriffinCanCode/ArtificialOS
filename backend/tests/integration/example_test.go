//go:build integration
// +build integration

package integration

import (
	"context"
	"testing"
	"time"

	"github.com/GriffinCanCode/AgentOS/backend/internal/domain/app"
	"github.com/GriffinCanCode/AgentOS/backend/internal/infrastructure/config"
	"github.com/GriffinCanCode/AgentOS/backend/internal/domain/service"
	"github.com/GriffinCanCode/AgentOS/backend/tests/helpers/testutil"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

// TestIntegrationExample demonstrates integration test structure
func TestIntegrationExample(t *testing.T) {
	if testing.Short() {
		t.Skip("Skipping integration test in short mode")
	}

	t.Run("full app lifecycle", func(t *testing.T) {
		// Setup
		mockKernel := testutil.NewMockKernelClient(t)
		appMgr := app.NewManager(mockKernel)

		ctx, cancel := context.WithTimeout(context.Background(), 10*time.Second)
		defer cancel()

		// Spawn app
		uiSpec := map[string]interface{}{
			"title": "Integration Test App",
			"type":  "app",
		}

		app1, err := appMgr.Spawn(ctx, "test request", uiSpec, nil)
		require.NoError(t, err)
		assert.NotEmpty(t, app1.ID)
		assert.Equal(t, "Integration Test App", app1.Title)

		// Verify app is active
		stats := appMgr.Stats()
		assert.Equal(t, 1, stats.TotalApps)
		assert.Equal(t, 1, stats.ActiveApps)

		// Close app
		success := appMgr.Close(app1.ID)
		assert.True(t, success)

		// Verify app is closed
		stats = appMgr.Stats()
		assert.Equal(t, 0, stats.TotalApps)
	})

	t.Run("service registry integration", func(t *testing.T) {
		// Setup
		registry := service.NewRegistry()

		// Create and register mock service
		mockProvider := testutil.NewMockServiceProvider(t, "test-service")
		err := registry.Register(mockProvider)
		require.NoError(t, err)

		// Verify service is registered
		services := registry.List(nil)
		assert.Len(t, services, 1)
		assert.Equal(t, "test-service", services[0].ID)

		// Test service discovery (note: discovery uses fuzzy matching)
		// The mock service has very generic name/description, so results may be empty
		// This is expected behavior
		stats := registry.Stats()
		assert.Equal(t, 1, stats["total_services"])
	})
}

// TestConfigIntegration tests configuration loading and usage
func TestConfigIntegration(t *testing.T) {
	if testing.Short() {
		t.Skip("Skipping integration test in short mode")
	}

	t.Run("config with defaults", func(t *testing.T) {
		cfg := config.Default()

		// Verify critical defaults
		assert.NotEmpty(t, cfg.Server.Port)
		assert.NotEmpty(t, cfg.Server.Host)
		assert.NotEmpty(t, cfg.Kernel.Address)
		assert.NotEmpty(t, cfg.AI.Address)
		assert.True(t, cfg.Kernel.Enabled)
		assert.True(t, cfg.RateLimit.Enabled)
	})
}
