package auth

import (
	"context"
	"testing"
)

func TestAuthRegisterLogin(t *testing.T) {
	kernel := newMockKernel()
	auth := NewProvider(kernel, 1, "/tmp/test")
	ctx := context.Background()

	// Register user
	result, err := auth.Execute(ctx, "auth.register", map[string]interface{}{
		"username": "alice",
		"password": "secret123",
		"email":    "alice@example.com",
	}, nil)

	if err != nil || !result.Success {
		t.Fatalf("Register failed: %v", err)
	}

	userID := result.Data["user_id"].(string)
	if userID == "" {
		t.Error("Expected user_id in response")
	}

	// Login
	result, err = auth.Execute(ctx, "auth.login", map[string]interface{}{
		"username": "alice",
		"password": "secret123",
	}, nil)

	if err != nil || !result.Success {
		t.Fatalf("Login failed: %v", err)
	}

	token := result.Data["token"].(string)
	if token == "" {
		t.Error("Expected token in login response")
	}

	// Verify token
	result, err = auth.Execute(ctx, "auth.verify", map[string]interface{}{
		"token": token,
	}, nil)

	if err != nil || !result.Success {
		t.Fatalf("Verify failed: %v", err)
	}

	if !result.Data["valid"].(bool) {
		t.Error("Expected token to be valid")
	}
}

func TestAuthInvalidCredentials(t *testing.T) {
	kernel := newMockKernel()
	auth := NewProvider(kernel, 1, "/tmp/test")
	ctx := context.Background()

	// Register user
	auth.Execute(ctx, "auth.register", map[string]interface{}{
		"username": "bob",
		"password": "password",
	}, nil)

	// Try wrong password
	result, _ := auth.Execute(ctx, "auth.login", map[string]interface{}{
		"username": "bob",
		"password": "wrong",
	}, nil)

	if result.Success {
		t.Error("Expected login to fail with wrong password")
	}

	// Try non-existent user
	result, _ = auth.Execute(ctx, "auth.login", map[string]interface{}{
		"username": "nobody",
		"password": "password",
	}, nil)

	if result.Success {
		t.Error("Expected login to fail for non-existent user")
	}
}

func TestAuthDuplicateUsername(t *testing.T) {
	kernel := newMockKernel()
	auth := NewProvider(kernel, 1, "/tmp/test")
	ctx := context.Background()

	// Register first user
	auth.Execute(ctx, "auth.register", map[string]interface{}{
		"username": "charlie",
		"password": "pass1",
	}, nil)

	// Try to register with same username
	result, _ := auth.Execute(ctx, "auth.register", map[string]interface{}{
		"username": "charlie",
		"password": "pass2",
	}, nil)

	if result.Success {
		t.Error("Expected duplicate username to fail")
	}
}

func TestAuthLogout(t *testing.T) {
	kernel := newMockKernel()
	auth := NewProvider(kernel, 1, "/tmp/test")
	ctx := context.Background()

	// Register and login
	auth.Execute(ctx, "auth.register", map[string]interface{}{
		"username": "dave",
		"password": "password",
	}, nil)

	loginResult, _ := auth.Execute(ctx, "auth.login", map[string]interface{}{
		"username": "dave",
		"password": "password",
	}, nil)

	token := loginResult.Data["token"].(string)

	// Logout
	result, err := auth.Execute(ctx, "auth.logout", map[string]interface{}{
		"token": token,
	}, nil)

	if err != nil || !result.Success {
		t.Fatalf("Logout failed")
	}

	// Verify token is invalid after logout
	verifyResult, _ := auth.Execute(ctx, "auth.verify", map[string]interface{}{
		"token": token,
	}, nil)

	if verifyResult.Data["valid"].(bool) {
		t.Error("Expected token to be invalid after logout")
	}
}

func TestAuthGetUser(t *testing.T) {
	kernel := newMockKernel()
	auth := NewProvider(kernel, 1, "/tmp/test")
	ctx := context.Background()

	// Register and login
	auth.Execute(ctx, "auth.register", map[string]interface{}{
		"username": "eve",
		"password": "password",
		"email":    "eve@example.com",
	}, nil)

	loginResult, _ := auth.Execute(ctx, "auth.login", map[string]interface{}{
		"username": "eve",
		"password": "password",
	}, nil)

	token := loginResult.Data["token"].(string)

	// Get user
	result, err := auth.Execute(ctx, "auth.getUser", map[string]interface{}{
		"token": token,
	}, nil)

	if err != nil || !result.Success {
		t.Fatalf("GetUser failed")
	}

	username := result.Data["username"].(string)
	if username != "eve" {
		t.Errorf("Expected username 'eve', got %s", username)
	}

	email := result.Data["email"].(string)
	if email != "eve@example.com" {
		t.Errorf("Expected email 'eve@example.com', got %s", email)
	}
}

type mockKernel struct{}

func newMockKernel() *mockKernel {
	return &mockKernel{}
}

func (m *mockKernel) ExecuteSyscall(ctx context.Context, pid uint32, syscall string, params map[string]interface{}) ([]byte, error) {
	return []byte{}, nil
}
