import { defineConfig } from "vite";

// These plugins seemed like they could be useful for importing CodeMirror,
// but weren't. If using them again, `npm i --save-dev` them first.
// import commonjs from "@rollup/plugin-commonjs";
// import { nodeResolve } from "@rollup/plugin-node-resolve";
// import externalGlobals from "rollup-plugin-external-globals";

import checker from "vite-plugin-checker";

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
      // 3. tell vite to ignore watching `src-tauri` (ilyagr note: likely no longer relevant)
      ignored: ["**/src-tauri/**"],
    },
  },
  plugins: [
    checker({
      // See https://vite-plugin-checker.netlify.app/introduction/getting-started.html
      // for disabling during testing.
      typescript: true,
    }),
  ],
  build: {
    commonjsOptions: {
      // https://github.com/vitejs/vite/issues/13672#issuecomment-1784110536
      // It *seems* that this workaround makes a difference. I'm
      // *guessing* this is related to CodeMirror5 being CommonJS and
      // possibly having some crazy recursive includes.
      strictRequires: true
    },
    rollupOptions: {
      // https://github.com/vitejs/vite/issues/10506#issuecomment-1367718113
      // TODO: It's possible that this workaround makes no difference
      maxParallelFileOps: 1,
      plugins: [
        // externalGlobals() would be nice, but doesn't seem to work
        // externalGlobals({ diff_match_patch: "diff_match_patch" }),
        // commonjs(),
        // nodeResolve(),
      ],
    },
  },
});
