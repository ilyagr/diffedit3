// This file is loaded directly in `index.html`, in the hopes that this will put
// everything in the global scope as CodeMirror 5 expects. CodeMirror is imported
// again in modules using it, to make Typescript happy

// TODO: Consider converting this file to Javascript

import "codemirror";
// import "codemirror/mode/htmlmixed/htmlmixed.js";
import "codemirror/lib/codemirror.css";
import "codemirror/addon/merge/merge.css";
import "codemirror/addon/merge/merge";

// diff_match_patch needs to be in the global scope for merge addon to work
// Conceivably, it could be imported from the HTML, but I have not found a way to convince
// Vite/TS/Rollup to do that. If using Webpack instead of Rollup, it's possible that this
// can be done as discussed in
// https://discuss.codemirror.net/t/issues-on-using-merge-addon-and-diff-match-patch-solved/4371/4.
import {
  diff_match_patch,
  DIFF_ADD,
  DIFF_EQUAL,
  DIFF_INSERT,
  DIFF_DELETE,
} from "diff_match_patch";

declare global {
  // Make Typescript happier about this
  var diff_match_patch: any,
    DIFF_ADD: any,
    DIFF_EQUAL: any,
    DIFF_INSERT: any,
    DIFF_DELETE: any;
}
globalThis.diff_match_patch = diff_match_patch;
globalThis.DIFF_ADD = DIFF_ADD;
globalThis.DIFF_EQUAL = DIFF_EQUAL;
globalThis.DIFF_INSERT = DIFF_INSERT;
globalThis.DIFF_DELETE = DIFF_DELETE;
