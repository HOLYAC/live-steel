module.exports = [
  {
    files: ["capture_live_steel_proof.js", "tools/*.cjs"],
    languageOptions: {
      ecmaVersion: 2022,
      sourceType: "commonjs",
      globals: {
        Buffer: "readonly",
        console: "readonly",
        __dirname: "readonly",
        process: "readonly",
        require: "readonly",
        WebAssembly: "readonly",
        Uint8Array: "readonly",
        Float32Array: "readonly",
        performance: "readonly",
        window: "readonly",
      },
    },
    rules: {
      "no-dupe-keys": "error",
      "no-undef": "error",
      "no-unused-vars": ["error", { "argsIgnorePattern": "^_", "varsIgnorePattern": "^_" }],
      "no-implicit-globals": "error",
    },
  },
];
