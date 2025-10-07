# Native Apps Directory

This directory contains native TypeScript/React applications for the OS.

## Structure

```
native/
├── vite.config.base.ts    # Shared Vite configuration base
├── README.md              # This file
└── <app-id>/              # Individual apps
    ├── manifest.json      # App metadata
    ├── package.json       # Dependencies
    ├── vite.config.ts     # Build config (extends base)
    ├── tsconfig.json      # TypeScript config
    ├── src/               # Source code
    └── README.md          # App documentation
```

## Creating a New App

```bash
# Using Makefile (recommended)
make create-native-app name="My App"

# Or directly
./scripts/create-native-app.sh "My App"
```

## Development

```bash
# Watch single app with HMR
make watch-native-app name=my-app

# Watch all apps
make watch-native-apps

# Build all apps
make build-native-apps

# Validate apps
make validate-native-apps

# Lint apps
make lint-native-apps
make fix-native-apps    # Auto-fix issues
```

## What Are Native Apps?

Native apps are **full React applications** with:

- ✅ Complete freedom to use any npm packages
- ✅ Custom React components (NO prebuilt components)
- ✅ Full TypeScript support
- ✅ Hot Module Replacement (HMR)
- ✅ Access to OS APIs via SDK

**Unlike Blueprint apps**, they:

- ❌ Do NOT use JSON definitions
- ❌ Do NOT use prebuilt UI components
- ✅ Are hand-coded TypeScript/React

## Build Output

Apps are built as ES modules and output to `../dist/<app-id>/`:

```
dist/
└── my-app/
    ├── index.js        # Main bundle
    └── assets/         # CSS and other assets
```

## Shared Configuration

All apps extend the shared Vite configuration (`vite.config.base.ts`) which provides:

- **Fast Refresh**: React HMR
- **Optimized Builds**: Tree-shaking, minification
- **External Dependencies**: React/ReactDOM shared with host
- **TypeScript**: Full type checking
- **Code Splitting**: Intelligent chunking

## Documentation

- **Developer Guide**: `../../docs/NATIVE_APPS_DEV_GUIDE.md`
- **SDK Reference**: `../../ui/src/core/sdk/index.ts`
- **Architecture**: `../../docs/NATIVE_APPS_PLAN.md`

## Best Practices

1. **Use the SDK**: Access OS services via `context.executor`
2. **Type Safety**: Enable TypeScript strict mode
3. **Small Components**: Keep components focused and testable
4. **State Management**: Use `context.state` for app-level state
5. **Error Handling**: Gracefully handle service call failures
6. **Performance**: Use React.memo, useMemo, useCallback

## Available Services

Apps have access to all OS services:

- **Filesystem**: Read, write, list, create, delete files
- **Storage**: Persistent key-value storage
- **HTTP**: Make external API calls
- **System**: Get system info, time, logging
- **UI**: Toasts, notifications, window controls

See the Developer Guide for detailed API reference.

## Getting Help

1. Read `NATIVE_APPS_DEV_GUIDE.md`
2. Check example apps in this directory
3. Review SDK source code
4. Run validation: `make validate-native-apps`

---

**Happy Building! 🚀**
