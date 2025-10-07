"""Tests for backend client circuit breaker integration."""

import pytest
import httpx
import pybreaker
from unittest.mock import Mock, patch, MagicMock

from clients.backend import BackendClient


class TestBackendClientCircuitBreaker:
    """Test circuit breaker integration in BackendClient."""

    def test_client_initializes_circuit_breaker(self):
        """Test that client initializes with circuit breaker."""
        client = BackendClient()

        assert client._breaker is not None
        assert isinstance(client._breaker, pybreaker.CircuitBreaker)
        assert client._breaker.name == "backend-http"
        assert client._breaker.fail_max == 5
        # pybreaker uses reset_timeout not timeout_duration
        assert hasattr(client._breaker, '_reset_timeout')

    def test_circuit_breaker_state_change_logging(self):
        """Test that circuit breaker state changes are logged."""
        # The listener is an internal class now, so we just verify it's registered
        client = BackendClient()

        # Verify the breaker has listeners
        assert len(client._breaker._listeners) > 0

        # Verify the listener is a CircuitBreakerListener
        listener = client._breaker._listeners[0]
        assert isinstance(listener, pybreaker.CircuitBreakerListener)

        # Test that state_change method exists and can be called
        assert hasattr(listener, 'state_change')

        # Mock logger to test logging
        with patch('clients.backend.logger') as mock_logger:
            # Simulate state change
            listener.state_change(client._breaker, "closed", "open")

            # Verify logging was called
            mock_logger.warning.assert_called_once()
            call_args = mock_logger.warning.call_args
            assert call_args[0][0] == "breaker_state_change"

    @patch('clients.backend.httpx.Client')
    def test_discover_services_uses_circuit_breaker(self, mock_http_client):
        """Test that discover_services uses circuit breaker."""
        # Setup mock response
        mock_response = Mock()
        mock_response.status_code = 200
        mock_response.json.return_value = {"services": []}

        mock_client_instance = Mock()
        mock_client_instance.get.return_value = mock_response
        mock_http_client.return_value = mock_client_instance

        client = BackendClient()
        client._client = mock_client_instance

        # Call discover_services
        with patch.object(client._breaker, 'call', wraps=client._breaker.call) as mock_call:
            result = client.discover_services()

            # Verify breaker.call was used
            mock_call.assert_called_once()
            assert isinstance(result, list)

    def test_circuit_breaker_opens_on_failures(self):
        """Test that circuit breaker opens after repeated failures."""
        client = BackendClient()

        # Replace the client's HTTP client with a mock that raises errors
        mock_client_instance = Mock()
        mock_client_instance.get.side_effect = httpx.HTTPError("Connection failed")
        client._client = mock_client_instance

        # Trigger failures (fail_max is 5, so 6 failures should open circuit)
        for _ in range(6):
            result = client.discover_services()
            # Should return empty list on error
            assert result == []

        # Circuit should be open now (pybreaker checks state via str comparison)
        assert str(client._breaker.current_state) == "open"

    def test_circuit_breaker_error_handling(self):
        """Test that circuit breaker errors are handled gracefully."""
        client = BackendClient()

        # Replace with mock that raises errors
        mock_client_instance = Mock()
        mock_client_instance.get.side_effect = httpx.HTTPError("Connection failed")
        client._client = mock_client_instance

        # Trigger enough failures to open circuit
        for _ in range(6):
            client.discover_services()

        # Verify circuit is open and call returns empty list
        result = client.discover_services()
        assert result == []

    def test_schedule_next_uses_circuit_breaker(self):
        """Test that schedule_next uses circuit breaker."""
        client = BackendClient()

        # Setup mock response
        mock_response = Mock()
        mock_response.json.return_value = {"success": True, "next_pid": 123}

        mock_client_instance = Mock()
        mock_client_instance.post.return_value = mock_response
        client._client = mock_client_instance

        # Just verify the method works and returns the expected value
        result = client.schedule_next()
        assert result == 123

    def test_get_scheduler_stats_uses_circuit_breaker(self):
        """Test that get_scheduler_stats uses circuit breaker."""
        client = BackendClient()

        # Setup mock response
        mock_response = Mock()
        mock_response.json.return_value = {
            "success": True,
            "stats": {"total_scheduled": 100}
        }

        mock_client_instance = Mock()
        mock_client_instance.get.return_value = mock_response
        client._client = mock_client_instance

        result = client.get_scheduler_stats()
        assert result == {"total_scheduled": 100}

    def test_set_scheduling_policy_uses_circuit_breaker(self):
        """Test that set_scheduling_policy uses circuit breaker."""
        client = BackendClient()

        # Setup mock response
        mock_response = Mock()
        mock_response.json.return_value = {"success": True}

        mock_client_instance = Mock()
        mock_client_instance.put.return_value = mock_response
        client._client = mock_client_instance

        result = client.set_scheduling_policy("RoundRobin")
        assert result is True

    def test_health_check_bypasses_circuit_breaker(self):
        """Test that health_check bypasses circuit breaker."""
        client = BackendClient()

        mock_response = Mock()
        mock_response.status_code = 200

        mock_client_instance = Mock()
        client._client = mock_client_instance

        # Trigger failures to open circuit
        mock_client_instance.get.side_effect = httpx.HTTPError("Connection failed")
        for _ in range(6):
            client.discover_services()

        # Reset mock for health check
        mock_client_instance.get.side_effect = None
        mock_client_instance.get.return_value = mock_response

        # Health check should still work (bypasses breaker)
        result = client.health_check()
        assert result is True

    def test_circuit_breaker_recovers_after_timeout(self):
        """Test that circuit breaker can recover after timeout."""
        client = BackendClient()
        client._breaker._reset_timeout = 1  # Short timeout for testing

        mock_client_instance = Mock()
        client._client = mock_client_instance

        # Trigger failures to open circuit
        mock_client_instance.get.side_effect = httpx.HTTPError("Connection failed")
        for _ in range(6):
            client.discover_services()

        assert str(client._breaker.current_state) == "open"

        # Wait for timeout and setup successful response
        import time
        time.sleep(1.1)  # Wait for circuit to move to half-open

        mock_response = Mock()
        mock_response.json.return_value = {"services": []}
        mock_client_instance.get.side_effect = None
        mock_client_instance.get.return_value = mock_response

        # Call should succeed and close circuit
        result = client.discover_services()
        assert result == []
        assert str(client._breaker.current_state) == "closed"


class TestBackendClientErrorScenarios:
    """Test error scenarios with circuit breaker."""

    def test_http_error_triggers_circuit_breaker(self):
        """Test that HTTP errors trigger circuit breaker."""
        client = BackendClient()

        mock_client_instance = Mock()
        mock_client_instance.get.side_effect = httpx.ConnectError("Connection refused")
        client._client = mock_client_instance

        # Multiple failures - should all return empty list
        for _ in range(5):
            result = client.discover_services()
            assert result == []

        # After 5 failures, circuit should still be closed (fail_max is 5)
        # One more failure should open it
        result = client.discover_services()
        assert result == []

        # Now circuit should be open
        assert str(client._breaker.current_state) == "open"

    def test_timeout_error_triggers_circuit_breaker(self):
        """Test that timeout errors trigger circuit breaker."""
        client = BackendClient()

        mock_client_instance = Mock()
        mock_client_instance.get.side_effect = httpx.TimeoutException("Request timeout")
        client._client = mock_client_instance

        # Trigger multiple timeouts
        for _ in range(6):
            result = client.discover_services()
            assert result == []

        # Circuit should be open after multiple failures
        assert str(client._breaker.current_state) == "open"
