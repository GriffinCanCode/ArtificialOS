# Journey Tracking System

## Overview

The Journey Tracking System provides debugging and observability capabilities for AgentOS by tracking user journeys across windows, apps, and components. It records the complete cause-and-effect chain of user actions, system responses, and cross-window interactions.

## Architecture

### Core Components

1. **MonitorProvider** - Top-level provider that initializes the monitoring system
2. **JourneyProvider** - Context-based provider for journey tracking in specific contexts
3. **WindowJourneyProvider** - Specialized journey provider for window-level tracking
4. **useJourney** - React hook for component-level journey tracking
5. **Causality Tracker** - Links events across the entire system to establish cause-and-effect relationships

### Data Flow

```
User Action (Click)
  → Causality Chain Started
    → Journey Step Added
      → API Call Made
        → Response Received
          → Window Opened
            → Journey Completed
```

## Integration Points

### 1. App Level (App.tsx)

```tsx
<MonitorProvider autoStart={true} desktopContext={{ environment: 'development' }}>
  <QueryClientProvider client={queryClient}>
    <WebSocketProvider>
      <AppContent />
    </WebSocketProvider>
  </QueryClientProvider>
</MonitorProvider>
```

**Purpose:**
- Initializes journey tracking system on app startup
- Sets desktop-level context (environment, session ID)
- Enables global monitoring features
- Manages tracker lifecycle and health monitoring

### 2. Window Level (Window.tsx)

```tsx
<WindowJourneyProvider
  windowId={window.id}
  windowTitle={window.title}
  appId={window.appId}
>
  {/* Window content */}
</WindowJourneyProvider>
```

**Purpose:**
- Automatically tracks when windows are opened/closed
- Records window focus changes
- Links journeys across multiple windows
- Provides window-specific context to all child components

### 3. Component Level (Any component)

```tsx
const journey = useJourney("ComponentName", true, "User opened component");

// Track user interactions
const handleClick = () => {
  journey.trackInteraction('submit-button', 'click');
  // Your click handler logic
};

// Track async operations
const handleAsync = async () => {
  try {
    const result = await someAsyncOperation();
    journey.trackResponse('operation_name', performance.now(), true);
  } catch (error) {
    journey.trackError(error);
  }
};
```

### 4. WebSocket Events (WebSocketContext.tsx)

```tsx
// Causality tracking for UI generation
const chainId = startCausalChain('user_action', 'User requested UI generation');
addCausalEvent('api_call', 'Sending request to AI service');
```

**Tracked events:**
- Chat messages sent
- UI generation requests
- Builder window creation
- WebSocket responses
- API state changes

## Current Implementation Status

### Implemented Features

1. **App.tsx**
   - MonitorProvider wrapping entire app
   - Desktop context initialization
   - Journey tracking in AppContent
   - Form submission tracking
   - App launch tracking with success/error handling

2. **Window.tsx**
   - WindowJourneyProvider per window
   - Automatic window lifecycle tracking
   - Cross-window journey correlation

3. **WebSocketContext.tsx**
   - Causality chains for chat messages
   - UI generation tracking
   - Builder window creation tracking
   - API call event tracking

### Tracked Events

**User Actions:**
- Form submissions
- Button clicks
- App launches (Native and Blueprint)
- Window operations (open, close, focus, minimize, maximize)

**System Responses:**
- API calls (fetch requests, WebSocket messages)
- Window navigation
- Success/failure outcomes
- Error occurrences with full context

**Performance Data:**
- Operation durations
- Response times
- Bottleneck identification

**Cross-Window Relationships:**
- Window parent-child relationships
- Communication patterns between windows
- Shared journey context across apps

## Usage Examples

### Example 1: Tracking a Complete User Flow

```tsx
// User opens app → types prompt → submits → UI generates → window opens

// 1. Journey starts automatically when AppContent mounts
const journey = useJourney("AppContent", true, "User opened AgentOS");

// 2. Form submission tracked
journey.trackInteraction('spotlight-form', 'submit');
journey.addStep('user_action', `User submitted prompt: "${message}"`, {
  promptLength: message.length,
});

// 3. WebSocket call tracked
const chainId = startCausalChain('user_action', 'User requested UI generation');
addCausalEvent('navigation', 'Opening builder window');
addCausalEvent('api_call', 'Sending UI generation request');

// 4. Window opens (automatically tracked by WindowJourneyProvider)
// 5. Response received (tracked by WebSocket handler)
// 6. Journey completes when window closes
```

### Example 2: Debugging Multi-Window Workflow

```tsx
// User opens Hub → clicks on app → new window opens → user interacts

// Access journey data in development console:
window.agentOSMonitoring.DEBUG.exportCausalityData();

// Returns complete chain:
{
  chainId: "chain_123456",
  rootCause: "User clicked app icon in Hub",
  events: [
    { type: "user_action", description: "User clicked app icon" },
    { type: "api_call", description: "Fetching app metadata" },
    { type: "navigation", description: "Opening window for app" },
    { type: "system_response", description: "App loaded successfully" }
  ]
}
```

### Example 3: Error Debugging

```tsx
// When an error occurs:
try {
  await launchApp(appId);
} catch (error) {
  // Error is automatically linked to the journey
  journey.trackError(error, { appId, action: 'launch_app' });
}

// In console, view journey with error:
window.agentOSMonitoring.DEBUG.getStats();

// Shows causality context:
{
  causality: {
    totalChains: 5,
    currentChainId: "chain_123456",
    context: {
      causalityRootCause: "User launching app: my-app",
      causalityEventCount: 3,
      causalityChainDuration: 145
    }
  }
}
```

## Debug Tools

### Development Console

```javascript
// Available in development mode via window.agentOSMonitoring

// Get current stats
await window.agentOSMonitoring.DEBUG.getStats();

// Export all causality chains
window.agentOSMonitoring.DEBUG.exportCausalityData();

// Get performance report
window.agentOSMonitoring.DEBUG.getPerformanceReport();

// Test the system
window.agentOSMonitoring.DEBUG.test();
```

### Viewing Journey Data

```typescript
import { useJourneyStore } from '@/core/monitoring/journey';

// Get current journey
const currentJourney = useJourneyStore.getState().getCurrentJourney();

// Get journey analytics
const analytics = useJourneyStore.getState().getAnalytics();

// Query journeys
const errorJourneys = useJourneyStore.getState().queryJourneys({ 
  hasErrors: true 
});

// Export journey data
const exportData = useJourneyStore.getState().export();
```

## Benefits

### Debugging Capability

Complete visibility into multi-window workflows with automatic cause-and-effect tracking. Full journey history available when errors occur.

### Performance Insights

Identify slow operations linked to user actions and detect performance bottlenecks across window boundaries.

### User Interaction Analysis

Understand which workflows are used most frequently and identify pain points in multi-step processes.

### Production Monitoring

Real-time health monitoring, automatic error reporting with full context, and detection of performance degradation patterns.

## Configuration

### Adjust Tracking Granularity

```tsx
<MonitorProvider 
  config={{
    features: {
      journeyTracking: true,        // User flow tracking
      causalityTracking: true,      // Cause-and-effect chains
      performanceMonitoring: true,  // Performance metrics
      errorTracking: true,          // Error capture
    },
    performance: {
      sampling: {
        journeys: 1.0,     // Track all journeys
        performance: 1.0,  // Track all perf metrics
      },
    },
  }}
>
```

### Privacy Controls

```tsx
<MonitorProvider 
  config={{
    privacy: {
      anonymizeUserData: true,
      excludeInputValues: true,   // Don't log form inputs
      excludePersonalInfo: true,
    },
  }}
>
```

## Performance Impact

- **Memory**: Approximately 5-10MB for 100 concurrent journeys
- **CPU**: Less than 2% overhead for tracking
- **Network**: Local processing only, no external calls
- **Storage**: Automatic cleanup of old journeys every 60 seconds

## Future Enhancements

- Backend persistence of journey data
- Real-time journey visualization dashboard
- Pattern analysis for common user workflows
- Journey export and replay for debugging
- Cross-session journey correlation

## Troubleshooting

### Journey not starting

Verify that MonitorProvider is wrapping your app:

```tsx
// Correct
<MonitorProvider>
  <App />
</MonitorProvider>

// Incorrect
<App>
  <MonitorProvider />
</App>
```

### Events not being linked

Ensure causality functions are called for major actions:

```tsx
// Start a chain for major actions
const chainId = startCausalChain('user_action', 'Description');

// Add events to the chain
addCausalEvent('api_call', 'Making API call');
```

### Memory usage growing

Journey store performs automatic cleanup every 60 seconds. Force cleanup manually:

```tsx
useJourneyStore.getState().cleanup();
```

## Summary

The Journey Tracking System is fully integrated into AgentOS and provides:

- Cross-window journey tracking
- Automatic causality chain linking
- Performance monitoring
- Error tracking with full context
- Debug tools in development mode
- Privacy-aware data collection

Start the app and journey tracking begins automatically. Access debugging tools via the browser console in development mode.
