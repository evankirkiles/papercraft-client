import { resolve } from "path";
import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";
import tsconfigPaths from "vite-tsconfig-paths";
import dts from "vite-plugin-dts";
import wasmPack from "vite-plugin-wasm-pack";

export default defineConfig({
  resolve: {
    alias: {
      "@": resolve(__dirname, "src"),
    },
  },
  plugins: [
    react(),
    tsconfigPaths(),
    dts({
      include: ["src/**/*.{ts,tsx}"],
      beforeWriteFile: (filePath, content) => ({
        filePath: filePath.replace("lib", "dist"),
        content,
      }),
    }),
    // wasmPack(["./crates/pp_control2"]),
  ],
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
    },
  },
});
