import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";
import path from "path";

export default defineConfig({
  plugins: [
    react({
      // Automatic JSX runtime for cleaner code (Fast Refresh enabled by default)
      jsxRuntime: "automatic",
    })
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
      ignored: ["**/node_modules/**", "**/dist/**"]
    }
  },

  preview: {
    port: 4173,
    strictPort: true,
    cors: true
  },

  base: "./",

  resolve: {
    alias: {
      "@": path.resolve(__dirname, "./src"),
    },
    // Optimize extension resolution order
    extensions: [".ts", ".tsx", ".js", ".jsx", ".json"]
  },

  build: {
    // Enable source maps only in development
    sourcemap: process.env.NODE_ENV !== "production",

    // Optimize chunk size
    chunkSizeWarningLimit: 1000,

    // Optimize CSS
    cssMinify: "lightningcss",

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
          if (id.includes("@tanstack/react-query") ||
              id.includes("zustand") ||
              id.includes("react-hook-form") ||
              id.includes("zod") ||
              id.includes("sonner")) {
            return "ui-core";
          }

          // Animation libraries - heavy but often used together
          if (id.includes("gsap") ||
              id.includes("@react-spring") ||
              id.includes("@use-gesture")) {
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
          if (id.includes("date-fns") ||
              id.includes("colord") ||
              id.includes("mathjs") ||
              id.includes("clsx") ||
              id.includes("class-variance-authority") ||
              id.includes("lucide-react")) {
            return "utils";
          }

          // Virtual scrolling libraries
          if (id.includes("@tanstack/react-virtual") ||
              id.includes("react-window")) {
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
      "lucide-react"
    ],
    exclude: [
      // Don't pre-bundle lazy-loaded heavy deps
      "recharts",
      "reactflow"
    ],
    // Use esbuild for fast pre-bundling
    esbuildOptions: {
      target: "esnext",
      supported: {
        "top-level-await": true
      }
    }
  },

  // Esbuild optimization options
  esbuild: {
    drop: process.env.NODE_ENV === "production" ? ["console", "debugger"] : [],
    legalComments: "none",
    logOverride: {
      "this-is-undefined-in-esm": "silent"
    }
  },

  // JSON optimization
  json: {
    stringify: true, // Faster parsing for large JSON
    namedExports: true
  },
});
