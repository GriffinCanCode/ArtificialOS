package config

import (
	"fmt"

	"github.com/kelseyhightower/envconfig"
)

// Config holds all application configuration.
type Config struct {
	Server    ServerConfig
	Kernel    KernelConfig
	AI        AIConfig
	Logging   LogConfig
	RateLimit RateLimitConfig
}

// ServerConfig holds HTTP server configuration.
type ServerConfig struct {
	Port string `envconfig:"PORT" default:"8000"`
	Host string `envconfig:"HOST" default:"0.0.0.0"`
}

// KernelConfig holds kernel service configuration.
type KernelConfig struct {
	Address string `envconfig:"KERNEL_ADDR" default:"localhost:50051"`
	Enabled bool   `envconfig:"KERNEL_ENABLED" default:"true"`
}

// AIConfig holds AI service configuration.
type AIConfig struct {
	Address string `envconfig:"AI_ADDR" default:"localhost:50052"`
}

// LogConfig holds logging configuration.
type LogConfig struct {
	Level       string `envconfig:"LOG_LEVEL" default:"info"`
	Development bool   `envconfig:"LOG_DEV" default:"false"`
}

// RateLimitConfig holds rate limiting configuration.
type RateLimitConfig struct {
	RequestsPerSecond int  `envconfig:"RATE_LIMIT_RPS" default:"100"`
	Burst             int  `envconfig:"RATE_LIMIT_BURST" default:"200"`
	Enabled           bool `envconfig:"RATE_LIMIT_ENABLED" default:"true"`
}

// Load loads configuration from environment variables.
func Load() (*Config, error) {
	var cfg Config
	if err := envconfig.Process("", &cfg); err != nil {
		return nil, fmt.Errorf("failed to load config: %w", err)
	}
	return &cfg, nil
}

// LoadOrDefault loads configuration from environment or returns default.
func LoadOrDefault() *Config {
	cfg, err := Load()
	if err != nil {
		return Default()
	}
	return cfg
}

// Default returns default configuration.
func Default() *Config {
	return &Config{
		Server: ServerConfig{
			Port: "8000",
			Host: "0.0.0.0",
		},
		Kernel: KernelConfig{
			Address: "localhost:50051",
			Enabled: true,
		},
		AI: AIConfig{
			Address: "localhost:50052",
		},
		Logging: LogConfig{
			Level:       "info",
			Development: false,
		},
		RateLimit: RateLimitConfig{
			RequestsPerSecond: 100,
			Burst:             200,
			Enabled:           true,
		},
	}
}
