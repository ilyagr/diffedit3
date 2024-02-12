import { invoke } from "@tauri-apps/api/tauri";

import CodeMirror from "codemirror";
// import "codemirror/mode/htmlmixed/htmlmixed.js";
import "codemirror/lib/codemirror.css";
import "codemirror/addon/merge/merge.css";
import "codemirror/addon/merge/merge";

// diff_match_patch needs to be in the global scope for merge addon to work
// Conceivably, it could be imported from the HTML, but I have not found a way to convince
// Vite/TS/Rollup to do that. If using Webpack instead of Rollup, it's possible that this
// can be done as disucussed in
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

let greetInputEl: HTMLInputElement | null;
let greetMsgEl: HTMLElement | null;

async function greet() {
  if (greetMsgEl && greetInputEl) {
    // Learn more about Tauri commands at https://tauri.app/v1/guides/features/command
    greetMsgEl.textContent = await invoke("greet", {
      name: greetInputEl.value,
    });
  }
}

window.addEventListener("DOMContentLoaded", () => {
  greetInputEl = document.querySelector("#greet-input");
  greetMsgEl = document.querySelector("#greet-msg");
  document.querySelector("#greet-form")?.addEventListener("submit", (e) => {
    e.preventDefault();
    greet();
  });

  // https://github.com/tauri-apps/tauri/discussions/6119
  if ("__TAURI__" in globalThis) {
    console.log("In Tauri");
  } else {
    console.log("Not in Tauri");
  }

  let cmEl = document.getElementById("cm");
  if (cmEl == null) return;
  cmEl.innerHTML = "";
  let /* panes = 2, */
    highlight = true,
    connect = "align",
    collapse = false;
  let base = "First\nThird\nFourth\nFifth one\n----\none two";
  let value = "First\nSecond\nThird\nFifth one\n----\none\n";
  let config = {
    value: value,
    origLeft: base, // Set to null for 2 panes
    orig: value,
    lineNumbers: true,
    mode: "text/plain",
    highlightDifferences: highlight,
    connect: connect,
    collapseIdentical: collapse,
  };
  /* let merge_view = */ CodeMirror.MergeView(cmEl, config);
  // TODO: Resizing. See https://codemirror.net/5/demo/merge.html
});
