#!/usr/bin/env python3
"""
Test Blueprint Parser
"""

import sys
import json
sys.path.insert(0, 'ai-service/src')

from blueprint.parser import BlueprintParser

def test_parser():
    print("Testing Blueprint Parser...")
    
    # Read test file
    with open('apps/utilities/test-app.bp', 'r') as f:
        content = f.read()
    
    print(f"\n📄 Input Blueprint ({len(content)} bytes):")
    print("=" * 60)
    print(content[:500] + "..." if len(content) > 500 else content)
    
    # Parse
    parser = BlueprintParser()
    try:
        result = parser.parse(content)
        print("\n✅ Parse successful!")
        
        # Show metadata
        print(f"\n📦 Package Metadata:")
        print(f"   ID: {result['id']}")
        print(f"   Name: {result['name']}")
        print(f"   Icon: {result['icon']}")
        print(f"   Services: {result['services']}")
        print(f"   Permissions: {result['permissions']}")
        
        # Show UI structure
        ui_spec = result['ui_spec']
        print(f"\n🎨 UI Spec:")
        print(f"   Title: {ui_spec['title']}")
        print(f"   Layout: {ui_spec['layout']}")
        print(f"   Components: {len(ui_spec.get('components', []))}")
        
        # Show first few components
        components = ui_spec.get('components', [])
        for i, comp in enumerate(components[:3]):
            print(f"\n   Component {i+1}:")
            print(f"      Type: {comp['type']}")
            print(f"      ID: {comp.get('id', 'N/A')}")
            if 'children' in comp:
                print(f"      Children: {len(comp['children'])}")
            if 'on_event' in comp:
                print(f"      Events: {list(comp['on_event'].keys())}")
        
        # Output full JSON
        print(f"\n📋 Full JSON output:")
        print("=" * 60)
        print(json.dumps(result, indent=2)[:1000] + "...")
        
        return True
        
    except Exception as e:
        print(f"\n❌ Parse failed: {e}")
        import traceback
        traceback.print_exc()
        return False

if __name__ == '__main__':
    success = test_parser()
    sys.exit(0 if success else 1)

