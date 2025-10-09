package sandbox

import (
	"context"
	"testing"
	"time"
)

func TestRuntimeExecution(t *testing.T) {
	config := DefaultConfig()
	runtime, err := New(config)
	if err != nil {
		t.Fatalf("Failed to create runtime: %v", err)
	}
	defer runtime.Close()

	tests := []struct {
		name    string
		script  string
		wantErr bool
	}{
		{
			name:    "simple return",
			script:  "42",
			wantErr: false,
		},
		{
			name:    "console log",
			script:  "console.log('hello'); 'test'",
			wantErr: false,
		},
		{
			name:    "math operations",
			script:  "Math.sqrt(16)",
			wantErr: false,
		},
		{
			name:    "string operations",
			script:  "'hello'.toUpperCase()",
			wantErr: false,
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			ctx := context.Background()
			result, err := runtime.Execute(ctx, tt.script, nil)

			if (err != nil) != tt.wantErr {
				t.Errorf("Execute() error = %v, wantErr %v", err, tt.wantErr)
				return
			}

			if !tt.wantErr && result == nil {
				t.Error("Execute() returned nil result")
			}
		})
	}
}

func TestRuntimeSecurity(t *testing.T) {
	config := DefaultConfig()
	runtime, err := New(config)
	if err != nil {
		t.Fatalf("Failed to create runtime: %v", err)
	}
	defer runtime.Close()

	dangerousScripts := []struct {
		name   string
		script string
	}{
		{
			name:   "require blocked",
			script: "require('fs')",
		},
		{
			name:   "process blocked",
			script: "process.exit(1)",
		},
		{
			name:   "module blocked",
			script: "module.exports = {}",
		},
	}

	for _, tt := range dangerousScripts {
		t.Run(tt.name, func(t *testing.T) {
			ctx := context.Background()
			result, _ := runtime.Execute(ctx, tt.script, nil)

			// Should either error or return undefined
			if result != nil && result.Value != nil {
				t.Errorf("Dangerous script executed successfully: %v", result.Value)
			}
		})
	}
}

func TestRuntimeTimeout(t *testing.T) {
	config := Config{
		MaxMemoryMB:   50,
		Timeout:       100 * time.Millisecond,
		EnableConsole: true,
		EnableDOM:     false,
	}

	runtime, err := New(config)
	if err != nil {
		t.Fatalf("Failed to create runtime: %v", err)
	}
	defer runtime.Close()

	ctx := context.Background()
	script := `
		let i = 0;
		while(true) {
			i++;
		}
	`

	result, err := runtime.Execute(ctx, script, nil)

	if err == nil {
		t.Error("Expected timeout error, got nil")
	}

	if result != nil && result.Error == nil {
		t.Error("Expected error in result")
	}
}

func TestRuntimeConsoleCapture(t *testing.T) {
	config := DefaultConfig()
	runtime, err := New(config)
	if err != nil {
		t.Fatalf("Failed to create runtime: %v", err)
	}
	defer runtime.Close()

	ctx := context.Background()
	script := `
		console.log('info message');
		console.warn('warning message');
		console.error('error message');
		'done'
	`

	result, err := runtime.Execute(ctx, script, nil)
	if err != nil {
		t.Fatalf("Execute() error = %v", err)
	}

	if len(result.Console) != 3 {
		t.Errorf("Expected 3 console entries, got %d", len(result.Console))
	}

	levels := []string{"log", "warn", "error"}
	for i, entry := range result.Console {
		if entry.Level != levels[i] {
			t.Errorf("Console entry %d: expected level %s, got %s", i, levels[i], entry.Level)
		}
	}
}

func TestPoolAcquireRelease(t *testing.T) {
	config := DefaultConfig()
	pool, err := NewPool(config, 2)
	if err != nil {
		t.Fatalf("Failed to create pool: %v", err)
	}
	defer pool.Close()

	ctx := context.Background()

	// Acquire runtime
	runtime, err := pool.Acquire(ctx)
	if err != nil {
		t.Fatalf("Failed to acquire runtime: %v", err)
	}

	// Execute script
	result, err := runtime.Execute(ctx, "42", nil)
	if err != nil {
		t.Fatalf("Execute() error = %v", err)
	}

	if result.Value == nil {
		t.Error("Expected non-nil result value")
	}

	// Release back to pool
	if err := pool.Release(runtime); err != nil {
		t.Errorf("Failed to release runtime: %v", err)
	}
}

func TestPoolExecute(t *testing.T) {
	config := DefaultConfig()
	pool, err := NewPool(config, 2)
	if err != nil {
		t.Fatalf("Failed to create pool: %v", err)
	}
	defer pool.Close()

	ctx := context.Background()
	script := "Math.sqrt(16)"

	result, err := pool.Execute(ctx, script, nil)
	if err != nil {
		t.Fatalf("Pool.Execute() error = %v", err)
	}

	if result.Value == nil {
		t.Error("Expected non-nil result value")
	}

	// Execute multiple times to test pool reuse
	for i := 0; i < 5; i++ {
		_, err := pool.Execute(ctx, script, nil)
		if err != nil {
			t.Errorf("Iteration %d: Execute() error = %v", i, err)
		}
	}
}

func TestDOMQuery(t *testing.T) {
	dom := NewDOM()

	// Add test element
	elem := &Element{
		TagName:    "div",
		ID:         "test-id",
		ClassName:  "test-class",
		Attributes: make(map[string]string),
	}
	dom.root.AddElement(elem)

	tests := []struct {
		name     string
		selector string
		wantLen  int
	}{
		{
			name:     "ID selector",
			selector: "#test-id",
			wantLen:  1,
		},
		{
			name:     "class selector",
			selector: ".test-class",
			wantLen:  1,
		},
		{
			name:     "tag selector",
			selector: "div",
			wantLen:  1,
		},
		{
			name:     "non-existent",
			selector: "#not-found",
			wantLen:  0,
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			results := dom.Query(tt.selector)
			if len(results) != tt.wantLen {
				t.Errorf("Query(%s) returned %d elements, want %d", tt.selector, len(results), tt.wantLen)
			}
		})
	}
}
