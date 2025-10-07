#!/bin/bash
# Create a new native TypeScript/React app from template
# Usage: ./create-native-app.sh "My App Name"

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

# Check arguments
if [ $# -eq 0 ]; then
  echo "Usage: $0 <app-name>"
  echo "Example: $0 \"My Awesome App\""
  exit 1
fi

APP_NAME=$1
APP_ID=$(echo "$APP_NAME" | tr '[:upper:]' '[:lower:]' | tr ' ' '-')
APP_DIR="$PROJECT_ROOT/apps/native/$APP_ID"

echo "======================================"
echo "Creating Native App"
echo "======================================"
echo "Name:        $APP_NAME"
echo "ID:          $APP_ID"
echo "Directory:   $APP_DIR"
echo ""

# Check if app already exists
if [ -d "$APP_DIR" ]; then
  echo "❌ Error: App directory already exists: $APP_DIR"
  exit 1
fi

# Create directory structure
echo "📁 Creating directory structure..."
mkdir -p "$APP_DIR/src/components"
mkdir -p "$APP_DIR/src/hooks"
mkdir -p "$APP_DIR/src/styles"

# Create manifest.json
echo "📄 Creating manifest.json..."
cat > "$APP_DIR/manifest.json" <<EOF
{
  "id": "$APP_ID",
  "name": "$APP_NAME",
  "type": "native_web",
  "version": "1.0.0",
  "icon": "📦",
  "category": "utilities",
  "author": "system",
  "description": "A native $APP_NAME application",
  "permissions": ["STANDARD"],
  "services": [],
  "exports": {
    "component": "App"
  },
  "tags": []
}
EOF

# Create package.json
echo "📄 Creating package.json..."
cat > "$APP_DIR/package.json" <<EOF
{
  "name": "@os-apps/$APP_ID",
  "version": "1.0.0",
  "type": "module",
  "description": "A native $APP_NAME application",
  "scripts": {
    "dev": "vite",
    "build": "tsc && vite build",
    "preview": "vite preview",
    "type-check": "tsc --noEmit",
    "lint": "eslint src --ext .ts,.tsx",
    "lint:fix": "eslint src --ext .ts,.tsx --fix",
    "format": "prettier --write \\"src/**/*.{ts,tsx,css}\\"",
    "format:check": "prettier --check \\"src/**/*.{ts,tsx,css}\\""
  },
  "dependencies": {
    "react": "^18.2.0",
    "react-dom": "^18.2.0",
    "clsx": "^2.1.1"
  },
  "devDependencies": {
    "@types/react": "^18.2.0",
    "@types/react-dom": "^18.2.0",
    "@vitejs/plugin-react": "^4.3.0",
    "typescript": "^5.3.0",
    "vite": "^5.0.0",
    "eslint": "^8.56.0",
    "prettier": "^3.1.0"
  }
}
EOF

# Create tsconfig.json
echo "📄 Creating tsconfig.json..."
cat > "$APP_DIR/tsconfig.json" <<EOF
{
  "compilerOptions": {
    "target": "ES2020",
    "useDefineForClassFields": true,
    "lib": ["ES2020", "DOM", "DOM.Iterable"],
    "module": "ESNext",
    "skipLibCheck": true,
    "moduleResolution": "bundler",
    "allowImportingTsExtensions": true,
    "resolveJsonModule": true,
    "isolatedModules": true,
    "noEmit": true,
    "jsx": "react-jsx",
    "strict": true,
    "noUnusedLocals": true,
    "noUnusedParameters": true,
    "noFallthroughCasesInSwitch": true
  },
  "include": ["src"]
}
EOF

# Create vite.config.ts (using shared base config)
echo "📄 Creating vite.config.ts..."
cat > "$APP_DIR/vite.config.ts" <<EOF
import { defineNativeAppConfig } from '../vite.config.base';

/**
 * Vite configuration for $APP_NAME
 * Extends the shared base configuration
 */
export default defineNativeAppConfig('$COMPONENT_NAME', {
  // Add app-specific overrides here
  // Example:
  // server: {
  //   port: 5175,
  // },
});
EOF

# Create index.tsx
echo "📄 Creating src/index.tsx..."
# Get capitalized component name (first letter uppercase)
COMPONENT_NAME="$(echo "$APP_ID" | sed 's/-/ /g' | awk '{for(i=1;i<=NF;i++)sub(/./,toupper(substr($i,1,1)),$i)}1' | sed 's/ //g')App"
cat > "$APP_DIR/src/index.tsx" <<EOF
import React from 'react';
import type { NativeAppProps } from '@os/sdk';
import App from './App';

/**
 * Entry point for $APP_NAME
 * This is the component that will be loaded by the OS
 */
export default function $COMPONENT_NAME(props: NativeAppProps) {
  return <App {...props} />;
}
EOF

# Create App.tsx
echo "📄 Creating src/App.tsx..."
cat > "$APP_DIR/src/App.tsx" <<EOF
import React, { useEffect } from 'react';
import type { NativeAppProps } from '@os/sdk';
import './styles/App.css';

/**
 * $APP_NAME
 * Main application component
 */
export default function App({ context }: NativeAppProps) {
  const { state, executor, window } = context;

  useEffect(() => {
    // Lifecycle: on mount
    console.log('$APP_NAME mounted');

    return () => {
      // Lifecycle: on unmount
      console.log('$APP_NAME unmounted');
    };
  }, []);

  return (
    <div className="app-container">
      <header className="app-header">
        <h1>$APP_NAME</h1>
      </header>
      <main className="app-main">
        <p>Your native app is ready! 🚀</p>
        <p>Edit <code>src/App.tsx</code> to get started.</p>

        <div className="app-info">
          <h2>What you can do:</h2>
          <ul>
            <li>Use any npm packages</li>
            <li>Write custom React components</li>
            <li>Access OS APIs via <code>executor</code></li>
            <li>Manage state with <code>state</code></li>
            <li>Control the window with <code>window</code></li>
          </ul>
        </div>
      </main>
    </div>
  );
}
EOF

# Create CSS
echo "📄 Creating src/styles/App.css..."
cat > "$APP_DIR/src/styles/App.css" <<EOF
.app-container {
  width: 100%;
  height: 100%;
  display: flex;
  flex-direction: column;
  padding: 20px;
  background: linear-gradient(135deg, #1a1a2e 0%, #16213e 100%);
  color: #e4e4e4;
  font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', 'Roboto', sans-serif;
}

.app-header {
  margin-bottom: 30px;
  padding-bottom: 20px;
  border-bottom: 2px solid rgba(255, 255, 255, 0.1);
}

.app-header h1 {
  margin: 0;
  font-size: 32px;
  font-weight: 700;
  background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
  -webkit-background-clip: text;
  -webkit-text-fill-color: transparent;
  background-clip: text;
}

.app-main {
  flex: 1;
  overflow-y: auto;
}

.app-main p {
  font-size: 16px;
  line-height: 1.6;
  margin-bottom: 15px;
}

.app-main code {
  background: rgba(255, 255, 255, 0.1);
  padding: 2px 6px;
  border-radius: 4px;
  font-family: 'Monaco', 'Courier New', monospace;
  font-size: 14px;
}

.app-info {
  margin-top: 30px;
  padding: 20px;
  background: rgba(255, 255, 255, 0.05);
  border-radius: 8px;
  border: 1px solid rgba(255, 255, 255, 0.1);
}

.app-info h2 {
  margin-top: 0;
  margin-bottom: 15px;
  font-size: 20px;
  color: #667eea;
}

.app-info ul {
  margin: 0;
  padding-left: 25px;
}

.app-info li {
  margin-bottom: 10px;
  line-height: 1.6;
}
EOF

# Create .gitignore
echo "📄 Creating .gitignore..."
cat > "$APP_DIR/.gitignore" <<EOF
# Dependencies
node_modules

# Build output
dist

# OS files
.DS_Store
Thumbs.db

# Logs
*.log
npm-debug.log*

# Editor directories
.vscode
.idea

# Environment variables
.env
.env.local
EOF

# Create .eslintrc.json
echo "📄 Creating .eslintrc.json..."
cat > "$APP_DIR/.eslintrc.json" <<EOF
{
  "extends": [
    "eslint:recommended",
    "plugin:@typescript-eslint/recommended",
    "plugin:react/recommended",
    "plugin:react-hooks/recommended"
  ],
  "parser": "@typescript-eslint/parser",
  "parserOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module",
    "ecmaFeatures": {
      "jsx": true
    }
  },
  "plugins": ["@typescript-eslint", "react", "react-hooks"],
  "rules": {
    "react/react-in-jsx-scope": "off",
    "@typescript-eslint/no-explicit-any": "warn",
    "@typescript-eslint/no-unused-vars": ["warn", { "argsIgnorePattern": "^_" }]
  },
  "settings": {
    "react": {
      "version": "detect"
    }
  }
}
EOF

# Create .prettierrc
echo "📄 Creating .prettierrc..."
cat > "$APP_DIR/.prettierrc" <<EOF
{
  "semi": true,
  "trailingComma": "es5",
  "singleQuote": true,
  "printWidth": 100,
  "tabWidth": 2,
  "useTabs": false,
  "arrowParens": "always",
  "endOfLine": "lf"
}
EOF

# Create README.md
echo "📄 Creating README.md..."
cat > "$APP_DIR/README.md" <<EOF
# $APP_NAME

A native TypeScript/React application for the OS.

## Development

\`\`\`bash
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
\`\`\`

## Structure

\`\`\`
$APP_ID/
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
\`\`\`

## Available APIs

The app has access to the OS SDK via the \`context\` prop:

- **State Management**: \`context.state\` - Reactive state store
- **Service Calls**: \`context.executor\` - Execute backend services
- **Window Controls**: \`context.window\` - Control the app window

### Example Service Calls

\`\`\`typescript
// Filesystem
await context.executor.execute('filesystem.read', { path: '/path/to/file' });
await context.executor.execute('filesystem.write', { path: '/path', content: 'data' });
await context.executor.execute('filesystem.list', { path: '/path' });

// Storage
await context.executor.execute('storage.set', { key: 'mykey', value: data });
const result = await context.executor.execute('storage.get', { key: 'mykey' });

// HTTP
await context.executor.execute('http.get', { url: 'https://api.example.com' });
\`\`\`

## Building

The app is built as an ES module that exports a React component. The build output goes to \`../../dist/$APP_ID/\`.

## License

MIT
EOF

echo ""
echo "======================================"
echo "✅ Native app created successfully!"
echo "======================================"
echo ""
echo "📦 App: $APP_NAME"
echo "🆔 ID:  $APP_ID"
echo "📁 Dir: apps/native/$APP_ID"
echo ""
echo "Next steps:"
echo ""
echo "  1. Install dependencies:"
echo "     cd $APP_DIR"
echo "     npm install"
echo ""
echo "  2. Start development server (with HMR):"
echo "     npm run dev"
echo "     # Or use: make watch-native-app name=$APP_ID"
echo ""
echo "  3. Build for production:"
echo "     npm run build"
echo "     # Or use: make build-native-apps"
echo ""
echo "  4. Validate and lint:"
echo "     make validate-native-apps"
echo "     make lint-native-app name=$APP_ID"
echo ""
echo "  5. Output will be in: apps/dist/$APP_ID/"
echo ""
echo "📚 Documentation:"
echo "   - App README:    $APP_DIR/README.md"
echo "   - SDK Reference: ui/src/core/sdk/index.ts"
echo "   - Examples:      apps/native/"
echo ""
echo "Happy coding! 🎉"
