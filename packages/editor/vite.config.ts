import tailwindcss from "@tailwindcss/vite";
import react from "@vitejs/plugin-react";
import path from "path";
import { defineConfig } from "vite";
import dts from "vite-plugin-dts";
import topLevelAwait from "vite-plugin-top-level-await";
import wasm from "vite-plugin-wasm";

export default defineConfig({
  resolve: {
    alias: {
      "@": path.resolve(__dirname, "src"),
    },
  },
  plugins: [
    wasm(),
    react(),
    tailwindcss(),
    topLevelAwait(),
    dts({
      include: ["src/**/*.{ts,tsx}"],
      beforeWriteFile: (filePath, content) => ({
        filePath: filePath.replace("lib", "dist"),
        content,
      }),
    }),
  ],
  assetsInclude: ["**/*.wasm"],
  optimizeDeps: {
    // Workspace package rebuilt on every Rust/TS change during `pnpm dev` -
    // don't pre-bundle/cache it, always import the fresh build from disk.
    exclude: ["@paperarium/client"],
  },
  server: {
    watch: {
      // Vite's watcher ignores node_modules by default, but this workspace
      // package is a live-rebuilt local symlink, not a real dependency.
      ignored: ["!**/node_modules/@paperarium/client/**"],
    },
  },
  build: {
    minify: false,
    target: "esnext",
    outDir: "dist",
    rollupOptions: {
      output: {
        assetFileNames: "assets/[name][extname]",
        entryFileNames: "[name].js",
      },
      input: {
        "index.html": "index.html",
      },
    },
  },
});
