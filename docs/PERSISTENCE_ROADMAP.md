# Persistence & Full Environment Roadmap

## Vision: Complete Computing Environment

Build a persistent, self-contained OS experience where users can:
- **Save applications** for later use
- **Resume sessions** exactly where they left off
- **Install app packages** from a catalog
- **Manage workspaces** like virtual desktops
- **Have a full digital life** inside the environment

## Architecture: Multi-Layer Persistence

```
┌────────────────────────────────────────────────────────┐
│  USER EXPERIENCE                                        │
│  - Launch saved apps instantly                          │
│  - Resume entire workspace on startup                   │
│  - Install apps from catalog                            │
│  - Multiple workspaces (work/personal/gaming)           │
└────────────────────────────────────────────────────────┘
                        ↓
┌────────────────────────────────────────────────────────┐
│  PERSISTENCE LAYER                                      │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐ │
│  │ App Registry │  │   Session    │  │  Workspace   │ │
│  │  (Library)   │  │  Snapshots   │  │  Manager     │ │
│  └──────────────┘  └──────────────┘  └──────────────┘ │
└────────────────────────────────────────────────────────┘
                        ↓
┌────────────────────────────────────────────────────────┐
│  STORAGE BACKEND (Already Exists!)                     │
│  - Kernel syscalls (sandboxed file access)             │
│  - JSON storage per app                                │
│  - App-specific directories                            │
└────────────────────────────────────────────────────────┘
```

---

## Phase 1: App Registry (1-2 days)

**What:** Persistent application library - save app definitions for instant relaunch

### Features
- Save app UISpecs with metadata (name, icon, category)
- Browse installed apps in App Launcher
- Launch saved apps instantly (no AI generation needed!)
- Update/delete saved apps
- Export/import app packages (.aiapp files)

### Implementation

#### 1. App Package Format (.aiapp)
```json
{
  "id": "calculator-v1",
  "name": "Calculator",
  "description": "Standard calculator with scientific functions",
  "icon": "🧮",
  "category": "productivity",
  "version": "1.0.0",
  "author": "system",
  "created_at": "2025-10-03T10:30:00Z",
  "ui_spec": { /* Full UISpec */ },
  "services": ["storage"],
  "permissions": ["STANDARD"],
  "tags": ["math", "calculator", "utility"]
}
```

#### 2. Backend: AppRegistry Service
```python
# ai-service/src/services/builtin/app_registry.py

class AppRegistry:
    """Manages installed applications."""
    
    def __init__(self, storage_path="/tmp/ai-os-storage/system/apps"):
        self.storage_path = storage_path
        self._ensure_directory()
    
    async def save_app(self, app_package: AppPackage) -> str:
        """Save an app to the registry."""
        path = f"{self.storage_path}/{app_package.id}.aiapp"
        content = json.dumps(app_package.dict(), indent=2)
        await self.kernel_tools.file_write(path, content)
        return app_package.id
    
    async def load_app(self, app_id: str) -> AppPackage:
        """Load an app from the registry."""
        path = f"{self.storage_path}/{app_id}.aiapp"
        content = await self.kernel_tools.file_read(path)
        return AppPackage(**json.loads(content))
    
    async def list_apps(self, category: Optional[str] = None) -> List[AppPackage]:
        """List all installed apps."""
        files = await self.kernel_tools.directory_list(self.storage_path)
        apps = []
        for file in files:
            if file.endswith('.aiapp'):
                app_id = file.replace('.aiapp', '')
                apps.append(await self.load_app(app_id))
        
        if category:
            apps = [a for a in apps if a.category == category]
        
        return sorted(apps, key=lambda a: a.name)
    
    async def delete_app(self, app_id: str) -> bool:
        """Delete an app from registry."""
        path = f"{self.storage_path}/{app_id}.aiapp"
        return await self.kernel_tools.file_delete(path)
    
    async def export_app(self, app_id: str, export_path: str) -> str:
        """Export app package to external file."""
        app = await self.load_app(app_id)
        # Write to user-specified location
        await self.kernel_tools.file_write(export_path, json.dumps(app.dict()))
        return export_path
    
    async def import_app(self, import_path: str) -> str:
        """Import app package from file."""
        content = await self.kernel_tools.file_read(import_path)
        app_package = AppPackage(**json.loads(content))
        return await self.save_app(app_package)
```

#### 3. New API Endpoints
```python
# ai-service/src/main.py

@app.post("/apps/save")
async def save_app_to_registry(request: SaveAppRequest):
    """Save a running app to the registry."""
    app = app_manager.get_app(request.app_id)
    if not app:
        raise HTTPException(404, "App not found")
    
    # Create app package
    package = AppPackage(
        id=f"{app.title.lower().replace(' ', '-')}-v1",
        name=app.title,
        description=request.description,
        icon=request.icon or "📦",
        category=request.category or "general",
        ui_spec=app.ui_spec,
        services=app.services
    )
    
    app_id = await app_registry.save_app(package)
    return {"success": True, "app_id": app_id}

@app.get("/apps/registry")
async def list_installed_apps(category: Optional[str] = None):
    """List all installed apps from registry."""
    apps = await app_registry.list_apps(category)
    return {"apps": [a.dict() for a in apps]}

@app.post("/apps/launch/{app_id}")
async def launch_saved_app(app_id: str):
    """Launch an app from the registry (no AI needed!)."""
    package = await app_registry.load_app(app_id)
    
    # Spawn directly from saved UISpec
    app = app_manager.spawn_app(
        request=f"Launch {package.name}",
        ui_spec=package.ui_spec,
        metadata={"from_registry": True, "package_id": app_id}
    )
    
    return {
        "app_id": app.id,
        "title": app.title,
        "ui_spec": app.ui_spec
    }
```

#### 4. Frontend: App Launcher UI
```typescript
// New component: ui/src/components/AppLauncher.tsx

interface SavedApp {
  id: string;
  name: string;
  icon: string;
  category: string;
}

export function AppLauncher() {
  const [apps, setApps] = useState<SavedApp[]>([]);
  
  useEffect(() => {
    fetch('/apps/registry')
      .then(r => r.json())
      .then(data => setApps(data.apps));
  }, []);
  
  const launchApp = async (appId: string) => {
    const res = await fetch(`/apps/launch/${appId}`, { method: 'POST' });
    const data = await res.json();
    // Render the app immediately (it's already generated!)
    onAppSpawned(data);
  };
  
  return (
    <div className="app-launcher">
      <h2>Installed Applications</h2>
      <div className="app-grid">
        {apps.map(app => (
          <div key={app.id} className="app-card" onClick={() => launchApp(app.id)}>
            <div className="app-icon">{app.icon}</div>
            <div className="app-name">{app.name}</div>
          </div>
        ))}
      </div>
      <button onClick={() => createNewApp()}>
        ➕ Create New App
      </button>
    </div>
  );
}
```

---

## Phase 2: Session Persistence (2-3 days)

**What:** Save/restore entire workspace state - all open apps with their data

### Features
- Auto-save workspace every 30 seconds
- Manual "Save Session" button
- Restore session on startup
- Named sessions (work, personal, gaming)
- Session snapshots (save points you can return to)

### Implementation

#### 1. Session Format
```json
{
  "id": "session-20251003-103045",
  "name": "Work Session",
  "created_at": "2025-10-03T10:30:45Z",
  "updated_at": "2025-10-03T15:22:10Z",
  "workspace": {
    "apps": [
      {
        "id": "app-uuid-1",
        "title": "Todo List",
        "ui_spec": { /* ... */ },
        "state": "active",
        "position": { "x": 100, "y": 100 },
        "size": { "width": 600, "height": 400 },
        "component_state": {
          "todos": ["Finish persistence layer", "Test app registry"],
          "filter": "all"
        }
      },
      {
        "id": "app-uuid-2",
        "title": "Notes",
        "state": "background",
        "component_state": {
          "content": "Meeting notes...",
          "cursor_position": 142
        }
      }
    ],
    "focused_app_id": "app-uuid-1",
    "layout": "tiled"
  }
}
```

#### 2. Backend: SessionManager
```python
# ai-service/src/agents/session_manager.py

class SessionManager:
    """Manages workspace sessions."""
    
    def __init__(self, app_manager: AppManager, storage_path: str):
        self.app_manager = app_manager
        self.storage_path = storage_path
        self.auto_save_task = None
    
    async def save_session(self, name: str = "default") -> str:
        """Save current workspace state."""
        session_data = {
            "id": f"session-{datetime.now().strftime('%Y%m%d-%H%M%S')}",
            "name": name,
            "created_at": datetime.now().isoformat(),
            "workspace": {
                "apps": [
                    {
                        "id": app.id,
                        "title": app.title,
                        "ui_spec": app.ui_spec,
                        "state": app.state,
                        "services": app.services,
                        "component_state": await self._capture_app_state(app.id)
                    }
                    for app in self.app_manager.list_apps()
                ],
                "focused_app_id": self.app_manager.focused_app_id
            }
        }
        
        path = f"{self.storage_path}/sessions/{session_data['id']}.json"
        await self.kernel_tools.file_write(path, json.dumps(session_data))
        return session_data['id']
    
    async def restore_session(self, session_id: str = "default") -> bool:
        """Restore workspace from saved session."""
        path = f"{self.storage_path}/sessions/{session_id}.json"
        content = await self.kernel_tools.file_read(path)
        session_data = json.loads(content)
        
        # Close all current apps
        for app in self.app_manager.list_apps():
            self.app_manager.close_app(app.id)
        
        # Restore each app
        for app_data in session_data['workspace']['apps']:
            restored_app = self.app_manager.spawn_app(
                request=f"Restore {app_data['title']}",
                ui_spec=app_data['ui_spec'],
                metadata={"restored_from_session": True}
            )
            
            # Restore component state
            if 'component_state' in app_data:
                await self._restore_app_state(restored_app.id, app_data['component_state'])
        
        # Restore focus
        if session_data['workspace']['focused_app_id']:
            self.app_manager.focus_app(session_data['workspace']['focused_app_id'])
        
        return True
    
    async def start_auto_save(self, interval_seconds: int = 30):
        """Start auto-saving session."""
        async def auto_save_loop():
            while True:
                await asyncio.sleep(interval_seconds)
                await self.save_session("default")
                logger.info("Auto-saved session")
        
        self.auto_save_task = asyncio.create_task(auto_save_loop())
```

---

## Phase 3: App Store / Catalog (3-4 days)

**What:** Built-in app marketplace with curated applications

### Features
- Browse app categories (productivity, games, utilities, creative)
- Search apps by name/tags
- Install apps with one click
- Rate and review apps
- Auto-update installed apps
- Submit your own apps

### Implementation

#### 1. App Catalog Structure
```
/tmp/ai-os-storage/system/
  ├── catalog/
  │   ├── productivity/
  │   │   ├── calculator.aiapp
  │   │   ├── todo-list.aiapp
  │   │   ├── notes.aiapp
  │   │   └── calendar.aiapp
  │   ├── games/
  │   │   ├── tic-tac-toe.aiapp
  │   │   └── snake.aiapp
  │   ├── utilities/
  │   │   ├── timer.aiapp
  │   │   └── weather.aiapp
  │   └── creative/
  │       ├── drawing.aiapp
  │       └── music-player.aiapp
  └── installed/
      └── [user's installed apps]
```

#### 2. Seed Default Apps
```python
# ai-service/src/agents/default_apps.py

DEFAULT_APPS = [
    {
        "id": "calculator",
        "name": "Calculator",
        "icon": "🧮",
        "category": "productivity",
        "description": "Standard calculator with scientific functions",
        "ui_spec": { /* Calculator UISpec */ }
    },
    {
        "id": "todo-list",
        "name": "Todo List",
        "icon": "✅",
        "category": "productivity",
        "description": "Simple task management",
        "ui_spec": { /* Todo UISpec */ }
    },
    {
        "id": "notes",
        "name": "Notes",
        "icon": "📝",
        "category": "productivity",
        "description": "Quick note-taking app",
        "ui_spec": { /* Notes UISpec */ }
    },
    # Add 10-15 default apps
]

async def seed_app_catalog():
    """Populate catalog with default apps."""
    for app_data in DEFAULT_APPS:
        package = AppPackage(**app_data)
        await app_catalog.add_app(package)
```

---

## Phase 4: Workspace Management (2-3 days)

**What:** Multiple named workspaces like virtual desktops

### Features
- Create named workspaces (Work, Personal, Gaming, etc.)
- Switch between workspaces
- Each workspace has its own set of apps
- Workspace-specific settings
- Quick workspace switcher (Ctrl+1, Ctrl+2, etc.)

### Implementation

```python
# ai-service/src/agents/workspace_manager.py

class Workspace:
    id: str
    name: str
    icon: str
    apps: List[str]  # App IDs
    settings: Dict[str, Any]
    created_at: datetime

class WorkspaceManager:
    def __init__(self):
        self.workspaces: Dict[str, Workspace] = {}
        self.active_workspace_id: Optional[str] = None
    
    async def create_workspace(self, name: str, icon: str = "🖥️") -> Workspace:
        """Create a new workspace."""
        workspace = Workspace(
            id=str(uuid.uuid4()),
            name=name,
            icon=icon,
            apps=[],
            settings={},
            created_at=datetime.now()
        )
        self.workspaces[workspace.id] = workspace
        return workspace
    
    async def switch_workspace(self, workspace_id: str) -> bool:
        """Switch to a different workspace."""
        if workspace_id not in self.workspaces:
            return False
        
        # Save current workspace state
        await self.session_manager.save_session(self.active_workspace_id)
        
        # Load new workspace
        self.active_workspace_id = workspace_id
        await self.session_manager.restore_session(workspace_id)
        
        return True
```

---

## Phase 5: Advanced Features (Ongoing)

### A. App Lifecycle Hooks
```typescript
// Apps can define lifecycle methods
{
  "hooks": {
    "on_mount": "storage.data.load('state')",
    "on_suspend": "storage.data.save('state', current_state)",
    "on_resume": "storage.data.load('state')",
    "on_destroy": "storage.data.save('final_state', current_state)"
  }
}
```

### B. Inter-App Communication
```python
# Apps can send messages to each other
await ipc.send_message(
    from_app="todo-list",
    to_app="calendar",
    message={
        "type": "add_event",
        "data": {"title": "Meeting", "date": "2025-10-05"}
    }
)
```

### C. Background Services
```python
# System services that run always
class BackgroundService:
    async def start(self):
        while True:
            # Sync data, check notifications, etc.
            await asyncio.sleep(60)
```

### D. Settings & Preferences
```json
{
  "system": {
    "theme": "dark",
    "auto_save_interval": 30,
    "default_workspace": "work",
    "startup_apps": ["notes", "todo-list"]
  },
  "user": {
    "name": "Griffin",
    "avatar": "👨‍💻"
  }
}
```

---

## Why This is Better Than Bare Metal

| Feature | Bare Metal OS | Your Approach |
|---------|--------------|---------------|
| **Development Time** | 2-3 years | 2-3 weeks |
| **Cross-Platform** | ❌ Single arch | ✅ Works everywhere |
| **AI Integration** | Complex | ✅ Native |
| **Sandboxing** | Manual | ✅ Built-in |
| **Hot Reload** | ❌ Requires reboot | ✅ Instant |
| **Cloud Sync** | Complex | ✅ Easy (JSON files) |
| **Web Access** | ❌ Difficult | ✅ Natural |

---

## Implementation Timeline

| Phase | Duration | Effort | Impact |
|-------|----------|--------|--------|
| Phase 1: App Registry | 1-2 days | Medium | ⭐⭐⭐⭐⭐ |
| Phase 2: Sessions | 2-3 days | Medium | ⭐⭐⭐⭐⭐ |
| Phase 3: App Store | 3-4 days | High | ⭐⭐⭐⭐ |
| Phase 4: Workspaces | 2-3 days | Medium | ⭐⭐⭐⭐ |
| Phase 5: Advanced | Ongoing | Variable | ⭐⭐⭐ |

**Total: ~2 weeks for full environment vs 2-3 years for bare metal**

---

## Next Steps

1. **Implement App Registry first** - Biggest immediate value
2. **Add Session Persistence** - Makes it feel permanent
3. **Seed default apps** - Give users instant value
4. **Polish the UX** - Make it feel like a real OS

You'll have a complete computing environment that feels more magical than a traditional OS, works everywhere, and took weeks instead of years.

**Want me to start implementing Phase 1 (App Registry)?**

