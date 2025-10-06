package config

import (
	"os"
	"testing"

	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

func TestDefault(t *testing.T) {
	cfg := Default()

	// Server config
	assert.Equal(t, "8000", cfg.Server.Port)
	assert.Equal(t, "0.0.0.0", cfg.Server.Host)

	// Kernel config
	assert.Equal(t, "localhost:50051", cfg.Kernel.Address)
	assert.True(t, cfg.Kernel.Enabled)

	// AI config
	assert.Equal(t, "localhost:50052", cfg.AI.Address)

	// Logging config
	assert.Equal(t, "info", cfg.Logging.Level)
	assert.False(t, cfg.Logging.Development)

	// Rate limit config
	assert.Equal(t, 100, cfg.RateLimit.RequestsPerSecond)
	assert.Equal(t, 200, cfg.RateLimit.Burst)
	assert.True(t, cfg.RateLimit.Enabled)
}

func TestLoadOrDefault(t *testing.T) {
	// Should return default when no env vars set
	cfg := LoadOrDefault()

	assert.NotNil(t, cfg)
	assert.Equal(t, "8000", cfg.Server.Port)
	assert.Equal(t, "info", cfg.Logging.Level)
}

func TestLoadWithEnvironmentVariables(t *testing.T) {
	// Setup environment variables
	envVars := map[string]string{
		"PORT":               "9000",
		"HOST":               "127.0.0.1",
		"KERNEL_ADDR":        "kernel:50051",
		"KERNEL_ENABLED":     "false",
		"AI_ADDR":            "ai:50052",
		"LOG_LEVEL":          "debug",
		"LOG_DEV":            "true",
		"RATE_LIMIT_RPS":     "500",
		"RATE_LIMIT_BURST":   "1000",
		"RATE_LIMIT_ENABLED": "false",
	}

	// Set environment variables
	for key, value := range envVars {
		err := os.Setenv(key, value)
		require.NoError(t, err)
		defer os.Unsetenv(key)
	}

	cfg, err := Load()
	require.NoError(t, err)

	// Verify server config
	assert.Equal(t, "9000", cfg.Server.Port)
	assert.Equal(t, "127.0.0.1", cfg.Server.Host)

	// Verify kernel config
	assert.Equal(t, "kernel:50051", cfg.Kernel.Address)
	assert.False(t, cfg.Kernel.Enabled)

	// Verify AI config
	assert.Equal(t, "ai:50052", cfg.AI.Address)

	// Verify logging config
	assert.Equal(t, "debug", cfg.Logging.Level)
	assert.True(t, cfg.Logging.Development)

	// Verify rate limit config
	assert.Equal(t, 500, cfg.RateLimit.RequestsPerSecond)
	assert.Equal(t, 1000, cfg.RateLimit.Burst)
	assert.False(t, cfg.RateLimit.Enabled)
}

func TestLoadWithPartialEnvironmentVariables(t *testing.T) {
	// Only set some environment variables
	err := os.Setenv("PORT", "3000")
	require.NoError(t, err)
	defer os.Unsetenv("PORT")

	err = os.Setenv("LOG_LEVEL", "warn")
	require.NoError(t, err)
	defer os.Unsetenv("LOG_LEVEL")

	cfg, err := Load()
	require.NoError(t, err)

	// Verify overridden values
	assert.Equal(t, "3000", cfg.Server.Port)
	assert.Equal(t, "warn", cfg.Logging.Level)

	// Verify default values still apply
	assert.Equal(t, "0.0.0.0", cfg.Server.Host)
	assert.Equal(t, "localhost:50051", cfg.Kernel.Address)
	assert.True(t, cfg.Kernel.Enabled)
}

func TestServerConfig(t *testing.T) {
	tests := []struct {
		name     string
		port     string
		host     string
		wantPort string
		wantHost string
	}{
		{
			name:     "default values",
			port:     "",
			host:     "",
			wantPort: "8000",
			wantHost: "0.0.0.0",
		},
		{
			name:     "custom port",
			port:     "9000",
			host:     "",
			wantPort: "9000",
			wantHost: "0.0.0.0",
		},
		{
			name:     "custom host",
			port:     "",
			host:     "localhost",
			wantPort: "8000",
			wantHost: "localhost",
		},
		{
			name:     "custom port and host",
			port:     "3000",
			host:     "127.0.0.1",
			wantPort: "3000",
			wantHost: "127.0.0.1",
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			// Clean environment
			os.Unsetenv("PORT")
			os.Unsetenv("HOST")

			// Set test values
			if tt.port != "" {
				err := os.Setenv("PORT", tt.port)
				require.NoError(t, err)
				defer os.Unsetenv("PORT")
			}
			if tt.host != "" {
				err := os.Setenv("HOST", tt.host)
				require.NoError(t, err)
				defer os.Unsetenv("HOST")
			}

			cfg := LoadOrDefault()

			assert.Equal(t, tt.wantPort, cfg.Server.Port)
			assert.Equal(t, tt.wantHost, cfg.Server.Host)
		})
	}
}

func TestKernelConfig(t *testing.T) {
	tests := []struct {
		name        string
		address     string
		enabled     string
		wantAddress string
		wantEnabled bool
	}{
		{
			name:        "default values",
			address:     "",
			enabled:     "",
			wantAddress: "localhost:50051",
			wantEnabled: true,
		},
		{
			name:        "custom address",
			address:     "kernel-service:50051",
			enabled:     "",
			wantAddress: "kernel-service:50051",
			wantEnabled: true,
		},
		{
			name:        "disabled",
			address:     "",
			enabled:     "false",
			wantAddress: "localhost:50051",
			wantEnabled: false,
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			// Clean environment
			os.Unsetenv("KERNEL_ADDR")
			os.Unsetenv("KERNEL_ENABLED")

			// Set test values
			if tt.address != "" {
				err := os.Setenv("KERNEL_ADDR", tt.address)
				require.NoError(t, err)
				defer os.Unsetenv("KERNEL_ADDR")
			}
			if tt.enabled != "" {
				err := os.Setenv("KERNEL_ENABLED", tt.enabled)
				require.NoError(t, err)
				defer os.Unsetenv("KERNEL_ENABLED")
			}

			cfg := LoadOrDefault()

			assert.Equal(t, tt.wantAddress, cfg.Kernel.Address)
			assert.Equal(t, tt.wantEnabled, cfg.Kernel.Enabled)
		})
	}
}

func TestLoggingConfig(t *testing.T) {
	tests := []struct {
		name      string
		level     string
		dev       string
		wantLevel string
		wantDev   bool
	}{
		{
			name:      "default values",
			level:     "",
			dev:       "",
			wantLevel: "info",
			wantDev:   false,
		},
		{
			name:      "debug level",
			level:     "debug",
			dev:       "",
			wantLevel: "debug",
			wantDev:   false,
		},
		{
			name:      "development mode",
			level:     "",
			dev:       "true",
			wantLevel: "info",
			wantDev:   true,
		},
		{
			name:      "error level production",
			level:     "error",
			dev:       "false",
			wantLevel: "error",
			wantDev:   false,
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			// Clean environment
			os.Unsetenv("LOG_LEVEL")
			os.Unsetenv("LOG_DEV")

			// Set test values
			if tt.level != "" {
				err := os.Setenv("LOG_LEVEL", tt.level)
				require.NoError(t, err)
				defer os.Unsetenv("LOG_LEVEL")
			}
			if tt.dev != "" {
				err := os.Setenv("LOG_DEV", tt.dev)
				require.NoError(t, err)
				defer os.Unsetenv("LOG_DEV")
			}

			cfg := LoadOrDefault()

			assert.Equal(t, tt.wantLevel, cfg.Logging.Level)
			assert.Equal(t, tt.wantDev, cfg.Logging.Development)
		})
	}
}

func TestRateLimitConfig(t *testing.T) {
	tests := []struct {
		name        string
		rps         string
		burst       string
		enabled     string
		wantRPS     int
		wantBurst   int
		wantEnabled bool
	}{
		{
			name:        "default values",
			rps:         "",
			burst:       "",
			enabled:     "",
			wantRPS:     100,
			wantBurst:   200,
			wantEnabled: true,
		},
		{
			name:        "high limits",
			rps:         "1000",
			burst:       "2000",
			enabled:     "",
			wantRPS:     1000,
			wantBurst:   2000,
			wantEnabled: true,
		},
		{
			name:        "disabled",
			rps:         "",
			burst:       "",
			enabled:     "false",
			wantRPS:     100,
			wantBurst:   200,
			wantEnabled: false,
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			// Clean environment
			os.Unsetenv("RATE_LIMIT_RPS")
			os.Unsetenv("RATE_LIMIT_BURST")
			os.Unsetenv("RATE_LIMIT_ENABLED")

			// Set test values
			if tt.rps != "" {
				err := os.Setenv("RATE_LIMIT_RPS", tt.rps)
				require.NoError(t, err)
				defer os.Unsetenv("RATE_LIMIT_RPS")
			}
			if tt.burst != "" {
				err := os.Setenv("RATE_LIMIT_BURST", tt.burst)
				require.NoError(t, err)
				defer os.Unsetenv("RATE_LIMIT_BURST")
			}
			if tt.enabled != "" {
				err := os.Setenv("RATE_LIMIT_ENABLED", tt.enabled)
				require.NoError(t, err)
				defer os.Unsetenv("RATE_LIMIT_ENABLED")
			}

			cfg := LoadOrDefault()

			assert.Equal(t, tt.wantRPS, cfg.RateLimit.RequestsPerSecond)
			assert.Equal(t, tt.wantBurst, cfg.RateLimit.Burst)
			assert.Equal(t, tt.wantEnabled, cfg.RateLimit.Enabled)
		})
	}
}
