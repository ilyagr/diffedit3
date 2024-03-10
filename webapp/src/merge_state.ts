import { html, render as lit_html_render } from "lit-html";

import CodeMirror, { Editor } from "codemirror";
import { MergeView } from "codemirror/addon/merge/merge";

import {
  MergeInput,
  SingleFileMergeInput,
  to_text,
} from "./backend_interactions";

class MergeState {
  merge_views: Record<string, MergeView>;

  constructor(merge_views: Record<string, MergeView>) {
    this.merge_views = merge_views;
  }

  values(): Record<string, string> {
    const result: Record<string, string> = {};
    for (const k in this.merge_views) {
      // TODO: Treat deleted values properly. Currently, the server guesses
      // whether an empty string means an empty file or a missing file.
      result[k] = this.merge_views[k].editor().getValue();
    }
    return result;
  }
}

// TODO: Split off drawing one editor. Only draw a single div in a loop.
// Or not? Is it reasonable to render lit-html in an element that was just rendered in lit-html?
// If not, could have two functions.
// Or just don't use `lit` for creating the divs in a loop; leave a comment instead.
//
/// Renders the input inside the HTML element with id `unique_id`.
export function render_input(unique_id: string, merge_input: MergeInput) {
  let templates = [];
  let k_uid = (k: string) => `${k}_${unique_id}`;
  let to_error = (input: SingleFileMergeInput) => {
    let unsupported_value = Array.from([
      { file: input.left, side: "left" },
      { file: input.right, side: "right" },
      { file: input.edit, side: "middle" },
    ]).find((v) => v.file.type == "Unsupported");
    if (unsupported_value == null) {
      return null;
    } else if (unsupported_value.file.type != "Unsupported") {
      throw new Error("this statement is unreachable; this check exists to make TS happy");
    }
    return html`<b>error</b>: ${unsupported_value.file.value} (occurred on the
      ${unsupported_value.side} side)`;
  };

  for (const k in merge_input) {
    const error = to_error(merge_input[k]);
    if (error != null) {
      templates.push(html` <details id="details_${k_uid(k)}">
        <summary><code>${k}</code>: ${error}</summary>
        <!-- TODO: Allow inserting error details here, perhaps grey out the triangle
          -- if there are no details.
          -->
      </details>`);
    } else {
      templates.push(html`
        <details open id="details_${k_uid(k)}">
          <!-- We will close this with javascript shortly. See below. -->
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
            <button id="linewrap_${k_uid(k)}" hidden>
              <!--Buggy with collapseIdentical, see comment below -->
              (Un)Wrap Lines
            </button>
          </summary>
          <div id="cm_${k_uid(k)}"></div>
        </details>
      `);
    }
  }

  const target_element = document.getElementById(unique_id)!;
  target_element.innerHTML = "";
  lit_html_render(html`${templates}`, target_element);

  let merge_views: Record<string, MergeView> = {};
  for (let k in merge_input) {
    if (to_error(merge_input[k]) != null) {
      continue;
    }
    const collapseButtonEl = document.getElementById(`collapse_${k_uid(k)}`)!;
    const linewrapButtonEl = document.getElementById(`linewrap_${k_uid(k)}`)!;
    const prevChangeButtonEl = document.getElementById(`prevChange_${k_uid(k)}`)!;
    const nextChangeButtonEl = document.getElementById(`nextChange_${k_uid(k)}`)!;
    const detailsButtonEl = <HTMLDetailsElement>(
      document.getElementById(`details_${k_uid(k)}`)!
    );
    const cmEl = document.getElementById(`cm_${k_uid(k)}`)!;

    const config = {
      value: to_text(merge_input[k].edit) ?? "",
      origLeft: to_text(merge_input[k].left) ?? "", // Set to null for 2 panes
      orig: to_text(merge_input[k].right) ?? "",
      lineNumbers: true,
      /* TODO: Toggling line wrapping breaks `collapseIdentical`. Need a
      settings system where the user can decide whether they want line wrapping,
      save, and reload. */
      lineWrapping: false,
      mode: "text/plain",
      connect: "align",
      collapseIdentical: true,
    };
    const merge_view = CodeMirror.MergeView(cmEl, config);
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
    // Starting with details closed breaks CodeMirror, especially line numbers
    // in left and right merge view.
    detailsButtonEl.open = false;
    detailsButtonEl.ontoggle = () => merge_view.editor().refresh();
    console.log(detailsButtonEl);

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
