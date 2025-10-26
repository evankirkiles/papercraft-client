import library from "@repo/eslint-config/library";
import { defineConfig } from "eslint/config";

export default defineConfig([
  library,
  { ignores: ["dist"] },
  {
    files: ["**/*.ts", "**/*.tsx"],
    languageOptions: {
      parserOptions: {
        // @ts-expect-error - We are stuck on TypeScript 5.8, where `dirname` is not present
        tsconfigRootDir: import.meta.dirname,
      },
    },
  },
]);
