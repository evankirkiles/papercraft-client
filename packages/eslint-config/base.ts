import eslint from "@eslint/js";
import tseslint from "typescript-eslint";
import turbo from "eslint-plugin-turbo";
import { defineConfig } from "eslint/config";

/* Custom ESLint configuration for use with Next.js apps. */
export default defineConfig([
  eslint.configs.recommended,
  tseslint.configs.strict,
  turbo.configs["flat/recommended"],
]);
