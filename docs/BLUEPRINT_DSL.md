# Blueprint DSL (.bp) Specification

## Overview

Blueprint is a concise, YAML-based DSL for describing full-stack applications in the AI OS ecosystem. It combines UI/UX specifications with backend service integration in a format that's easy for both humans and AI to read and write.

## Why YAML?

- **80% less verbose** than JSON
- No excessive brackets, braces, or quotes
- Supports comments
- Widely understood by AI models
- Excellent Python/Go parsing support
- Human-readable indentation-based structure

## File Format

Blueprint files use the `.bp` extension and are interpreted at runtime (no compilation needed).

## Basic Structure

```yaml
---
# Metadata
app:
  id: notes
  name: Notes
  icon: 📝
  category: productivity
  version: 1.0.0
  author: system
  tags: [notes, markdown, productivity]
  permissions: [STANDARD]

# Backend Services
services:
  - storage
  - filesystem

# UI Specification
ui:
  title: Notes
  layout: horizontal
  
  lifecycle:
    on_mount: storage.get
  
  components:
    - sidebar:
        layout: vertical
        gap: 8
        padding: medium
        style:
          width: 200px
          borderRight: 1px solid rgba(255,255,255,0.1)
        
        children:
          - button#new-note:
              text: "+ New Note"
              variant: primary
              fullWidth: true
              @click: ui.set
          
          - list#notes-list:
              variant: default
    
    - editor:
        layout: vertical
        gap: 12
        padding: large
        style: { flex: 1 }
        
        children:
          - input#note-title:
              placeholder: "Note title..."
              type: text
              style: { fontSize: 24px, fontWeight: bold }
              @change: storage.set
          
          - textarea#note-content:
              placeholder: "Start typing..."
              rows: 20
              resize: vertical
              @change: storage.set
```

## Syntax Features

### 1. Component Declaration

**Concise syntax with inline ID:**
```yaml
button#my-button:
  text: Click me
  variant: primary
```

**Equivalent to JSON:**
```json
{
  "type": "button",
  "id": "my-button",
  "props": {
    "text": "Click me",
    "variant": "primary"
  }
}
```

### 2. Event Handlers

Use `"@"` prefix for events (must be quoted in YAML):
```yaml
button#submit:
  text: Submit
  "@click": form.submit
  "@hover": ui.highlight
```

### 3. Service Integration

**Simple import (all tools):**
```yaml
services:
  - storage              # All tools: set, get, remove, list, clear
  - filesystem           # All tools: list, stat, read, write, create, mkdir, delete, move, copy, exists
```

**Explicit tool selection:**
```yaml
services:
  - storage: [get, set]              # Only specific tools
  - filesystem: [list, read, mkdir]  # Limited filesystem access
  - system: [info, time]             # Read-only system info
```

**Explicit all tools:**
```yaml
services:
  - storage: *           # Same as simple import, but explicit
  - filesystem: *
```

**With configuration (future):**
```yaml
services:
  - storage:
      tools: [get, set, remove]
      scope: app
      persist: true
  - filesystem:
      tools: *
      root: /tmp/ai-os-storage
      readonly: false
```

### 4. Lifecycle Hooks

```yaml
lifecycle:
  on_mount: storage.get
  on_unmount: storage.save
  on_focus: ui.refresh
```

Multiple actions:
```yaml
lifecycle:
  on_mount:
    - storage.get
    - system.log
    - ui.init
```

### 5. Layout Shortcuts

**Shorthand for common patterns:**
```yaml
# Instead of verbose container definitions
row:          # horizontal container
  gap: 16
  children:
    - text: Hello
    - text: World

col:          # vertical container
  gap: 8
  children:
    - button: Top
    - button: Bottom

grid:
  columns: 3
  gap: 20
  children:
    - card: Item 1
    - card: Item 2
    - card: Item 3
```

### 6. Component Shortcuts

For simple components:
```yaml
# Short form
text: "Hello World"

# Expands to
text#auto-id:
  content: "Hello World"
  variant: body
```

### 7. Inline Styles

YAML's flow style for compact CSS:
```yaml
style: { width: 300px, height: 200px, backgroundColor: #1a1a1a }

# Or multiline
style:
  width: 300px
  height: 200px
  backgroundColor: "#1a1a1a"
```

### 8. Templates & Reuse

```yaml
templates:
  action-button:
    type: button
    variant: primary
    size: medium
  
  card-style:
    padding: large
    borderRadius: 8px
    backgroundColor: rgba(0,0,0,0.2)

components:
  - button#save:
      $template: action-button
      text: Save
  
  - container#card:
      $template: card-style
      children:
        - text: Content
```

## Complete Example: File Explorer

```yaml
---
app:
  id: file-explorer
  name: File Explorer
  icon: 📁
  category: system
  version: 1.0.0
  permissions: [READ_FILE, WRITE_FILE, CREATE_FILE, DELETE_FILE, LIST_DIRECTORY]
  tags: [files, explorer, system]

services:
  - filesystem:
      root: /tmp/ai-os-storage
  - storage:
      scope: app

ui:
  title: File Explorer
  layout: horizontal
  
  lifecycle:
    on_mount: filesystem.list
  
  components:
    # Sidebar
    - sidebar:
        layout: vertical
        gap: 8
        padding: medium
        style: { width: 240px, borderRight: 1px solid rgba(255,255,255,0.1) }
        
        children:
          - text#sidebar-title:
              content: Locations
              variant: h3
              weight: bold
          
          - list#locations:
              variant: default
              spacing: small
              children:
                - button#home-btn:
                    text: 🏠 Home
                    variant: ghost
                    fullWidth: true
                    @click: filesystem.list
                
                - button#documents-btn:
                    text: 📄 Documents
                    variant: ghost
                    fullWidth: true
                    @click: filesystem.list
    
    # Main panel
    - main:
        layout: vertical
        gap: 12
        padding: large
        style: { flex: 1 }
        
        children:
          # Toolbar
          - row:
              gap: 8
              align: center
              children:
                - button#back: { text: "←", variant: outline, size: small, @click: ui.set }
                - button#forward: { text: "→", variant: outline, size: small, @click: ui.set }
                - button#up: { text: "↑", variant: outline, size: small, @click: ui.set }
                - input#path-input:
                    placeholder: /Users/...
                    value: /tmp/ai-os-storage
                    type: text
                    style: { flex: 1 }
                    @change: filesystem.list
                - button#refresh: { text: "⟳", variant: outline, size: small, @click: filesystem.list }
                - button#new-folder: { text: "+ Folder", variant: primary, size: small, @click: filesystem.mkdir }
          
          # Breadcrumbs
          - text#current-path:
              content: /tmp/ai-os-storage
              variant: caption
          
          - divider:
              orientation: horizontal
          
          # File list
          - col:
              gap: 0
              style: { flex: 1, overflowY: auto, backgroundColor: rgba(0,0,0,0.1), borderRadius: 8px }
              children:
                - list#file-list:
                    variant: default
                    spacing: small
```

## Available Services Reference

### Storage Service
**ID**: `storage`  
**Tools**: `set`, `get`, `remove`, `list`, `clear`  
**Use for**: Persistent key-value data, app settings, user preferences

```yaml
services:
  - storage: [get, set]  # Most common for simple apps
  - storage: *           # All tools for advanced storage needs
```

### Filesystem Service
**ID**: `filesystem`  
**Tools**: `list`, `stat`, `read`, `write`, `create`, `mkdir`, `delete`, `move`, `copy`, `exists`  
**Use for**: File management, directory browsing, file I/O

```yaml
services:
  - filesystem: [list, read]           # Read-only file browser
  - filesystem: [list, read, write]    # File editor
  - filesystem: *                      # Full file manager
```

### System Service
**ID**: `system`  
**Tools**: `info`, `time`, `log`, `getLogs`, `ping`  
**Use for**: System monitoring, logging, diagnostics

```yaml
services:
  - system: [info, time]  # System info display
  - system: [log]         # Logging only
  - system: *             # Full system access
```

### Auth Service
**ID**: `auth`  
**Tools**: `register`, `login`, `logout`, `verify`, `getUser`  
**Use for**: User authentication, session management

```yaml
services:
  - auth: [login, logout, verify]  # Basic auth
  - auth: *                        # Full auth including registration
```

## Parser Implementation

The Blueprint parser will:

1. **Parse YAML** → Internal representation
2. **Expand shortcuts** → Full component tree
3. **Validate services** → Check against registry
4. **Resolve templates** → Apply reusable patterns
5. **Generate UISpec** → Compatible with existing DynamicRenderer

### Python Parser (ai-service)

```python
# ai-service/src/blueprint/parser.py
import yaml
from typing import Dict, Any

class BlueprintParser:
    def parse(self, bp_content: str) -> Dict[str, Any]:
        """Parse Blueprint YAML to UISpec JSON"""
        bp = yaml.safe_load(bp_content)
        
        return {
            "id": bp["app"]["id"],
            "name": bp["app"]["name"],
            "icon": bp["app"].get("icon"),
            "category": bp["app"].get("category"),
            "version": bp["app"].get("version", "1.0.0"),
            "services": self._expand_services(bp.get("services", [])),
            "permissions": bp["app"].get("permissions", ["STANDARD"]),
            "tags": bp["app"].get("tags", []),
            "ui_spec": self._expand_ui(bp["ui"])
        }
    
    def _expand_ui(self, ui: Dict) -> Dict:
        """Expand UI shortcuts to full component tree"""
        return {
            "type": "app",
            "title": ui.get("title"),
            "layout": ui.get("layout", "vertical"),
            "lifecycle_hooks": self._expand_lifecycle(ui.get("lifecycle", {})),
            "components": [
                self._expand_component(comp) 
                for comp in ui.get("components", [])
            ]
        }
    
    def _expand_component(self, comp: Any) -> Dict:
        """Expand component shortcuts"""
        if isinstance(comp, str):
            # Simple text: "Hello" -> {type: text, content: "Hello"}
            return {"type": "text", "props": {"content": comp}}
        
        if isinstance(comp, dict):
            # Extract type and ID from key
            for key, props in comp.items():
                type_id = key.split("#") if "#" in key else [key, None]
                comp_type = type_id[0]
                comp_id = type_id[1] if len(type_id) > 1 else None
                
                # Handle shortcuts
                if comp_type == "row":
                    comp_type = "container"
                    props["layout"] = "horizontal"
                elif comp_type == "col":
                    comp_type = "container"
                    props["layout"] = "vertical"
                
                # Extract event handlers (@click, @change, etc.)
                events = {}
                clean_props = {}
                for k, v in props.items():
                    if k.startswith("@"):
                        events[k[1:]] = v  # Remove @ prefix
                    elif k == "children":
                        clean_props[k] = [
                            self._expand_component(child) 
                            for child in v
                        ]
                    else:
                        clean_props[k] = v
                
                result = {
                    "type": comp_type,
                    "props": clean_props
                }
                
                if comp_id:
                    result["id"] = comp_id
                
                if events:
                    result["on_event"] = events
                
                return result
        
        return comp
```

### Go Parser (backend)

```go
// backend/internal/blueprint/parser.go
package blueprint

import (
    "gopkg.in/yaml.v3"
)

type Blueprint struct {
    App      App                    `yaml:"app"`
    Services []interface{}          `yaml:"services"`
    UI       UI                     `yaml:"ui"`
}

func Parse(content []byte) (*types.Package, error) {
    var bp Blueprint
    if err := yaml.Unmarshal(content, &bp); err != nil {
        return nil, err
    }
    
    // Convert to Package (existing format)
    return &types.Package{
        ID:          bp.App.ID,
        Name:        bp.App.Name,
        Icon:        bp.App.Icon,
        // ... expand to full Package struct
    }, nil
}
```

## Migration Strategy

1. **Phase 1**: Keep `.aiapp` (JSON) as fallback
2. **Phase 2**: Add `.bp` parser alongside `.aiapp` loader
3. **Phase 3**: Migrate existing apps to `.bp`
4. **Phase 4**: Deprecate `.aiapp` format

## Benefits

| Feature | JSON | Blueprint (YAML) |
|---------|------|------------------|
| File size | 4.2 KB | **0.9 KB** (78% smaller) |
| Lines | 97 | **32** (67% fewer) |
| Readability | Medium | **High** |
| Comments | No | **Yes** |
| AI generation | Good | **Excellent** |
| Type safety | Runtime | Runtime |
| Parsing speed | Fast | Fast |

## AI System Prompt

To teach AI models the Blueprint DSL:

```
You are generating Blueprint (.bp) files for the AI OS platform.

Blueprint is a YAML-based DSL with these conventions:
1. Components use type#id syntax: `button#save:`
2. Events use @ prefix: `@click: tool.name`
3. Layouts use shortcuts: `row`, `col`, `grid`
4. Services are simple imports: `services: [storage, filesystem]`
5. Keep it concise - no unnecessary nesting

Example:
```yaml
app: { id: calc, name: Calculator, icon: 🧮 }
services: [storage]
ui:
  title: Calculator
  layout: vertical
  components:
    - display#result:
        type: input
        value: "0"
        readonly: true
    - grid:
        columns: 4
        children:
          - button#btn-1: { text: "1", @click: calc.digit }
          - button#btn-2: { text: "2", @click: calc.digit }
```

Always output valid YAML. Test your indentation.
```

## Next Steps

1. Implement Blueprint parser in Python (ai-service)
2. Implement Blueprint parser in Go (backend)
3. Update registry seeder to support `.bp` files
4. Add Blueprint validation
5. Create migration tool: `.aiapp` → `.bp`
6. Update documentation

