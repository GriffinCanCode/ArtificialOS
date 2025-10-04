# Testing Guide

Comprehensive testing setup for the AI Service.

## Table of Contents

- [Overview](#overview)
- [Testing Libraries](#testing-libraries)
- [Running Tests](#running-tests)
- [Test Organization](#test-organization)
- [Writing Tests](#writing-tests)
- [Coverage](#coverage)
- [Best Practices](#best-practices)

## Overview

This project uses a comprehensive testing stack to ensure code quality and reliability:

- **75%+ coverage requirement** enforced in CI
- **Unit, integration, and property-based tests**
- **Performance benchmarking**
- **Parallel test execution** for speed
- **Comprehensive fixtures** for easy test writing

## Testing Libraries

### Core Testing

- **pytest** - Test framework with excellent fixtures and plugins
- **pytest-asyncio** - Async test support
- **pytest-cov** - Coverage reporting
- **pytest-mock** - Mocking utilities

### Specialized Testing

- **hypothesis** - Property-based testing (generative testing)
- **pytest-benchmark** - Performance benchmarking
- **pytest-xdist** - Parallel test execution
- **pytest-timeout** - Test timeout handling

### Mocking & Utilities

- **respx** - httpx request mocking
- **grpcio-testing** - gRPC testing utilities
- **faker** - Test data generation
- **freezegun** - Time mocking

## Running Tests

### Basic Commands

```bash
# Run all tests with coverage
make test

# Run unit tests only (fast)
make test-unit

# Run integration tests
make test-integration

# Run tests in parallel (fastest)
make test-fast

# Run without coverage (faster)
make test-no-cov
```

### Specialized Test Runs

```bash
# Run performance benchmarks
make test-benchmark

# Run slow tests
make test-slow

# Run gRPC-specific tests
make test-grpc

# Watch mode (re-run on file changes)
make test-watch
```

### Direct pytest Commands

```bash
# Run specific test file
PYTHONPATH=src pytest tests/unit/test_agents.py -v

# Run specific test function
PYTHONPATH=src pytest tests/unit/test_agents.py::test_chat_agent_stream -v

# Run tests matching pattern
PYTHONPATH=src pytest -k "chat" -v

# Show print statements
PYTHONPATH=src pytest tests/ -v -s

# Stop on first failure
PYTHONPATH=src pytest tests/ -x

# Run last failed tests only
PYTHONPATH=src pytest tests/ --lf
```

## Test Organization

### Test Markers

Tests are organized using pytest markers:

```python
@pytest.mark.unit           # Fast, isolated unit tests
@pytest.mark.integration    # Tests with external dependencies
@pytest.mark.slow           # Slow-running tests
@pytest.mark.grpc           # gRPC service tests
@pytest.mark.llm            # Tests requiring LLM/API (expensive)
@pytest.mark.property       # Property-based tests with hypothesis
@pytest.mark.benchmark      # Performance benchmarks
```

### Test Files

```
tests/
├── conftest.py              # Shared fixtures
├── README.md                # This file
└── unit/                    # Unit and integration tests
    ├── __init__.py
    ├── test_agents.py           # Agent tests (chat, UI generator)
    ├── test_backend_client.py   # Backend client tests
    ├── test_blueprint.py        # Blueprint parser tests
    ├── test_cache.py            # Cache implementation tests
    ├── test_config.py           # Configuration tests
    ├── test_handlers.py         # gRPC handler tests
    ├── test_hash.py             # Hashing utility tests
    ├── test_models.py           # Model config/loader tests
    ├── test_parsers.py          # JSON parser tests
    ├── test_stream.py           # Stream utility tests
    └── test_validation.py       # Validation tests
```

## Writing Tests

### Using Fixtures

We provide comprehensive fixtures in `conftest.py`:

```python
def test_ui_generation(ui_generator, tool_registry):
    """Test with pre-configured fixtures."""
    spec = ui_generator.generate_ui("calculator")
    assert spec.title == "Calculator"

def test_with_mock_backend(mock_httpx_client, mock_backend_services):
    """Test with mocked HTTP client."""
    # HTTP requests are automatically mocked
    pass
```

### Unit Tests

```python
@pytest.mark.unit
def test_chat_message_creation():
    """Test basic functionality in isolation."""
    msg = ChatMessage(role="user", content="Hello", timestamp=123)
    assert msg.role == "user"
    assert msg.content == "Hello"
```

### Async Tests

```python
@pytest.mark.unit
@pytest.mark.asyncio
async def test_async_function(chat_agent):
    """Test async functions."""
    response = await chat_agent.get_response("Hello")
    assert isinstance(response, str)
```

### Integration Tests

```python
@pytest.mark.integration
@respx.mock
def test_backend_integration(mock_backend_services):
    """Test with external services mocked."""
    respx.get("http://localhost:8000/services").mock(
        return_value=httpx.Response(200, json=mock_backend_services)
    )
    
    client = BackendClient("http://localhost:8000")
    services = client.discover_services()
    assert len(services) > 0
```

### Property-Based Tests

```python
from hypothesis import given, strategies as st

@pytest.mark.property
@given(st.text(min_size=1, max_size=1000))
def test_hash_deterministic(text):
    """Test with automatically generated inputs."""
    hash1 = hash_string(text)
    hash2 = hash_string(text)
    assert hash1 == hash2
```

### Performance Benchmarks

```python
@pytest.mark.benchmark
def test_cache_performance(benchmark):
    """Benchmark cache operations."""
    def cache_operations():
        cache = LRUCache(max_size=100)
        for i in range(1000):
            cache.set(f"key-{i}", f"value-{i}")
    
    result = benchmark(cache_operations)
    assert result is not None
```

### Mocking Examples

```python
# Mock with pytest-mock
def test_with_mocker(mocker):
    mock_llm = mocker.MagicMock()
    mock_llm.invoke.return_value = "response"
    
    agent = ChatAgent(mock_llm)
    result = agent.get_response("Hello")
    assert result == "response"

# Mock HTTP with respx
@respx.mock
def test_http_request():
    respx.get("http://api.example.com/data").mock(
        return_value=httpx.Response(200, json={"result": "success"})
    )
    
    # Your code that makes HTTP requests
    pass

# Mock time with freezegun
from freezegun import freeze_time

@freeze_time("2024-01-01 12:00:00")
def test_with_frozen_time():
    # Time is frozen at 2024-01-01 12:00:00
    pass
```

## Coverage

### Viewing Coverage

```bash
# Run tests and generate HTML coverage report
make test

# Open coverage report in browser
make coverage
```

### Coverage Configuration

Coverage is configured in `pytest.ini`:

- **Minimum 75% coverage required**
- Branch coverage enabled
- Generated files excluded (protobuf)
- HTML and JSON reports generated

### Coverage Goals

- Core utilities: 90%+ coverage
- Business logic: 85%+ coverage
- Integration code: 70%+ coverage
- Overall project: 75%+ coverage

## Best Practices

### Test Naming

```python
# Good test names
def test_chat_agent_stream_response()
def test_blueprint_parser_handles_invalid_yaml()
def test_cache_evicts_least_recently_used_entry()

# Bad test names
def test_chat()
def test_parser()
def test_1()
```

### Test Structure (AAA Pattern)

```python
def test_example():
    # Arrange - Set up test data and dependencies
    cache = LRUCache(max_size=2)
    
    # Act - Execute the code under test
    cache.set("key", "value")
    result = cache.get("key")
    
    # Assert - Verify the results
    assert result == "value"
```

### Test Independence

```python
# Good - Each test is independent
def test_add_user():
    user = User(name="Alice")
    user.save()
    assert user.id is not None

def test_delete_user():
    user = User(name="Bob")
    user.save()
    user.delete()
    assert User.get(user.id) is None

# Bad - Tests depend on each other
user_id = None

def test_add_user():
    global user_id
    user = User(name="Alice")
    user.save()
    user_id = user.id

def test_delete_user():
    User.get(user_id).delete()  # Depends on previous test
```

### Fixtures Over Setup/Teardown

```python
# Good - Use fixtures
@pytest.fixture
def sample_user():
    user = User(name="Test")
    yield user
    user.delete()  # Cleanup after test

def test_user_operations(sample_user):
    assert sample_user.name == "Test"

# Less ideal - setUp/tearDown
class TestUser:
    def setUp(self):
        self.user = User(name="Test")
    
    def tearDown(self):
        self.user.delete()
```

### Mock External Dependencies

```python
# Good - Mock external services
@respx.mock
def test_api_call():
    respx.get("https://api.example.com").mock(
        return_value=httpx.Response(200, json={"data": "test"})
    )
    result = fetch_data()
    assert result["data"] == "test"

# Bad - Make real API calls in tests
def test_api_call():
    result = fetch_data()  # Makes real HTTP request
    assert result["data"] == "test"
```

### Parameterized Tests

```python
# Test multiple scenarios efficiently
@pytest.mark.parametrize("input,expected", [
    ("calculator", "Calculator"),
    ("todo app", "Todo App"),
    ("counter", "Counter"),
])
def test_ui_generation(ui_generator, input, expected):
    spec = ui_generator.generate_ui(input)
    assert spec.title == expected
```

## Continuous Integration

Tests run automatically on:
- Pull requests
- Commits to main
- Pre-commit hooks (optional)

CI will fail if:
- Any test fails
- Coverage drops below 75%
- Linting errors found
- Type checking fails

## Troubleshooting

### Common Issues

**Tests fail with import errors:**
```bash
# Ensure PYTHONPATH is set
PYTHONPATH=src pytest tests/
```

**Coverage not updating:**
```bash
# Clean cache and re-run
make clean
make test
```

**Parallel tests failing:**
```bash
# Some tests may not be parallel-safe
# Run sequentially instead
make test
```

**Fixture not found:**
```bash
# Ensure conftest.py is in tests/ directory
# Check fixture scope (session, module, function)
```

## Examples

See the `tests/unit/` directory for comprehensive examples of:
- Unit tests
- Integration tests
- Property-based tests
- Performance benchmarks
- Async tests
- Mocking strategies

## Resources

- [pytest documentation](https://docs.pytest.org/)
- [hypothesis documentation](https://hypothesis.readthedocs.io/)
- [respx documentation](https://lundberg.github.io/respx/)
- [pytest-asyncio documentation](https://pytest-asyncio.readthedocs.io/)

