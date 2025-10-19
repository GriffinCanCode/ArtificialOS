# AgentOS Monitoring System v2.0

**Production-grade, intelligent monitoring system for the AgentOS frontend.**

Unified logging, metrics collection, journey tracking, causality chains, and hierarchical context tracking in one cohesive, extensible system.

---

## 📁 Architecture

The monitoring system follows a **modular, feature-based architecture** with clear separation of concerns:

```
monitoring/
├── core/              # Core infrastructure
├── hooks/             # React integration
├── journey/           # End-to-end user tracking
├── tracker/           # System orchestration
├── causality/         # Cause-effect chains
├── context/           # Hierarchical context
└── visualization/     # Dashboard & charts
```

### Core Principles

1. **Elegance through Simplicity** - Each module does one thing exceptionally well
2. **Extensibility** - Plugin-based architecture for custom features
3. **Type Safety** - Comprehensive TypeScript types throughout
4. **Zero Runtime Cost** - Intelligent sampling and batching
5. **Developer Experience** - Clean APIs, excellent debugging tools

---

## 🚀 Quick Start

```tsx
import { MonitorProvider, useLogger, useJourney } from '@/core/monitoring';

function App() {
  return (
    <MonitorProvider autoStart={true}>
      <YourApp />
    </MonitorProvider>
  );
}

function YourComponent() {
  const log = useLogger('YourComponent');
  const journey = useJourney('YourComponent', true, 'User opened component');

  const handleClick = () => {
    log.info('Button clicked');
    journey.trackInteraction('button', 'click');
  };

  return <button onClick={handleClick}>Click me</button>;
}
```

---

## 📦 Modules

### Core (`core/`)

**Foundational infrastructure for the entire monitoring system.**

```typescript
import { logger, metricsCollector, performanceMonitor, initWebVitals } from '@/core/monitoring/core';

// Structured logging
logger.info('Operation completed', { userId: '123', duration: 45 });

// Metrics collection (Prometheus-compatible)
metricsCollector.incCounter('api_calls_total', 1, { endpoint: '/api/data' });
metricsCollector.observeHistogram('request_duration', 0.145);

// Performance monitoring
performanceMonitor.start('expensive_operation');
// ... do work ...
performanceMonitor.end('expensive_operation');

// Web Vitals
initWebVitals(); // Auto-tracks LCP, FID, CLS, FCP, TTFB, INP
```

**Files:**
- `logger.ts` - Structured logging with hierarchical context
- `buffer.ts` - High-performance log buffering (batching, priority lanes)
- `config.ts` - Environment-based configuration
- `metrics.ts` - Prometheus-compatible metrics collector
- `vitals.ts` - Core Web Vitals tracking
- `performance.ts` - Performance monitoring utilities
- `types.ts` - Shared type definitions

---

### Hooks (`hooks/`)

**React integration for component-level monitoring.**

```typescript
import { useLogger, useJourney, useTracker } from '@/core/monitoring/hooks';

// Component-scoped logging
const log = useLogger('MyComponent', { feature: 'payments' });
log.info('Payment initiated');

// Journey tracking
const journey = useJourney('PaymentFlow', true, 'User started checkout');
journey.trackInteraction('pay-button', 'click');
journey.completeJourney('payment_successful');

// System tracker
const tracker = useTracker(true);
console.log('Health score:', tracker.health.score);
```

**Files:**
- `useLogger.ts` - Component-scoped logger hook
- `useJourney.ts` - Journey tracking hook (+ interaction, async, performance variants)
- `useTracker.ts` - System tracker hook (+ feature, plugin, health variants)

---

### Journey (`journey/`)

**End-to-end user interaction tracking across windows and apps.**

Tracks complete user flows from initial action to completion, spanning multiple windows, apps, and components. Essential for understanding user behavior and debugging complex multi-step issues.

```typescript
import { useJourneyStore, JourneyProvider } from '@/core/monitoring/journey';

// Zustand store
const { startJourney, addStep, completeJourney } = useJourneyStore();

const journeyId = startJourney('User checkout flow', {
  windowId: 'win_123',
  appId: 'payments'
});

addStep('user_action', 'Selected payment method', {
  interaction: { element: 'paypal-button' }
});

addStep('system_response', 'Payment processed', {
  systemResponse: { operation: 'process_payment', duration: 245, success: true }
});

completeJourney(journeyId, 'completed');

// Provider for automatic tracking
<JourneyProvider contextName="CheckoutFlow" autoStart={true}>
  <CheckoutForm />
</JourneyProvider>
```

**Features:**
- Cross-window journey correlation
- Performance bottleneck detection
- Pattern recognition (single_task, multi_window, workflow, etc.)
- Experience classification (excellent, good, poor, etc.)
- Analytics and trend analysis

**Files:**
- `store.ts` - Zustand store for journey state
- `hooks.ts` - React hooks (re-exported from `../hooks/useJourney.ts`)
- `types.ts` - Journey type definitions
- `providers.tsx` - React providers (Journey, Window, App, Form)

---

### Tracker (`tracker/`)

**Overall monitoring system orchestration and plugin management.**

Central coordinator that manages all monitoring features, plugins, and system health.

```typescript
import { useTrackerStore, MonitorProvider } from '@/core/monitoring/tracker';

// System configuration
const tracker = useTrackerStore();
tracker.updateConfig({
  features: {
    journeyTracking: true,
    causalityTracking: true,
    performanceMonitoring: true,
  },
  performance: {
    sampling: { journeys: 1.0, performance: 0.1 }
  }
});

// Plugin system
await tracker.registerPlugin({
  meta: { name: 'CustomAnalytics', version: '1.0.0' },
  hooks: {
    onEvent: (event) => {
      // Custom event processing
    }
  }
});

// Health monitoring
const health = tracker.getHealth();
console.log('Overall:', health.overall); // 'healthy' | 'degraded' | 'critical'
console.log('Score:', health.score); // 0-100
```

**Files:**
- `store.ts` - Zustand store for tracker state
- `hooks.ts` - React hooks (re-exported from `../hooks/useTracker.ts`)
- `types.ts` - Tracker type definitions
- `providers.tsx` - MonitorProvider (top-level orchestration)

---

### Causality (`causality/`)

**Cause-and-effect chain tracking for debugging complex async flows.**

Tracks how user actions propagate through the system, essential for debugging multi-step operations and understanding system behavior.

```typescript
import { startCausalChain, addCausalEvent, completeCausalEvent } from '@/core/monitoring/causality';

// Start chain
const chainId = startCausalChain('user_action', 'User clicked checkout button', {
  componentId: 'CheckoutButton',
  windowId: 'win_123'
});

// Add events as they occur
const eventId1 = addCausalEvent('api_call', 'Fetching payment methods');
completeCausalEvent(eventId1);

const eventId2 = addCausalEvent('state_change', 'Updated payment UI');
completeCausalEvent(eventId2);

// End chain
endCurrentChain();

// Analyze chains
const chains = causalityTracker.getChains({ hasError: true });
chains.forEach(chain => {
  console.log(chain.exportChain(chain.id));
});
```

**Features:**
- Automatic chain propagation across async boundaries
- Timeline reconstruction
- Performance impact analysis
- Cross-boundary tracking (components, WebSocket, APIs)
- Memory-bounded with auto-cleanup

**Files:**
- `tracker.ts` - Causality tracker implementation
- `index.ts` - Clean exports with type re-exports

---

### Context (`context/`)

**Hierarchical context tracking: Desktop > Window > App > Component.**

Automatically tracks UI hierarchy so every log knows exactly where it came from. Makes debugging multi-window, multi-app systems trivial.

```typescript
import { 
  WindowContextProvider, 
  AppContextProvider, 
  useComponentTracking,
  getHierarchicalLogContext 
} from '@/core/monitoring/context';

// Automatic context
<WindowContextProvider
  windowId="win_123"
  appId="payments"
  title="Payment Portal"
  appType="native_web"
  zIndex={100}
  isFocused={true}
>
  <AppContextProvider
    appId="payments"
    instanceId="pay_001"
    type="native_web"
  >
    <PaymentForm />
  </AppContextProvider>
</WindowContextProvider>

// Manual tracking
function MyComponent() {
  useComponentTracking('MyComponent', 'react_component');
  
  // All logs now include hierarchy
  logger.info('Event occurred'); // Includes window, app, component context
}

// Debugging
const context = getHierarchicalLogContext();
console.log(context.breadcrumbPath); // "window:abc123 > app:payments > component:MyComponent"
```

**Features:**
- Automatic context propagation
- Breadcrumb path generation
- Development debugger (Ctrl/Cmd + Shift + L)
- Zero performance impact
- Clean HOC wrappers

**Files:**
- `store.ts` - Zustand store for hierarchical context
- `providers.tsx` - React providers (Window, App, Component)
- `index.ts` - Clean exports

---

### Visualization (`visualization/`)

**Dashboard, charts, and metrics visualization.**

```typescript
import { 
  LiveMetricsDashboard, 
  getAllMetrics, 
  downloadMetrics 
} from '@/core/monitoring/visualization';

// Real-time dashboard
<LiveMetricsDashboard />

// Programmatic access
const metrics = await getAllMetrics();
console.log('UI metrics:', metrics.ui);
console.log('Web Vitals:', metrics.webVitals);
console.log('Backend:', metrics.backend);

// Export for analysis
downloadMetrics(); // Downloads JSON file

// Console helpers
window.agentOSMetrics.log(); // Pretty print all metrics
window.agentOSMetrics.openDashboard(); // Open web dashboard
```

**Components:**
- Performance Chart (latency trends)
- Error Rate Chart (error tracking)
- Tool Execution Chart (tool usage)
- Web Vitals Chart (Core Web Vitals)
- Live Metrics Dashboard (real-time overview)

**Files:**
- `dashboard.ts` - Dashboard API and utilities
- `charts.tsx` - React chart components (Recharts)
- `index.ts` - Clean exports

---

## 🎯 Advanced Patterns

### Plugin Development

```typescript
import { TrackerPlugin } from '@/core/monitoring/tracker';

const customPlugin: TrackerPlugin = {
  meta: {
    name: 'CustomAnalytics',
    version: '1.0.0',
    description: 'Custom analytics integration',
    author: 'Your Team'
  },
  
  hooks: {
    async onInitialize() {
      // Plugin setup
    },
    
    async onEvent(event) {
      // Process monitoring events
      if (event.type === 'journey_completed') {
        // Send to analytics service
      }
    },
    
    async onError(error) {
      // Handle plugin errors
    }
  },
  
  api: {
    trackCustomMetric(name: string, value: number) {
      // Custom API methods
    }
  }
};

// Register plugin
const tracker = useTrackerStore();
await tracker.registerPlugin(customPlugin);
```

### Custom Metrics

```typescript
import { metricsCollector } from '@/core/monitoring/core';

// Counter (monotonically increasing)
metricsCollector.incCounter('user_signups_total', 1);

// Gauge (can go up or down)
metricsCollector.setGauge('active_users', 42);
metricsCollector.incGauge('queue_size', 5);
metricsCollector.decGauge('queue_size', 2);

// Histogram (distribution)
metricsCollector.observeHistogram('api_latency_seconds', 0.145);

// With labels
metricsCollector.incCounter('http_requests_total', 1, {
  method: 'GET',
  endpoint: '/api/users',
  status: '200'
});
```

### Error Tracking with Context

```typescript
import { logger } from '@/core/monitoring/core';
import { addCausalEvent } from '@/core/monitoring/causality';

try {
  // Start tracking operation
  const eventId = addCausalEvent('async_operation', 'Processing payment');
  
  // Do work
  await processPayment();
  
  completeCausalEvent(eventId);
} catch (error) {
  // Full context automatically included
  logger.error('Payment failed', error, {
    paymentId: 'pay_123',
    amount: 99.99,
    userId: 'user_456'
  });
  
  // Causality chain preserved
  completeCausalEvent(eventId, error);
}
```

---

## 🧪 Testing

```typescript
import { useJourneyStore, useTrackerStore } from '@/core/monitoring';
import { logger } from '@/core/monitoring/core';

describe('Monitoring Integration', () => {
  beforeEach(() => {
    // Reset state
    useJourneyStore.getState().reset();
    useTrackerStore.getState().reset();
  });
  
  it('tracks journey completion', () => {
    const { startJourney, completeJourney, getJourney } = useJourneyStore.getState();
    
    const journeyId = startJourney('Test journey');
    completeJourney(journeyId, 'completed');
    
    const journey = getJourney(journeyId);
    expect(journey?.classification.outcome).toBe('completed');
  });
  
  it('logs with proper context', () => {
    const spy = jest.spyOn(console, 'info');
    logger.info('Test message', { test: true });
    
    expect(spy).toHaveBeenCalledWith(
      expect.stringContaining('Test message'),
      expect.objectContaining({ test: true })
    );
  });
});
```

---

## 🔧 Configuration

Environment-based configuration with intelligent defaults:

```typescript
import { createLoggingConfig } from '@/core/monitoring/core';

const config = createLoggingConfig('production')
  .setLevel('info')
  .setConsole(false)
  .setBackendStream(true)
  .setPerformance({
    sampleRate: 0.1, // Sample 10% of operations
    logSlowOperations: true,
    slowThresholdMs: 500
  })
  .build();
```

**Environment Variables:**
- `LOG_LEVEL` - Log level (error, warn, info, debug, verbose)
- `LOG_CONSOLE` - Enable console output (true/false)
- `LOG_BACKEND` - Enable backend streaming (true/false)
- `LOG_BUFFER_SIZE` - Buffer size before flush
- `LOG_SAMPLE_RATE` - Performance sampling rate (0.0-1.0)

---

## 📊 Performance

**Design Goals:**
- **<2% CPU overhead** - Adaptive sampling, efficient buffering
- **<10MB memory** - Bounded buffers, automatic cleanup
- **<50ms latency** - Batch processing, priority lanes
- **Production-ready** - Battle-tested patterns, zero crashes

**Optimizations:**
- Batched log processing (100ms intervals)
- Priority lanes (errors flush immediately)
- Memory-bounded buffers (automatic eviction)
- SIMD-optimized JSON parsing (>1KB payloads)
- Intelligent sampling (high-frequency events)

---

## 🐛 Debugging

### Development Tools

```typescript
// Access global debug utilities
window.agentOSMonitoring.DEBUG.test(); // Run system test
window.agentOSMonitoring.DEBUG.getStats(); // Get comprehensive stats
window.agentOSMonitoring.DEBUG.enableDebugMode(); // Enable debug visualizations

// Quick access
window.agentOSMonitoring.log('Test message');
window.agentOSMonitoring.startChain('user_action', 'Test chain');
```

### Context Debugger

Press **Ctrl/Cmd + Shift + L** in development to toggle the context debugger overlay.

Shows:
- Current hierarchical context (Desktop > Window > App > Component)
- Active causality chains
- Journey tracking status
- System health metrics

### Breadcrumb Visualization

Add `<ContextBreadcrumbs />` to your component tree to see real-time context path.

---

## 🔥 Migration from v1.0

**Before:**
```typescript
import { logger } from '@/core/utils/monitoring/logger';
import { initWebVitals } from '@/core/monitoring';
```

**After:**
```typescript
import { logger } from '@/core/monitoring/core/logger';
import { initWebVitals } from '@/core/monitoring';
```

**Automated Migration:**
All imports have been automatically updated. The API surface is backward-compatible.

---

## 📚 API Reference

See individual module README files and inline TypeScript documentation for comprehensive API details.

**Key Exports:**
- Core: `logger`, `metricsCollector`, `performanceMonitor`, `initWebVitals`
- Hooks: `useLogger`, `useJourney`, `useTracker`
- Stores: `useJourneyStore`, `useTrackerStore`, `useHierarchicalContextStore`
- Providers: `MonitorProvider`, `JourneyProvider`, `WindowContextProvider`
- Causality: `startCausalChain`, `addCausalEvent`, `causalityTracker`
- Visualization: `LiveMetricsDashboard`, `getAllMetrics`, `downloadMetrics`

---

## 🤝 Contributing

When adding new features:

1. **Follow the module pattern** - Keep files focused and concise (<500 lines)
2. **Export through index.ts** - Maintain clean barrel exports
3. **Add TypeScript types** - Strong typing prevents bugs
4. **Document with JSDoc** - Help others understand your code
5. **Write tests** - Ensure reliability
6. **Update this README** - Keep documentation current

---

## 📄 License

Part of the AgentOS project. See root LICENSE file.

---

**Built with ❤️ by the AgentOS Team**

v2.0.0 | 2025


