// For more info, see https://github.com/storybookjs/eslint-plugin-storybook#configuration-flat-config-format
import storybook from "eslint-plugin-storybook";

import tseslint from "@typescript-eslint/eslint-plugin";
import tsparser from "@typescript-eslint/parser";
import vueplugin from "eslint-plugin-vue";

export default [{
  ignores: ["node_modules/**", "dist/**", "src-tauri/target/**"],
}, // TypeScript files
{
  files: ["src/**/*.ts"],
  languageOptions: {
    parser: tsparser,
    parserOptions: {
      ecmaVersion: "latest",
      sourceType: "module",
    },
  },
  plugins: { "@typescript-eslint": tseslint },
  rules: {
    "@typescript-eslint/naming-convention": [
      "error",
      { selector: "function", format: ["camelCase", "PascalCase"] },
      { selector: "variable", format: ["camelCase", "UPPER_CASE"] },
    ],
    "max-lines-per-function": [
      "warn",
      { max: 200, skipBlankLines: true, skipComments: true },
    ],
    "no-duplicate-imports": "warn",
  },
}, // Vue files
...vueplugin.configs["flat/essential"], {
  files: ["src/**/*.vue"],
  languageOptions: {
    parserOptions: {
      parser: tsparser,
      ecmaVersion: "latest",
      sourceType: "module",
    },
  },
  rules: {
    "max-lines-per-function": [
      "warn",
      { max: 200, skipBlankLines: true, skipComments: true },
    ],
  },
}, ...storybook.configs["flat/recommended"]];
