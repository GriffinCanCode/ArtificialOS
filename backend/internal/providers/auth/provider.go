package auth

import (
	"context"
	"crypto/rand"
	"encoding/base64"
	"encoding/json"
	"fmt"
	"sync"
	"time"

	"github.com/GriffinCanCode/AgentOS/backend/internal/shared/types"
	"github.com/GriffinCanCode/AgentOS/backend/internal/shared/utils"
	"golang.org/x/crypto/bcrypt"
)

// KernelClient interface for syscall execution
type KernelClient interface {
	ExecuteSyscall(ctx context.Context, pid uint32, syscall string, params map[string]interface{}) ([]byte, error)
}

// Provider implements authentication and session management
type Provider struct {
	kernel      KernelClient
	storagePID  uint32
	storagePath string
	sessions    sync.Map
	users       sync.Map
	// Note: sync.Map provides its own internal synchronization
}

// User represents an authenticated user
type User struct {
	ID           string    `json:"id"`
	Username     string    `json:"username"`
	PasswordHash string    `json:"password_hash"`
	Email        string    `json:"email,omitempty"`
	CreatedAt    time.Time `json:"created_at"`
}

// Session represents an active session
type Session struct {
	ID        string    `json:"id"`
	UserID    string    `json:"user_id"`
	Token     string    `json:"token"`
	CreatedAt time.Time `json:"created_at"`
	ExpiresAt time.Time `json:"expires_at"`
}

// NewProvider creates an auth provider
func NewProvider(kernel KernelClient, storagePID uint32, storagePath string) *Provider {
	return &Provider{
		kernel:      kernel,
		storagePID:  storagePID,
		storagePath: storagePath,
	}
}

// Definition returns service metadata
func (a *Provider) Definition() types.Service {
	return types.Service{
		ID:          "auth",
		Name:        "Authentication Service",
		Description: "User authentication and session management",
		Category:    types.CategoryAuth,
		Capabilities: []string{
			"register",
			"login",
			"logout",
			"verify",
		},
		Tools: []types.Tool{
			{
				ID:          "auth.register",
				Name:        "Register User",
				Description: "Create a new user account",
				Parameters: []types.Parameter{
					{Name: "username", Type: "string", Description: "Username", Required: true},
					{Name: "password", Type: "string", Description: "Password", Required: true},
					{Name: "email", Type: "string", Description: "Email address", Required: false},
				},
				Returns: "object",
			},
			{
				ID:          "auth.login",
				Name:        "Login",
				Description: "Authenticate and create session",
				Parameters: []types.Parameter{
					{Name: "username", Type: "string", Description: "Username", Required: true},
					{Name: "password", Type: "string", Description: "Password", Required: true},
				},
				Returns: "object",
			},
			{
				ID:          "auth.logout",
				Name:        "Logout",
				Description: "End current session",
				Parameters: []types.Parameter{
					{Name: "token", Type: "string", Description: "Session token", Required: true},
				},
				Returns: "boolean",
			},
			{
				ID:          "auth.verify",
				Name:        "Verify Token",
				Description: "Check if session token is valid",
				Parameters: []types.Parameter{
					{Name: "token", Type: "string", Description: "Session token", Required: true},
				},
				Returns: "object",
			},
			{
				ID:          "auth.getUser",
				Name:        "Get Current User",
				Description: "Get authenticated user details",
				Parameters: []types.Parameter{
					{Name: "token", Type: "string", Description: "Session token", Required: true},
				},
				Returns: "object",
			},
		},
	}
}

// Execute runs an auth operation
func (a *Provider) Execute(ctx context.Context, toolID string, params map[string]interface{}, appCtx *types.Context) (*types.Result, error) {
	switch toolID {
	case "auth.register":
		return a.register(ctx, params)
	case "auth.login":
		return a.login(params)
	case "auth.logout":
		return a.logout(params)
	case "auth.verify":
		return a.verify(params)
	case "auth.getUser":
		return a.getUser(params)
	default:
		return failure(fmt.Sprintf("unknown tool: %s", toolID))
	}
}

func (a *Provider) register(ctx context.Context, params map[string]interface{}) (*types.Result, error) {
	username, ok := params["username"].(string)
	if !ok || username == "" {
		return failure("username required")
	}

	password, ok := params["password"].(string)
	if !ok || password == "" {
		return failure("password required")
	}

	// Validate username
	if err := utils.ValidateUsername(username); err != nil {
		return failure(err.Error())
	}

	// Validate password
	if err := utils.ValidatePassword(password); err != nil {
		return failure(err.Error())
	}

	// Validate email if provided
	if email, ok := params["email"].(string); ok && email != "" {
		if err := utils.ValidateEmail(email, false); err != nil {
			return failure(err.Error())
		}
	}

	// Check if user exists
	if _, exists := a.users.Load(username); exists {
		return failure("username already exists")
	}

	// Hash password
	hash, err := bcrypt.GenerateFromPassword([]byte(password), bcrypt.DefaultCost)
	if err != nil {
		return failure(fmt.Sprintf("password hashing failed: %v", err))
	}

	// Create user
	user := &User{
		ID:           generateID(),
		Username:     username,
		PasswordHash: string(hash),
		Email:        getStringParam(params, "email"),
		CreatedAt:    time.Now(),
	}

	// Store user
	a.users.Store(username, user)
	a.users.Store(user.ID, user) // Also store by ID for lookups

	// Persist to disk
	if err := a.saveUser(ctx, user); err != nil {
		return failure(fmt.Sprintf("failed to persist user: %v", err))
	}

	return success(map[string]interface{}{
		"user_id":  user.ID,
		"username": user.Username,
		"email":    user.Email,
	})
}

func (a *Provider) login(params map[string]interface{}) (*types.Result, error) {
	username, ok := params["username"].(string)
	if !ok || username == "" {
		return failure("username required")
	}

	password, ok := params["password"].(string)
	if !ok || password == "" {
		return failure("password required")
	}

	// Validate username format (prevent injection attacks)
	if err := utils.ValidateUsername(username); err != nil {
		return failure("invalid credentials") // Don't reveal validation error
	}

	// Validate password length (prevent DoS with extremely long passwords)
	if err := utils.ValidatePassword(password); err != nil {
		return failure("invalid credentials") // Don't reveal validation error
	}

	// Get user
	userVal, exists := a.users.Load(username)
	if !exists {
		return failure("invalid credentials")
	}

	user := userVal.(*User)

	// Verify password
	if err := bcrypt.CompareHashAndPassword([]byte(user.PasswordHash), []byte(password)); err != nil {
		return failure("invalid credentials")
	}

	// Create session
	token := generateToken()
	session := &Session{
		ID:        generateID(),
		UserID:    user.ID,
		Token:     token,
		CreatedAt: time.Now(),
		ExpiresAt: time.Now().Add(24 * time.Hour),
	}

	a.sessions.Store(token, session)

	return success(map[string]interface{}{
		"token":      token,
		"user_id":    user.ID,
		"username":   user.Username,
		"expires_at": session.ExpiresAt.Unix(),
	})
}

func (a *Provider) logout(params map[string]interface{}) (*types.Result, error) {
	token, ok := params["token"].(string)
	if !ok || token == "" {
		return failure("token required")
	}

	// Validate token format (prevent injection)
	if err := utils.ValidateString(token, "token", 1, 128, true); err != nil {
		return failure("invalid token")
	}

	a.sessions.Delete(token)

	return success(map[string]interface{}{"logged_out": true})
}

func (a *Provider) verify(params map[string]interface{}) (*types.Result, error) {
	token, ok := params["token"].(string)
	if !ok || token == "" {
		return failure("token required")
	}

	// Validate token format (prevent injection)
	if err := utils.ValidateString(token, "token", 1, 128, true); err != nil {
		return success(map[string]interface{}{"valid": false})
	}

	sessionVal, exists := a.sessions.Load(token)
	if !exists {
		return success(map[string]interface{}{"valid": false})
	}

	session := sessionVal.(*Session)

	// Check expiration
	if time.Now().After(session.ExpiresAt) {
		a.sessions.Delete(token)
		return success(map[string]interface{}{"valid": false, "reason": "expired"})
	}

	return success(map[string]interface{}{
		"valid":      true,
		"user_id":    session.UserID,
		"expires_at": session.ExpiresAt.Unix(),
	})
}

func (a *Provider) getUser(params map[string]interface{}) (*types.Result, error) {
	token, ok := params["token"].(string)
	if !ok || token == "" {
		return failure("token required")
	}

	// Validate token format (prevent injection)
	if err := utils.ValidateString(token, "token", 1, 128, true); err != nil {
		return failure("invalid token")
	}

	// Verify session
	sessionVal, exists := a.sessions.Load(token)
	if !exists {
		return failure("invalid token")
	}

	session := sessionVal.(*Session)
	if time.Now().After(session.ExpiresAt) {
		return failure("token expired")
	}

	// Get user
	userVal, exists := a.users.Load(session.UserID)
	if !exists {
		return failure("user not found")
	}

	user := userVal.(*User)

	return success(map[string]interface{}{
		"user_id":  user.ID,
		"username": user.Username,
		"email":    user.Email,
	})
}

func (a *Provider) saveUser(ctx context.Context, user *User) error {
	if a.kernel == nil {
		return nil // Skip if no kernel
	}

	data, err := json.Marshal(user)
	if err != nil {
		return err
	}

	path := fmt.Sprintf("%s/users/%s.json", a.storagePath, user.ID)
	_, err = a.kernel.ExecuteSyscall(ctx, a.storagePID, "write_file", map[string]interface{}{
		"path": path,
		"data": data,
	})

	return err
}

func generateID() string {
	b := make([]byte, 16)
	if _, err := rand.Read(b); err != nil {
		// This is a critical security issue - if crypto/rand fails,
		// we must not fall back to weak randomness
		panic(fmt.Sprintf("crypto/rand failed: %v - cannot generate secure ID", err))
	}
	return base64.URLEncoding.EncodeToString(b)
}

func generateToken() string {
	b := make([]byte, 32)
	if _, err := rand.Read(b); err != nil {
		// This is a critical security issue - if crypto/rand fails,
		// we must not fall back to weak randomness
		panic(fmt.Sprintf("crypto/rand failed: %v - cannot generate secure token", err))
	}
	return base64.URLEncoding.EncodeToString(b)
}

func getStringParam(params map[string]interface{}, key string) string {
	if val, ok := params[key].(string); ok {
		return val
	}
	return ""
}

func success(data map[string]interface{}) (*types.Result, error) {
	return &types.Result{Success: true, Data: data}, nil
}

func failure(message string) (*types.Result, error) {
	msg := message
	return &types.Result{Success: false, Error: &msg}, nil
}
