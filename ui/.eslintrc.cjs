module.exports = {
  root: true,
  env: {
    browser: true,
    es2020: true,
    node: true,
  },
  globals: {
    NodeJS: 'readonly',
    NodeListOf: 'readonly',
    EventListener: 'readonly',
    React: 'readonly',
    JSX: 'readonly',
  },
  extends: [
    'eslint:recommended',
    'plugin:@typescript-eslint/recommended',
    'plugin:react/recommended',
    'plugin:react-hooks/recommended',
  ],
  ignorePatterns: ['dist', 'dist-electron', 'node_modules', '.eslintrc.cjs'],
  parser: '@typescript-eslint/parser',
  plugins: ['react-refresh', '@typescript-eslint'],
  settings: {
    react: {
      version: 'detect',
    },
  },
  rules: {
    // React specific
    'react-refresh/only-export-components': [
      'warn',
      { allowConstantExport: true },
    ],
    'react/react-in-jsx-scope': 'off',
    'react/prop-types': 'off', // TypeScript handles this

    // TypeScript specific
    '@typescript-eslint/no-unused-vars': [
      'warn',
      {
        argsIgnorePattern: '^_',
        varsIgnorePattern: '^_',
        caughtErrorsIgnorePattern: '^_'
      }
    ],
    '@typescript-eslint/no-explicit-any': 'warn',
    '@typescript-eslint/ban-ts-comment': 'warn',

    // Disable conflicting base rules
    'no-unused-vars': 'off', // Use TypeScript version instead
    'no-undef': 'off', // TypeScript handles undefined checking

    // General rules
    'no-console': 'warn',
    'no-debugger': 'error',
    'prefer-const': 'error',
  },
  overrides: [
    {
      files: ['**/*.test.ts', '**/*.test.tsx'],
      env: {
        jest: true,
      },
    },
    // STRATEGIC EXCEPTION: Floating UI Refs During Render
    //
    // Why disabled:
    // - @floating-ui/react intentionally accesses refs during render for positioning
    // - This pattern is safe and battle-tested across millions of React apps
    // - The library predates React 18's stricter rules but works correctly in practice
    // - Refactoring would require major changes across 6+ component types for minimal benefit
    //
    // The Alternative (Not Worth It):
    // - Major refactor: Tooltip, Popover, Select, Dropdown, ContextMenu, HoverCard
    // - Risk: Breaking production-tested positioning logic
    // - Benefit: Satisfying a linting rule that's overly conservative for this pattern
    //
    // Future Migration Path:
    // - CSS Anchor Positioning API (Chrome 125+, coming to Safari/Firefox)
    // - Will eliminate JavaScript positioning entirely (more performant)
    // - When broadly supported, migrate away from floating-ui to native CSS
    //
    // References:
    // - CSS Anchor: https://developer.chrome.com/blog/anchor-positioning-api
    // - Floating UI: https://floating-ui.com/docs/react
    //
    // Last reviewed: 2025-10-17
    {
      files: ['src/features/floating/**/*.{ts,tsx}'],
      rules: {
        'react-hooks/refs': 'off',
      },
    },
  ],
};
