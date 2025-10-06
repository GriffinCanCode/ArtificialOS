package filesystem

import (
	"context"
	"time"

	"github.com/GriffinCanCode/AgentOS/backend/internal/shared/types"
)

// FileInfo represents file metadata
type FileInfo struct {
	Name      string    `json:"name"`
	Path      string    `json:"path"`
	Size      int64     `json:"size"`
	IsDir     bool      `json:"is_dir"`
	Mode      string    `json:"mode"`
	Modified  time.Time `json:"modified"`
	Extension string    `json:"extension,omitempty"`
}

// KernelClient interface for filesystem operations
type KernelClient interface {
	ExecuteSyscall(ctx context.Context, pid uint32, syscall string, params map[string]interface{}) ([]byte, error)
}

// FilesystemOps provides common filesystem operation helpers
type FilesystemOps struct {
	Kernel      KernelClient
	StoragePID  uint32
	StoragePath string
}

// GetPID returns the appropriate PID for the operation
func (ops *FilesystemOps) GetPID(appCtx *types.Context) uint32 {
	if appCtx != nil && appCtx.SandboxPID != nil {
		return *appCtx.SandboxPID
	}
	return ops.StoragePID
}

// Success helper
func Success(data map[string]interface{}) (*types.Result, error) {
	return &types.Result{Success: true, Data: data}, nil
}

// Failure helper
func Failure(message string) (*types.Result, error) {
	msg := message
	return &types.Result{Success: false, Error: &msg}, nil
}
