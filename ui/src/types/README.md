# Type-Safe API Communication with Zod

This directory contains Zod schemas and TypeScript types for type-safe communication with the backend API.

## Overview

All API communication is validated at runtime using [Zod](https://zod.dev/) schemas, providing:
- **Runtime type validation**: Catch API contract mismatches immediately
- **TypeScript types**: Auto-generated types from Zod schemas
- **Type safety**: Compile-time and runtime guarantees
- **Better developer experience**: Auto-completion and inline documentation

## Files

- `api.ts` - All Zod schemas, TypeScript types, and validation helpers

## Usage

### WebSocket Communication

#### Using the WebSocketProvider (Recommended)

The easiest way to communicate with the backend is through the `WebSocketProvider`:

```tsx
import { WebSocketProvider, useWebSocket } from '../contexts/WebSocketContext';

function MyComponent() {
  const { sendChat, generateUI, isConnected } = useWebSocket();
  
  // Send a chat message
  const handleChat = () => {
    sendChat('Hello, AI!', { userId: '123' });
  };
  
  // Request UI generation
  const handleGenerateUI = () => {
    generateUI('create a calculator', {});
  };
  
  return (
    <div>
      <p>Status: {isConnected ? 'Connected' : 'Disconnected'}</p>
      <button onClick={handleChat}>Send Chat</button>
      <button onClick={handleGenerateUI}>Generate UI</button>
    </div>
  );
}

// Wrap your app
function App() {
  const handleMessage = (message: ServerMessage) => {
    console.log('Received:', message);
    
    switch (message.type) {
      case 'token':
        // Handle token
        break;
      case 'thought':
        // Handle thought
        break;
      // ... other types
    }
  };
  
  return (
    <WebSocketProvider onMessage={handleMessage}>
      <MyComponent />
    </WebSocketProvider>
  );
}
```

#### Using the WebSocket Client Directly

For more control, use the `WebSocketClient` directly:

```typescript
import { WebSocketClient } from '../utils/websocketClient';
import { ServerMessage } from '../types/api';

const client = new WebSocketClient({
  url: 'ws://localhost:8000/stream',
  autoReconnect: true,
  reconnectDelay: 1000
});

// Subscribe to messages
const unsubscribe = client.onMessage((message: ServerMessage) => {
  // All messages are validated and typed!
  switch (message.type) {
    case 'system':
      console.log('System:', message.message);
      break;
    case 'token':
      console.log('Token:', message.content);
      break;
    case 'thought':
      console.log('Thought:', message.content);
      break;
    case 'ui_generated':
      console.log('UI:', message.ui_spec.title);
      break;
  }
});

// Connect
client.connect();

// Send messages (type-safe!)
client.sendChat('Hello, world!', { foo: 'bar' });
client.generateUI('create a todo app');
client.ping();

// Clean up
unsubscribe();
client.disconnect();
```

### HTTP API Communication

Use the `APIClient` for HTTP requests:

```typescript
import APIClient from '../utils/apiClient';

// All methods return validated, typed responses!

// Health check
const health = await APIClient.health();
console.log(health.model); // ✅ Type-safe!
console.log(health.invalidField); // ❌ TypeScript error!

// Generate UI (non-streaming)
const result = await APIClient.generateUI({
  message: 'create a calculator',
  context: {}
});
if (result.ui_spec) {
  console.log(result.ui_spec.title);
}

// List apps
const apps = await APIClient.listApps();
apps.apps.forEach(app => {
  console.log(`${app.id}: ${app.title}`);
});

// Focus an app
await APIClient.focusApp('app-123');

// Close an app
await APIClient.closeApp('app-123');

// List services
const services = await APIClient.listServices();
console.log(services.services);

// Discover services
const discovered = await APIClient.discoverServices({
  message: 'I need storage',
  context: {}
});

// Execute a service tool
const toolResult = await APIClient.executeServiceTool({
  tool_id: 'storage.save',
  params: { key: 'user', value: { name: 'John' } },
  app_id: 'app-123'
});
```

## Message Types

### Client → Server (WebSocket)

- **`ChatMessage`**: Send a chat message
- **`GenerateUIMessage`**: Request UI generation
- **`PingMessage`**: Keep-alive ping

### Server → Client (WebSocket)

- **`SystemMessage`**: System notifications
- **`TokenMessage`**: Streaming LLM tokens
- **`ThoughtMessage`**: AI reasoning steps
- **`CompleteMessage`**: Stream completed
- **`GenerationStartMessage`**: UI generation started
- **`UIGeneratedMessage`**: UI generation complete
- **`ErrorMessage`**: Error occurred
- **`HistoryUpdateMessage`**: Chat history update
- **`PongMessage`**: Ping response

## Type Definitions

All types are auto-generated from Zod schemas using `z.infer`:

```typescript
// Example: ServerMessage is a discriminated union
type ServerMessage = 
  | SystemMessage
  | TokenMessage
  | ThoughtMessage
  | CompleteMessage
  | GenerationStartMessage
  | UIGeneratedMessage
  | ErrorMessage
  | HistoryUpdateMessage
  | PongMessage;

// Each message type has a unique 'type' field
// TypeScript can narrow the type based on it:
function handleMessage(msg: ServerMessage) {
  if (msg.type === 'ui_generated') {
    // TypeScript knows this is UIGeneratedMessage
    console.log(msg.ui_spec.title);
    console.log(msg.app_id);
  }
}
```

## Validation Helpers

### `parseServerMessage(data: unknown): ServerMessage | null`

Safely parse and validate server messages:

```typescript
import { parseServerMessage } from '../types/api';

const data = JSON.parse(event.data);
const message = parseServerMessage(data);

if (message) {
  // Validated and typed!
  console.log(message.type);
} else {
  // Invalid message format
  console.error('Failed to parse message');
}
```

### Creating Messages

Use the helper functions to create type-safe messages:

```typescript
import { 
  createChatMessage,
  createGenerateUIMessage,
  createPingMessage 
} from '../types/api';

const chatMsg = createChatMessage('Hello!', { user: 'john' });
const uiMsg = createGenerateUIMessage('create a calculator', {});
const pingMsg = createPingMessage();
```

## Error Handling

All validation errors are logged to the console:

```typescript
// Invalid message will:
// 1. Log error to console
// 2. Return null (or throw, depending on context)
const message = parseServerMessage({ invalid: 'data' });
// Console: "Failed to parse server message: ..."
// message === null
```

## Extending

To add new message types:

1. **Define the Zod schema**:
```typescript
export const NewMessageSchema = z.object({
  type: z.literal('new_message'),
  data: z.string()
});
```

2. **Add to discriminated union**:
```typescript
export const ServerMessageSchema = z.discriminatedUnion('type', [
  // ... existing schemas
  NewMessageSchema
]);
```

3. **Export the TypeScript type**:
```typescript
export type NewMessage = z.infer<typeof NewMessageSchema>;
```

4. **Types are automatically updated everywhere!** ✨

## Benefits

### Before (Without Zod)
```typescript
// ❌ No validation
const data = JSON.parse(event.data);
console.log(data.message); // What if 'message' doesn't exist?

// ❌ No type safety
function handleMessage(msg: any) {
  if (msg.type === 'ui_generatd') { // Typo! No error.
    console.log(msg.ui_spce.title); // Another typo!
  }
}
```

### After (With Zod)
```typescript
// ✅ Runtime validation
const message = parseServerMessage(JSON.parse(event.data));
if (message) {
  console.log(message.message); // TypeScript knows this exists!
}

// ✅ Type safety + auto-completion
function handleMessage(msg: ServerMessage) {
  if (msg.type === 'ui_generatd') { // ❌ TypeScript error!
    console.log(msg.ui_spce.title); // ❌ TypeScript error!
  }
  if (msg.type === 'ui_generated') { // ✅ Correct!
    console.log(msg.ui_spec.title); // ✅ Auto-completion!
  }
}
```

## Best Practices

1. **Always use the provided clients**: `WebSocketClient` and `APIClient` handle validation automatically
2. **Handle all message types**: Use exhaustive switch statements with TypeScript's discriminated unions
3. **Check validation results**: Always check if `parseServerMessage` returns `null`
4. **Don't bypass validation**: Never use `as any` or cast types – let Zod do its job
5. **Keep schemas up to date**: When the backend API changes, update the schemas immediately

## Resources

- [Zod Documentation](https://zod.dev/)
- [TypeScript Discriminated Unions](https://www.typescriptlang.org/docs/handbook/2/narrowing.html#discriminated-unions)
- [Backend API Documentation](../../ai-service/src/main.py)

