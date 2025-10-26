import base from "@repo/eslint-config/base";
import { defineConfig } from "eslint/config";

export default defineConfig([
  base,
  { ignores: ["ts/wasm/**/*", "dist", ".rollup.cache"] },
  {
    files: ["ts/**/*.ts", "eslint.config.ts", "rollup.config.js"],
    languageOptions: {
      parserOptions: {
        // @ts-expect-error - We are stuck on TypeScript 5.8, where `dirname` is not present
        tsconfigRootDir: import.meta.dirname,
      },
    },
  },
]);
