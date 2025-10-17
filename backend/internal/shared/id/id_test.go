package id

import (
	"strings"
	"sync"
	"testing"
	"time"
)

func TestGenerate(t *testing.T) {
	gen := NewGenerator()

	id1 := gen.Generate()
	id2 := gen.Generate()

	if id1.String() == id2.String() {
		t.Error("Generated IDs should be unique")
	}
}

func TestGenerateString(t *testing.T) {
	gen := NewGenerator()

	id := gen.GenerateString()

	if len(id) != 26 {
		t.Errorf("ULID should be 26 characters, got %d", len(id))
	}
}

func TestGenerateWithPrefix(t *testing.T) {
	gen := NewGenerator()

	tests := []struct {
		prefix string
	}{
		{"app"},
		{"sess"},
		{"req"},
	}

	for _, tt := range tests {
		id := gen.GenerateWithPrefix(tt.prefix)

		if !strings.HasPrefix(id, tt.prefix+"_") {
			t.Errorf("ID should start with '%s_', got: %s", tt.prefix, id)
		}

		// Verify ULID part is valid
		parts := strings.Split(id, "_")
		if len(parts) != 2 {
			t.Errorf("Prefixed ID should have format 'prefix_ulid', got: %s", id)
		}

		if !IsValid(parts[1]) {
			t.Errorf("ULID part should be valid: %s", parts[1])
		}
	}
}

func TestTypedIDGeneration(t *testing.T) {
	appID := NewAppID()
	sessID := NewSessionID()
	reqID := NewRequestID()

	if !strings.HasPrefix(string(appID), "app_") {
		t.Errorf("AppID should start with 'app_', got: %s", appID)
	}

	if !strings.HasPrefix(string(sessID), "sess_") {
		t.Errorf("SessionID should start with 'sess_', got: %s", sessID)
	}

	if !strings.HasPrefix(string(reqID), "req_") {
		t.Errorf("RequestID should start with 'req_', got: %s", reqID)
	}
}

func TestIsValid(t *testing.T) {
	gen := NewGenerator()

	validID := gen.GenerateString()
	if !IsValid(validID) {
		t.Error("Generated ULID should be valid")
	}

	invalidIDs := []string{
		"",
		"invalid",
		"1234567890",
		"zzzzzzzzzzzzzzzzzzzzzzzzzzz", // Invalid characters
	}

	for _, id := range invalidIDs {
		if IsValid(id) {
			t.Errorf("ID should be invalid: %s", id)
		}
	}
}

func TestParse(t *testing.T) {
	gen := NewGenerator()

	original := gen.Generate()
	str := original.String()

	parsed, err := Parse(str)
	if err != nil {
		t.Fatalf("Failed to parse ULID: %v", err)
	}

	if parsed.String() != str {
		t.Errorf("Parsed ULID doesn't match original: %s != %s", parsed.String(), str)
	}
}

func TestTimestamp(t *testing.T) {
	gen := NewGenerator()

	before := time.Now()
	id := gen.GenerateString()
	after := time.Now()

	ts, err := Timestamp(id)
	if err != nil {
		t.Fatalf("Failed to extract timestamp: %v", err)
	}

	// ULID timestamps have millisecond precision, so allow small variance
	beforeMs := before.UnixMilli()
	afterMs := after.UnixMilli()
	tsMs := ts.UnixMilli()

	if tsMs < beforeMs || tsMs > afterMs {
		t.Errorf("Timestamp should be between %d and %d ms, got %d ms", beforeMs, afterMs, tsMs)
	}
}

func TestGenerateBatch(t *testing.T) {
	gen := NewGenerator()

	count := 100
	ids := gen.GenerateBatch(count)

	if len(ids) != count {
		t.Errorf("Expected %d IDs, got %d", count, len(ids))
	}

	// Check uniqueness
	seen := make(map[string]bool)
	for _, id := range ids {
		str := id.String()
		if seen[str] {
			t.Errorf("Duplicate ID found: %s", str)
		}
		seen[str] = true
	}
}

func TestIDFormatConsistency(t *testing.T) {
	// All IDs should follow the format: prefix_ULID
	ids := map[string]string{
		"app":  string(NewAppID()),
		"sess": string(NewSessionID()),
		"req":  string(NewRequestID()),
		"svc":  string(NewServiceID()),
		"tool": string(NewToolID()),
		"win":  string(NewWindowID()),
		"pkg":  string(NewPackageID()),
	}

	for prefix, id := range ids {
		parts := strings.Split(id, "_")
		if len(parts) != 2 {
			t.Errorf("ID should have format 'prefix_ulid', got: %s", id)
		}

		if parts[0] != prefix {
			t.Errorf("Expected prefix '%s', got '%s' in ID: %s", prefix, parts[0], id)
		}

		if len(parts[1]) != 26 {
			t.Errorf("ULID should be 26 characters, got %d in ID: %s", len(parts[1]), id)
		}
	}
}

func TestConcurrentGeneration(t *testing.T) {
	gen := NewGenerator()

	const goroutines = 100
	const idsPerGoroutine = 100

	var wg sync.WaitGroup
	idChan := make(chan string, goroutines*idsPerGoroutine)

	for i := 0; i < goroutines; i++ {
		wg.Add(1)
		go func() {
			defer wg.Done()
			for j := 0; j < idsPerGoroutine; j++ {
				idChan <- gen.GenerateString()
			}
		}()
	}

	wg.Wait()
	close(idChan)

	// Check uniqueness
	seen := make(map[string]bool)
	count := 0
	for id := range idChan {
		if seen[id] {
			t.Errorf("Duplicate ID found in concurrent generation: %s", id)
		}
		seen[id] = true
		count++
	}

	expected := goroutines * idsPerGoroutine
	if count != expected {
		t.Errorf("Expected %d unique IDs, got %d", expected, count)
	}
}

func TestLexicographicSorting(t *testing.T) {
	gen := NewGenerator()

	// Generate IDs with delays to ensure different timestamps
	ids := make([]string, 5)
	for i := 0; i < 5; i++ {
		ids[i] = gen.GenerateString()
		time.Sleep(2 * time.Millisecond)
	}

	// Verify they're in ascending order (k-sortable)
	for i := 1; i < len(ids); i++ {
		if ids[i] <= ids[i-1] {
			t.Errorf("IDs should be lexicographically sorted: %s should be > %s", ids[i], ids[i-1])
		}
	}
}

func TestDefaultGenerator(t *testing.T) {
	// Test singleton pattern
	gen1 := Default()
	gen2 := Default()

	if gen1 != gen2 {
		t.Error("Default() should return the same instance")
	}

	// Test it works
	id := gen1.GenerateString()
	if !IsValid(id) {
		t.Error("Default generator should produce valid IDs")
	}
}

func BenchmarkGenerate(b *testing.B) {
	gen := NewGenerator()

	b.ResetTimer()
	for i := 0; i < b.N; i++ {
		_ = gen.Generate()
	}
}

func BenchmarkGenerateString(b *testing.B) {
	gen := NewGenerator()

	b.ResetTimer()
	for i := 0; i < b.N; i++ {
		_ = gen.GenerateString()
	}
}

func BenchmarkGenerateWithPrefix(b *testing.B) {
	gen := NewGenerator()

	b.ResetTimer()
	for i := 0; i < b.N; i++ {
		_ = gen.GenerateWithPrefix("app")
	}
}

func BenchmarkGenerateBatch(b *testing.B) {
	gen := NewGenerator()

	b.ResetTimer()
	for i := 0; i < b.N; i++ {
		_ = gen.GenerateBatch(100)
	}
}

func BenchmarkConcurrentGenerate(b *testing.B) {
	gen := NewGenerator()

	b.RunParallel(func(pb *testing.PB) {
		for pb.Next() {
			_ = gen.Generate()
		}
	})
}
