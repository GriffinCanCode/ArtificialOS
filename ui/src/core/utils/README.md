# Frontend Logging System

This directory contains the logging infrastructure for the AI-OS frontend. The logging system is built on `electron-log` and provides comprehensive logging capabilities across both Electron main and renderer processes.

## Features

- **Unified API**: Single logger interface for both main and renderer processes
- **Structured Logging**: Support for contextual metadata in all log messages
- **Multiple Log Levels**: ERROR, WARN, INFO, DEBUG, VERBOSE
- **File & Console Output**: Logs written to both file and console
- **Performance Tracking**: Built-in performance measurement utilities
- **React Integration**: Custom hooks for component-specific logging
- **Error Serialization**: Automatic serialization of Error objects

## Log Files

Logs are automatically written to:
- **macOS**: `~/Library/Logs/ai-os-ui/main.log`
- **Windows**: `%USERPROFILE%\AppData\Roaming\ai-os-ui\logs\main.log`
- **Linux**: `~/.config/ai-os-ui/logs/main.log`

## Usage

### Basic Usage in React Components

```typescript
import { logger } from '@/utils/logger';

function MyComponent() {
  useEffect(() => {
    logger.info('Component initialized', { 
      component: 'MyComponent',
      timestamp: Date.now() 
    });
  }, []);

  const handleClick = () => {
    logger.interaction('Button clicked', 'submit-button');
  };

  return <button onClick={handleClick}>Submit</button>;
}
```

### Using the React Hook

```typescript
import { useLogger } from '@/utils/useLogger';

function ChatInterface({ userId }) {
  const log = useLogger('ChatInterface', { userId });

  const sendMessage = async (message: string) => {
    log.info('Sending message', { messageLength: message.length });
    
    try {
      await api.sendMessage(message);
      log.info('Message sent successfully');
    } catch (error) {
      log.error('Failed to send message', error);
    }
  };

  return <div>...</div>;
}
```

### Performance Logging

```typescript
import { useLogger } from '@/utils/useLogger';

function DataTable({ data }) {
  const log = useLogger('DataTable');

  useEffect(() => {
    const startTime = performance.now();
    
    // Expensive operation
    processData(data);
    
    const duration = performance.now() - startTime;
    log.performance('Data processing', duration);
  }, [data]);

  return <table>...</table>;
}
```

### Logging Async Operations

```typescript
import { withLogging } from '@/utils/useLogger';

const fetchUserData = withLogging(
  async (userId: string) => {
    const response = await fetch(`/api/users/${userId}`);
    return response.json();
  },
  'fetchUserData',
  { service: 'api' }
);
```

### API Call Logging

```typescript
import { logger } from '@/utils/logger';

async function apiRequest(method: string, endpoint: string, data?: any) {
  logger.api(method, endpoint);
  
  try {
    const response = await fetch(endpoint, {
      method,
      body: JSON.stringify(data)
    });
    
    logger.api(method, endpoint, response.status);
    return response.json();
  } catch (error) {
    logger.error('API request failed', error, { method, endpoint });
    throw error;
  }
}
```

### WebSocket Logging

```typescript
import { logger } from '@/utils/logger';

const ws = new WebSocket('ws://localhost:8080');

ws.onopen = () => {
  logger.websocket('connected');
};

ws.onmessage = (event) => {
  logger.websocket('message', { 
    type: event.data.type,
    size: event.data.size 
  });
};

ws.onerror = (error) => {
  logger.error('WebSocket error', error);
};
```

### Child Loggers

Create child loggers with persistent context:

```typescript
import { logger } from '@/utils/logger';

const authLogger = logger.child({ 
  module: 'authentication',
  version: '1.0.0' 
});

authLogger.info('User login attempt', { username: 'john' });
// Automatically includes module and version in the log
```

### Setting Global Context

```typescript
import { logger } from '@/utils/logger';

// Set persistent context at app initialization
logger.setContext({
  sessionId: generateSessionId(),
  appVersion: '1.0.0',
  environment: process.env.NODE_ENV
});

// All subsequent logs will include this context
logger.info('Application started'); 
// Includes sessionId, appVersion, environment
```

## Log Levels

Choose the appropriate log level for your message:

- **ERROR**: Application errors, exceptions, failed operations
- **WARN**: Warnings, deprecated features, potential issues
- **INFO**: General application flow, user actions, significant events
- **DEBUG**: Detailed information for debugging (not in production)
- **VERBOSE**: Very detailed information, typically only for development

## Configuration

The logger is configured in `main.js`:

```javascript
log.transports.file.level = 'debug';
log.transports.console.level = 'debug';
log.transports.file.maxSize = 10 * 1024 * 1024; // 10MB
```

Adjust these settings based on your needs.

## Best Practices

1. **Include Context**: Always provide relevant context with your logs
   ```typescript
   log.info('User action', { action: 'submit', formId: 'contact' });
   ```

2. **Use Appropriate Levels**: Don't use `info` for debugging information
   
3. **Log Errors Properly**: Pass the actual Error object, not just the message
   ```typescript
   try {
     // ...
   } catch (error) {
     log.error('Operation failed', error, { operationId: 123 });
   }
   ```

4. **Performance Logging**: Track expensive operations
   ```typescript
   const start = performance.now();
   await expensiveOperation();
   log.performance('Expensive operation', performance.now() - start);
   ```

5. **Structured Data**: Use objects for complex data
   ```typescript
   log.info('User profile updated', {
     userId: user.id,
     fields: ['email', 'name'],
     timestamp: Date.now()
   });
   ```

6. **Avoid Logging Sensitive Data**: Never log passwords, tokens, or PII
   ```typescript
   // ❌ Bad
   log.info('Login', { password: user.password });
   
   // ✅ Good
   log.info('Login attempt', { username: user.username });
   ```

## Troubleshooting

### Logs not appearing

1. Check that electron-log is imported correctly
2. Verify log level settings allow your messages
3. Check file permissions for log directory

### Finding log files

Run in the Electron main process:
```javascript
console.log('Log path:', log.transports.file.getFile().path);
```

Or check the standard locations listed above.

### Performance issues

If logging impacts performance:
1. Reduce log level in production
2. Use asynchronous logging for heavy operations
3. Limit context data size

