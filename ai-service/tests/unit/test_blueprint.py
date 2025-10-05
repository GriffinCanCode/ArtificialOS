"""Tests for Blueprint parser."""

import pytest
from src.blueprint.parser import BlueprintParser, parse_blueprint
from src.core import ValidationError


@pytest.mark.unit
def test_blueprint_parser_basic(sample_blueprint):
    """Test basic blueprint parsing."""
    parser = BlueprintParser()
    result = parser.parse(sample_blueprint)
    
    assert result["id"] == "test-app"
    assert result["name"] == "Test App"
    assert result["category"] == "utilities"
    assert "storage" in str(result["services"])


@pytest.mark.unit
def test_blueprint_parser_services(sample_blueprint):
    """Test service expansion."""
    parser = BlueprintParser()
    result = parser.parse(sample_blueprint)
    
    services = result["services"]
    assert len(services) > 0
    
    # Should have storage service with specific tools
    storage_service = None
    for svc in services:
        if isinstance(svc, dict) and svc.get("service") == "storage":
            storage_service = svc
            break
        elif isinstance(svc, str) and svc == "storage":
            storage_service = svc
            break
    
    assert storage_service is not None


@pytest.mark.unit
def test_blueprint_parser_ui_components(sample_blueprint):
    """Test UI component parsing."""
    parser = BlueprintParser()
    result = parser.parse(sample_blueprint)
    
    ui_spec = result["ui_spec"]
    assert ui_spec["title"] == "Test App"
    assert ui_spec["layout"] == "vertical"
    assert len(ui_spec["components"]) == 2


@pytest.mark.unit
def test_blueprint_parser_type_id_syntax():
    """Test type#id syntax parsing."""
    blueprint = """{
  "app": {
    "id": "test",
    "name": "Test"
  },
  "services": [],
  "ui": {
    "title": "Test",
    "components": [
      {
        "button#save": {
          "text": "Save",
          "@click": "ui.save"
        }
      }
    ]
  }
}"""
    parser = BlueprintParser()
    result = parser.parse(blueprint)
    
    components = result["ui_spec"]["components"]
    assert len(components) == 1
    assert components[0]["type"] == "button"
    assert components[0]["id"] == "save"


@pytest.mark.unit
def test_blueprint_parser_event_handlers():
    """Test event handler parsing."""
    blueprint = """{
  "app": {
    "id": "test",
    "name": "Test"
  },
  "services": [],
  "ui": {
    "title": "Test",
    "components": [
      {
        "button#submit": {
          "text": "Submit",
          "@click": "ui.submit",
          "@hover": "ui.highlight"
        }
      }
    ]
  }
}"""
    parser = BlueprintParser()
    result = parser.parse(blueprint)
    
    button = result["ui_spec"]["components"][0]
    assert "on_event" in button
    assert button["on_event"]["click"] == "ui.submit"
    assert button["on_event"]["hover"] == "ui.highlight"


@pytest.mark.unit
def test_blueprint_parser_layout_shortcuts():
    """Test row/col layout shortcuts."""
    blueprint = """{
  "app": {
    "id": "test",
    "name": "Test"
  },
  "services": [],
  "ui": {
    "title": "Test",
    "components": [
      {
        "row": {
          "gap": 12,
          "children": [
            {"button#btn1": {"text": "Button 1"}}
          ]
        }
      },
      {
        "col": {
          "gap": 8,
          "children": [
            {"text": "Text"}
          ]
        }
      }
    ]
  }
}"""
    parser = BlueprintParser()
    result = parser.parse(blueprint)
    
    components = result["ui_spec"]["components"]
    
    # Row becomes horizontal container
    assert components[0]["type"] == "container"
    assert components[0]["props"]["layout"] == "horizontal"
    
    # Col becomes vertical container
    assert components[1]["type"] == "container"
    assert components[1]["props"]["layout"] == "vertical"


@pytest.mark.unit
def test_blueprint_parser_templates():
    """Test template expansion."""
    blueprint = """{
  "app": {
    "id": "test",
    "name": "Test"
  },
  "services": [],
  "templates": {
    "primary-btn": {
      "variant": "primary",
      "size": "large"
    }
  },
  "ui": {
    "title": "Test",
    "components": [
      {
        "button#save": {
          "$template": "primary-btn",
          "text": "Save"
        }
      }
    ]
  }
}"""
    parser = BlueprintParser()
    result = parser.parse(blueprint)
    
    button = result["ui_spec"]["components"][0]
    assert button["props"]["variant"] == "primary"
    assert button["props"]["size"] == "large"
    assert button["props"]["text"] == "Save"


@pytest.mark.unit
def test_blueprint_parser_lifecycle_hooks():
    """Test lifecycle hook parsing."""
    blueprint = """{
  "app": {
    "id": "test",
    "name": "Test"
  },
  "services": [],
  "ui": {
    "title": "Test",
    "lifecycle": {
      "on_mount": "storage.get",
      "on_unmount": ["storage.save", "ui.cleanup"]
    },
    "components": []
  }
}"""
    parser = BlueprintParser()
    result = parser.parse(blueprint)
    
    lifecycle = result["ui_spec"]["lifecycle_hooks"]
    assert "on_mount" in lifecycle
    assert lifecycle["on_mount"] == ["storage.get"]
    assert lifecycle["on_unmount"] == ["storage.save", "ui.cleanup"]


@pytest.mark.unit
def test_blueprint_parser_nested_children():
    """Test nested component parsing."""
    blueprint = """{
  "app": {
    "id": "test",
    "name": "Test"
  },
  "services": [],
  "ui": {
    "title": "Test",
    "components": [
      {
        "container#main": {
          "layout": "vertical",
          "children": [
            {"text#header": {"content": "Header"}},
            {
              "container#nested": {
                "layout": "horizontal",
                "children": [
                  {"button#btn1": {"text": "Button 1"}},
                  {"button#btn2": {"text": "Button 2"}}
                ]
              }
            }
          ]
        }
      }
    ]
  }
}"""
    parser = BlueprintParser()
    result = parser.parse(blueprint)
    
    main = result["ui_spec"]["components"][0]
    assert len(main["children"]) == 2
    
    nested = main["children"][1]
    assert nested["type"] == "container"
    assert len(nested["children"]) == 2


@pytest.mark.unit
def test_blueprint_parser_missing_app_section():
    """Test error on missing app section."""
    blueprint = """{
  "ui": {
    "title": "Test"
  }
}"""
    parser = BlueprintParser()
    
    with pytest.raises(ValidationError, match="missing 'app' section"):
        parser.parse(blueprint)


@pytest.mark.unit
def test_blueprint_parser_missing_required_fields():
    """Test error on missing required fields."""
    blueprint = """{
  "app": {
    "name": "Test"
  }
}"""
    parser = BlueprintParser()
    
    with pytest.raises(ValidationError, match="app.id is required"):
        parser.parse(blueprint)


@pytest.mark.unit
def test_blueprint_parser_invalid_json():
    """Test error on invalid JSON."""
    blueprint = "{not: valid json: ["
    parser = BlueprintParser()
    
    with pytest.raises(ValidationError, match="Invalid JSON"):
        parser.parse(blueprint)


@pytest.mark.unit
def test_parse_blueprint_convenience_function(sample_blueprint):
    """Test convenience function."""
    result = parse_blueprint(sample_blueprint)
    
    assert result["id"] == "test-app"
    assert result["name"] == "Test App"


@pytest.mark.unit
def test_blueprint_service_wildcard():
    """Test service wildcard expansion."""
    blueprint = """{
  "app": {
    "id": "test",
    "name": "Test"
  },
  "services": [
    {"storage": "*"},
    "filesystem"
  ],
  "ui": {
    "title": "Test",
    "components": []
  }
}"""
    parser = BlueprintParser()
    result = parser.parse(blueprint)
    
    services = result["services"]
    # Both should expand to simple strings (all tools)
    assert "storage" in services or any(s == "storage" for s in services if isinstance(s, str))
    assert "filesystem" in services or any(s == "filesystem" for s in services if isinstance(s, str))


@pytest.mark.unit
def test_blueprint_service_explicit_tools():
    """Test explicit tool list."""
    blueprint = """{
  "app": {
    "id": "test",
    "name": "Test"
  },
  "services": [
    {"storage": ["get", "set", "list"]}
  ],
  "ui": {
    "title": "Test",
    "components": []
  }
}"""
    parser = BlueprintParser()
    result = parser.parse(blueprint)
    
    services = result["services"]
    storage_service = next((s for s in services if isinstance(s, dict) and s.get("service") == "storage"), None)
    
    assert storage_service is not None
    assert "tools" in storage_service
    assert "get" in storage_service["tools"]
    assert "set" in storage_service["tools"]