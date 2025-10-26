import base from "./base";
import storybook from "eslint-plugin-storybook";
import { defineConfig } from "eslint/config";

/* Custom ESLint configuration for use with Next.js apps. */
export default defineConfig([base, ...storybook.configs["flat/recommended"]]);
