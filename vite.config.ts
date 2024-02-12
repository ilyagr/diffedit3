import { defineConfig } from "vite";

// These plugins seemed like they could be useful for importing CodeMirror,
// but weren't. If using them again, `npm i --save-dev` them first.
// import commonjs from "@rollup/plugin-commonjs";
// import { nodeResolve } from "@rollup/plugin-node-resolve";
// import externalGlobals from "rollup-plugin-external-globals";

// https://vitejs.dev/config/
export default defineConfig({
  // Vite options tailored for Tauri development and only applied in `tauri dev` or `tauri build`
  //
  // 1. prevent vite from obscuring rust errors
  clearScreen: false,
  // 2. tauri expects a fixed port, fail if that port is not available
  server: {
    port: 1420,
    strictPort: true,
    watch: {
      // 3. tell vite to ignore watching `src-tauri`
      ignored: ["**/src-tauri/**"],
    },
  },
  build: {
    rollupOptions: {
      plugins: [
        // externalGlobals() would be nice, but doesn't seem to work
        // externalGlobals({ diff_match_patch: "diff_match_patch" }),
        // commonjs(),
        // nodeResolve(),
      ],
    },
  },
});
