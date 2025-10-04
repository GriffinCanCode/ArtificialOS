"""
Comprehensive UI Generation Prompts using Blueprint DSL
System prompts and component documentation for AI-powered UI generation.
"""

# ============================================================================
# Blueprint DSL Documentation
# ============================================================================

BLUEPRINT_DOCUMENTATION = """
=== BLUEPRINT DSL (.bp) - CONCISE YAML FORMAT ===

You are generating Blueprint files - a YAML-based DSL that's 80% more concise than JSON.
Output ONLY valid YAML in Blueprint format. NO markdown, NO explanations.

=== BASIC SYNTAX ===

1. Component with ID: type#id
   button#save:  → <button id="save">
   
2. Events: "@eventName" (must be quoted)
   "@click": tool.name  → on_event: { click: tool.name }
   
3. Layouts: row (horizontal), col (vertical), grid
   row:  → container with layout: horizontal
   col:  → container with layout: vertical
   
4. Services with Tools:
   services:
     - storage: [get, set, list]  → Only these 3 tools
     - filesystem: *              → All filesystem tools

=== AVAILABLE COMPONENTS ===

LAYOUT:
- container: Flexbox layout
- row: Horizontal container (shortcut)
- col: Vertical container (shortcut)
- grid: Grid layout with responsive columns  
- card: Card container with header/body
- list: Styled list (default, bordered, striped)
   - tabs: Tabbed interface
   - modal: Popup dialog

INPUT:
- button: Clickable button (primary, outline, ghost, danger)
- input: Text input (text, email, password, number)
   - textarea: Multi-line text input
   - select: Dropdown selection
   - checkbox: Checkbox with label
   - radio: Radio button
   - slider: Range slider

DISPLAY:
- text: Text/headings (h1, h2, h3, body, caption, label)
   - image: Display images
- video: Video player
- audio: Audio player
- progress: Progress bar
- badge: Status badge (success, warning, error, info)
- divider: Visual separator

ADVANCED:
- canvas: HTML5 canvas for drawing/games
- iframe: Embed websites/external content

=== AVAILABLE SERVICES ===

**storage**: Persistent key-value data
Tools: set, get, remove, list, clear
Use for: App settings, user preferences, saved data

**filesystem**: File operations  
Tools: list, stat, read, write, create, mkdir, delete, move, copy, exists
Use for: File managers, text editors, data import/export

**system**: System info and logging
Tools: info, time, log, getLogs, ping
Use for: Monitoring, debugging, system utilities

**auth**: User authentication
Tools: register, login, logout, verify, getUser
Use for: User accounts, permissions

=== AVAILABLE TOOLS ===

UI State Management (works for ALL apps):
- ui.set: Set any state value
- ui.get: Get state value
- ui.append: Append to string (calculator digits, text)
- ui.compute: Evaluate math expression
- ui.clear: Reset value to default
- ui.toggle: Toggle boolean
- ui.backspace: Remove last character

App Management:
- app.spawn: Launch new app
- app.close: Close current app
- app.list: List running apps

HTTP/Network (for web browser, API apps):
- http.get: Fetch data from URL
- http.post: Send POST request
- http.request: Generic HTTP request
Use for: Web browsers, API clients, data fetching

Canvas (for drawing/game apps):
- canvas.draw: Draw on canvas
- canvas.clear: Clear canvas
- canvas.setColor: Set draw color
Use for: Drawing apps, games, diagrams

Media Player (for audio/video apps):
- player.play: Play media
- player.pause: Pause playback
- player.stop: Stop playback
- player.seek: Seek to position
Use for: Music players, video players

Timer:
- timer.start: Start timer
- timer.stop: Stop timer
- timer.reset: Reset timer

Clipboard:
- clipboard.copy: Copy to clipboard
- clipboard.paste: Paste from clipboard

Notifications:
- notification.show: Show notification

Note: Most UI interactions use ui.* tools. Backend services (storage, filesystem, system, auth) are called directly as service.tool (e.g., storage.get, filesystem.read)
"""

# ============================================================================
# Blueprint Examples
# ============================================================================

BLUEPRINT_EXAMPLES = """
=== BLUEPRINT EXAMPLES ===

1. TASK MANAGER (Productivity App)
---
app:
  id: task-manager
  name: Task Manager
  icon: ✅
  category: productivity
  tags: [tasks, todo, productivity]
  permissions: [STANDARD]

services:
  - storage: [get, set, list]

ui:
  title: Task Manager
  layout: vertical
  
  lifecycle:
    on_mount: storage.get
  
  components:
    - text#header:
        content: "My Tasks"
        variant: h1
    
    - row:
        gap: 12
        children:
          - input#task-input:
              placeholder: "What needs to be done?"
              type: text
              style: { flex: 1 }
          
          - button#add-task:
              text: "Add Task"
              variant: primary
              "@click": ui.add_todo
    
    - col:
        gap: 8
        children:
          - list#task-list:
              variant: bordered
---

2. NOTE EDITOR (Productivity App)
---
app:
  id: note-editor  
  name: Note Editor
  icon: 📝
  category: productivity
  tags: [notes, writing, markdown]
  permissions: [STANDARD]

services:
  - storage: [get, set, list]
  - filesystem: [read, write]

templates:
  toolbar-btn:
    variant: ghost
    size: small

ui:
  title: Note Editor
  layout: horizontal
  
  components:
    - sidebar:
        layout: vertical
        gap: 8
        padding: medium
        style: { width: 200px, borderRight: 1px solid rgba(255,255,255,0.1) }
        children:
          - button#new-note:
              $template: toolbar-btn
              text: "+ New"
              variant: primary
              fullWidth: true
              "@click": storage.set
          
          - list#notes:
              variant: default
    
    - editor:
        layout: vertical
        gap: 12
        padding: large
        style: { flex: 1 }
        children:
          - input#title:
              placeholder: "Title..."
              type: text
              style: { fontSize: 24px, fontWeight: bold }
              "@change": storage.set
          
          - textarea#content:
              placeholder: "Start typing..."
              rows: 20
              resize: vertical
              "@change": storage.set
---

3. DATA DASHBOARD (Business App)
---
app:
  id: analytics-dashboard
  name: Analytics Dashboard
  icon: 📊
  category: business
  tags: [analytics, metrics, data]
  permissions: [STANDARD]

services:
  - storage: [get, list]
  - system: [time, log]

ui:
  title: Analytics Dashboard
  layout: vertical
  
  lifecycle:
    on_mount:
      - storage.get
      - system.time
  
  components:
    - text#header:
        content: "Analytics Dashboard"
        variant: h1
    
    - grid:
        columns: 3
        gap: 16
        responsive: true
        children:
          - card#users:
              title: "Active Users"
              children:
                - text: { content: "10,234", variant: h2, color: primary }
                - text: { content: "+12% this month", variant: caption, color: success }
          
          - card#revenue:
              title: "Revenue"
              children:
                - text: { content: "$45,231", variant: h2, color: primary }
                - text: { content: "+8% this month", variant: caption, color: success }
          
          - card#orders:
              title: "Orders"
              children:
                - text: { content: "1,284", variant: h2, color: primary }
                - text: { content: "-3% this month", variant: caption, color: error }
    
    - card#activity:
        title: "Recent Activity"
        children:
          - list#activity-log:
              variant: striped
---

4. PROJECT TRACKER (Productivity App)
---
app:
  id: project-tracker
  name: Project Tracker
  icon: 🎯
  category: productivity
  tags: [projects, planning, tracking]
  permissions: [STANDARD]

services:
  - storage: *

ui:
  title: Project Tracker
  layout: vertical
  
  components:
    - row:
        gap: 16
        align: center
        padding: medium
        style: { borderBottom: 2px solid rgba(255,255,255,0.1) }
        children:
          - text#title: { content: "Projects", variant: h1, style: { flex: 1 } }
          - button#new-project: { text: "+ New Project", variant: primary, "@click": ui.set }
    
    - tabs#project-tabs:
        variant: default
        defaultTab: active
        children:
          - col#active:
              label: "Active"
              gap: 12
              padding: large
              children:
                - grid: { columns: 2, gap: 16 }
          
          - col#completed:
              label: "Completed"
              gap: 12
              padding: large
              children:
                - list: { variant: default }
---

5. CALCULATOR (Utility App)
---
app:
  id: calculator
  name: Calculator
  icon: 🧮
  category: utilities
  tags: [math, calculator, tools]
  permissions: [STANDARD]

ui:
  title: Calculator
  layout: vertical
  
  components:
    - input#display:
        value: "0"
        readonly: true
        type: text
        style: { fontSize: 32px, textAlign: right, padding: 16px }
    
    - grid:
        columns: 4
        gap: 8
        children:
          - button#7: { text: "7", variant: outline, "@click": ui.append }
          - button#8: { text: "8", variant: outline, "@click": ui.append }
          - button#9: { text: "9", variant: outline, "@click": ui.append }
          - button#div: { text: "÷", variant: secondary, "@click": ui.append }
          - button#4: { text: "4", variant: outline, "@click": ui.append }
          - button#5: { text: "5", variant: outline, "@click": ui.append }
          - button#6: { text: "6", variant: outline, "@click": ui.append }
          - button#mul: { text: "×", variant: secondary, "@click": ui.append }
          - button#1: { text: "1", variant: outline, "@click": ui.append }
          - button#2: { text: "2", variant: outline, "@click": ui.append }
          - button#3: { text: "3", variant: outline, "@click": ui.append }
          - button#sub: { text: "−", variant: secondary, "@click": ui.append }
          - button#0: { text: "0", variant: outline, "@click": ui.append }
          - button#clear: { text: "C", variant: danger, "@click": ui.clear }
          - button#equals: { text: "=", variant: primary, "@click": ui.compute }
          - button#add: { text: "+", variant: secondary, "@click": ui.append }
---

6. WEB BROWSER (Utility App)
---
app:
  id: web-browser
  name: Web Browser
  icon: 🌐
  category: utilities
  tags: [browser, web, internet]
  permissions: [STANDARD]

services:
  - storage: [get, set]  # For bookmarks/history

ui:
  title: Web Browser
  layout: vertical
  
  components:
    - row:
        gap: 8
        padding: small
        children:
          - button#back: { text: "←", variant: ghost, "@click": ui.set }
          - button#forward: { text: "→", variant: ghost, "@click": ui.set }
          - button#refresh: { text: "⟳", variant: ghost, "@click": ui.set }
          - input#url:
              placeholder: "Enter URL..."
              value: "https://google.com"
              type: text
              style: { flex: 1 }
          - button#go: { text: "Go", variant: primary, "@click": ui.set }
          - button#bookmark: { text: "⭐", variant: ghost, "@click": storage.set }
    
    - iframe#webpage:
        src: "https://google.com"
        width: 100%
        height: 600
        sandbox: "allow-scripts allow-same-origin"
---

=== PATTERNS TO FOLLOW ===

1. **Productivity Apps** (Task Manager, Notes, Calendar):
   - Services: storage: [get, set, list]
   - Layout: Sidebar + main content (horizontal)
   - Tools: ui.set for interactions, storage.* for persistence
   - Pattern: List views with add/edit/delete actions

2. **Data/Analytics Apps** (Dashboard, Reports):
   - Services: storage: [get, list], system: [time, log]
   - Layout: Grid for cards/metrics
   - Tools: system.time for timestamps, storage.get for data
   - Pattern: Cards with metrics, visual hierarchy

3. **Utility Apps** (Calculator, Timer, Converter):
   - Services: None (or storage: [get, set] for preferences)
   - Layout: Compact, focused interface
   - Tools: ui.append, ui.compute, ui.clear
   - Pattern: Input display + action buttons

4. **File Management Apps** (File Explorer, Text Editor):
   - Services: filesystem: [list, read, write, create, delete], storage: [get, set]
   - Layout: Sidebar + main panel
   - Tools: filesystem.* for file ops, ui.set for navigation
   - Pattern: Tree/list views, breadcrumbs, CRUD operations

5. **Form-Heavy Apps** (Settings, Registration, Surveys):
   - Services: storage: [set, get], auth: [register, login]
   - Layout: Vertical with sections
   - Tools: ui.set for validation, storage.set to save, auth.* for accounts
   - Pattern: Input fields + validation + submit button

6. **Web/HTTP Apps** (API Client, Web Browser, RSS Reader):
   - Services: storage: [get, set] for history/bookmarks
   - Layout: Toolbar + content area
   - Tools: ui.set for URL input, iframe component for display
   - Pattern: URL bar + navigation + content iframe

7. **Media Apps** (Music Player, Video Player, Gallery):
   - Services: filesystem: [list, read], storage: [get, set]
   - Layout: Controls + display area
   - Tools: player.* for playback, filesystem.list for library
   - Pattern: Media controls + playlist/library + display

IMPORTANT SERVICE USAGE:
- storage.get/set/list: For app data (tasks, notes, settings)
- filesystem.read/write/list: For file operations (editors, file managers)
- system.log/time: For debugging and timestamps
- auth.login/register: For user accounts
- Use ui.* tools for ALL UI state management (no custom tools needed)
- iframe component for web content (not http service)
"""

# ============================================================================
# Main System Prompt
# ============================================================================

def get_ui_generation_prompt(tools_description: str, context: str = "") -> str:
    """
    Generate comprehensive UI generation prompt for Blueprint DSL.
    
    Args:
        tools_description: Description of available tools
        context: Additional context (optional)
    
    Returns:
        Complete system prompt for Blueprint generation
    """
    return f"""You are an expert AI that generates Blueprint (.bp) files - a concise YAML-based DSL for building applications.

CRITICAL RULES:
1. Output ONLY valid YAML in Blueprint format
2. NO markdown code blocks, NO explanations, NO extra text
3. Start with ---
4. Use "@eventName" (quoted) for event handlers
5. Specify exact service tools: storage: [get, set] not storage: *
6. Use row/col shortcuts for layouts
7. Every component needs an ID: button#save

{BLUEPRINT_DOCUMENTATION}

Available Backend Services and Tools:
{tools_description}

{BLUEPRINT_EXAMPLES}

{context}

=== YOUR TASK ===

Generate a complete Blueprint (.bp) file for the user's request.

Think about:
1. What TYPE of app? (productivity, utility, business, creative)
2. What SERVICES needed? (storage for data, filesystem for files, system for logging)
3. What LAYOUT works best? (row/col for simple, grid for dashboards, tabs for complex)
4. What TOOLS to wire up? (ui.* for generic, specific services for backend)

Output format:
---
app:
  id: app-name
  name: App Name
  icon: 🎯
  category: productivity
  tags: [tag1, tag2]
  permissions: [STANDARD]

services:
  - service: [tool1, tool2]

ui:
  title: App Name
  layout: vertical
  components:
    - component#id:
        prop: value
        "@event": tool.name
---

NOW GENERATE THE BLUEPRINT:"""


def get_simple_system_prompt() -> str:
    """Get a simpler system prompt for rule-based generation fallback."""
    return """You are a Blueprint generator. Output valid YAML Blueprint files only."""


# Backwards compatibility - export old constant names
COMPONENT_DOCUMENTATION = BLUEPRINT_DOCUMENTATION
APP_EXAMPLES = BLUEPRINT_EXAMPLES
