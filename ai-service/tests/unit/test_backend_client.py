"""Tests for backend client."""

import pytest
import respx
import httpx
from src.clients.backend import BackendClient, ServiceDefinition, ToolDefinition, ToolParameter


@pytest.mark.unit
def test_backend_client_initialization():
    """Test backend client initialization."""
    client = BackendClient("http://localhost:8000", timeout=5.0)
    
    assert client.backend_url == "http://localhost:8000"
    assert client.timeout == 5.0


@pytest.mark.unit
@respx.mock
def test_backend_client_health_check():
    """Test health check."""
    respx.get("http://localhost:8000/health").mock(return_value=httpx.Response(200, json={"status": "ok"}))
    
    client = BackendClient("http://localhost:8000")
    result = client.health_check()
    
    assert result is True


@pytest.mark.unit
@respx.mock
def test_backend_client_health_check_failure():
    """Test health check failure."""
    respx.get("http://localhost:8000/health").mock(return_value=httpx.Response(500))
    
    client = BackendClient("http://localhost:8000")
    result = client.health_check()
    
    assert result is False


@pytest.mark.unit
@respx.mock
def test_backend_client_discover_services(mock_backend_services):
    """Test service discovery."""
    respx.get("http://localhost:8000/services").mock(
        return_value=httpx.Response(200, json=mock_backend_services)
    )
    
    client = BackendClient("http://localhost:8000")
    services = client.discover_services()
    
    assert len(services) == 1
    assert services[0].id == "storage"
    assert services[0].name == "Storage Service"
    assert services[0].category == "data"
    assert len(services[0].tools) == 2


@pytest.mark.unit
@respx.mock
def test_backend_client_discover_services_with_category(mock_backend_services):
    """Test service discovery with category filter."""
    respx.get("http://localhost:8000/services", params={"category": "data"}).mock(
        return_value=httpx.Response(200, json=mock_backend_services)
    )
    
    client = BackendClient("http://localhost:8000")
    services = client.discover_services(category="data")
    
    assert len(services) == 1
    assert services[0].category == "data"


@pytest.mark.unit
@respx.mock
def test_backend_client_discover_services_empty():
    """Test service discovery with no services."""
    respx.get("http://localhost:8000/services").mock(
        return_value=httpx.Response(200, json={"services": []})
    )
    
    client = BackendClient("http://localhost:8000")
    services = client.discover_services()
    
    assert len(services) == 0


@pytest.mark.unit
@respx.mock
def test_backend_client_discover_services_error():
    """Test service discovery error handling."""
    respx.get("http://localhost:8000/services").mock(return_value=httpx.Response(500))
    
    client = BackendClient("http://localhost:8000")
    services = client.discover_services()
    
    # Should return empty list on error
    assert len(services) == 0


@pytest.mark.unit
@respx.mock
def test_backend_client_discover_services_invalid_response():
    """Test service discovery with invalid response."""
    respx.get("http://localhost:8000/services").mock(
        return_value=httpx.Response(200, json="invalid")
    )
    
    client = BackendClient("http://localhost:8000")
    services = client.discover_services()
    
    # Should return empty list on invalid response
    assert len(services) == 0


@pytest.mark.unit
def test_backend_client_get_tools_description():
    """Test formatting tools description."""
    tool_param = ToolParameter(
        name="key",
        type="string",
        description="Storage key",
        required=True
    )
    
    tool = ToolDefinition(
        id="storage.get",
        name="Get Value",
        description="Get value from storage",
        parameters=[tool_param],
        returns="any"
    )
    
    service = ServiceDefinition(
        id="storage",
        name="Storage Service",
        description="Key-value storage",
        category="data",
        capabilities=["persistent"],
        tools=[tool]
    )
    
    client = BackendClient("http://localhost:8000")
    description = client.get_tools_description([service])
    
    assert "Storage Service" in description
    assert "storage.get" in description
    assert "Get value from storage" in description


@pytest.mark.unit
def test_backend_client_get_tools_description_empty():
    """Test formatting empty tools description."""
    client = BackendClient("http://localhost:8000")
    description = client.get_tools_description([])
    
    assert description == ""


@pytest.mark.unit
def test_backend_client_context_manager():
    """Test client as context manager."""
    with BackendClient("http://localhost:8000") as client:
        assert client is not None
    
    # Client should be closed after context


@pytest.mark.integration
@respx.mock
def test_backend_client_full_workflow(mock_backend_services):
    """Test full discovery workflow."""
    # Mock health check
    respx.get("http://localhost:8000/health").mock(
        return_value=httpx.Response(200, json={"status": "ok"})
    )
    
    # Mock service discovery
    respx.get("http://localhost:8000/services").mock(
        return_value=httpx.Response(200, json=mock_backend_services)
    )
    
    with BackendClient("http://localhost:8000") as client:
        # Check health
        assert client.health_check() is True
        
        # Discover services
        services = client.discover_services()
        assert len(services) == 1
        
        # Format description
        description = client.get_tools_description(services)
        assert "Storage Service" in description

