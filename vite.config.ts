import { resolve } from "path";
import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";
import tsconfigPaths from "vite-tsconfig-paths";
import dts from "vite-plugin-dts";
import wasm from "vite-plugin-wasm";
import topLevelAwait from "vite-plugin-top-level-await";

export default defineConfig({
  resolve: {
    alias: {
      "@": resolve(__dirname, "src"),
    },
  },
  plugins: [
    wasm(),
    react(),
    topLevelAwait(),
    tsconfigPaths(),
    dts({
      include: ["src/**/*.{ts,tsx}"],
      beforeWriteFile: (filePath, content) => ({
        filePath: filePath.replace("lib", "dist"),
        content,
      }),
    }),
  ],
  assetsInclude: ["**/*.wasm"],
  build: {
    minify: false,
    lib: {
      formats: ["es"],
      entry: {
        index: resolve(__dirname, "src/index.tsx"),
      },
    },
    rollupOptions: {
      external: ["react", "react/jsx-runtime"],
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
