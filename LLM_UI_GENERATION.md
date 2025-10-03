# LLM-Based UI Generation Architecture

## Overview
Replaced rule-based UI generation with intelligent LLM-powered generation while maintaining backward compatibility.

## What Was Implemented

### 1. Function/Tool Definitions for UI Components
**File:** `ai-service/src/agents/ui_generator.py` (lines 196-319)

Created LangChain `@tool` decorated functions that define the UI component schema:
- `create_button()` - Interactive buttons with click handlers
- `create_input()` - Text input fields
- `create_text()` - Text/label components
- `create_container()` - Layout containers (vertical/horizontal)
- `create_grid()` - Grid layouts

Each tool provides:
- Clear docstrings for the LLM
- Type-annotated parameters
- Returns properly structured component dictionaries

### 2. Enhanced System Prompt
**File:** `ai-service/src/agents/ui_generator.py` (lines 408-510)

Designed a comprehensive system prompt that includes:
- **Critical schema rules** - Exact requirements (id, children, props, on_event)
- **Component schemas** - Precise JSON structure for each component type
- **Complete examples** - Full calculator example showing proper structure
- **Tool descriptions** - Available tools for interactivity (calc.*, ui.*, system.*, app.*)
- **Step-by-step instructions** - Clear guidance on how to generate UIs

### 3. LLM-Based Generation Method
**File:** `ai-service/src/agents/ui_generator.py` (lines 567-620)

`_generate_with_llm()` method:
1. Constructs system + user prompts
2. Calls LLM via LangChain
3. Extracts JSON from response (handles markdown, extra text)
4. Validates against UISpec schema
5. Returns structured UISpec object

### 4. Robust JSON Extraction
**File:** `ai-service/src/agents/ui_generator.py` (lines 622-664)

`_extract_json()` method handles:
- Markdown code blocks (```json ... ```)
- Extra text before/after JSON
- Whitespace and formatting variations
- Detailed error logging for debugging

### 5. Hybrid Generation Strategy
**File:** `ai-service/src/agents/ui_generator.py` (lines 535-565)

`generate_ui()` method implements:
```
if LLM available:
    try:
        return LLM-generated UI
    except error:
        log warning, fall back to rules
else:
    use rule-based generation
```

Benefits:
- ✅ Smart generation for complex/custom UIs
- ✅ Reliable fallback for edge cases
- ✅ Zero breaking changes to existing code

### 6. Lazy LLM Loading
**Files:** `ai-service/src/main.py` (lines 235-248, 389-398)

Updated both WebSocket and HTTP endpoints to:
1. Check if LLM is loaded
2. Load LLM on first UI generation request
3. Attach LLM to UIGeneratorAgent
4. Enable LLM-based generation
5. Log all steps for debugging

Benefits:
- No startup delay
- Model loads only when needed
- Graceful degradation if model unavailable

## Architecture Decisions

### Why Not Use LangChain's bind_tools()?
We considered using `.bind_tools(UI_COMPONENT_TOOLS)` but chose a prompt-based approach because:
1. **Better control** - Explicit schema in prompt vs relying on tool parsing
2. **More flexible** - Can provide detailed examples and rules
3. **Easier debugging** - Clear prompts vs opaque tool binding
4. **Better results** - Direct JSON generation vs function calling round-trips

### Why Hybrid Approach?
The hybrid LLM + rule-based strategy provides:
1. **Reliability** - Always works, even if LLM fails
2. **Performance** - Fast rule-based for simple cases
3. **Quality** - LLM for complex/creative requests
4. **Gradual rollout** - Can test LLM without breaking production

## Configuration

### Model Setup
**File:** `ai-service/src/main.py` (lines 41-48)

```python
config = ModelConfig(
    backend=ModelBackend.OLLAMA,
    size=ModelSize.SMALL,  # gpt-oss:20b
    context_length=8192,
    max_tokens=2048,
    temperature=0.7,
    streaming=True,
)
```

### Tool Registry
**File:** `ai-service/src/agents/ui_generator.py` (lines 64-157)

Available tools for binding to UI events:
- **Compute:** calc.add, calc.subtract, calc.multiply, calc.divide
- **UI:** ui.set_state, ui.get_state
- **System:** system.alert, system.log
- **App:** app.spawn, app.close, app.list

## Testing

### Test Script
**File:** `ai-service/test_ui_generation.py`

Comprehensive test suite:
1. **Test 1:** Rule-based generation (3 cases)
2. **Test 2:** LLM-based generation (3 cases)

Run with:
```bash
cd ai-service
source venv/bin/activate
python3 test_ui_generation.py
```

### Quick Test
**File:** `ai-service/test_llm_quick.py`

Fast single-request test for iteration:
```bash
cd ai-service
source venv/bin/activate
python3 test_llm_quick.py
```

## Example Requests

### Simple Calculator
```
Request: "create a calculator"
Generated: Calculator UI with display + 4x4 button grid
```

### Todo App
```
Request: "build a todo list app"
Generated: Todo UI with input, add button, task list
```

### Custom Widget
```
Request: "make a weather widget showing temperature and conditions"
Generated: Custom weather UI with appropriate components
```

## Schema Reference

### UISpec Structure
```json
{
  "type": "app",
  "title": "App Name",
  "layout": "vertical",
  "components": [
    {
      "type": "button|input|text|container|grid",
      "id": "unique-id",
      "props": { /* component-specific props */ },
      "children": [ /* nested components */ ],
      "on_event": { "click": "tool.name" }
    }
  ],
  "style": { /* optional global styles */ }
}
```

### Component Types

**Button:**
```json
{
  "type": "button",
  "id": "btn-id",
  "props": {"text": "Click Me", "variant": "default", "size": "medium"},
  "on_event": {"click": "calc.add"}
}
```

**Input:**
```json
{
  "type": "input",
  "id": "input-id",
  "props": {"placeholder": "Enter text", "value": "", "type": "text"},
  "on_event": null
}
```

**Text:**
```json
{
  "type": "text",
  "id": "text-id",
  "props": {"content": "Hello", "variant": "body"},
  "on_event": null
}
```

**Container:**
```json
{
  "type": "container",
  "id": "container-id",
  "props": {"layout": "vertical", "gap": 8},
  "children": [ /* nested components */ ]
}
```

**Grid:**
```json
{
  "type": "grid",
  "id": "grid-id",
  "props": {"columns": 3, "gap": 8},
  "children": [ /* grid items */ ]
}
```

## Integration Points

### Backend
1. **UIGeneratorAgent** - Generates UI specs from natural language
2. **AppManager** - Manages app lifecycle, spawning, focus
3. **API Endpoints** - `/generate-ui` (HTTP), WebSocket `generate_ui` type

### Frontend
1. **DynamicRenderer** - Renders UISpec → React components
2. **ToolExecutor** - Executes tool calls on user interaction
3. **ComponentState** - Manages per-app state

## Future Enhancements

### Phase 2: True Function Calling
- Bind UI component tools to LLM
- Let LLM call functions directly
- Build UI from function call results

### Phase 3: Streaming Thoughts
- Stream generation progress to UI
- Show "thinking" steps to user
- Real-time component preview

### Phase 4: Iterative Refinement
- "Make the buttons bigger"
- "Change color scheme to dark"
- Edit existing UIs based on feedback

## Known Limitations

1. **LLM may generate invalid JSON** - Falls back to rules
2. **Schema validation strict** - Missing `id` fails validation
3. **No UI editing yet** - Only generates new UIs
4. **Single language** - No i18n support yet

## Debugging

### Enable Debug Logging
```python
logging.basicConfig(level=logging.DEBUG)
```

### Check LLM Response
Look for these log lines:
```
agents.ui_generator - DEBUG - LLM response: {...
agents.ui_generator - ERROR - Failed to validate UISpec: ...
```

### Common Issues

**Empty JSON:**
- LLM returned no JSON object
- Check Ollama is running: `curl http://localhost:11434/api/tags`

**Validation Error:**
- Missing required fields (id, props, etc.)
- Check error logs for exact field
- Improve system prompt with more examples

**Fallback to Rules:**
- LLM generation failed
- Check logs for specific error
- Rule-based generation used as backup

## Success Metrics

✅ LLM integration complete
✅ Hybrid approach implemented  
✅ Zero breaking changes
✅ Comprehensive testing
✅ Graceful fallback
✅ Clear documentation

## Summary

We successfully replaced rule-based UI generation with an intelligent LLM-powered system that:
- Generates custom UIs from natural language
- Falls back to rules for reliability
- Maintains full backward compatibility
- Provides clear debugging and error handling
- Sets foundation for future enhancements

The system is production-ready with a solid architecture for continued iteration!

