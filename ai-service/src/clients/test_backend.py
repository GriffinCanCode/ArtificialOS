"""
Tests for BackendClient service discovery
"""

import sys
from pathlib import Path

# Add parent directory to path
sys.path.insert(0, str(Path(__file__).parent.parent))

import pytest
from clients.backend import BackendClient, ServiceDefinition, ToolDefinition


def test_backend_client_initialization():
    """Test BackendClient can be initialized"""
    client = BackendClient("http://localhost:8000")
    assert client.backend_url == "http://localhost:8000"
    assert client.timeout == 5.0
    client.close()


def test_backend_client_context_manager():
    """Test BackendClient works as context manager"""
    with BackendClient("http://localhost:8000") as client:
        assert client is not None


@pytest.mark.integration
def test_discover_services():
    """
    Integration test: Discover services from running backend
    Requires backend to be running on localhost:8000
    """
    client = BackendClient("http://localhost:8000")
    
    try:
        # Check health first
        if not client.health_check():
            pytest.skip("Backend not available")
        
        # Discover all services
        services = client.discover_services()
        
        # Should have at least the core services (storage, auth, system)
        assert len(services) >= 3
        
        # Check service structure
        for service in services:
            assert isinstance(service, ServiceDefinition)
            assert service.id
            assert service.name
            assert service.description
            assert service.category
            assert isinstance(service.tools, list)
        
        # Find storage service
        storage = next((s for s in services if s.id == "storage"), None)
        assert storage is not None
        assert storage.category == "storage"
        assert len(storage.tools) >= 5  # set, get, remove, list, clear
        
        # Check tool structure
        for tool in storage.tools:
            assert isinstance(tool, ToolDefinition)
            assert tool.id.startswith("storage.")
            assert tool.description
        
        print(f"✅ Discovered {len(services)} services successfully")
        
    finally:
        client.close()


@pytest.mark.integration
def test_tools_description_formatting():
    """Test formatted tool descriptions for AI context"""
    client = BackendClient("http://localhost:8000")
    
    try:
        if not client.health_check():
            pytest.skip("Backend not available")
        
        services = client.discover_services()
        description = client.get_tools_description(services)
        
        # Should contain backend services header
        assert "BACKEND SERVICES" in description
        
        # Should contain service categories
        assert any(cat in description for cat in ["STORAGE", "AUTH", "SYSTEM"])
        
        # Should contain tool IDs
        assert "storage.set" in description or "storage.get" in description
        
        print("✅ Tools description formatted correctly")
        print(description)
        
    finally:
        client.close()


if __name__ == "__main__":
    # Run integration tests if backend is available
    print("Testing BackendClient...")
    test_backend_client_initialization()
    test_backend_client_context_manager()
    
    print("\nRunning integration tests (requires running backend)...")
    try:
        test_discover_services()
        test_tools_description_formatting()
        print("\n✅ All tests passed!")
    except Exception as e:
        print(f"\n❌ Integration tests failed: {e}")

