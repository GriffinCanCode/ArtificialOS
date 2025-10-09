package permissions

import (
	"context"
	"encoding/json"
	"fmt"

	"github.com/GriffinCanCode/AgentOS/backend/internal/shared/types"
)

// KernelClient interface for syscall operations
type KernelClient interface {
	ExecuteSyscall(ctx context.Context, pid uint32, syscallType string, params map[string]interface{}) ([]byte, error)
}

// Provider implements permission management UI layer
type Provider struct {
	kernel     KernelClient
	storagePID uint32
}

// PermissionEntry represents a permission grant
type PermissionEntry struct {
	AppID      string   `json:"app_id"`
	Resource   string   `json:"resource"`
	Actions    []string `json:"actions"`
	GrantedAt  int64    `json:"granted_at"`
	ExpiresAt  *int64   `json:"expires_at,omitempty"`
	Persistent bool     `json:"persistent"`
}

// AuditEntry represents a permission check audit log
type AuditEntry struct {
	Timestamp int64  `json:"timestamp"`
	AppID     string `json:"app_id"`
	Resource  string `json:"resource"`
	Action    string `json:"action"`
	Allowed   bool   `json:"allowed"`
	Reason    string `json:"reason,omitempty"`
}

// NewProvider creates a permissions provider
func NewProvider(kernel KernelClient, storagePID uint32) *Provider {
	return &Provider{
		kernel:     kernel,
		storagePID: storagePID,
	}
}

// Definition returns service metadata
func (p *Provider) Definition() types.Service {
	return types.Service{
		ID:          "permissions",
		Name:        "Permissions Manager",
		Description: "Manage app permissions and view audit logs",
		Category:    types.CategorySystem,
		Capabilities: []string{
			"list",
			"grant",
			"revoke",
			"audit",
		},
		Tools: []types.Tool{
			{
				ID:          "permissions.list",
				Name:        "List Permissions",
				Description: "List all permissions for an app",
				Parameters: []types.Parameter{
					{Name: "app_id", Type: "string", Description: "App ID (optional)", Required: false},
				},
				Returns: "array",
			},
			{
				ID:          "permissions.grant",
				Name:        "Grant Permission",
				Description: "Grant a permission to an app",
				Parameters: []types.Parameter{
					{Name: "app_id", Type: "string", Description: "App ID", Required: true},
					{Name: "resource", Type: "string", Description: "Resource type", Required: true},
					{Name: "actions", Type: "array", Description: "Actions to grant", Required: true},
					{Name: "persistent", Type: "boolean", Description: "Persist across sessions", Required: false},
				},
				Returns: "boolean",
			},
			{
				ID:          "permissions.revoke",
				Name:        "Revoke Permission",
				Description: "Revoke a permission from an app",
				Parameters: []types.Parameter{
					{Name: "app_id", Type: "string", Description: "App ID", Required: true},
					{Name: "resource", Type: "string", Description: "Resource type", Required: true},
					{Name: "actions", Type: "array", Description: "Actions to revoke", Required: false},
				},
				Returns: "boolean",
			},
			{
				ID:          "permissions.audit",
				Name:        "Get Audit Log",
				Description: "Get permission check audit log",
				Parameters: []types.Parameter{
					{Name: "app_id", Type: "string", Description: "Filter by app ID", Required: false},
					{Name: "limit", Type: "number", Description: "Max entries to return", Required: false},
				},
				Returns: "array",
			},
			{
				ID:          "permissions.resources",
				Name:        "List Resources",
				Description: "List all available resource types",
				Parameters:  []types.Parameter{},
				Returns:     "array",
			},
		},
	}
}

// Execute runs a permissions operation
func (p *Provider) Execute(ctx context.Context, toolID string, params map[string]interface{}, appCtx *types.Context) (*types.Result, error) {
	switch toolID {
	case "permissions.list":
		return p.list(ctx, params)
	case "permissions.grant":
		return p.grant(ctx, params)
	case "permissions.revoke":
		return p.revoke(ctx, params)
	case "permissions.audit":
		return p.audit(ctx, params)
	case "permissions.resources":
		return p.resources(ctx)
	default:
		return failure(fmt.Sprintf("unknown tool: %s", toolID))
	}
}

func (p *Provider) list(ctx context.Context, params map[string]interface{}) (*types.Result, error) {
	appID, _ := params["app_id"].(string)

	if p.kernel == nil {
		return success(map[string]interface{}{"permissions": []PermissionEntry{}})
	}

	// Get permissions from kernel
	resp, err := p.kernel.ExecuteSyscall(ctx, p.storagePID, "list_permissions", map[string]interface{}{
		"app_id": appID,
	})
	if err != nil {
		return failure(fmt.Sprintf("failed to list permissions: %v", err))
	}

	var result map[string]interface{}
	if err := json.Unmarshal(resp, &result); err != nil {
		return failure(fmt.Sprintf("failed to parse permissions: %v", err))
	}

	return success(result)
}

func (p *Provider) grant(ctx context.Context, params map[string]interface{}) (*types.Result, error) {
	appID, ok := params["app_id"].(string)
	if !ok || appID == "" {
		return failure("app_id parameter required")
	}

	resource, ok := params["resource"].(string)
	if !ok || resource == "" {
		return failure("resource parameter required")
	}

	actions, ok := params["actions"].([]interface{})
	if !ok {
		return failure("actions parameter required and must be an array")
	}

	persistent, _ := params["persistent"].(bool)

	if p.kernel == nil {
		return failure("kernel client not available")
	}

	// Grant permission via kernel
	_, err := p.kernel.ExecuteSyscall(ctx, p.storagePID, "grant_permission", map[string]interface{}{
		"app_id":     appID,
		"resource":   resource,
		"actions":    actions,
		"persistent": persistent,
	})
	if err != nil {
		return failure(fmt.Sprintf("failed to grant permission: %v", err))
	}

	return success(map[string]interface{}{"granted": true})
}

func (p *Provider) revoke(ctx context.Context, params map[string]interface{}) (*types.Result, error) {
	appID, ok := params["app_id"].(string)
	if !ok || appID == "" {
		return failure("app_id parameter required")
	}

	resource, ok := params["resource"].(string)
	if !ok || resource == "" {
		return failure("resource parameter required")
	}

	actions, _ := params["actions"].([]interface{})

	if p.kernel == nil {
		return failure("kernel client not available")
	}

	// Revoke permission via kernel
	_, err := p.kernel.ExecuteSyscall(ctx, p.storagePID, "revoke_permission", map[string]interface{}{
		"app_id":   appID,
		"resource": resource,
		"actions":  actions,
	})
	if err != nil {
		return failure(fmt.Sprintf("failed to revoke permission: %v", err))
	}

	return success(map[string]interface{}{"revoked": true})
}

func (p *Provider) audit(ctx context.Context, params map[string]interface{}) (*types.Result, error) {
	appID, _ := params["app_id"].(string)
	limit, _ := params["limit"].(float64)
	if limit == 0 {
		limit = 100
	}

	if p.kernel == nil {
		return success(map[string]interface{}{"audit_log": []AuditEntry{}})
	}

	// Get audit log from kernel
	resp, err := p.kernel.ExecuteSyscall(ctx, p.storagePID, "get_audit_log", map[string]interface{}{
		"app_id": appID,
		"limit":  int(limit),
	})
	if err != nil {
		return failure(fmt.Sprintf("failed to get audit log: %v", err))
	}

	var result map[string]interface{}
	if err := json.Unmarshal(resp, &result); err != nil {
		return failure(fmt.Sprintf("failed to parse audit log: %v", err))
	}

	return success(result)
}

func (p *Provider) resources(ctx context.Context) (*types.Result, error) {
	// List of all available resource types
	resources := []map[string]interface{}{
		{"type": "file", "description": "File system access", "actions": []string{"read", "write", "delete", "create"}},
		{"type": "network", "description": "Network access", "actions": []string{"connect", "listen", "send", "receive"}},
		{"type": "process", "description": "Process management", "actions": []string{"spawn", "kill", "inspect"}},
		{"type": "ipc", "description": "Inter-process communication", "actions": []string{"send", "receive", "create"}},
		{"type": "system", "description": "System information", "actions": []string{"inspect", "configure"}},
		{"type": "storage", "description": "Persistent storage", "actions": []string{"read", "write", "delete"}},
	}

	return success(map[string]interface{}{"resources": resources})
}

// Helper functions
func success(data map[string]interface{}) (*types.Result, error) {
	return &types.Result{Success: true, Data: data}, nil
}

func failure(message string) (*types.Result, error) {
	errMsg := message
	return &types.Result{Success: false, Error: &errMsg}, nil
}
