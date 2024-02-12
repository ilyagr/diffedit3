import { InvokeArgs, invoke } from "@tauri-apps/api/tauri";

import CodeMirror from "codemirror";
// import "codemirror/mode/htmlmixed/htmlmixed.js";
import "codemirror/lib/codemirror.css";
import "codemirror/addon/merge/merge.css";
import "codemirror/addon/merge/merge";
import { MergeView } from "codemirror/addon/merge/merge";

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

type SingleMerge = {
  left: string | null;
  right: string | null;
  edit: string | null;
};
type MergeInput = Record<string, SingleMerge>;
let INPUT: MergeInput = {
  edited_file: {
    left: "First\nThird\nFourth\nFifth one\n----\none two",
    edit: "First\nSecond\nThird\nFifth one\n----\none\n",
    right: "",
  },
  added_file: {
    left: null,
    edit: "Added",
    right: "",
  },
  removed_file: {
    left: "Deleted",
    edit: null,
    right: null,
  },
};
for (let x in INPUT) {
  INPUT[x].right = INPUT[x].edit;
}

import { html, render as lit_html_render } from "lit-html";
function render_input(unique_id: string, merge_input: MergeInput) {
  let templates = [];
  for (let k in merge_input) {
    templates.push(html`
        <details open>
          <summary>
            <code>${k}</code>
            <button id = "save_${k}_${unique_id}">Save (non-functional)</button>
          </summary>
          <div id="cm_${k}_${unique_id}"></div>
        </details>
      `);
  }

  lit_html_render(html`${templates}`, document.getElementById(unique_id)!);

  let merge_views: Record<string, MergeView> = {};
  for (let k in merge_input) {
    let cmEl = document.getElementById(`cm_${k}_${unique_id}`)!;
    cmEl.innerHTML = "";
    let /* panes = 2, */
      highlight = true,
      connect = "align",
      collapse = false;
    let config = {
      value: merge_input[k].edit ?? "",
      origLeft: merge_input[k].left ?? "", // Set to null for 2 panes
      orig: merge_input[k].right ?? "",
      lineNumbers: true,
      mode: "text/plain",
      highlightDifferences: highlight,
      connect: connect,
      collapseIdentical: collapse,
    };
    // TODO: Resizing. See https://codemirror.net/5/demo/merge.html
    merge_views[k] = CodeMirror.MergeView(cmEl, config);
  }

  return new MergeState(merge_views)
}

class MergeState {
  merge_views: Record<string, MergeView>

  constructor(merge_views: Record<string, MergeView>) {
    this.merge_views = merge_views
  }

  values(): Record<string, string> {
    let result: Record<string, string> = {};
    for (let k in this.merge_views) {
      result[k] = this.merge_views[k].editor().getValue();
    }
    return result;
  }
}

async function command_line_args(): Promise<string[]> {
  return await invoke("args");
}

async function logoutput(result: InvokeArgs) {
  console.log(result);
  await invoke("logoutput", {result: result});
}

window.addEventListener("DOMContentLoaded", () => {
  // https://github.com/tauri-apps/tauri/discussions/6119
  if ("__TAURI__" in globalThis) {
    console.log("In Tauri");
  } else {
    console.log("Not in Tauri");
  }

  let merge_views = render_input("lit", INPUT);
  document.getElementById("button_show")!.onclick = () => logoutput(merge_views.values());
});

window.addEventListener("DOMContentLoaded", async () => {
  let args: string[] = await command_line_args();
  let one_arg_tmpl = (arg: string) => html`<code>${arg}</code>`;
  lit_html_render(html`<p>Args: ${args.map(one_arg_tmpl)}</p>`, document.getElementById("args")!);
});
