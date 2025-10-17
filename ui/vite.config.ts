import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";
import path from "path";
import fs from "fs";

// Plugin to serve native app bundles from /native-apps/ path
const nativeAppsPlugin = () => ({
  name: "native-apps-server",
  configureServer(server: any) {
    server.middlewares.use((req: any, res: any, next: any) => {
      // Match /native-apps/[app-id]/[file] (strip query params)
      const urlWithoutQuery = req.url?.split("?")[0];
      const nativeAppsMatch = urlWithoutQuery?.match(/^\/native-apps\/([^/]+)\/(.+)/);

      if (nativeAppsMatch) {
        const [, appId, file] = nativeAppsMatch;
        const filePath = path.resolve(__dirname, `../apps/dist/${appId}/${file}`);

        // Check if file exists
        if (fs.existsSync(filePath)) {
          // Set appropriate content type and CORS headers
          res.setHeader("Access-Control-Allow-Origin", "*");
          res.setHeader("Access-Control-Allow-Methods", "GET, OPTIONS");
          res.setHeader("Access-Control-Allow-Headers", "*");

          if (file.endsWith(".js")) {
            res.setHeader("Content-Type", "application/javascript; charset=utf-8");

            // Read and transform JS to use window globals for React
            let content = fs.readFileSync(filePath, "utf-8");

            // Transform bare imports to window globals
            // Replace "as" with ":" for destructuring syntax
            content = content
              .replace(
                /import\s+\*\s+as\s+(\w+)\s+from\s+["']react["'];?/g,
                "const $1 = window.React;"
              )
              .replace(
                /import\s+\*\s+as\s+(\w+)\s+from\s+["']react-dom["'];?/g,
                "const $1 = window.ReactDOM;"
              )
              .replace(
                /import\s+{([^}]+)}\s+from\s+["']react\/jsx-runtime["'];?/g,
                (match, imports) => {
                  const fixed = imports.replace(/\s+as\s+/g, ": ");
                  return `const {${fixed}} = window.ReactJSXRuntime;`;
                }
              )
              .replace(
                /import\s+{([^}]+)}\s+from\s+["']react\/jsx-dev-runtime["'];?/g,
                (match, imports) => {
                  const fixed = imports.replace(/\s+as\s+/g, ": ");
                  return `const {${fixed}} = window.ReactJSXRuntime;`;
                }
              )
              .replace(/import\s+{([^}]+)}\s+from\s+["']react["'];?/g, (match, imports) => {
                const fixed = imports.replace(/\s+as\s+/g, ": ");
                return `const {${fixed}} = window.React;`;
              })
              .replace(/import\s+{([^}]+)}\s+from\s+["']react-dom["'];?/g, (match, imports) => {
                const fixed = imports.replace(/\s+as\s+/g, ": ");
                return `const {${fixed}} = window.ReactDOM;`;
              });

            res.end(content);
            return;
          } else if (file.endsWith(".css")) {
            res.setHeader("Content-Type", "text/css; charset=utf-8");
            const content = fs.readFileSync(filePath, "utf-8");
            res.end(content);
            return;
          }
        }
      }

      next();
    });
  },
});

export default defineConfig({
  plugins: [
    react({
      // Automatic JSX runtime for cleaner code (Fast Refresh enabled by default)
      jsxRuntime: "automatic",
    }),
    nativeAppsPlugin(),
  ],

  server: {
    port: 5173,
    strictPort: true,
    // Enable HMR for better DX
    hmr: {
      overlay: true,
    },
    // Better CORS handling
    cors: true,
    // Optimize file watching
    watch: {
      // Ignore node_modules for better performance
      ignored: ["**/node_modules/**", "**/dist/**"],
    },
    // Proxy configuration for backend API and assets
    proxy: {
      // Proxy /apps/native/* to backend for icon assets
      '/apps/native': {
        target: 'http://localhost:8000',
        changeOrigin: true,
        secure: false,
      },
      // Proxy other API endpoints to backend
      '/api': {
        target: 'http://localhost:8000',
        changeOrigin: true,
        secure: false,
      },
    },
    // Serve native app bundles
    fs: {
      allow: [
        // Allow serving files from the project root
        path.resolve(__dirname, ".."),
      ],
    },
  },

  preview: {
    port: 4173,
    strictPort: true,
    cors: true,
  },

  base: "./",

  resolve: {
    alias: {
      "@": path.resolve(__dirname, "./src"),
    },
    // Optimize extension resolution order
    extensions: [".ts", ".tsx", ".js", ".jsx", ".json"],
  },

  build: {
    // Enable source maps only in development
    sourcemap: process.env.NODE_ENV !== "production",

    // Optimize chunk size
    chunkSizeWarningLimit: 1000,

    // CSS minification (esbuild is fast and efficient)
    cssMinify: "esbuild",

    // Improve build performance
    reportCompressedSize: false,

    // Manual chunk splitting for better caching and loading
    rollupOptions: {
      output: {
        // Better naming for chunks
        chunkFileNames: "assets/js/[name]-[hash].js",
        entryFileNames: "assets/js/[name]-[hash].js",
        assetFileNames: "assets/[ext]/[name]-[hash].[ext]",

        manualChunks: (id) => {
          // React core - most stable, rarely changes
          if (id.includes("node_modules/react/") || id.includes("node_modules/react-dom/")) {
            return "react-vendor";
          }

          // UI core libraries - stable, always needed
          if (
            id.includes("@tanstack/react-query") ||
            id.includes("zustand") ||
            id.includes("react-hook-form") ||
            id.includes("zod") ||
            id.includes("sonner")
          ) {
            return "ui-core";
          }

          // Animation libraries - heavy but often used together
          if (id.includes("gsap") || id.includes("@react-spring") || id.includes("@use-gesture")) {
            return "animation";
          }

          // DnD kit - heavy but isolated feature
          if (id.includes("@dnd-kit")) {
            return "dnd";
          }

          // Floating UI - isolated feature
          if (id.includes("@floating-ui")) {
            return "floating";
          }

          // Charts - heavy, lazy loaded
          if (id.includes("recharts")) {
            return "charts";
          }

          // Graph visualization - very heavy, lazy loaded
          if (id.includes("reactflow")) {
            return "graph";
          }

          // Utilities - smaller, less frequently changing
          if (
            id.includes("date-fns") ||
            id.includes("colord") ||
            id.includes("mathjs") ||
            id.includes("clsx") ||
            id.includes("class-variance-authority") ||
            id.includes("lucide-react")
          ) {
            return "utils";
          }

          // Virtual scrolling libraries
          if (id.includes("@tanstack/react-virtual") || id.includes("react-window")) {
            return "virtualization";
          }

          // Other node_modules as vendor
          if (id.includes("node_modules")) {
            return "vendor";
          }
        },
      },
    },

    // Advanced minification with esbuild
    minify: "esbuild",

    // Target modern browsers for optimal bundle size
    target: "esnext",

    // Enable CSS code splitting
    cssCodeSplit: true,

    // Optimize assets
    assetsInlineLimit: 4096, // 4kb - inline smaller assets as base64
  },

  // Optimize dependency pre-bundling
  optimizeDeps: {
    include: [
      // Pre-bundle frequently used deps for faster cold start
      "react",
      "react-dom",
      "react/jsx-runtime",
      "@tanstack/react-query",
      "zustand",
      "gsap",
      "clsx",
      "lucide-react",
    ],
    exclude: [
      // Don't pre-bundle lazy-loaded heavy deps
      "recharts",
      "reactflow",
    ],
    // Use esbuild for fast pre-bundling
    esbuildOptions: {
      target: "esnext",
      supported: {
        "top-level-await": true,
      },
    },
  },

  // Esbuild optimization options
  esbuild: {
    drop: process.env.NODE_ENV === "production" ? ["console", "debugger"] : [],
    legalComments: "none",
    logOverride: {
      "this-is-undefined-in-esm": "silent",
    },
  },

  // JSON optimization
  json: {
    stringify: true, // Faster parsing for large JSON
    namedExports: true,
  },
});
