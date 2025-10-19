# Filesystem Structure

## Overview

The AgentOS filesystem is **automatically initialized** on first kernel startup with a standardized directory structure. No manual setup required!

## Standard Directory Layout

```
/storage/
   native-apps/   # Prebuilt OS applications (synced from apps/dist)
      browser/
      file-explorer/
      hub/
      settings/
      terminal/
   apps/          # User/AI-generated applications
      {app-id}/
          data/
          config/
   user/          # User files
      documents/
      downloads/
      projects/
   system/        # System configuration
   lib/           # Shared libraries

/tmp/              # Temporary files (100MB limit, in-memory)
   {app-id}/    # Per-app temp directories

/cache/            # Cache files (50MB limit, in-memory)
   {app-id}/    # Per-app cache directories
```

## Automatic Initialization

### What Happens on First Startup

1. **VFS Mounts** are created automatically:
   - `/storage` - Persistent local filesystem
   - `/tmp` - In-memory filesystem (100MB)
   - `/cache` - In-memory filesystem (50MB)

2. **Standard directories** are created:
   - All directories shown above are automatically created
   - File watching (observability) is enabled for all mounts

3. **Native apps are synced**:
   - If `WORKSPACE_ROOT` is set, native apps from `apps/dist/` are copied to `/storage/native-apps/`
   - This happens automatically without user intervention

### Environment Variables

```bash
# Optional: Custom storage location
export KERNEL_STORAGE_PATH="/path/to/storage"  # Default: /tmp/ai-os-storage

# Optional: For native app syncing
export WORKSPACE_ROOT="/path/to/project"       # Auto-set by Makefile
```

### Code Location

The initialization logic is centralized in:
- **Kernel (Rust)**: `kernel/src/vfs/init.rs`
- **Backend (Go)**: `backend/internal/shared/paths/`
- **Frontend (TypeScript)**: `ui/src/core/utils/paths.ts`

All three layers use the **same path constants** for consistency.

## Path Utilities

### Rust (Kernel)

```rust
use ai_os_kernel::vfs::{storage, app, user};

// Standard paths
let native_apps = storage::NATIVE_APPS;  // "/storage/native-apps"
let user_docs = user::DOCUMENTS;         // "/storage/user/documents"

// App-specific paths
let app_data = app::data_dir("my-app");   // "/storage/apps/my-app/data"
let app_cache = app::cache_dir("my-app"); // "/cache/my-app"
```

### Go (Backend)

```go
import "github.com/GriffinCanCode/AgentOS/backend/internal/shared/paths"

// Standard paths
nativeApps := paths.NativeApps  // "/storage/native-apps"
userDocs := paths.Documents     // "/storage/user/documents"

// App-specific paths
app := paths.AppPath("my-app")
dataDir := app.DataDir()   // "/storage/apps/my-app/data"
cacheDir := app.CacheDir() // "/cache/my-app"

// Path validation
if paths.IsUserspacePath(somePath) {
    // Safe to access
}
```

### TypeScript (Frontend)

```typescript
import { STORAGE, USER, appPath, normalizePath } from '@/core/utils/paths';

// Standard paths
const nativeApps = STORAGE.NATIVE_APPS;  // "/storage/native-apps"
const userDocs = USER.DOCUMENTS;         // "/storage/user/documents"

// App-specific paths
const app = appPath("my-app");
const dataDir = app.dataDir();    // "/storage/apps/my-app/data"
const cacheDir = app.cacheDir();  // "/cache/my-app"

// Path utilities
const clean = normalizePath("/storage//user/../user/docs");
```

## Security & Sandboxing

### Userspace Restrictions

Apps can only access paths within:
- `/storage/apps/{their-app-id}/` (their own directory)
- `/storage/user/` (with user permission)
- `/tmp/{their-app-id}/` (their temp directory)
- `/cache/{their-app-id}/` (their cache directory)

System paths (`/storage/system/`) and other apps' directories are protected.

### Path Validation

All path operations are validated at multiple layers:

1. **Kernel**: Permission checks via sandbox manager
2. **Backend**: Path validation via `paths.IsUserspacePath()`
3. **Frontend**: Path normalization and validation

```go
// Example: Backend validation
func validatePath(path string, appCtx *types.Context) error {
    if !paths.IsUserspacePath(path) {
        return fmt.Errorf("access denied: outside userspace")
    }
    return nil
}
```

## File Watching

All filesystem mounts have **built-in file watching** (observable events):

```rust
// Kernel automatically emits events
FileEvent::Created { path }
FileEvent::Modified { path }
FileEvent::Deleted { path }
FileEvent::Renamed { from, to }
```

Apps can subscribe to file changes via the `watch` syscall (see `kernel/src/syscalls/impls/watch.rs`).

## Development Tips

### Testing Locally

```bash
# Start with custom storage location
export KERNEL_STORAGE_PATH="$HOME/.ai-os-dev"
make start

# Check filesystem structure
ls -la /tmp/ai-os-storage  # or $HOME/.ai-os-dev
```

### Resetting Storage

```bash
# Remove all data (be careful!)
rm -rf /tmp/ai-os-storage

# Restart to reinitialize
make start
```

### Adding New Standard Directories

1. Add constant to `kernel/src/vfs/paths.rs`
2. Add to `standard_directories()` function
3. Mirror in `backend/internal/shared/paths/paths.go`
4. Mirror in `ui/src/core/utils/paths.ts`
5. Update this documentation

## Troubleshooting

### Native apps not syncing

**Problem**: `/storage/native-apps/` is empty

**Solution**: Ensure `WORKSPACE_ROOT` is set:
```bash
export WORKSPACE_ROOT="$(pwd)"
make start
```

### Permission denied errors

**Problem**: Can't write to `/storage/system/`

**Solution**: System paths are protected. Use `/storage/apps/{your-app}/` instead.

### Out of space errors

**Problem**: `/tmp` or `/cache` is full

**Solution**: These are in-memory filesystems with size limits:
- `/tmp`: 100MB limit
- `/cache`: 50MB limit

Clean up old files or increase limits in `kernel/src/core/limits.rs`.

## Future Enhancements

Planned features:
- Content-addressable storage (automatic deduplication)
- File versioning and snapshots
- Intelligent caching layer
- Virtual filesystems (`/proc`, `/sys`)

For implementation details, see:
- `docs/FILESYSTEM_QUICK_START.md`
- `docs/FILESYSTEM_REDESIGN.md`

