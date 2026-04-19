// Flat ESLint config for Binderbase frontend (TypeScript + React).
// https://eslint.org/docs/latest/use/configure/configuration-files

import js from "@eslint/js";
import tseslint from "@typescript-eslint/eslint-plugin";
import tsParser from "@typescript-eslint/parser";
import reactHooks from "eslint-plugin-react-hooks";
import reactRefresh from "eslint-plugin-react-refresh";

export default [
  {
    ignores: [
      "dist/**",
      "dist-ssr/**",
      "coverage/**",
      "node_modules/**",
      "src-tauri/target/**",
      "src-tauri/gen/**",
    ],
  },
  js.configs.recommended,
  {
    files: ["**/*.{ts,tsx}"],
    languageOptions: {
      parser: tsParser,
      parserOptions: {
        ecmaVersion: 2022,
        sourceType: "module",
        ecmaFeatures: { jsx: true },
      },
      globals: {
        window: "readonly",
        document: "readonly",
        navigator: "readonly",
        console: "readonly",
        setTimeout: "readonly",
        clearTimeout: "readonly",
        setInterval: "readonly",
        clearInterval: "readonly",
        fetch: "readonly",
        URL: "readonly",
        Uint8Array: "readonly",
        HTMLElement: "readonly",
        HTMLDivElement: "readonly",
        HTMLInputElement: "readonly",
        HTMLButtonElement: "readonly",
        HTMLSelectElement: "readonly",
        HTMLVideoElement: "readonly",
        HTMLCanvasElement: "readonly",
        HTMLDialogElement: "readonly",
        MediaStream: "readonly",
        MouseEvent: "readonly",
        Node: "readonly",
        File: "readonly",
        FileReader: "readonly",
        Blob: "readonly",
      },
    },
    plugins: {
      "@typescript-eslint": tseslint,
      "react-hooks": reactHooks,
      "react-refresh": reactRefresh,
    },
    rules: {
      ...tseslint.configs.recommended.rules,
      ...reactHooks.configs.recommended.rules,
      "react-refresh/only-export-components": ["warn", { allowConstantExport: true }],
      "@typescript-eslint/no-unused-vars": [
        "error",
        { argsIgnorePattern: "^_", varsIgnorePattern: "^_" },
      ],
      "@typescript-eslint/consistent-type-imports": [
        "error",
        { prefer: "type-imports", fixStyle: "separate-type-imports" },
      ],
      "no-console": ["warn", { allow: ["warn", "error"] }],
      eqeqeq: ["error", "always"],
    },
  },
  {
    files: ["vite.config.ts", "eslint.config.js"],
    languageOptions: {
      globals: {
        process: "readonly",
      },
    },
  },
];
