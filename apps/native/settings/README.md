# Settings

A native TypeScript/React application for the OS.

## Development

```bash
# Install dependencies
npm install

# Start development server (with HMR)
npm run dev

# Build for production
npm run build

# Type check
npm run type-check

# Lint code
npm run lint
npm run lint:fix

# Format code
npm run format
```

## Structure

```
settings/
├── manifest.json      # App metadata and configuration
├── package.json       # npm dependencies and scripts
├── tsconfig.json      # TypeScript configuration
├── vite.config.ts     # Build configuration
├── src/
│   ├── index.tsx      # Entry point (exports default component)
│   ├── App.tsx        # Main application component
│   ├── components/    # Reusable components
│   ├── hooks/         # Custom React hooks
│   └── styles/        # CSS styles
└── README.md          # This file
```

## Available APIs

The app has access to the OS SDK via the `context` prop:

- **State Management**: `context.state` - Reactive state store
- **Service Calls**: `context.executor` - Execute backend services
- **Window Controls**: `context.window` - Control the app window

### Example Service Calls

```typescript
// Filesystem
await context.executor.execute('filesystem.read', { path: '/path/to/file' });
await context.executor.execute('filesystem.write', { path: '/path', content: 'data' });
await context.executor.execute('filesystem.list', { path: '/path' });

// Storage
await context.executor.execute('storage.set', { key: 'mykey', value: data });
const result = await context.executor.execute('storage.get', { key: 'mykey' });

// HTTP
await context.executor.execute('http.get', { url: 'https://api.example.com' });
```

## Building

The app is built as an ES module that exports a React component. The build output goes to `../../dist/settings/`.

## License

MIT
