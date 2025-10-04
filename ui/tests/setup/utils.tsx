/**
 * Test Utilities
 * Custom render functions and test helpers
 */

import React, { ReactElement } from "react";
import { render, RenderOptions } from "@testing-library/react";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { WebSocketProvider } from "../../src/contexts/WebSocketContext";

// Create a custom render function that includes providers
interface AllTheProvidersProps {
  children: React.ReactNode;
}

function AllTheProviders({ children }: AllTheProvidersProps) {
  // Create a new QueryClient for each test to ensure isolation
  const queryClient = new QueryClient({
    defaultOptions: {
      queries: {
        retry: false, // Disable retries in tests
        cacheTime: 0, // Disable caching in tests
      },
    },
  });

  return (
    <QueryClientProvider client={queryClient}>
      <WebSocketProvider>{children}</WebSocketProvider>
    </QueryClientProvider>
  );
}

export interface CustomRenderOptions extends Omit<RenderOptions, "wrapper"> {
  // Add custom options here if needed
}

export function renderWithProviders(
  ui: ReactElement,
  options?: CustomRenderOptions
) {
  return render(ui, { wrapper: AllTheProviders, ...options });
}

// Re-export everything from React Testing Library
export * from "@testing-library/react";
export { renderWithProviders as render };

// Custom user event helpers
export { default as userEvent } from "@testing-library/user-event";

/**
 * Wait for async operations with a reasonable timeout
 */
export const waitFor = async (callback: () => void, timeout = 3000) => {
  const { waitFor: rtlWaitFor } = await import("@testing-library/react");
  return rtlWaitFor(callback, { timeout });
};

/**
 * Mock WebSocket for testing
 */
export class MockWebSocket {
  static instance: MockWebSocket | null = null;
  public readyState = WebSocket.OPEN;
  public onopen: ((event: Event) => void) | null = null;
  public onclose: ((event: CloseEvent) => void) | null = null;
  public onmessage: ((event: MessageEvent) => void) | null = null;
  public onerror: ((event: Event) => void) | null = null;
  public sentMessages: any[] = [];

  constructor(public url: string) {
    MockWebSocket.instance = this;
    setTimeout(() => {
      if (this.onopen) {
        this.onopen(new Event("open"));
      }
    }, 0);
  }

  send(data: string) {
    this.sentMessages.push(JSON.parse(data));
  }

  close() {
    this.readyState = WebSocket.CLOSED;
    if (this.onclose) {
      this.onclose(new CloseEvent("close"));
    }
  }

  // Simulate receiving a message
  simulateMessage(data: any) {
    if (this.onmessage) {
      this.onmessage(
        new MessageEvent("message", {
          data: JSON.stringify(data),
        })
      );
    }
  }

  // Simulate error
  simulateError() {
    if (this.onerror) {
      this.onerror(new Event("error"));
    }
  }

  static reset() {
    MockWebSocket.instance = null;
  }
}

/**
 * Create mock fetch responses
 */
export function createMockResponse<T>(data: T, status = 200): Response {
  return {
    ok: status >= 200 && status < 300,
    status,
    json: async () => data,
    text: async () => JSON.stringify(data),
    headers: new Headers(),
  } as Response;
}

/**
 * Mock fetch helper
 */
export function mockFetch(responses: Record<string, any>) {
  global.fetch = vi.fn((url: string) => {
    const response = responses[url];
    if (!response) {
      return Promise.resolve(
        createMockResponse({ error: "Not found" }, 404)
      );
    }
    return Promise.resolve(createMockResponse(response));
  }) as any;
}

/**
 * Reset all mocks
 */
export function resetAllMocks() {
  vi.clearAllMocks();
  MockWebSocket.reset();
}

/**
 * Create a mock UISpec for testing
 */
export function createMockUISpec(overrides?: Partial<any>) {
  return {
    type: "app",
    title: "Test App",
    layout: "vertical",
    components: [
      {
        type: "text",
        id: "test-text",
        props: {
          content: "Test Content",
        },
      },
    ],
    ...overrides,
  };
}

/**
 * Create a mock window state for testing
 */
export function createMockWindow(overrides?: Partial<any>) {
  return {
    id: "window-1",
    appId: "app-1",
    title: "Test Window",
    uiSpec: createMockUISpec(),
    position: { x: 100, y: 100 },
    size: { width: 800, height: 600 },
    isMinimized: false,
    isFocused: true,
    zIndex: 1,
    ...overrides,
  };
}

/**
 * Flush all promises
 */
export const flushPromises = () => new Promise((resolve) => setImmediate(resolve));

