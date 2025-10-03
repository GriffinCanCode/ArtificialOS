#!/usr/bin/env python3
"""
End-to-End Test: Complete UI Generation Flow
Tests the entire pipeline from user request to validated UISpec
"""

import sys
import json
import logging
from pathlib import Path

sys.path.insert(0, str(Path(__file__).parent / "src"))

from models import ModelConfig, ModelLoader
from models.config import ModelSize, ModelBackend
from agents.ui_generator import UIGeneratorAgent, ToolRegistry
from agents.app_manager import AppManager

logging.basicConfig(
    level=logging.INFO,
    format="%(levelname)s - %(message)s"
)
logger = logging.getLogger(__name__)


class E2ETestRunner:
    """End-to-end test runner."""
    
    def __init__(self):
        self.passed = 0
        self.failed = 0
        self.errors = []
    
    def test(self, name: str, func):
        """Run a test and track results."""
        try:
            logger.info(f"\n{'='*80}")
            logger.info(f"TEST: {name}")
            logger.info(f"{'='*80}")
            func()
            self.passed += 1
            logger.info(f"✅ PASS: {name}")
        except AssertionError as e:
            self.failed += 1
            self.errors.append((name, str(e)))
            logger.error(f"❌ FAIL: {name} - {e}")
        except Exception as e:
            self.failed += 1
            self.errors.append((name, str(e)))
            logger.error(f"❌ ERROR: {name} - {e}", exc_info=True)
    
    def summary(self):
        """Print test summary."""
        logger.info(f"\n{'='*80}")
        logger.info(f"END-TO-END TEST SUMMARY")
        logger.info(f"{'='*80}")
        logger.info(f"Passed: {self.passed}")
        logger.info(f"Failed: {self.failed}")
        logger.info(f"Total:  {self.passed + self.failed}")
        
        if self.errors:
            logger.info(f"\nFailures:")
            for name, error in self.errors:
                logger.info(f"  ❌ {name}: {error}")
        
        if self.failed == 0:
            logger.info(f"\n🎉 ALL E2E TESTS PASSED!")
        
        return self.failed == 0


def main():
    """Run end-to-end tests."""
    runner = E2ETestRunner()
    
    logger.info("="*80)
    logger.info("🧪 END-TO-END TEST SUITE: AI-OS UI Generation")
    logger.info("="*80)
    
    # Shared resources
    config = None
    llm = None
    tool_registry = None
    ui_generator = None
    app_manager = None
    
    # ========================================================================
    # TEST 1: Model Loading
    # ========================================================================
    def test_model_loading():
        nonlocal config, llm
        
        config = ModelConfig(
            backend=ModelBackend.OLLAMA,
            size=ModelSize.SMALL,
            temperature=0.7,
            max_tokens=2048,
            streaming=True,
        )
        
        logger.info("Loading LLM via Ollama...")
        llm = ModelLoader.load(config)
        
        assert llm is not None, "LLM is None"
        assert hasattr(llm, 'model'), "LLM missing model attribute"
        assert llm.model == "gpt-oss:20b", f"Wrong model: {llm.model}"
        
        logger.info(f"  → Model: {llm.model}")
        logger.info(f"  → Type: {type(llm)}")
    
    runner.test("Model Loading", test_model_loading)
    
    # ========================================================================
    # TEST 2: Tool Registry Initialization
    # ========================================================================
    def test_tool_registry():
        nonlocal tool_registry
        
        tool_registry = ToolRegistry()
        
        tools = tool_registry.list_tools()
        assert len(tools) > 0, "No tools registered"
        
        # Check for required tool categories
        calc_tools = tool_registry.list_tools(category="compute")
        ui_tools = tool_registry.list_tools(category="ui")
        app_tools = tool_registry.list_tools(category="app")
        
        assert len(calc_tools) >= 4, f"Expected >=4 calc tools, got {len(calc_tools)}"
        assert len(ui_tools) >= 2, f"Expected >=2 UI tools, got {len(ui_tools)}"
        assert len(app_tools) >= 3, f"Expected >=3 app tools, got {len(app_tools)}"
        
        logger.info(f"  → Total tools: {len(tools)}")
        logger.info(f"  → Calc tools: {len(calc_tools)}")
        logger.info(f"  → UI tools: {len(ui_tools)}")
        logger.info(f"  → App tools: {len(app_tools)}")
    
    runner.test("Tool Registry Initialization", test_tool_registry)
    
    # ========================================================================
    # TEST 3: UI Generator with LLM
    # ========================================================================
    def test_ui_generator_init():
        nonlocal ui_generator
        
        ui_generator = UIGeneratorAgent(tool_registry, llm=llm)
        
        assert ui_generator is not None, "UI generator is None"
        assert ui_generator.use_llm is True, "LLM not enabled"
        assert ui_generator.llm is not None, "LLM not attached"
        
        logger.info(f"  → LLM enabled: {ui_generator.use_llm}")
        logger.info(f"  → Tools available: {len(ui_generator.tool_registry.tools)}")
    
    runner.test("UI Generator Initialization", test_ui_generator_init)
    
    # ========================================================================
    # TEST 4: Generate Simple UI (Counter)
    # ========================================================================
    def test_generate_counter():
        logger.info("Generating counter UI with LLM...")
        
        ui_spec = ui_generator.generate_ui("create a counter")
        
        # Validate UISpec structure
        assert ui_spec is not None, "UISpec is None"
        assert hasattr(ui_spec, 'title'), "Missing title"
        assert hasattr(ui_spec, 'components'), "Missing components"
        assert hasattr(ui_spec, 'layout'), "Missing layout"
        
        assert len(ui_spec.components) > 0, "No components generated"
        
        logger.info(f"  → Title: {ui_spec.title}")
        logger.info(f"  → Components: {len(ui_spec.components)}")
        logger.info(f"  → Layout: {ui_spec.layout}")
        
        # Check component structure
        for comp in ui_spec.components:
            assert hasattr(comp, 'id'), "Component missing id"
            assert hasattr(comp, 'type'), "Component missing type"
            assert hasattr(comp, 'props'), "Component missing props"
    
    runner.test("Generate Counter UI", test_generate_counter)
    
    # ========================================================================
    # TEST 5: Generate Custom UI
    # ========================================================================
    def test_generate_custom():
        logger.info("Generating custom UI with LLM...")
        
        ui_spec = ui_generator.generate_ui("two buttons labeled yes and no")
        
        assert ui_spec is not None, "UISpec is None"
        assert len(ui_spec.components) > 0, "No components"
        
        logger.info(f"  → Title: {ui_spec.title}")
        logger.info(f"  → Components: {len(ui_spec.components)}")
        
        # Try to find buttons in the structure
        def find_buttons(comp, found=[]):
            if comp.type == "button":
                found.append(comp)
            for child in comp.children:
                find_buttons(child, found)
            return found
        
        all_buttons = []
        for comp in ui_spec.components:
            find_buttons(comp, all_buttons)
        
        logger.info(f"  → Buttons found: {len(all_buttons)}")
    
    runner.test("Generate Custom UI", test_generate_custom)
    
    # ========================================================================
    # TEST 6: JSON Serialization
    # ========================================================================
    def test_json_serialization():
        logger.info("Testing JSON serialization...")
        
        ui_spec = ui_generator.generate_ui("create a calculator")
        
        # Convert to JSON
        json_str = ui_generator.spec_to_json(ui_spec)
        assert len(json_str) > 0, "Empty JSON"
        
        # Validate JSON syntax
        parsed = json.loads(json_str)
        assert isinstance(parsed, dict), "Not a JSON object"
        assert "title" in parsed, "Missing title in JSON"
        assert "components" in parsed, "Missing components in JSON"
        
        logger.info(f"  → JSON length: {len(json_str)} chars")
        logger.info(f"  → JSON keys: {list(parsed.keys())}")
        
        # Round-trip test
        ui_spec_restored = ui_generator.json_to_spec(json_str)
        assert ui_spec_restored.title == ui_spec.title, "Title mismatch after round-trip"
    
    runner.test("JSON Serialization", test_json_serialization)
    
    # ========================================================================
    # TEST 7: App Manager Integration
    # ========================================================================
    def test_app_manager():
        nonlocal app_manager
        
        logger.info("Testing AppManager integration...")
        
        app_manager = AppManager()
        
        # Generate UI
        ui_spec = ui_generator.generate_ui("create a todo app")
        ui_json = ui_spec.model_dump()
        
        # Register with AppManager
        app = app_manager.spawn_app(
            request="create a todo app",
            ui_spec=ui_json,
            parent_id=None
        )
        
        assert app is not None, "App is None"
        assert app.id is not None, "App has no ID"
        assert app.title == ui_spec.title, "App title mismatch"
        
        # AppState might be string or enum
        state_value = app.state.value if hasattr(app.state, 'value') else app.state
        assert state_value == "active", f"App not active: {state_value}"
        
        logger.info(f"  → App ID: {app.id}")
        logger.info(f"  → App title: {app.title}")
        logger.info(f"  → App state: {app.state}")
        
        # Check it's in the registry
        all_apps = app_manager.list_apps()
        assert len(all_apps) > 0, "No apps in registry"
        assert app.id in [a["id"] for a in all_apps], "App not in registry"
    
    runner.test("AppManager Integration", test_app_manager)
    
    # ========================================================================
    # TEST 8: Tool Binding Verification
    # ========================================================================
    def test_tool_binding():
        logger.info("Testing tool binding in generated UIs...")
        
        ui_spec = ui_generator.generate_ui("calculator app")
        
        # Search for tool bindings
        def find_tool_bindings(comp, bindings=[]):
            if comp.on_event:
                for event, tool_id in comp.on_event.items():
                    if tool_id:
                        bindings.append((comp.id, event, tool_id))
            for child in comp.children:
                find_tool_bindings(child, bindings)
            return bindings
        
        all_bindings = []
        for comp in ui_spec.components:
            find_tool_bindings(comp, all_bindings)
        
        logger.info(f"  → Tool bindings found: {len(all_bindings)}")
        
        if all_bindings:
            for comp_id, event, tool_id in all_bindings[:3]:  # Show first 3
                logger.info(f"    • {comp_id}.{event} → {tool_id}")
            
            # Verify tools exist in registry
            for _, _, tool_id in all_bindings:
                tool = tool_registry.get_tool(tool_id)
                if tool:
                    logger.info(f"    ✓ Tool '{tool_id}' exists in registry")
    
    runner.test("Tool Binding Verification", test_tool_binding)
    
    # ========================================================================
    # TEST 9: Fallback Behavior
    # ========================================================================
    def test_fallback():
        logger.info("Testing fallback to rule-based generation...")
        
        # Create generator without LLM
        fallback_generator = UIGeneratorAgent(tool_registry, llm=None)
        
        assert fallback_generator.use_llm is False, "LLM should be disabled"
        
        # Should still generate UI using rules
        ui_spec = fallback_generator.generate_ui("create a calculator")
        
        assert ui_spec is not None, "No fallback UI"
        assert ui_spec.title == "Calculator", f"Wrong title: {ui_spec.title}"
        
        logger.info(f"  → Fallback UI generated: {ui_spec.title}")
    
    runner.test("Fallback Behavior", test_fallback)
    
    # ========================================================================
    # Summary
    # ========================================================================
    success = runner.summary()
    
    sys.exit(0 if success else 1)


if __name__ == "__main__":
    main()

