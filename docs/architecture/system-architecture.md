# AI-OS Architecture

## Core Principle: Generate Once, Execute Many

The system generates UI specifications once, then executes tools to handle all interactions. No re-prompting needed for normal use.

---

## Data Flow Architecture

```
1. USER/APP: "create a calculator"
   
2. AppManager (Orchestrator)
   - Coordinates app lifecycle
   - Tracks all running apps
   - Handles parent-child relationships
   - Manages focus/foreground/background
   
3. UIGeneratorAgent
   - LLM generates UISpec (JSON) ONCE
   - Includes all components + tool bindings
   - NO CODE EXECUTION - pure data
   
4. Backend returns: {app_id, ui_spec, thoughts}
   
5. DynamicRenderer (Frontend)
   - Parses UISpec JSON
   - Renders React components
   - Creates ComponentState
   - Initializes ToolExecutor
   
6. USER CLICKS BUTTON
   
7. Event Handler -> ToolExecutor.execute(tool_id, params)
   - calc.add: Arithmetic operations
   - ui.set_state: Update component state
   - system.alert: Show dialogs
   - app.spawn: CREATE NEW APP! (back to step 1)
   
8. ComponentState updates -> React re-renders
   (automatic, no LLM needed)
```

---

## Module Responsibilities

### AppManager (`ai-service/src/agents/app_manager.py`)
**Role:** Central orchestrator for all apps

**Responsibilities:**
- Track all running app instances
- Handle app spawning (from user OR from apps)
- Manage app lifecycle (spawning → active → background → destroyed)
- Coordinate parent-child relationships
- Handle focus management
- Clean up destroyed apps

**Key Methods:**
- `spawn_app()`: Create new app instance
- `get_app(app_id)`: Retrieve app by ID
- `focus_app(app_id)`: Bring app to foreground
- `close_app(app_id)`: Destroy app and children
- `list_apps()`: Get all apps
- `get_stats()`: Get metrics

**State:**
```python
{
  "apps": {
    "uuid-1": AppInstance(title="Calculator", state="active", ...),
    "uuid-2": AppInstance(title="Todo List", state="background", parent_id="uuid-1", ...)
  },
  "focused_app_id": "uuid-1"
}
```

---

### UIGeneratorAgent (`ai-service/src/agents/ui_generator.py`)
**Role:** Generate UI specifications from natural language

**Responsibilities:**
- Parse user intent
- Generate structured UISpec (JSON)
- Bind tools to UI events
- Provide component templates
- Currently uses rule-based generation with LLM fallback

**Tool Registry:**
- `calc.*`: Arithmetic (add, subtract, multiply, divide, etc.)
- `ui.*`: State management (set_state, get_state, add_todo)
- `system.*`: System operations (alert, log)
- `app.*`: App management (spawn, close, list)

**Output Format:**
```json
{
  "type": "app",
  "title": "Calculator",
  "layout": "vertical",
  "components": [
    {
      "type": "button",
      "id": "btn-7",
      "props": {"text": "7"},
      "on_event": {"click": "calc.append_digit"}
    }
  ]
}
```

---

### DynamicRenderer (`ui/src/components/DynamicRenderer.tsx`)
**Role:** Render UI specs as React components

**Responsibilities:**
- Parse UISpec JSON
- Render components dynamically
- Manage component state
- Execute tools on user interaction
- Handle app spawning messages

**Supported Components:**
- `button`: Interactive button
- `input`: Text input field
- `text`: Static text (h1, h2, p, caption)
- `container`: Layout container (vertical/horizontal)
- `grid`: Grid layout

**Flow:**
1. Fetch UISpec from `/generate-ui`
2. Create `ComponentState` for this app
3. Create `ToolExecutor` with state
4. Render components recursively
5. On user interaction → execute tool → update state → React re-renders

---

### ToolExecutor (`ui/src/components/DynamicRenderer.tsx`)
**Role:** Execute tools and update state

**Responsibilities:**
- Execute tool functions
- Update ComponentState
- Handle async operations
- Trigger app spawning via IPC

**Tool Categories:**
- `calc.*`: Local arithmetic
- `ui.*`: Local state updates
- `system.*`: Browser APIs (alert, console)
- `app.*`: Backend API calls (spawn, close, list)

**App Spawning Flow:**
```typescript
// User clicks button bound to app.spawn
executeAppTool('spawn', {request: 'create a todo list'})
   fetch('/generate-ui', {message: 'create a todo list'})
   Backend: AppManager.spawn_app()
   Backend: UIGeneratorAgent.generate_ui()
   Backend: returns {app_id, ui_spec}
   Frontend: window.postMessage({type: 'spawn_app', ui_spec})
   Parent component renders new app
```

---

### ComponentState (`ui/src/components/DynamicRenderer.tsx`)
**Role:** Per-app reactive state management

**Responsibilities:**
- Store component state (key-value pairs)
- Notify subscribers on changes
- Enable reactive updates

**Features:**
- Observable pattern
- Per-component subscriptions
- Automatic React re-rendering

```typescript
// Usage
state.set('display', '42');  // Updates state
state.get('display');         // Returns '42'
state.subscribe('display', (value) => {
  // Called when 'display' changes
  forceUpdate();
});
```

---

## App Lifecycle States

```
SPAWNING  ->  ACTIVE  ->  BACKGROUND  ->  SUSPENDED  ->  DESTROYED
   
  LLM        Running      Unfocused        Paused          Closed
  Gen
```

**States:**
1. **SPAWNING**: AI is generating UI (brief)
2. **ACTIVE**: App is running and focused
3. **BACKGROUND**: App is running but not focused
4. **SUSPENDED**: App is paused (future: for performance)
5. **DESTROYED**: App is closed and cleaned up

---

## Key Features

### Apps Can Spawn Apps
Apps can create other apps via the `app.spawn` tool:

```json
{
  "type": "button",
  "id": "create-todo-btn",
  "props": {"text": "Create Todo List"},
  "on_event": {"click": "app.spawn"}
}
```

**Use Cases:**
- App launcher app
- Settings that spawn dialogs
- Wizard flows that spawn next steps
- Productivity apps that spawn sub-apps

---

### No Re-Prompting for Normal Use

**Traditional Approach (Inefficient):**
```
User clicks button -> LLM -> Generate new UI -> Render
(Slow, expensive, unpredictable)
```

**This System's Approach:**
```
User clicks button -> ToolExecutor -> Update state -> React re-renders
(Fast, deterministic, efficient)
```

**When to Re-Prompt:**
- Initial app creation
- App evolution ("add a dark mode toggle")
- App spawning (creating new apps)

---

### Multi-App Support

The system supports multiple concurrent apps:
- Each app has its own UISpec
- Each app has its own ComponentState
- Apps can be in foreground/background
- Apps can spawn child apps
- Closing parent closes children

**Example:**
```
Calculator (active, focused)
Todo List (background)
   Add Task Dialog (active, child)
Settings (background)
```

---

## API Endpoints

### Backend (`ai-service`)

**POST `/generate-ui`**
```json
// Request
{"message": "create a calculator", "context": {"parent_app_id": "uuid"}}

// Response
{
  "app_id": "uuid-123",
  "ui_spec": {...},
  "thoughts": ["Analyzed request", "Generated Calculator", ...]
}
```

**GET `/apps`**
```json
// Response
{
  "apps": [
    {"id": "uuid-1", "title": "Calculator", "state": "active", ...}
  ],
  "stats": {"total_apps": 3, "active_apps": 1, ...}
}
```

**POST `/apps/{app_id}/focus`**
```json
// Response
{"success": true, "app_id": "uuid-1"}
```

**DELETE `/apps/{app_id}`**
```json
// Response
{"success": true, "app_id": "uuid-1"}
```

---

## Design Principles

1. **Separation of Concerns**: Generation (LLM) != Execution (Tools)
2. **Security**: No arbitrary code execution - only structured data
3. **Performance**: Generate once, execute many times
4. **Composability**: Apps can spawn apps
5. **Predictability**: Tools have defined behavior
6. **Debuggability**: All state changes are traceable
7. **Scalability**: Multi-app support from the start

---

## Example Scenarios

### Scenario 1: Simple Calculator
```
User: "create a calculator"
   AppManager.spawn_app()
   UIGeneratorAgent.generate_ui() [rule-based + LLM]
   Returns UISpec with buttons bound to calc.* tools
   DynamicRenderer renders
   User clicks "7" button
   ToolExecutor.execute("calc.append_digit", {digit: "7"})
   ComponentState.set("display", "7")
   React re-renders display
```

### Scenario 2: App Spawning Another App
```
User: "create an app launcher"
   UIGeneratorAgent generates launcher with buttons
   Each button bound to app.spawn with different requests
   User clicks "Calculator" button
   ToolExecutor.execute("app.spawn", {request: "create calculator"})
   Fetch /generate-ui
   AppManager.spawn_app() [parent_id set]
   DynamicRenderer renders new calculator
   Now 2 apps running: Launcher (background) + Calculator (active)
```

### Scenario 3: Complex Workflow
```
User: "create a project manager"
   Generates project list view
   User clicks "New Project" button
   app.spawn("create a project creation form")
   Form app spawns (child of project manager)
   User fills form, clicks "Create"
   Tool calls back to parent via IPC (future)
   Parent updates, form closes
```

---

## Summary

This architecture enables the following capabilities:
- Users can create any app with natural language
- Apps are fast and responsive (no LLM in the loop for normal interaction)
- Apps can create other apps (composability)
- System is secure (no code execution)
- System is scalable (multi-app from day 1)
- System is debuggable (structured data flow)

**Implementation Status:**
1. AppManager: Implemented
2. App spawning tools: Implemented
3. API endpoints: Implemented
4. Rule-based UI generation: Implemented
5. LLM UI generation: In progress
6. IPC integration for tools: Planned

