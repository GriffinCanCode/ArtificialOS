# Scheduler API Integration Status

## Summary

After reviewing the kernel refactoring with the builder pattern and scheduler integration, here's the status of API integration across the stack:

## ✅ What's Working

### 1. **Kernel Implementation** (Rust)
- ✅ ProcessManager now uses builder pattern correctly
- ✅ Scheduler integrated via builder: `.with_scheduler(Policy::Fair)`
- ✅ gRPC server receives ProcessManager with scheduler
- ✅ All scheduler methods exposed via gRPC:
  - `schedule_next()` → `ProcessManager::schedule_next()`
  - `get_scheduler_stats()` → `ProcessManager::get_scheduler_stats()`
  - `set_scheduling_policy()` → `ProcessManager::set_scheduling_policy()`

### 2. **Protocol Buffers**
- ✅ Proto definitions updated with scheduler RPCs:
  ```protobuf
  rpc ScheduleNext(ScheduleNextRequest) returns (ScheduleNextResponse);
  rpc GetSchedulerStats(GetSchedulerStatsRequest) returns (GetSchedulerStatsResponse);
  rpc SetSchedulingPolicy(SetSchedulingPolicyRequest) returns (SetSchedulingPolicyResponse);
  ```
- ✅ Scheduler syscalls defined in SyscallRequest (fields 66-73)
- ✅ Generated Go code is up to date

### 3. **Go Backend** (Kernel Client)
- ✅ Full gRPC client implementation in `backend/internal/grpc/kernel.go`:
  ```go
  func (k *KernelClient) ScheduleNext(ctx context.Context) (*uint32, error)
  func (k *KernelClient) GetSchedulerStats(ctx context.Context) (*pb.SchedulerStats, error)
  func (k *KernelClient) SetSchedulingPolicy(ctx context.Context, policy string) error
  ```
- ✅ Proper timeout handling (5s)
- ✅ Error handling with context

## ⚠️ What Needs Attention

### 1. **Go Backend HTTP API** (Missing)
Currently, the scheduler APIs are **NOT exposed via HTTP** to the Python AI service. The Go backend only has these HTTP endpoints:

**Existing:**
- `/apps` - App management
- `/services` - Service discovery/execution
- `/generate-ui` - AI operations
- `/registry/*` - App registry
- `/sessions/*` - Session management

**Missing:**
- ❌ No `/kernel/schedule-next` endpoint
- ❌ No `/kernel/scheduler/stats` endpoint
- ❌ No `/kernel/scheduler/policy` endpoint

### 2. **Python AI Service** (No Direct Access)
The Python AI service uses `BackendClient` which communicates via HTTP with the Go backend. Since the Go backend doesn't expose scheduler HTTP endpoints, the Python service **cannot access scheduler functionality**.

## 🔧 Required Changes

### Option 1: Add HTTP Handlers (Recommended)
Expose scheduler APIs via HTTP in Go backend:

**File: `backend/internal/http/kernel_handlers.go` (NEW)**
```go
package http

import (
    "net/http"
    "github.com/gin-gonic/gin"
)

// ScheduleNext schedules the next process
func (h *Handlers) ScheduleNext(c *gin.Context) {
    ctx := c.Request.Context()
    
    nextPID, err := h.kernel.ScheduleNext(ctx)
    if err != nil {
        c.JSON(http.StatusInternalServerError, gin.H{
            "error": err.Error(),
        })
        return
    }
    
    c.JSON(http.StatusOK, gin.H{
        "success": true,
        "next_pid": nextPID,
    })
}

// GetSchedulerStats retrieves scheduler statistics
func (h *Handlers) GetSchedulerStats(c *gin.Context) {
    ctx := c.Request.Context()
    
    stats, err := h.kernel.GetSchedulerStats(ctx)
    if err != nil {
        c.JSON(http.StatusInternalServerError, gin.H{
            "error": err.Error(),
        })
        return
    }
    
    c.JSON(http.StatusOK, gin.H{
        "success": true,
        "stats": gin.H{
            "total_scheduled":   stats.TotalScheduled,
            "context_switches":  stats.ContextSwitches,
            "preemptions":       stats.Preemptions,
            "active_processes":  stats.ActiveProcesses,
            "policy":            stats.Policy,
            "quantum_micros":    stats.QuantumMicros,
        },
    })
}

// SetSchedulingPolicy changes the scheduling policy
func (h *Handlers) SetSchedulingPolicy(c *gin.Context) {
    ctx := c.Request.Context()
    
    var req struct {
        Policy string `json:"policy" binding:"required"`
    }
    
    if err := c.ShouldBindJSON(&req); err != nil {
        c.JSON(http.StatusBadRequest, gin.H{
            "error": "Invalid request: " + err.Error(),
        })
        return
    }
    
    // Validate policy
    validPolicies := map[string]bool{
        "RoundRobin": true,
        "Priority":   true,
        "Fair":       true,
    }
    
    if !validPolicies[req.Policy] {
        c.JSON(http.StatusBadRequest, gin.H{
            "error": "Invalid policy. Must be RoundRobin, Priority, or Fair",
        })
        return
    }
    
    if err := h.kernel.SetSchedulingPolicy(ctx, req.Policy); err != nil {
        c.JSON(http.StatusInternalServerError, gin.H{
            "error": err.Error(),
        })
        return
    }
    
    c.JSON(http.StatusOK, gin.H{
        "success": true,
        "policy": req.Policy,
    })
}
```

**File: `backend/internal/server/server.go` (UPDATE)**
Add routes:
```go
// Kernel/Scheduler operations
router.POST("/kernel/schedule-next", handlers.ScheduleNext)
router.GET("/kernel/scheduler/stats", handlers.GetSchedulerStats)
router.PUT("/kernel/scheduler/policy", handlers.SetSchedulingPolicy)
```

### Option 2: Python Direct gRPC (Alternative)
Add a Python gRPC client for direct kernel communication:

**File: `ai-service/src/clients/kernel.py` (NEW)**
```python
import grpc
from typing import Optional
from dataclasses import dataclass

# Import generated protobuf (would need to generate Python protobufs)
# from proto import kernel_pb2, kernel_pb2_grpc

@dataclass
class SchedulerStats:
    total_scheduled: int
    context_switches: int
    preemptions: int
    active_processes: int
    policy: str
    quantum_micros: int

class KernelClient:
    def __init__(self, addr: str = "localhost:50051"):
        self.channel = grpc.insecure_channel(addr)
        # self.stub = kernel_pb2_grpc.KernelServiceStub(self.channel)
    
    def schedule_next(self) -> Optional[int]:
        # Implementation
        pass
    
    def get_scheduler_stats(self) -> SchedulerStats:
        # Implementation
        pass
    
    def set_scheduling_policy(self, policy: str) -> bool:
        # Implementation
        pass
```

## 📋 Recommendations

1. **Implement Option 1** (HTTP handlers in Go backend)
   - Consistent with current architecture (Python → HTTP → Go → gRPC → Kernel)
   - No need to generate Python protobufs
   - Easier to maintain
   - Better for the Python AI service

2. **Update Python BackendClient** once HTTP handlers are added:
   ```python
   # In ai-service/src/clients/backend.py
   
   def schedule_next(self) -> Optional[int]:
       """Schedule the next process"""
       response = self._client.post(f"{self.backend_url}/kernel/schedule-next")
       response.raise_for_status()
       data = response.json()
       return data.get("next_pid")
   
   def get_scheduler_stats(self) -> dict:
       """Get scheduler statistics"""
       response = self._client.get(f"{self.backend_url}/kernel/scheduler/stats")
       response.raise_for_status()
       return response.json()["stats"]
   
   def set_scheduling_policy(self, policy: str) -> bool:
       """Set scheduling policy"""
       response = self._client.put(
           f"{self.backend_url}/kernel/scheduler/policy",
           json={"policy": policy}
       )
       response.raise_for_status()
       return response.json()["success"]
   ```

## 🎯 Current Architecture Flow

```
┌─────────────────┐
│  Python AI      │
│  Service        │
└────────┬────────┘
         │ HTTP (via BackendClient)
         ▼
┌─────────────────┐
│  Go Backend     │
│  (HTTP Server)  │
└────────┬────────┘
         │ gRPC (KernelClient)
         ▼
┌─────────────────┐
│  Rust Kernel    │
│  (gRPC Server)  │
│  - ProcessMgr   │
│  - Scheduler    │
└─────────────────┘
```

## ✅ No Breaking Changes Needed

The builder pattern refactoring in the kernel **does not break existing APIs**:
- gRPC interface remains the same
- Proto definitions are unchanged
- Go client implementation is correct
- Kernel initialization properly uses builder pattern in `main.rs`

The only gap is the **missing HTTP layer** in the Go backend for Python access.

## 📝 Action Items

1. [ ] Create `backend/internal/http/kernel_handlers.go` with scheduler HTTP handlers
2. [ ] Add routes in `backend/internal/server/server.go`
3. [ ] Update `ai-service/src/clients/backend.py` with scheduler methods
4. [ ] Test full stack integration (Python → Go → Kernel)
5. [ ] Add integration tests for scheduler HTTP endpoints
6. [ ] Update documentation

## 🔍 Testing Checklist

Once HTTP handlers are added:

```bash
# Test Go → Kernel (already works)
curl http://localhost:8000/kernel/scheduler/stats

# Test policy change
curl -X PUT http://localhost:8000/kernel/scheduler/policy \
  -H "Content-Type: application/json" \
  -d '{"policy": "Priority"}'

# Test schedule next
curl -X POST http://localhost:8000/kernel/schedule-next

# Test from Python
python3 -c "
from clients import BackendClient
client = BackendClient('http://localhost:8000')
stats = client.get_scheduler_stats()
print(stats)
"
```
