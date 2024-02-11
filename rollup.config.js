import commonjs from "@rollup/plugin-commonjs";
import { nodeResolve } from "@rollup/plugin-node-resolve";

export default {
  input: "src/index.js",
  output: {
    dir: "output",
    format: "cjs",
  },
  plugins: [
    commonjs(),
    nodeResolve(),
    externalGlobals({ diff_match_patch: "diff_match_patch" }),
  ],
};
