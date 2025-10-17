# Prebuilt Apps System

## Overview

The prebuilt apps system allows the OS to ship with essential system applications as JSON UISpec files. These apps are loaded on startup and can be launched instantly without AI generation.

## Architecture

### Directory Structure

```
apps/
├── system/              # Core system applications
│   ├── file-explorer.aiapp
│   ├── task-manager.aiapp
│   └── settings.aiapp (seeded)
├── productivity/        # Productivity applications
│   ├── calculator.aiapp (seeded)
│   └── notes.aiapp
├── utilities/           # Utility applications
└── creative/            # Creative applications
```

### Components Built

#### 1. **Filesystem Service Provider** (`backend/internal/providers/filesystem.go`)
- **Purpose**: Sandboxed file system operations for apps
- **Category**: `CategoryFilesystem`
- **Tools**:
  - `filesystem.list` - List directory contents
  - `filesystem.stat` - Get file/directory metadata
  - `filesystem.read` - Read file contents
  - `filesystem.write` - Write to file
  - `filesystem.create` - Create new file
  - `filesystem.mkdir` - Create directory
  - `filesystem.delete` - Delete file/directory
  - `filesystem.move` - Move/rename file/directory
  - `filesystem.copy` - Copy file/directory
  - `filesystem.exists` - Check file existence
- **Pattern**: Follows exact provider pattern (storage.go, auth.go)
- **Security**: Uses kernel syscalls with sandbox PID for isolation

#### 2. **Enhanced Kernel Syscalls**
- **Proto Updates** (`proto/kernel.proto`):
  - Added `FileStatCall` - Get file metadata
  - Added `MoveFileCall` - Move/rename operations
  - Added `CopyFileCall` - Copy operations
  - Added `CreateDirectoryCall` - Directory creation
- **Rust Implementation** (`kernel/src/syscall.rs`):
  - `file_stat()` - Returns JSON with name, path, size, is_dir, mode, modified, extension
  - `move_file()` - Uses `fs::rename` with sandbox checks
  - `copy_file()` - Uses `fs::copy` with sandbox checks
  - `create_directory()` - Uses `fs::create_dir_all` with sandbox checks
- **gRPC Integration** (`kernel/src/grpc_server.rs`):
  - Added proto conversion for all new syscalls
  - Maintains permission model (ReadFile, WriteFile, CreateFile capabilities)

#### 3. **App Registry Seeder** (`backend/internal/registry/seeder.go`)
- **Purpose**: Load prebuilt apps on server startup
- **Functions**:
  - `SeedApps()` - Walks apps directory and loads all .aiapp files
  - `SeedDefaultApps()` - Creates essential system apps if missing (app-launcher, settings, calculator)
  - `loadApp()` - Parses and registers individual .aiapp file
- **Integration**: Called in `server.go` during initialization
- **Logging**: Detailed startup logs showing loaded/failed apps

#### 4. **Frontend Service Integration**
- **Update**: `ui/src/components/dynamics/DynamicRenderer.executor.ts`
- **Change**: Added `"filesystem"` and `"system"` to service prefixes
- **Effect**: Filesystem tools are routed to backend via `/services/execute` endpoint

#### 5. **Prebuilt Applications**

##### **File Explorer** (`apps/system/file-explorer.aiapp`)
- **Description**: Modern file browser with sidebar navigation
- **Services**: `filesystem`, `storage`
- **Permissions**: READ_FILE, WRITE_FILE, CREATE_FILE, DELETE_FILE, LIST_DIRECTORY
- **Features**:
  - Sidebar with quick locations (Home, Documents, Downloads, Desktop)
  - Toolbar with navigation (back, forward, up, refresh)
  - Path input with breadcrumbs
  - File list with columns (Name, Modified, Size, Actions)
  - Create folder button
  - Status bar with item counts
- **Lifecycle**: Calls `filesystem.list` on mount

##### **Notes** (`apps/productivity/notes.aiapp`)
- **Description**: Simple note-taking app
- **Services**: `storage`
- **Features**:
  - Sidebar with note list and "New Note" button
  - Editor with title input and content textarea
  - Auto-save to backend storage
- **Lifecycle**: Loads saved notes on mount

##### **Task Manager** (`apps/system/task-manager.aiapp`)
- **Description**: System monitor and app manager
- **Services**: `system`
- **Features**:
  - Tabbed interface (Applications, Performance)
  - Applications list with running apps
  - Performance metrics (CPU, Memory)
  - Refresh button
- **Use Case**: Monitor system resources and manage running apps

##### **Seeded Defaults**
Built into seeder if missing:
- **App Launcher** - Browse and launch installed apps
- **Settings** - System configuration
- **Calculator** - Basic arithmetic

## Technical Decisions

### 1. **Language: JSON UISpec (Not New Code)**
- ✅ Consistent with existing app generation system
- ✅ Can be modified by AI later ("add preview to file explorer")
- ✅ Secure (data, not code)
- ✅ Dynamic rendering already implemented
- ✅ No new execution environment needed

### 2. **Backend: Go Service Providers**
- ✅ Follows existing pattern (storage.go, auth.go, system.go)
- ✅ Strong typing with types.Service interface
- ✅ Registered in service registry
- ✅ Uses kernel syscalls for sandboxing
- ✅ Short, focused functions (50-100 lines each)

### 3. **Kernel: Rust Syscalls**
- ✅ Platform-specific code properly handled (#[cfg(unix)])
- ✅ Sandbox permission checks on every operation
- ✅ Detailed logging with PID tracking
- ✅ JSON serialization for complex data (file metadata)
- ✅ Error handling with SyscallResult enum

### 4. **In-Repo Organization**
- ✅ All prebuilt apps in `apps/` at project root
- ✅ Categorized by purpose (system, productivity, utilities, creative)
- ✅ Easy to add more apps (just drop .aiapp files)
- ✅ Seeder automatically picks them up

### 5. **Strong Typing & Tech Debt Reduction**
- **Go**: types.Service, types.Tool, types.Parameter, types.Result
- **Rust**: Syscall enum, SyscallResult enum, FileInfo struct
- **Proto**: Strongly typed messages for all syscalls
- **No duplicated responsibilities**: Each provider handles one domain

## File Organization

### One Word, Memorable Names ✅
- `filesystem.go` - Filesystem operations
- `seeder.go` - App seeding logic
- `file-explorer.aiapp` - File explorer app
- `task-manager.aiapp` - Task manager app
- `notes.aiapp` - Notes app

### Short, Focused Files ✅
- `filesystem.go`: 480 lines (10 tools, clean separation)
- `seeder.go`: 170 lines (2 main functions, clear purpose)
- Each app: ~100-300 lines of JSON (readable, maintainable)

### Testing & Extensibility ✅
- **Service Registry Pattern**: Easy to mock providers
- **Interface-Based**: KernelClient interface for dependency injection
- **Sandbox Isolation**: Each app runs in separate process with PID
- **Permission Model**: Fine-grained capabilities (ReadFile, WriteFile, etc.)
- **Extensible Tools**: New tools added by implementing types.Tool

## Usage

### Adding New Prebuilt Apps
1. Create `.aiapp` file in appropriate category directory
2. Define UISpec with components, tools, services
3. App loads automatically on next server start
4. Access via `/registry/apps` endpoint

### Launching Prebuilt Apps
```typescript
// Frontend
const response = await fetch('/registry/apps/file-explorer/launch', { method: 'POST' });
const { app_id, ui_spec } = await response.json();
// DynamicRenderer renders ui_spec
```

### Creating New Service Providers
1. Create `providers/newservice.go`
2. Implement `Definition()` and `Execute()` methods
3. Register in `server.go` `registerProviders()`
4. Add category to `types/service.go` if new
5. Frontend automatically routes `newservice.*` tools to backend

## Integration Points

### Backend (Go)
- `backend/internal/providers/filesystem.go` - New provider
- `backend/internal/types/service.go` - CategoryFilesystem added
- `backend/internal/registry/seeder.go` - New seeder
- `backend/internal/server/server.go` - Seeder integration & provider registration

### Kernel (Rust)
- `kernel/src/syscall.rs` - 4 new syscalls implemented
- `kernel/src/grpc_server.rs` - Proto conversion for new syscalls

### Proto
- `proto/kernel.proto` - 4 new message types added

### Frontend (TypeScript)
- `ui/src/components/dynamics/DynamicRenderer.executor.ts` - Service prefixes updated

## Future Enhancements

1. **Hot Reload**: Watch apps directory for changes
2. **App Store**: Publish/download apps from marketplace
3. **Versioning**: Semantic versioning for app updates
4. **Dependencies**: Apps can depend on other apps
5. **Permissions UI**: Visual permission requests
6. **App Signing**: Cryptographic verification
7. **Custom Components**: Register new component types
8. **Themes**: Per-app theme overrides

## Summary

✅ **All 4 Steps Completed**:
1. ✅ Filesystem service provider (Go)
2. ✅ File explorer UISpec (JSON)
3. ✅ Apps directory structure
4. ✅ Registry seeding on startup

✅ **Follows Exact Patterns**:
- Provider pattern matches storage.go/auth.go exactly
- Syscall pattern matches existing filesystem operations
- Server registration follows established flow
- Tool execution uses existing service infrastructure

✅ **Tech Debt Minimized**:
- Strong typing throughout (Go interfaces, Rust enums, Proto messages)
- Short, focused files (each <500 lines)
- One-word memorable names
- No duplicate responsibilities
- Highly testable (interfaces, dependency injection)

✅ **Extensible & Tested**:
- Easy to add new apps (just drop .aiapp files)
- Easy to add new services (follow provider pattern)
- Sandbox isolation for security
- Permission model for fine-grained control

