import { InvokeArgs, invoke } from "@tauri-apps/api/tauri";

import CodeMirror, { Editor } from "codemirror";
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
let thirds = Array(29).join("Third\n");
let INPUT: MergeInput = {
  edited_file: {
    left:
      "First\n" +
      thirds +
      "Fourth\nFourthAndAHalf\n\nFifth\nSixth\n----\none two",
    edit: "First\nSecond\n" + thirds + "\nFifth\nSixth\n----\none\n",
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
  let k_uid = (k: string) => `${k}_${unique_id}`;
  for (let k in merge_input) {
    templates.push(html`
      <details open>
        <summary>
          <code>${k}</code>
          <button id="collapse_${k_uid(k)}" hidden>
            (Un)Collapse (Doesn't work)
          </button>
          <button
            id="prevChange_${k_uid(k)}"
            alt="Previous Change"
            title="Previous Change"
          >
            ⇧ Previous Change
          </button>
          <button
            id="nextChange_${k_uid(k)}"
            alt="Next Change"
            title="Next Change"
          >
            ⇩ Next Change
          </button>
          <button id="linewrap_${k_uid(k)}">(Un)Wrap Lines</button>
        </summary>
        <div id="cm_${k_uid(k)}"></div>
      </details>
    `);
  }

  let target_element = document.getElementById(unique_id)!;
  target_element.innerHTML = "";
  lit_html_render(html`${templates}`, target_element);

  let merge_views: Record<string, MergeView> = {};
  for (let k in merge_input) {
    let cmEl = document.getElementById(`cm_${k_uid(k)}`)!;
    cmEl.innerHTML = "";
    let collapseButtonEl = document.getElementById(`collapse_${k_uid(k)}`)!;
    let linewrapButtonEl = document.getElementById(`linewrap_${k_uid(k)}`)!;
    let prevChangeButtonEl = document.getElementById(`prevChange_${k_uid(k)}`)!;
    let nextChangeButtonEl = document.getElementById(`nextChange_${k_uid(k)}`)!;

    let config = {
      value: merge_input[k].edit ?? "",
      origLeft: merge_input[k].left ?? "", // Set to null for 2 panes
      orig: merge_input[k].right ?? "",
      lineNumbers: true,
      lineWrapping: true,
      mode: "text/plain",
      connect: "align",
      collapseIdentical: true,
    };
    let merge_view = CodeMirror.MergeView(cmEl, config);
    merge_view.editor().setOption("extraKeys", {
      "Alt-Down": cm_nextChange,
      "Option-Down": cm_nextChange,
      "Cmd-Down": cm_nextChange,
      "Alt-Up": cm_prevChange,
      "Option-Up": cm_prevChange,
      "Cmd-Up": cm_prevChange,
      Tab: cm_nextChange,
    });
    collapseButtonEl.onclick = () => cm_collapseSame(merge_view.editor());
    linewrapButtonEl.onclick = () => cm_toggleLineWrapping(merge_view.editor());
    prevChangeButtonEl.onclick = () => cm_prevChange(merge_view.editor());
    nextChangeButtonEl.onclick = () => cm_nextChange(merge_view.editor());

    // TODO: Resizing. See https://codemirror.net/5/demo/merge.html
    merge_views[k] = merge_view;
  }

  return new MergeState(merge_views);
}

function cm_collapseSame(cm: any) {
  // console.log(cm.getOption("collapseIdentical"));
  cm.setOption(
    /* TODO: Doesn't seem to work. Might need to recreate the whole editor */
    "collapseIdentical",
    !cm.getOption("collapseIdentical")
  );
  cm.setValue(cm.getValue());
  console.log(cm.getOption("collapseIdentical"));
  cm.scrollIntoView(null, 50);
}

function cm_toggleLineWrapping(cm: any) {
  cm.setOption(
    /* TODO: Interferes with collapseIdentical, always moves cursor to beginning */
    "lineWrapping",
    !cm.getOption("lineWrapping")
  );
  cm.setValue(cm.getValue());
  // cm.scrollIntoView(null, 50); // Always happens automatically
}

function cm_nextChange(cm: Editor) {
  cm.execCommand("goNextDiff");
  cm.scrollIntoView(null, 50);
}
function cm_prevChange(cm: Editor) {
  cm.execCommand("goPrevDiff");
  cm.scrollIntoView(null, 50);
}

class MergeState {
  merge_views: Record<string, MergeView>;

  constructor(merge_views: Record<string, MergeView>) {
    this.merge_views = merge_views;
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
  await invoke("logoutput", { result: result });
}

async function get_merge_data() {
  let data = await invoke("get_merge_data")
  for (let k in data) {
    data[k] = {left: data[k][0], right: data[k][1], edit: data[k][2]}
  }
  return data;
}

window.addEventListener("DOMContentLoaded", async () => {
  // https://github.com/tauri-apps/tauri/discussions/6119
  if ("__TAURI__" in globalThis) {
    console.log("In Tauri");
  } else {
    console.log("Not in Tauri");
  }

  let loading_elt = document.getElementById("loading_message");
  loading_elt.innerHTML="";
  lit_html_render(html`
      <h2>Loading...</h2>
      <p>Getting the data we want to merge...</p>
  `, loading_elt);
  let input = await get_merge_data();

  let merge_views = render_input("lit", input);

  lit_html_render(html``, loading_elt);
  document.getElementById("button_show")!.onclick = () =>
    logoutput(merge_views.values());
});

window.addEventListener("DOMContentLoaded", async () => {
  let args: string[] = await command_line_args();
  let one_arg_tmpl = (arg: string) => html`<code>${arg}</code>`;
  lit_html_render(
    html`<p>Args: ${args.map(one_arg_tmpl)}</p>`,
    document.getElementById("args")!
  );
});
