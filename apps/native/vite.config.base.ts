/**
 * Shared Vite Configuration Base for Native Apps
 * Provides optimized build settings for native app development
 *
 * Apps should extend this config in their own vite.config.ts
 */

import { defineConfig, type UserConfig } from 'vite';
import react from '@vitejs/plugin-react';
import path from 'path';

/**
 * Base configuration factory
 * @param appName - Name of the app (from package.json)
 * @param options - Additional config overrides
 */
export function defineNativeAppConfig(appName: string, options: Partial<UserConfig> = {}): UserConfig {
  return defineConfig({
    plugins: [
      react({
        // Fast Refresh for better DX
        fastRefresh: true,
        // Automatic JSX runtime
        jsxRuntime: 'automatic',
        // Babel optimizations
        babel: {
          plugins: [
            // Add any necessary babel plugins here
          ],
        },
      }),
      ...(options.plugins || []),
    ],

    // Development server config
    server: {
      port: 5174, // Different from main UI
      strictPort: false, // Auto-increment if taken
      cors: true,
      hmr: {
        overlay: true, // Show errors in overlay
        protocol: 'ws',
      },
      // Watch all source files
      watch: {
        usePolling: false,
        interval: 100,
      },
    },

    // Build configuration for library mode
    build: {
      lib: {
        entry: path.resolve(process.cwd(), 'src/index.tsx'),
        name: appName,
        fileName: () => 'index.js',
        formats: ['es'], // ES modules only
      },

      // Rollup options
      rollupOptions: {
        // Externalize React - will be provided by host via window.React
        external: [
          'react',
          'react-dom',
          'react/jsx-runtime',
          'react/jsx-dev-runtime',
        ],

        output: {
          // Map external modules to window globals
          globals: {
            react: 'React',
            'react-dom': 'ReactDOM',
            'react/jsx-runtime': 'ReactJSXRuntime',
            'react/jsx-dev-runtime': 'ReactJSXRuntime',
          },

          // Preserve modules for better tree-shaking
          preserveModules: false,

          // Code splitting strategy
          manualChunks: undefined,

          // Asset file names
          assetFileNames: 'assets/[name].[ext]',
        },
      },

      // Output directory (relative to app root)
      outDir: '../../dist/' + path.basename(process.cwd()),
      emptyOutDir: true,

      // Source maps (disabled in production for smaller bundles)
      sourcemap: process.env.NODE_ENV !== 'production',

      // Minification settings
      minify: 'esbuild',
      target: 'esnext',

      // Chunk size warnings
      chunkSizeWarningLimit: 500, // 500kb

      // CSS code splitting
      cssCodeSplit: true,

      // Asset inlining threshold
      assetsInlineLimit: 4096, // 4kb
    },

    // Dependency optimization
    optimizeDeps: {
      include: ['react', 'react-dom', 'react/jsx-runtime'],
      exclude: [],
      esbuildOptions: {
        target: 'esnext',
        supported: {
          'top-level-await': true,
        },
      },
    },

    // Esbuild configuration
    esbuild: {
      // Remove console/debugger in production
      drop: process.env.NODE_ENV === 'production' ? ['console', 'debugger'] : [],
      legalComments: 'none',
      logOverride: {
        'this-is-undefined-in-esm': 'silent',
      },
    },

    // JSON optimization
    json: {
      stringify: true,
      namedExports: true,
    },

    // Path resolution
    resolve: {
      alias: {
        '@': path.resolve(process.cwd(), '../../../ui/src'),
        '@app': path.resolve(process.cwd(), 'src'),
      },
      extensions: ['.ts', '.tsx', '.js', '.jsx', '.json'],
    },

    // Define environment variables
    define: {
      'process.env.NODE_ENV': JSON.stringify(process.env.NODE_ENV || 'development'),
    },

    // Merge with provided options
    ...options,
  });
}

/**
 * Development mode configuration
 * Optimized for fast rebuilds and HMR
 */
export function defineDevConfig(appName: string): UserConfig {
  return defineNativeAppConfig(appName, {
    build: {
      minify: false,
      sourcemap: true,
    },
    esbuild: {
      drop: [],
    },
  });
}

/**
 * Production mode configuration
 * Optimized for bundle size and performance
 */
export function defineProdConfig(appName: string): UserConfig {
  return defineNativeAppConfig(appName, {
    build: {
      minify: 'esbuild',
      sourcemap: false,
      reportCompressedSize: true,
    },
    esbuild: {
      drop: ['console', 'debugger'],
    },
  });
}
