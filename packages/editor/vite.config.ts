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
