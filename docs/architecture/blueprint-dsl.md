# Blueprint DSL (.bp) Specification

## Overview

Blueprint is a JSON-based DSL for describing full-stack applications in the AI OS ecosystem. It combines UI/UX specifications with backend service integration in a format optimized for real-time streaming and incremental rendering.

## Design Principles

- **Streaming-first** - Components render as they're generated, not at the end
- **Explicit structure** - All fields are clear key-value pairs (no special syntax in keys)
- **AI-friendly** - Clear, unambiguous structure for language models
- **Human-readable** - Easy to read, write, and understand
- **Type-safe** - Strong validation at runtime
- **Incrementally parsable** - Can extract complete components from partial JSON

## Why Explicit Format?

**Explicit format:**
```json
{"type": "button", "id": "save", "props": {"text": "Save"}}
```

**Why this works for streaming:**
- ✅ Type and ID are VALUES (easy to extract with regex during streaming)
- ✅ Complete objects render immediately as they finish
- ✅ No special parsing logic for key syntax
- ✅ Simpler parsers across all languages
- ✅ Clear component boundaries for incremental rendering

## File Format

Blueprint files use the `.bp` extension and are interpreted at runtime (no compilation needed).

## Basic Structure

```json
{
  "app": {
    "id": "notes",
    "name": "Notes",
    "icon": "",
    "category": "productivity",
    "version": "1.0.0",
    "author": "system",
    "tags": ["notes", "markdown", "productivity"],
    "permissions": ["STANDARD"]
  },
  "services": [{"storage": ["get", "set", "list"]}],
  "ui": {
    "title": "Notes",
    "layout": "horizontal",
    "lifecycle": {"on_mount": "storage.get"},
    "components": [
      {
        "type": "container",
        "id": "sidebar",
        "props": {
          "layout": "vertical",
          "gap": 8,
          "padding": "medium",
          "style": {"width": "200px", "borderRight": "1px solid rgba(255,255,255,0.1)"}
        },
        "children": [
          {
            "type": "button",
            "id": "new-note",
            "props": {"text": "+ New Note", "variant": "primary", "fullWidth": true},
            "on_event": {"click": "ui.set"}
          },
          {
            "type": "list",
            "id": "notes-list",
            "props": {"variant": "default"}
          }
        ]
      },
      {
        "type": "container",
        "id": "editor",
        "props": {
          "layout": "vertical",
          "gap": 12,
          "padding": "large",
          "style": {"flex": 1}
        },
        "children": [
          {
            "type": "input",
            "id": "note-title",
            "props": {
              "placeholder": "Note title...",
              "type": "text",
              "style": {"fontSize": "24px", "fontWeight": "bold"}
            },
            "on_event": {"change": "storage.set"}
          },
          {
            "type": "textarea",
            "id": "note-content",
            "props": {"placeholder": "Start typing...", "rows": 20, "resize": "vertical"},
            "on_event": {"change": "storage.set"}
          }
        ]
      }
    ]
  }
}
```

## Component Format

### Standard Component Structure

All components use explicit format:

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

**Required fields:**
- `type` - Component type (button, input, text, container, grid, etc.)
- `id` - Unique identifier within the app

**Optional fields:**
- `props` - Component properties (text, value, style, etc.)
- `on_event` - Event handlers (click, change, etc.)
- `children` - Nested components (for containers)

### Event Handlers

Use the `on_event` object:
```json
{
  "type": "button",
  "id": "submit",
  "props": {"text": "Submit"},
  "on_event": {
    "click": "form.submit",
    "hover": "ui.highlight"
  }
}
```

### Service Integration

**Simple import (all tools):**
```json
{
  "services": ["storage", "filesystem"]
}
```

**Explicit tool selection:**
```json
{
  "services": [
    {
      "storage": ["get", "set"]
    },
    {
      "filesystem": ["list", "read", "mkdir"]
    },
    {
      "system": ["info", "time"]
    }
  ]
}
```

**With configuration (future):**
```json
{
  "services": [
    {
      "storage": {
        "tools": ["get", "set", "remove"],
        "scope": "app",
        "persist": true
      }
    },
    {
      "filesystem": {
        "tools": "*",
        "root": "/tmp/ai-os-storage",
        "readonly": false
      }
    }
  ]
}
```

### Lifecycle Hooks

```json
{
  "lifecycle": {
    "on_mount": "storage.get",
    "on_unmount": "storage.save",
    "on_focus": "ui.refresh"
  }
}
```

Multiple actions:
```json
{
  "lifecycle": {
    "on_mount": [
      "storage.get",
      "system.log",
      "ui.init"
    ]
  }
}
```

### Layout Shortcuts

Use `type: "row"` or `type: "col"` for horizontal/vertical containers:

**Horizontal row:**
```json
{
  "type": "row",
  "id": "controls",
  "props": {"gap": 16},
  "children": [
    {"type": "text", "id": "label1", "props": {"content": "Hello"}},
    {"type": "text", "id": "label2", "props": {"content": "World"}}
  ]
}
```

**Vertical column:**
```json
{
  "type": "col",
  "id": "sidebar",
  "props": {"gap": 8},
  "children": [
    {"type": "button", "id": "top", "props": {"text": "Top"}},
    {"type": "button", "id": "bottom", "props": {"text": "Bottom"}}
  ]
}
```

```json
{
  "grid": {
    "columns": 3,
    "gap": 20,
    "children": [
      {
        "card": "Item 1"
      },
      {
        "card": "Item 2"
      },
      {
        "card": "Item 3"
      }
    ]
  }
}
```

**Semantic containers** (for better readability and styling):
```json
{
  "sidebar": {
    "gap": 8,
    "padding": "medium",
    "style": {
      "width": "240px"
    },
    "children": []
  }
}
```

Available semantic types: `sidebar`, `main`, `editor`, `header`, `footer`, `content`, `section`

These expand to `container` with a `role` property for semantic meaning:
- Adds `semantic-{role}` CSS class
- Adds `data-role="{role}"` attribute
- Defaults to `vertical` layout unless specified

### 6. Component Shortcuts

For simple components:
```json
// Short form
"text": "Hello World"

// Expands to
{
  "text#auto-id": {
    "content": "Hello World",
    "variant": "body"
  }
}
```

### 7. Inline Styles

JSON objects for CSS properties:
```json
{
  "style": {
    "width": "300px",
    "height": "200px",
    "backgroundColor": "#1a1a1a"
  }
}
```

### 8. Templates & Reuse

```json
{
  "templates": {
    "action-button": {
      "type": "button",
      "variant": "primary",
      "size": "medium"
    },
    "card-style": {
      "padding": "large",
      "borderRadius": "8px",
      "backgroundColor": "rgba(0,0,0,0.2)"
    }
  },
  "components": [
    {
      "button#save": {
        "$template": "action-button",
        "text": "Save"
      }
    },
    {
      "container#card": {
        "$template": "card-style",
        "children": [
          {
            "text": "Content"
          }
        ]
      }
    }
  ]
}
```

### 9. Multi-Page Navigation (Tabs/Pages)

For complex applications with multiple views, use the `tabs` component with container children:

```json
{
  "ui": {
    "title": "System Monitor",
    "layout": "vertical",
    "components": [
      {
        "type": "tabs",
        "id": "main-tabs",
        "props": {
          "defaultTab": "overview",
          "variant": "default"
        },
        "children": [
          {
            "type": "container",
            "id": "overview",
            "props": {
              "label": " Overview",
              "layout": "vertical",
              "padding": "medium"
            },
            "children": [
              {
                "type": "text",
                "id": "overview-title",
                "props": {
                  "content": "System Overview",
                  "variant": "h2"
                }
              }
            ]
          },
          {
            "type": "container",
            "id": "details",
            "props": {
              "label": " Details",
              "layout": "vertical",
              "padding": "medium"
            },
            "children": [
              {
                "type": "text",
                "id": "details-title",
                "props": {
                  "content": "Detailed Metrics",
                  "variant": "h2"
                }
              }
            ]
          }
        ]
      }
    ]
  }
}
```

**Why This Structure?**

Each page is a complete container with all its children. This structure:
- ✅ Allows AI to output complete pages sequentially
- ✅ Pages are stacked vertically in code (natural for generation)
- ✅ Each page is self-contained with its own layout and styling
- ✅ Easy to stream incrementally during generation

**Tab Properties:**
- `defaultTab` - ID of the tab to show initially (in tabs props)
- `variant` - Tab style: `default`, `pills`, `underline`, `vertical`
- Each child container must have:
  - `id` - Unique identifier (used as tab ID)
  - `props.label` - Display text for the tab (supports emojis)
  - `children` - Full page content

**Nested Tabs:**
You can nest tabs within tab pages for complex navigation:

```json
{
  "type": "tabs",
  "id": "main-tabs",
  "props": { "defaultTab": "metrics" },
  "children": [
    {
      "type": "container",
      "id": "metrics",
      "props": { "label": "Metrics", "layout": "vertical" },
      "children": [
        {
          "type": "tabs",
          "id": "metrics-subtabs",
          "props": {
            "variant": "pills",
            "defaultTab": "cpu"
          },
          "children": [
            {
              "type": "container",
              "id": "cpu",
              "props": { "label": "CPU", "layout": "vertical" },
              "children": []
            },
            {
              "type": "container",
              "id": "memory",
              "props": { "label": "Memory", "layout": "vertical" },
              "children": []
            }
          ]
        }
      ]
    }
  ]
}
```

## Complete Example: File Explorer

```json
{
  "app": {
    "id": "file-explorer",
    "name": "File Explorer",
    "icon": "",
    "category": "system",
    "version": "1.0.0",
    "permissions": ["READ_FILE", "WRITE_FILE", "CREATE_FILE", "DELETE_FILE", "LIST_DIRECTORY"],
    "tags": ["files", "explorer", "system"]
  },
  "services": [
    {
      "filesystem": {
        "root": "/tmp/ai-os-storage"
      }
    },
    {
      "storage": {
        "scope": "app"
      }
    }
  ],
  "ui": {
    "title": "File Explorer",
    "layout": "horizontal",
    "lifecycle": {
      "on_mount": "filesystem.list"
    },
    "components": [
      {
        "sidebar": {
          "layout": "vertical",
          "gap": 8,
          "padding": "medium",
          "style": {
            "width": "240px",
            "borderRight": "1px solid rgba(255,255,255,0.1)"
          },
          "children": [
            {
              "text#sidebar-title": {
                "content": "Locations",
                "variant": "h3",
                "weight": "bold"
              }
            },
            {
              "list#locations": {
                "variant": "default",
                "spacing": "small",
                "children": [
                  {
                    "button#home-btn": {
                      "text": " Home",
                      "variant": "ghost",
                      "fullWidth": true,
                      "@click": "filesystem.list"
                    }
                  },
                  {
                    "button#documents-btn": {
                      "text": " Documents",
                      "variant": "ghost",
                      "fullWidth": true,
                      "@click": "filesystem.list"
                    }
                  }
                ]
              }
            }
          ]
        }
      },
      {
        "main": {
          "layout": "vertical",
          "gap": 12,
          "padding": "large",
          "style": {
            "flex": 1
          },
          "children": [
            {
              "row": {
                "gap": 8,
                "align": "center",
                "children": [
                  {
                    "button#back": {
                      "text": "←",
                      "variant": "outline",
                      "size": "small",
                      "@click": "ui.set"
                    }
                  },
                  {
                    "button#forward": {
                      "text": "→",
                      "variant": "outline",
                      "size": "small",
                      "@click": "ui.set"
                    }
                  },
                  {
                    "button#up": {
                      "text": "↑",
                      "variant": "outline",
                      "size": "small",
                      "@click": "ui.set"
                    }
                  },
                  {
                    "input#path-input": {
                      "placeholder": "/Users/...",
                      "value": "/tmp/ai-os-storage",
                      "type": "text",
                      "style": {
                        "flex": 1
                      },
                      "@change": "filesystem.list"
                    }
                  },
                  {
                    "button#refresh": {
                      "text": "⟳",
                      "variant": "outline",
                      "size": "small",
                      "@click": "filesystem.list"
                    }
                  },
                  {
                    "button#new-folder": {
                      "text": "+ Folder",
                      "variant": "primary",
                      "size": "small",
                      "@click": "filesystem.mkdir"
                    }
                  }
                ]
              }
            },
            {
              "text#current-path": {
                "content": "/tmp/ai-os-storage",
                "variant": "caption"
              }
            },
            {
              "divider": {
                "orientation": "horizontal"
              }
            },
            {
              "col": {
                "gap": 0,
                "style": {
                  "flex": 1,
                  "overflowY": "auto",
                  "backgroundColor": "rgba(0,0,0,0.1)",
                  "borderRadius": "8px"
                },
                "children": [
                  {
                    "list#file-list": {
                      "variant": "default",
                      "spacing": "small"
                    }
                  }
                ]
              }
            }
          ]
        }
      }
    ]
  }
}
```

## Available Services Reference

### Storage Service
**ID**: `storage`  
**Tools**: `set`, `get`, `remove`, `list`, `clear`  
**Use for**: Persistent key-value data, app settings, user preferences

```json
{
  "services": [
    {
      "storage": ["get", "set"]
    }
  ]
}
```

### Filesystem Service
**ID**: `filesystem`  
**Tools**: `list`, `stat`, `read`, `write`, `create`, `mkdir`, `delete`, `move`, `copy`, `exists`  
**Use for**: File management, directory browsing, file I/O

```json
{
  "services": [
    {
      "filesystem": ["list", "read"]
    }
  ]
}
```

### System Service
**ID**: `system`  
**Tools**: `info`, `time`, `log`, `getLogs`, `ping`  
**Use for**: System monitoring, logging, diagnostics

```json
{
  "services": [
    {
      "system": ["info", "time"]
    }
  ]
}
```

### Auth Service
**ID**: `auth`  
**Tools**: `register`, `login`, `logout`, `verify`, `getUser`  
**Use for**: User authentication, session management

```json
{
  "services": [
    {
      "auth": ["login", "logout", "verify"]
    }
  ]
}
```

## Parser Implementation

The Blueprint parser will:

1. **Parse JSON** → Internal representation
2. **Expand shortcuts** → Full component tree
3. **Validate services** → Check against registry
4. **Resolve templates** → Apply reusable patterns
5. **Generate UISpec** → Compatible with existing DynamicRenderer

### Python Parser (ai-service)

```python
# ai-service/src/blueprint/parser.py
import json
from typing import Dict, Any

class BlueprintParser:
    def parse(self, bp_content: str) -> Dict[str, Any]:
        """Parse Blueprint JSON to UISpec"""
        bp = json.loads(bp_content)
        
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
    "encoding/json"
)

type Blueprint struct {
    App      App                    `json:"app"`
    Services []interface{}          `json:"services"`
    UI       UI                     `json:"ui"`
}

func Parse(content []byte) (*types.Package, error) {
    var bp Blueprint
    if err := json.Unmarshal(content, &bp); err != nil {
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

| Feature | Legacy .aiapp | Blueprint (.bp) |
|---------|---------------|-----------------|
| Format | Raw JSON | Structured JSON with shortcuts |
| Readability | Medium | **High** |
| IDE Support | Good | **Excellent** |
| AI generation | Good | **Excellent** |
| Type safety | Runtime | Runtime |
| Parsing speed | Fast | Fast |
| Shortcuts | No | **Yes** (row/col, @events, #id) |

## AI System Prompt

To teach AI models the Blueprint DSL:

```
You are generating Blueprint (.bp) files for the AI OS platform.

Blueprint is a JSON-based DSL with these conventions:
1. Components use type#id syntax: {"button#save": {...}}
2. Events use @ prefix: "@click": "tool.name"
3. Layouts use shortcuts: "row", "col", "grid"
4. Services are simple imports or objects: ["storage"] or [{"storage": ["get", "set"]}]
5. Keep it structured and clear

Example:
{
  "app": {"id": "calc", "name": "Calculator", "icon": ""},
  "services": [{"storage": ["get", "set"]}],
  "ui": {
    "title": "Calculator",
    "layout": "vertical",
    "components": [
      {
        "input#result": {
          "type": "input",
          "value": "0",
          "readonly": true
        }
      },
      {
        "grid": {
          "columns": 4,
          "children": [
            {"button#btn-1": {"text": "1", "@click": "calc.digit"}},
            {"button#btn-2": {"text": "2", "@click": "calc.digit"}}
          ]
        }
      }
    ]
  }
}

Always output valid JSON. Validate your syntax.
```

## Next Steps

1. ✓ Implement Blueprint parser in Python (ai-service)
2. ✓ Implement Blueprint parser in Go (backend)
3. ✓ Update registry seeder to support `.bp` files
4. Add Blueprint validation
5. Create migration tool: `.aiapp` → `.bp`
6. Update documentation
