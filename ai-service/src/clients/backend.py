"""Backend Service Client"""

from dataclasses import dataclass
import httpx
import pybreaker

from core import get_logger

logger = get_logger(__name__)


@dataclass
class ToolParameter:
    """Tool parameter definition"""

    name: str
    type: str
    description: str
    required: bool


@dataclass
class ToolDefinition:
    """Backend service tool definition"""

    id: str
    name: str
    description: str
    parameters: list[ToolParameter]
    returns: str


@dataclass
class ServiceDefinition:
    """Backend service provider definition"""

    id: str
    name: str
    description: str
    category: str
    capabilities: list[str]
    tools: list[ToolDefinition]


class BackendClient:
    """
    Client for querying backend services with circuit breaker protection.
    Discovers available service providers and their tools.
    """

    def __init__(self, backend_url: str = "http://localhost:8000", timeout: float = 5.0) -> None:
        """
        Initialize backend client with circuit breaker.

        Args:
            backend_url: Base URL of the Go backend
            timeout: Request timeout in seconds
        """
        self.backend_url = backend_url.rstrip("/")
        self.timeout = timeout
        self._client = httpx.Client(timeout=timeout)

        # Initialize circuit breaker for backend HTTP calls
        # Create listener for state change logging
        class BreakerListener(pybreaker.CircuitBreakerListener):
            """Listener for circuit breaker state changes."""

            def state_change(self, cb, old_state, new_state):
                """Called when circuit breaker state changes."""
                logger.warning(
                    "breaker_state_change",
                    breaker=cb.name,
                    from_state=str(old_state),
                    to_state=str(new_state),
                )

        self._breaker = pybreaker.CircuitBreaker(
            fail_max=5,
            reset_timeout=30,
            name="backend-http",
            listeners=[BreakerListener()],
        )

        logger.info("client_init", url=self.backend_url)

    def discover_services(self, category: str | None = None) -> list[ServiceDefinition]:
        """
        Discover available services from the backend with circuit breaker protection.

        Args:
            category: Optional category filter (storage, auth, system, etc.)

        Returns:
            List of service definitions
        """
        try:
            url = f"{self.backend_url}/services"
            params = {"category": category} if category else {}

            # Execute with circuit breaker protection
            def _make_request():
                return self._client.get(url, params=params)

            response = self._breaker.call(_make_request)
            response.raise_for_status()

            # Parse JSON response (httpx uses orjson if available, otherwise stdlib)
            data = response.json()

            # Validate response structure
            if not isinstance(data, dict):
                logger.error("invalid_response", type=type(data).__name__)
                return []

            services = data.get("services", [])

            # Convert to ServiceDefinition objects
            result = []
            for svc in services:
                tools = []
                for tool_data in svc.get("tools", []):
                    params_list = []
                    for param in tool_data.get("parameters", []):
                        params_list.append(
                            ToolParameter(
                                name=param["name"],
                                type=param["type"],
                                description=param["description"],
                                required=param["required"],
                            )
                        )

                    tools.append(
                        ToolDefinition(
                            id=tool_data["id"],
                            name=tool_data["name"],
                            description=tool_data["description"],
                            parameters=params_list,
                            returns=tool_data["returns"],
                        )
                    )

                result.append(
                    ServiceDefinition(
                        id=svc["id"],
                        name=svc["name"],
                        description=svc["description"],
                        category=svc["category"],
                        capabilities=svc.get("capabilities", []),
                        tools=tools,
                    )
                )

            total_tools = sum(len(s.tools) for s in result)
            logger.info("discovered", services=len(result), tools=total_tools)
            return result

        except pybreaker.CircuitBreakerError:
            logger.error("discover_failed", error="Circuit breaker open - backend unavailable")
            return []
        except httpx.HTTPError as e:
            logger.warning("http_error", error=str(e))
            return []
        except Exception as e:
            logger.error("discover_failed", error=str(e), exc_info=True)
            return []

    def get_tools_description(self, services: list[ServiceDefinition]) -> str:
        """
        Format service tools as a string for AI context.

        Args:
            services: List of service definitions

        Returns:
            Formatted tool descriptions
        """
        if not services:
            return ""

        lines = ["\nBACKEND SERVICES:"]

        for service in services:
            lines.append(f"\n{service.category.upper()} - {service.name}:")
            for tool in service.tools:
                params_str = ", ".join(
                    f"{p.name}: {p.type}" + (" (required)" if p.required else " (optional)")
                    for p in tool.parameters
                )
                params_display = f"({params_str})" if params_str else "(no params)"
                lines.append(f"  - {tool.id}: {tool.description} {params_display}")

        return "\n".join(lines)

    def health_check(self) -> bool:
        """
        Check if backend is reachable (bypasses circuit breaker).

        Returns:
            True if backend is healthy
        """
        try:
            # Health checks bypass circuit breaker to test actual connectivity
            response = self._client.get(f"{self.backend_url}/health", timeout=2.0)
            return response.status_code == 200
        except Exception:
            return False

    # ========================================================================
    # Kernel/Scheduler Operations
    # ========================================================================

    def schedule_next(self) -> int | None:
        """
        Schedule the next process with circuit breaker protection.

        Returns:
            PID of the next scheduled process, or None if no processes available

        Raises:
            httpx.HTTPError: If the request fails
        """
        try:
            url = f"{self.backend_url}/kernel/schedule-next"

            def _make_request():
                return self._client.post(url)

            response = self._breaker.call(_make_request)
            response.raise_for_status()

            data = response.json()
            if not data.get("success", False):
                logger.error("schedule_next_failed", error=data.get("error"))
                return None

            logger.info("schedule_next_success", next_pid=data.get("next_pid"))
            return data.get("next_pid")

        except pybreaker.CircuitBreakerError:
            logger.error("schedule_next_failed", error="Circuit breaker open")
            return None
        except httpx.HTTPError as e:
            logger.warning("schedule_next_http_error", error=str(e))
            raise
        except Exception as e:
            logger.error("schedule_next_failed", error=str(e), exc_info=True)
            return None

    def get_scheduler_stats(self) -> dict | None:
        """
        Get scheduler statistics with circuit breaker protection.

        Returns:
            Dictionary with scheduler statistics:
            - total_scheduled: Total number of processes scheduled
            - context_switches: Number of context switches
            - preemptions: Number of preemptions
            - active_processes: Number of active processes
            - policy: Current scheduling policy (RoundRobin, Priority, Fair)
            - quantum_micros: Time quantum in microseconds

        Raises:
            httpx.HTTPError: If the request fails
        """
        try:
            url = f"{self.backend_url}/kernel/scheduler/stats"

            def _make_request():
                return self._client.get(url)

            response = self._breaker.call(_make_request)
            response.raise_for_status()

            data = response.json()
            if not data.get("success", False):
                logger.error("get_scheduler_stats_failed", error=data.get("error"))
                return None

            stats = data.get("stats", {})
            logger.info("get_scheduler_stats_success", stats=stats)
            return stats

        except pybreaker.CircuitBreakerError:
            logger.error("get_scheduler_stats_failed", error="Circuit breaker open")
            return None
        except httpx.HTTPError as e:
            logger.warning("get_scheduler_stats_http_error", error=str(e))
            raise
        except Exception as e:
            logger.error("get_scheduler_stats_failed", error=str(e), exc_info=True)
            return None

    def set_scheduling_policy(self, policy: str) -> bool:
        """
        Set the scheduling policy with circuit breaker protection.

        Args:
            policy: Scheduling policy to set (RoundRobin, Priority, or Fair)

        Returns:
            True if successful, False otherwise

        Raises:
            httpx.HTTPError: If the request fails
            ValueError: If policy is invalid
        """
        valid_policies = {"RoundRobin", "Priority", "Fair"}
        if policy not in valid_policies:
            raise ValueError(f"Invalid policy '{policy}'. Must be one of: {valid_policies}")

        try:
            url = f"{self.backend_url}/kernel/scheduler/policy"

            def _make_request():
                return self._client.put(url, json={"policy": policy})

            response = self._breaker.call(_make_request)
            response.raise_for_status()

            data = response.json()
            success = data.get("success", False)

            if success:
                logger.info("set_scheduling_policy_success", policy=policy)
            else:
                logger.error("set_scheduling_policy_failed", error=data.get("error"))

            return success

        except pybreaker.CircuitBreakerError:
            logger.error("set_scheduling_policy_failed", error="Circuit breaker open")
            return False
        except httpx.HTTPError as e:
            logger.warning("set_scheduling_policy_http_error", error=str(e))
            raise
        except Exception as e:
            logger.error("set_scheduling_policy_failed", error=str(e), exc_info=True)
            return False

    def close(self) -> None:
        """Close HTTP client"""
        self._client.close()

    def __enter__(self) -> "BackendClient":
        return self

    def __exit__(self, *args: object) -> None:
        self.close()
