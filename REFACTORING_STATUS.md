# Critical Go Refactoring - Status Report

## 🎯 Objectives
1. **CRITICAL**: Add `context.Context` as first parameter to ALL public methods
2. **CRITICAL**: Fix race conditions in app.Manager by using single locking mechanism

---

## ✅ COMPLETED (Tasks 1-4)

###  1. app.Manager - Race Condition Fixed
**File**: `backend/internal/app/manager.go`

**Changes**:
- ❌ Removed: `sync.Map` (caused race conditions)
- ✅ Added: Regular `map[string]*types.App` with `sync.RWMutex`
- ✅ Fixed: All map operations now properly locked
- ✅ Fixed: Recursive Close() now handles locking correctly
- ✅ Added: `context.Context` parameter to `Spawn()` method
- ✅ Improved: Returns copies of apps to prevent external modifications

**Benefits**:
- No more race conditions between reads and writes
- Single, predictable locking mechanism
- Better performance (no lock contention from sync.Map)
- Safer API (returns copies, not mutable pointers)

### 2. grpc.KernelClient - Context Added
**File**: `backend/internal/grpc/kernel.go`

**Changes**:
```go
// Before
func (k *KernelClient) CreateProcess(name string, ...) (*uint32, error)
func (k *KernelClient) ExecuteSyscall(pid uint32, ...) ([]byte, error)

// After  
func (k *KernelClient) CreateProcess(ctx context.Context, name string, ...) (*uint32, error)
func (k *KernelClient) ExecuteSyscall(ctx context.Context, pid uint32, ...) ([]byte, error)
```

**Benefits**:
- Operations can now be cancelled
- Prevents goroutine leaks on timeout
- Better request tracking and tracing

### 3. grpc.AIClient - Context Added
**File**: `backend/internal/grpc/ai.go`

**Changes**:
```go
// Before
func (a *AIClient) GenerateUI(message string, ...) (*pb.UIResponse, error)
func (a *AIClient) StreamUI(message string, ...) (pb.AIService_StreamUIClient, error)
func (a *AIClient) StreamChat(message string, ...) (pb.AIService_StreamChatClient, error)

// After
func (a *AIClient) GenerateUI(ctx context.Context, message string, ...) (*pb.UIResponse, error)
func (a *AIClient) StreamUI(ctx context.Context, message string, ...) (pb.AIService_StreamUIClient, error)
func (a *AIClient) StreamChat(ctx context.Context, message string, ...) (pb.AIService_StreamChatClient, error)
```

**Benefits**:
- Streaming operations can be cancelled properly
- No more dangling streams on client disconnect

### 4. Storage Provider - Context Added
**File**: `backend/internal/providers/storage.go`

**Changes**:
```go
// KernelClient interface updated
type KernelClient interface {
    ExecuteSyscall(ctx context.Context, pid uint32, ...) ([]byte, error)
}

// Execute method updated
func (s *Storage) Execute(ctx context.Context, toolID string, params map[string]interface{}, appCtx *types.Context) (*types.Result, error)

// All internal methods updated:
- set(ctx context.Context, ...)
- get(ctx context.Context, ...)
- remove(ctx context.Context, ...)
- list(ctx context.Context, ...)
- clear(ctx context.Context, ...)
```

---

## 🚧 IN PROGRESS

### 5. Remaining Providers
Need to update these files:
- `backend/internal/providers/auth.go`
- `backend/internal/providers/filesystem.go`
- `backend/internal/providers/system.go`

---

## 📋 TODO (Remaining Tasks)

### 6. Update service.Registry
**File**: `backend/internal/service/registry.go`

Update Provider interface:
```go
type Provider interface {
    Definition() types.Service
    Execute(ctx context.Context, toolID string, params map[string]interface{}, appCtx *types.Context) (*types.Result, error)
}
```

### 7. Update registry.Manager
**Files**: 
- `backend/internal/registry/manager.go`
- `backend/internal/registry/seeder.go`

Add context to:
- `Save(ctx context.Context, ...)`
- `Load(ctx context.Context, ...)`
- `Delete(ctx context.Context, ...)`

### 8. Update session.Manager
**File**: `backend/internal/session/manager.go`

Add context to:
- `Save(ctx context.Context, ...)`
- `Load(ctx context.Context, ...)`
- `Restore(ctx context.Context, ...)`
- `Delete(ctx context.Context, ...)`

### 9. Update HTTP Handlers
**File**: `backend/internal/http/handlers.go`

Update all handlers to pass `c.Request.Context()` to service calls.

Example:
```go
func (h *Handlers) GenerateUI(c *gin.Context) {
    ctx := c.Request.Context()
    
    resp, err := h.aiClient.GenerateUI(ctx, req.Message, contextMap, req.ParentID)
    // ...
}
```

### 10. Update WebSocket Handler
**File**: `backend/internal/ws/handler.go`

Pass context from WebSocket connection to all service calls.

### 11. Update Tests
**Files**:
- `backend/internal/app/manager_test.go`
- `backend/internal/grpc/kernel_test.go`
- All provider tests

Update test calls to include `context.Background()` or `context.TODO()`.

### 12. Update server.go
**File**: `backend/internal/server/server.go`

Update initialization code to pass context where needed.

---

## 🔍 Verification Steps

After completing all tasks:

1. **Run Go vet**:
   ```bash
   cd backend
   go vet ./...
   ```

2. **Run tests**:
   ```bash
   go test ./...
   ```

3. **Run with race detector**:
   ```bash
   go test -race ./...
   ```

4. **Check for goroutine leaks** (manual testing):
   - Start server
   - Make requests
   - Monitor `runtime.NumGoroutine()`
   - Cancel requests mid-flight
   - Verify goroutines are cleaned up

---

## 📊 Impact Summary

### Before:
- ❌ Race conditions in app.Manager (sync.Map misuse)
- ❌ No request cancellation
- ❌ Goroutine leaks on timeout/disconnect
- ❌ No way to trace requests across services
- ❌ Resources not properly cleaned up

### After:
- ✅ Thread-safe app.Manager with proper locking
- ✅ Full request cancellation support
- ✅ Goroutines properly cleaned up
- ✅ Better observability (context for tracing)
- ✅ Resource cleanup on context cancellation

---

## ⚠️ Breaking Changes

All public APIs now require `context.Context` as first parameter. This affects:
- All gRPC clients
- All service providers
- All manager methods
- All HTTP/WebSocket handlers

**Migration Path**:
- For one-off calls: Use `context.Background()`
- For HTTP requests: Use `c.Request.Context()`
- For timed operations: Use `context.WithTimeout()`
- For cancellable operations: Use `context.WithCancel()`

---

## 🐛 Known Issues to Address

1. **Storage cache** still unbounded (will fix separately with LRU)
2. **Error wrapping** inconsistent (use `%w` everywhere)
3. **crypto/rand.Read** errors ignored in auth.go
4. **Input validation** missing in many handlers
5. **Structured logging** not used consistently

---

## 📝 Notes

- All changes maintain backward compatibility at the data layer
- No database schema changes required
- No breaking changes to protobuf definitions
- Tests will need minor updates (context.Background())

**Estimated remaining work**: ~2-3 hours to complete all remaining tasks.

