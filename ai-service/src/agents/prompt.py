"""
Comprehensive UI Generation Prompts
System prompts and component documentation for AI-powered UI generation.
"""

# ============================================================================
# Component Documentation
# ============================================================================

COMPONENT_DOCUMENTATION = """
=== AVAILABLE COMPONENTS ===

You can build ANY type of application using these components. Be creative and think beyond simple apps.

1. LAYOUT COMPONENTS:
   - container: Flexbox layout (vertical/horizontal)
     {"type": "container", "id": "main", "props": {"layout": "vertical", "gap": 12, "padding": "large", "align": "center", "justify": "start"}, "children": [...]}
   
   - grid: Grid layout with responsive columns
     {"type": "grid", "id": "grid1", "props": {"columns": 3, "gap": 16, "responsive": true}, "children": [...]}
   
   - card: Card container with header/body/footer
     {"type": "card", "id": "card1", "props": {"title": "Card Title", "footer": "Footer text", "style": {...}}, "children": [...]}
   
   - list: Styled list with variants (default, bordered, striped)
     {"type": "list", "id": "list1", "props": {"variant": "bordered", "spacing": "medium"}, "children": [...]}
   
   - tabs: Tabbed interface
     {"type": "tabs", "id": "tabs1", "props": {"variant": "default", "defaultTab": "tab1"}, "children": [
       {"type": "container", "id": "tab1", "props": {"label": "Tab 1"}, "children": [...]},
       {"type": "container", "id": "tab2", "props": {"label": "Tab 2"}, "children": [...]}
     ]}
   
   - modal: Popup dialog
     {"type": "modal", "id": "modal1", "props": {"open": false, "title": "Modal Title", "size": "medium"}, "children": [...]}

2. INPUT COMPONENTS:
   - button: Clickable button with variants
     {"type": "button", "id": "btn1", "props": {"text": "Click Me", "variant": "primary", "size": "medium"}, "on_event": {"click": "tool.name"}}
   
   - input: Text input field
     {"type": "input", "id": "input1", "props": {"placeholder": "Enter text...", "value": "", "type": "text", "variant": "default"}, "on_event": {"change": "tool.name"}}
   
   - textarea: Multi-line text input
     {"type": "textarea", "id": "textarea1", "props": {"placeholder": "Enter text...", "rows": 5, "resize": "vertical"}, "on_event": {"change": "tool.name"}}
   
   - select: Dropdown selection
     {"type": "select", "id": "select1", "props": {"value": "option1", "options": [{"value": "option1", "label": "Option 1"}, {"value": "option2", "label": "Option 2"}]}, "on_event": {"change": "tool.name"}}
   
   - checkbox: Checkbox with label
     {"type": "checkbox", "id": "check1", "props": {"label": "Enable feature", "checked": false, "variant": "primary"}, "on_event": {"change": "tool.name"}}
   
   - radio: Radio button
     {"type": "radio", "id": "radio1", "props": {"name": "group1", "value": "option1", "label": "Option 1"}, "on_event": {"change": "tool.name"}}
   
   - slider: Range slider
     {"type": "slider", "id": "slider1", "props": {"min": 0, "max": 100, "step": 1, "value": 50}, "on_event": {"change": "tool.name"}}

3. DISPLAY COMPONENTS:
   - text: Text/heading with typography variants
     {"type": "text", "id": "text1", "props": {"content": "Hello World", "variant": "h1", "weight": "bold", "color": "primary", "align": "center"}}
     Variants: h1, h2, h3, body, caption, label
   
   - image: Display images
     {"type": "image", "id": "img1", "props": {"src": "https://example.com/image.jpg", "alt": "Description", "fit": "cover", "rounded": "medium", "width": 300, "height": 200}, "on_event": {"click": "tool.name"}}
   
   - video: Video player
     {"type": "video", "id": "video1", "props": {"src": "https://example.com/video.mp4", "controls": true, "autoPlay": false, "loop": false, "width": 640, "height": 360}}
   
   - audio: Audio player
     {"type": "audio", "id": "audio1", "props": {"src": "https://example.com/audio.mp3", "controls": true, "variant": "default"}}
   
   - progress: Progress bar
     {"type": "progress", "id": "progress1", "props": {"value": 50, "max": 100, "variant": "primary", "size": "medium"}}
   
   - badge: Small status badge
     {"type": "badge", "id": "badge1", "props": {"text": "New", "variant": "success", "size": "small"}}
   
   - divider: Visual separator
     {"type": "divider", "id": "div1", "props": {"orientation": "horizontal", "variant": "solid"}}

4. ADVANCED COMPONENTS:
   - canvas: HTML5 canvas for drawing/games
     {"type": "canvas", "id": "canvas1", "props": {"width": 800, "height": 600, "bordered": true}, "on_event": {"mount": "canvas.init"}}
   
   - iframe: Embed external content (websites, videos)
     {"type": "iframe", "id": "iframe1", "props": {"src": "https://example.com", "title": "External Content", "width": "100%", "height": 600, "sandbox": "allow-scripts allow-same-origin"}}

=== STYLE PROPERTIES ===

All components support a "style" prop for custom CSS:
{"style": {"backgroundColor": "#fff", "padding": "20px", "borderRadius": "8px", "boxShadow": "0 2px 8px rgba(0,0,0,0.1)"}}

Common style properties:
- Layout: width, height, margin, padding, display, position, top, left, right, bottom
- Colors: backgroundColor, color, borderColor, opacity
- Typography: fontSize, fontWeight, textAlign, lineHeight
- Borders: border, borderRadius, borderWidth, borderStyle
- Effects: boxShadow, transform, transition, animation
"""

# ============================================================================
# App Examples
# ============================================================================

APP_EXAMPLES = """
=== EXAMPLE APPLICATIONS ===

1. BROWSER APP:
{
  "type": "app",
  "title": "Web Browser",
  "layout": "vertical",
  "components": [
    {"type": "container", "id": "toolbar", "props": {"layout": "horizontal", "gap": 8, "padding": "small"}, "children": [
      {"type": "button", "id": "back", "props": {"text": "←", "variant": "ghost"}, "on_event": {"click": "browser.back"}},
      {"type": "button", "id": "forward", "props": {"text": "→", "variant": "ghost"}, "on_event": {"click": "browser.forward"}},
      {"type": "button", "id": "refresh", "props": {"text": "⟳", "variant": "ghost"}, "on_event": {"click": "browser.refresh"}},
      {"type": "input", "id": "url", "props": {"placeholder": "Enter URL...", "value": "https://example.com", "style": {"flex": "1"}}, "on_event": {"change": "browser.navigate"}},
      {"type": "button", "id": "go", "props": {"text": "Go", "variant": "primary"}, "on_event": {"click": "browser.navigate"}}
    ]},
    {"type": "iframe", "id": "webpage", "props": {"src": "https://example.com", "width": "100%", "height": 600, "sandbox": "allow-scripts allow-same-origin"}}
  ]
}

2. CALCULATOR APP:
{
  "type": "app",
  "title": "Calculator",
  "layout": "vertical",
  "components": [
    {"type": "input", "id": "display", "props": {"value": "0", "readonly": true, "variant": "large"}},
    {"type": "grid", "id": "buttons", "props": {"columns": 4, "gap": 8}, "children": [
      {"type": "button", "id": "btn-7", "props": {"text": "7", "variant": "outline"}, "on_event": {"click": "ui.append"}},
      {"type": "button", "id": "btn-8", "props": {"text": "8", "variant": "outline"}, "on_event": {"click": "ui.append"}},
      {"type": "button", "id": "btn-9", "props": {"text": "9", "variant": "outline"}, "on_event": {"click": "ui.append"}},
      {"type": "button", "id": "btn-div", "props": {"text": "/", "variant": "secondary"}, "on_event": {"click": "ui.append"}},
      {"type": "button", "id": "btn-4", "props": {"text": "4", "variant": "outline"}, "on_event": {"click": "ui.append"}},
      {"type": "button", "id": "btn-5", "props": {"text": "5", "variant": "outline"}, "on_event": {"click": "ui.append"}},
      {"type": "button", "id": "btn-6", "props": {"text": "6", "variant": "outline"}, "on_event": {"click": "ui.append"}},
      {"type": "button", "id": "btn-mul", "props": {"text": "*", "variant": "secondary"}, "on_event": {"click": "ui.append"}},
      {"type": "button", "id": "btn-1", "props": {"text": "1", "variant": "outline"}, "on_event": {"click": "ui.append"}},
      {"type": "button", "id": "btn-2", "props": {"text": "2", "variant": "outline"}, "on_event": {"click": "ui.append"}},
      {"type": "button", "id": "btn-3", "props": {"text": "3", "variant": "outline"}, "on_event": {"click": "ui.append"}},
      {"type": "button", "id": "btn-sub", "props": {"text": "-", "variant": "secondary"}, "on_event": {"click": "ui.append"}},
      {"type": "button", "id": "btn-0", "props": {"text": "0", "variant": "outline"}, "on_event": {"click": "ui.append"}},
      {"type": "button", "id": "btn-clear", "props": {"text": "C", "variant": "danger"}, "on_event": {"click": "ui.clear"}},
      {"type": "button", "id": "btn-eq", "props": {"text": "=", "variant": "primary"}, "on_event": {"click": "ui.compute"}},
      {"type": "button", "id": "btn-add", "props": {"text": "+", "variant": "secondary"}, "on_event": {"click": "ui.append"}}
    ]}
  ]
}

3. BROWSER APP:
{
  "type": "app",
  "title": "Web Browser",
  "layout": "vertical",
  "components": [
    {"type": "container", "id": "toolbar", "props": {"layout": "horizontal", "gap": 8, "padding": "small"}, "children": [
      {"type": "button", "id": "back", "props": {"text": "←", "variant": "ghost"}, "on_event": {"click": "browser.back"}},
      {"type": "button", "id": "forward", "props": {"text": "→", "variant": "ghost"}, "on_event": {"click": "browser.forward"}},
      {"type": "button", "id": "refresh", "props": {"text": "⟳", "variant": "ghost"}, "on_event": {"click": "browser.refresh"}},
      {"type": "input", "id": "url-input", "props": {"placeholder": "Enter URL...", "value": "https://www.google.com"}, "on_event": {"change": "ui.set"}},
      {"type": "button", "id": "go", "props": {"text": "Go", "variant": "primary"}, "on_event": {"click": "browser.navigate"}},
      {"type": "button", "id": "bookmark", "props": {"text": "⭐", "variant": "ghost"}, "on_event": {"click": "browser.bookmark.add"}}
    ]},
    {"type": "iframe", "id": "webpage", "props": {"src": "https://www.google.com", "width": "100%", "height": 600, "sandbox": "allow-scripts allow-same-origin"}}
  ]
}

4. DASHBOARD:
{
  "type": "app",
  "title": "Analytics Dashboard",
  "layout": "vertical",
  "components": [
    {"type": "text", "id": "header", "props": {"content": "Analytics Dashboard", "variant": "h1"}},
    {"type": "grid", "id": "metrics", "props": {"columns": 3, "gap": 16}, "children": [
      {"type": "card", "id": "card1", "props": {"title": "Users"}, "children": [
        {"type": "text", "id": "users", "props": {"content": "10,234", "variant": "h2", "color": "primary"}},
        {"type": "text", "id": "users-change", "props": {"content": "+12% this month", "variant": "caption", "color": "success"}}
      ]},
      {"type": "card", "id": "card2", "props": {"title": "Revenue"}, "children": [
        {"type": "text", "id": "revenue", "props": {"content": "$45,231", "variant": "h2", "color": "primary"}},
        {"type": "text", "id": "revenue-change", "props": {"content": "+8% this month", "variant": "caption", "color": "success"}}
      ]},
      {"type": "card", "id": "card3", "props": {"title": "Orders"}, "children": [
        {"type": "text", "id": "orders", "props": {"content": "1,284", "variant": "h2", "color": "primary"}},
        {"type": "text", "id": "orders-change", "props": {"content": "-3% this month", "variant": "caption", "color": "error"}}
      ]}
    ]},
    {"type": "card", "id": "activity", "props": {"title": "Recent Activity"}, "children": [
      {"type": "list", "id": "activity-list", "props": {"variant": "striped"}, "children": [
        {"type": "text", "id": "act1", "props": {"content": "New user registered", "variant": "body"}},
        {"type": "text", "id": "act2", "props": {"content": "Order #1234 completed", "variant": "body"}},
        {"type": "text", "id": "act3", "props": {"content": "Payment received", "variant": "body"}}
      ]}
    ]}
  ]
}

5. FORM BUILDER:
{
  "type": "app",
  "title": "User Registration",
  "layout": "vertical",
  "components": [
    {"type": "text", "id": "title", "props": {"content": "Create Account", "variant": "h1"}},
    {"type": "container", "id": "form", "props": {"layout": "vertical", "gap": 16, "padding": "large", "style": {"maxWidth": "500px"}}, "children": [
      {"type": "input", "id": "name", "props": {"placeholder": "Full Name", "type": "text"}, "on_event": {"change": "form.validate"}},
      {"type": "input", "id": "email", "props": {"placeholder": "Email", "type": "email"}, "on_event": {"change": "form.validate"}},
      {"type": "input", "id": "password", "props": {"placeholder": "Password", "type": "password"}, "on_event": {"change": "form.validate"}},
      {"type": "textarea", "id": "bio", "props": {"placeholder": "Tell us about yourself...", "rows": 4}, "on_event": {"change": "form.validate"}},
      {"type": "checkbox", "id": "terms", "props": {"label": "I agree to the terms and conditions", "checked": false}, "on_event": {"change": "form.validate"}},
      {"type": "button", "id": "submit", "props": {"text": "Create Account", "variant": "primary", "fullWidth": true}, "on_event": {"click": "form.submit"}}
    ]}
  ]
}

6. TODO APP WITH BACKEND STORAGE:
{
  "type": "app",
  "title": "Todo List",
  "layout": "vertical",
  "services": ["storage"],
  "lifecycle_hooks": {"on_mount": ["storage.get"]},
  "components": [
    {"type": "text", "id": "header", "props": {"content": "My Todos", "variant": "h1"}},
    {"type": "container", "id": "add-row", "props": {"layout": "horizontal", "gap": 8}, "children": [
      {"type": "input", "id": "task-input", "props": {"placeholder": "What needs to be done?", "type": "text"}},
      {"type": "button", "id": "add-btn", "props": {"text": "Add", "variant": "primary"}, "on_event": {"click": "ui.list.add"}}
    ]},
    {"type": "list", "id": "todos", "props": {"variant": "striped"}, "children": []}
  ]
}
"""

# ============================================================================
# Main System Prompt
# ============================================================================

def get_ui_generation_prompt(tools_description: str, context: str = "") -> str:
    """
    Generate comprehensive UI generation prompt for Gemini.
    
    Args:
        tools_description: Description of available tools
        context: Additional context (optional)
    
    Returns:
        Complete system prompt for UI generation
    """
    return f"""You are an expert UI generation AI. Your task is to generate complete, valid JSON specifications for ANY type of application.

CRITICAL RULES:
1. Output ONLY valid JSON - no markdown, no explanations, no extra text
2. EVERY component MUST have: "id", "type", "props", "children", "on_event"
3. Use generic UI tools (ui.*, browser.*) for most functionality
4. Use appropriate components: iframe for browsers, grid for dashboards, container for forms
5. Wire up events properly with on_event handlers
6. Use proper layouts (grid for dashboards, container for forms, tabs for multi-section apps)

{COMPONENT_DOCUMENTATION}

Available Tools:
{tools_description}

{APP_EXAMPLES}

{context}

=== REQUIRED ROOT STRUCTURE ===
{{
  "type": "app",
  "title": "App Name",
  "layout": "vertical",
  "components": [/* array of components */],
  "style": {{}},
  "services": [],
  "service_bindings": {{}},
  "lifecycle_hooks": {{}}
}}

=== GENERIC UI TOOLS (Use These for ALL Apps) ===

Available tools work for ANY app type - no need for app-specific tools!

**ui.append** - Append value to state (calculator digits, text input)
  Button: {{"on_event": {{"click": "ui.append"}}, "props": {{"text": "7"}}}}
  → Automatically appends button's text to "display" state

**ui.compute** - Evaluate expression (calculator =, formula fields)
  Button: {{"on_event": {{"click": "ui.compute"}}}}
  → Evaluates expression in "display" state

**ui.clear** - Clear value (calculator C, form reset)  
  Button: {{"on_event": {{"click": "ui.clear"}}}}
  → Resets "display" to "0"

**ui.set** - Set any state (navigation, URL loading, toggles)
  Button: {{"on_event": {{"click": "ui.set"}}}}
  → Reads from input fields automatically or use params

**ui.toggle** - Toggle boolean (checkboxes, switches)
  Button: {{"on_event": {{"click": "ui.toggle"}}}}

**ui.backspace** - Remove last char (backspace buttons)
  Button: {{"on_event": {{"click": "ui.backspace"}}}}

=== REAL EXAMPLES ===

**Calculator:**
{{
  "components": [
    {{"type": "input", "id": "display", "props": {{"readonly": true, "value": "0"}}}},
    {{"type": "button", "id": "btn-7", "props": {{"text": "7"}}, "on_event": {{"click": "ui.append"}}}},
    {{"type": "button", "id": "btn-plus", "props": {{"text": "+"}}, "on_event": {{"click": "ui.append"}}}},
    {{"type": "button", "id": "btn-equals", "props": {{"text": "="}}, "on_event": {{"click": "ui.compute"}}}},
    {{"type": "button", "id": "btn-clear", "props": {{"text": "C"}}, "on_event": {{"click": "ui.clear"}}}}
  ]
}}

**Browser:**
{{
  "components": [
    {{"type": "input", "id": "url-input", "props": {{"placeholder": "Enter URL"}}}},
    {{"type": "button", "id": "go-btn", "props": {{"text": "Go"}}, "on_event": {{"click": "browser.navigate"}}}},
    {{"type": "iframe", "id": "webpage", "props": {{"src": ""}}}}
  ]
}}

**Todo List:**
{{
  "components": [
    {{"type": "input", "id": "task-input"}},
    {{"type": "button", "props": {{"text": "Add"}}, "on_event": {{"click": "ui.add_todo"}}}}
  ]
}}

=== APP TYPE PATTERNS ===

1. **Calculator/Keypad:** Use ui.append for all buttons, ui.compute for =
2. **Browser:** Use input for URL + browser.navigate button  
3. **Forms:** Use inputs with validation + ui.set for state management
4. **Media Players:** Use player.* tools for playback controls
5. **Canvas Apps:** Use canvas.* tools for drawing
6. **Data Apps:** Use ui.set/get for state, data.* for filtering/sorting

Think BIG - you can make ANY app! The tools are generic and composable.

Now generate a complete, valid JSON UI specification for the user's request. Output ONLY the JSON."""


def get_simple_system_prompt() -> str:
    """Get a simpler system prompt for rule-based generation fallback."""
    return """You are a UI generator. Output valid JSON only."""

