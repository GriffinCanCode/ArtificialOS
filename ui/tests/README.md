# Testing Guide

This directory contains test utilities and configuration for the UI project.

## Stack

- **Vitest**: Fast unit test framework (Jest-compatible API)
- **React Testing Library**: Component testing best practices
- **@testing-library/jest-dom**: Custom matchers for DOM assertions
- **@testing-library/user-event**: Realistic user interaction simulation
- **happy-dom**: Lightweight DOM environment

## Running Tests

```bash
# Run tests in watch mode (recommended during development)
npm test

# Run tests once
npm run test:run

# Run tests with UI (visual test runner)
npm run test:ui

# Run tests with coverage report
npm run test:coverage

# Run specific test file
npm test -- ChatInterface.test
```

## Test Structure

```
tests/                          # Centralized test directory
├── setup/
│   ├── setup.ts               # Global test setup
│   └── utils.tsx              # Custom render functions and helpers
├── components/
│   ├── chat/
│   │   └── ChatInterface.test.tsx
│   └── dynamics/
│       └── DynamicRenderer.state.test.ts
├── store/
│   └── appStore.test.ts       # State management tests
├── hooks/
│   └── useLogger.test.ts      # Hook tests
├── utils/
│   └── api/
│       └── websocketClient.test.ts
└── README.md                   # This file

src/                            # Source code (tests import from here)
├── components/
├── store/
├── hooks/
└── utils/
```

## Writing Tests

### Component Tests

Use the custom `render` function that includes all providers:

```tsx
import { render, screen, userEvent } from "../setup/utils";
import MyComponent from "../../src/components/MyComponent";

it("renders and handles interaction", async () => {
  const user = userEvent.setup();
  render(<MyComponent />);
  
  const button = screen.getByRole("button");
  await user.click(button);
  
  expect(screen.getByText("Clicked")).toBeInTheDocument();
});
```

### State Tests

Test Zustand stores directly:

```tsx
import { useAppStore } from "../../src/store/appStore";

it("updates state correctly", () => {
  const { addMessage } = useAppStore.getState();
  
  addMessage({ type: "user", content: "Test", timestamp: 1 });
  
  const { messages } = useAppStore.getState();
  expect(messages).toHaveLength(1);
});
```

### Integration Tests

Use mocks for external dependencies:

```tsx
import { MockWebSocket, mockFetch } from "../setup/utils";

beforeEach(() => {
  global.WebSocket = MockWebSocket as any;
});

it("handles WebSocket messages", () => {
  const client = new WebSocketClient();
  client.connect();
  
  MockWebSocket.instance?.simulateMessage({ type: "system", content: "Hello" });
  // Assert behavior
});
```

## Test Utilities

### Custom Render

`renderWithProviders()` wraps components with necessary providers:
- QueryClientProvider
- WebSocketProvider

### Mock WebSocket

`MockWebSocket` class simulates WebSocket behavior:
- `simulateMessage(data)` - Send message from server
- `simulateError()` - Trigger error event
- `sentMessages` - Array of sent messages

### Mock Fetch

`mockFetch(responses)` - Mock HTTP requests:

```tsx
mockFetch({
  "/api/endpoint": { success: true, data: "response" }
});
```

### Mock Data Factories

- `createMockUISpec()` - Generate test UI specs
- `createMockWindow()` - Generate test window states

## Best Practices

1. **Test Behavior, Not Implementation**
   - Focus on what the user sees and does
   - Avoid testing internal state directly

2. **Use Semantic Queries**
   - Prefer `getByRole`, `getByLabelText`, `getByText`
   - Avoid `getByTestId` unless necessary

3. **Clean Up**
   - Tests automatically clean up after each run
   - Use `beforeEach` to reset state

4. **Async Operations**
   - Use `async/await` with `userEvent`
   - Use `waitFor` for async state changes

5. **Isolation**
   - Each test should be independent
   - Reset stores and mocks in `beforeEach`

## Coverage

Run `npm run test:coverage` to generate coverage reports:
- Terminal summary
- HTML report in `coverage/` directory

Target coverage: 80%+ for critical paths

## Common Patterns

### Testing Form Submission

```tsx
it("submits form data", async () => {
  const user = userEvent.setup();
  const onSubmit = vi.fn();
  
  render(<Form onSubmit={onSubmit} />);
  
  await user.type(screen.getByLabelText("Name"), "John");
  await user.click(screen.getByRole("button", { name: /submit/i }));
  
  expect(onSubmit).toHaveBeenCalledWith({ name: "John" });
});
```

### Testing Async State Changes

```tsx
it("loads data on mount", async () => {
  mockFetch({ "/api/data": { items: ["A", "B"] } });
  
  render(<DataList />);
  
  expect(screen.getByText("Loading...")).toBeInTheDocument();
  
  await waitFor(() => {
    expect(screen.getByText("A")).toBeInTheDocument();
  });
});
```

### Testing Error States

```tsx
it("displays error message on failure", async () => {
  mockFetch({ "/api/data": { error: "Failed" } });
  
  render(<DataList />);
  
  await waitFor(() => {
    expect(screen.getByText(/failed/i)).toBeInTheDocument();
  });
});
```

## Troubleshooting

### Tests timeout
- Increase timeout in test: `it("test", { timeout: 10000 })`
- Or globally in `vitest.config.ts`

### Mock not working
- Ensure mock is set up in `beforeEach`
- Check if mock is being reset correctly

### Can't find element
- Check if element is rendered: `screen.debug()`
- Use `findBy` queries for async elements
- Check accessibility attributes

## Resources

- [Vitest Documentation](https://vitest.dev/)
- [React Testing Library](https://testing-library.com/react)
- [Testing Best Practices](https://kentcdodds.com/blog/common-mistakes-with-react-testing-library)

