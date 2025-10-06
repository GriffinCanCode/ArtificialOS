package logging

import (
	"os"

	"go.uber.org/zap"
	"go.uber.org/zap/zapcore"
)

// Logger wraps zap.Logger with convenience methods.
type Logger struct {
	*zap.Logger
}

// Config defines logger configuration.
type Config struct {
	Level       string // "debug", "info", "warn", "error"
	Development bool
	OutputPaths []string
}

// DefaultConfig returns production-ready logger configuration.
func DefaultConfig() Config {
	return Config{
		Level:       "info",
		Development: false,
		OutputPaths: []string{"stdout"},
	}
}

// DevelopmentConfig returns development logger configuration.
func DevelopmentConfig() Config {
	return Config{
		Level:       "debug",
		Development: true,
		OutputPaths: []string{"stdout"},
	}
}

// New creates a new logger with the provided configuration.
func New(cfg Config) (*Logger, error) {
	level, err := parseLevel(cfg.Level)
	if err != nil {
		return nil, err
	}

	zapCfg := zap.Config{
		Level:             zap.NewAtomicLevelAt(level),
		Development:       cfg.Development,
		Encoding:          encodingFormat(cfg.Development),
		EncoderConfig:     encoderConfig(cfg.Development),
		OutputPaths:       cfg.OutputPaths,
		ErrorOutputPaths:  []string{"stderr"},
		DisableCaller:     false,
		DisableStacktrace: !cfg.Development,
	}

	logger, err := zapCfg.Build()
	if err != nil {
		return nil, err
	}

	return &Logger{Logger: logger}, nil
}

// NewDefault creates a logger with default configuration.
func NewDefault() *Logger {
	logger, err := New(DefaultConfig())
	if err != nil {
		// Fallback to no-op logger
		return &Logger{Logger: zap.NewNop()}
	}
	return logger
}

// NewDevelopment creates a logger with development configuration.
func NewDevelopment() *Logger {
	logger, err := New(DevelopmentConfig())
	if err != nil {
		// Fallback to no-op logger
		return &Logger{Logger: zap.NewNop()}
	}
	return logger
}

// parseLevel converts string level to zapcore.Level.
func parseLevel(level string) (zapcore.Level, error) {
	var l zapcore.Level
	if err := l.UnmarshalText([]byte(level)); err != nil {
		return zapcore.InfoLevel, err
	}
	return l, nil
}

// encodingFormat returns encoding format based on environment.
func encodingFormat(development bool) string {
	if development {
		return "console"
	}
	return "json"
}

// encoderConfig returns encoder configuration based on environment.
func encoderConfig(development bool) zapcore.EncoderConfig {
	if development {
		return zapcore.EncoderConfig{
			TimeKey:        "T",
			LevelKey:       "L",
			NameKey:        "N",
			CallerKey:      "C",
			FunctionKey:    zapcore.OmitKey,
			MessageKey:     "M",
			StacktraceKey:  "S",
			LineEnding:     zapcore.DefaultLineEnding,
			EncodeLevel:    zapcore.CapitalColorLevelEncoder,
			EncodeTime:     zapcore.ISO8601TimeEncoder,
			EncodeDuration: zapcore.StringDurationEncoder,
			EncodeCaller:   zapcore.ShortCallerEncoder,
		}
	}

	return zapcore.EncoderConfig{
		TimeKey:        "timestamp",
		LevelKey:       "level",
		NameKey:        "logger",
		CallerKey:      "caller",
		FunctionKey:    zapcore.OmitKey,
		MessageKey:     "message",
		StacktraceKey:  "stacktrace",
		LineEnding:     zapcore.DefaultLineEnding,
		EncodeLevel:    zapcore.LowercaseLevelEncoder,
		EncodeTime:     zapcore.ISO8601TimeEncoder,
		EncodeDuration: zapcore.SecondsDurationEncoder,
		EncodeCaller:   zapcore.ShortCallerEncoder,
	}
}

// IsProduction checks if running in production environment.
func IsProduction() bool {
	env := os.Getenv("ENV")
	return env == "production" || env == "prod"
}

// IsDevelopment checks if running in development environment.
func IsDevelopment() bool {
	return !IsProduction()
}
