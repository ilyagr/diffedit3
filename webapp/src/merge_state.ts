import { html, render as lit_html_render } from "lit-html";

import CodeMirror from "codemirror";
import { MergeView } from "codemirror/addon/merge/merge";

import {
  MergeInput,
  SingleFileMergeInput,
  to_text,
} from "./backend_interactions";

class MergeState {
  merge_views: Record<string, MergeView>;
  dom_ids: Record<string, string>;
  initial_values: Record<string, SingleFileMergeInput>;

  constructor() {
    this.merge_views = {};
    this.dom_ids = {};
    this.initial_values = {};
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

  protected getSingleMergeState(filename: string): SingleMergeState {
    let editor = this.merge_views[filename].editor();
    return {
      input: {
        left: this.initial_values[filename].left,
        right: this.initial_values[filename].right,
        edit: {
          type: "Text",
          value: this.merge_views[filename].editor().getValue(),
        },
      },
      wrapLines:
        editor.getOption("lineWrapping") ??
        false /* TODO: is this ever undefined? */,
    };
  }

  // TODO: This method should NOT be exported. It should become protected,
  // e.g. move render_input into the constructor of this class
  createCodeMirrorMergeWidget(
    unique_id: string,
    filename: string,
    merge_state: SingleMergeState
  ) {
    const input = merge_state.input;
    // This method is tightly coupled with the DOM constructed in
    // `render_input`.
    const collapseButtonEl = document.getElementById(`collapse_${unique_id}`)!;
    const linewrapButtonEl = document.getElementById(`linewrap_${unique_id}`)!;
    const prevChangeButtonEl = document.getElementById(
      `prevChange_${unique_id}`
    )!;
    const nextChangeButtonEl = document.getElementById(
      `nextChange_${unique_id}`
    )!;
    const detailsButtonEl = <HTMLDetailsElement>(
      document.getElementById(`details_${unique_id}`)!
    );
    const cmEl = document.getElementById(`cm_${unique_id}`)!;

    const config = {
      value: to_text(input.edit) ?? "",
      origLeft: to_text(input.left) ?? "", // Set to null for 2 panes
      orig: to_text(input.right) ?? "",
      lineWrapping: merge_state.wrapLines ?? true,
      collapseIdentical: true,
      lineNumbers: true,
      mode: "text/plain",
      connect: "align",
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
    linewrapButtonEl.onclick = () =>
      this.recreateCodeMirrorFlippingOption(filename, "wrapLines");
    prevChangeButtonEl.onclick = () => cm_prevChange(merge_view.editor());
    nextChangeButtonEl.onclick = () => cm_nextChange(merge_view.editor());
    // Starting with details closed breaks CodeMirror, especially line numbers
    // in left and right merge view.
    detailsButtonEl.open = false;
    detailsButtonEl.ontoggle = () => merge_view.editor().refresh();
    console.log(detailsButtonEl);

    // TODO: Resizing. See https://codemirror.net/5/demo/merge.html
    this.merge_views[filename] = merge_view;
    this.dom_ids[filename] = unique_id;
    this.initial_values[filename] = input;
  }

  protected recreateCodeMirrorFlippingOption(
    filename: string,
    option: BooleandMergeStateOption
  ) {
    const old_merge_view = this.merge_views[filename];
    if (old_merge_view == null) {
      console.warn(
        `Trying to toggle \`${option}\` option on a non-existent editor`,
        filename,
        this
      );
      return;
    }
    let dom_id = this.dom_ids[filename];
    const codemirror_dom_id = `cm_${dom_id}`;

    const current_state = this.getSingleMergeState(filename);

    const new_codemirror_element = document.createElement("div");
    document
      .getElementById(codemirror_dom_id)
      ?.replaceWith(new_codemirror_element);
    new_codemirror_element.id = codemirror_dom_id;

    this.createCodeMirrorMergeWidget(
      dom_id,
      filename,
      flip(current_state, option)
    );
    const detailsButtonEl = <HTMLDetailsElement>(
      document.getElementById(`details_${dom_id}`)!
    );
    // TODO: Perhaps one day we'll want to toggle this option when the details view is closed.
    detailsButtonEl.open = true;
    // TODO: Preserve cursor position
    // cm.scrollIntoView(null, 50); // Always happens automatically
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
          <!-- We will close this details element with javascript shortly. See below. -->
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
            <button
              id="linewrap_${k_uid(k)}"
              alt="Toggle wrapping of long lines"
              title="Toggle wrapping of long lines"
            >
              (Un)Wrap
            </button>
            <!-- TODO: Toggle right pane-->
          </summary>
          <div id="cm_${k_uid(k)}"></div>
        </details>
      `);
    }
  }

  const target_element = document.getElementById(unique_id)!;
  target_element.innerHTML = ""; // TODO: Should use replaceWith or something
  lit_html_render(html`${templates}`, target_element);

  const merge_state = new MergeState();
  for (let k in merge_input) {
    if (to_error(merge_input[k]) != null) {
      continue;
    }
    merge_state.createCodeMirrorMergeWidget(
      k_uid(k),
      k,
      fillInDefaultSettings(merge_input[k])
    );
  }

  return merge_state;
}

type SingleMergeState = {
  input: SingleFileMergeInput;
  // cursorPosition
  wrapLines: boolean;
  // collapse identical
  // rightPane
  // collapse this merge pane?
};

type BooleandMergeStateOption = "wrapLines" /* | ... */;

function fillInDefaultSettings(input: SingleFileMergeInput): SingleMergeState {
  return { input: input, wrapLines: true };
}

function flip(
  settings: SingleMergeState,
  boolean_option: BooleandMergeStateOption
) {
  let result = Object.assign({}, settings);
  result[boolean_option] = !result[boolean_option];
  return result;
}

function to_error(input: SingleFileMergeInput) {
  let unsupported_value = Array.from([
    { file: input.left, side: "left" },
    { file: input.right, side: "right" },
    { file: input.edit, side: "middle" },
  ]).find((v) => v.file.type == "Unsupported");
  if (unsupported_value == null) {
    return null;
  } else if (unsupported_value.file.type != "Unsupported") {
    throw new Error(
      "this statement is unreachable; this check exists to make TS happy"
    );
  }
  return html`<b>error</b>: ${unsupported_value.file.value} (occurred on the
    ${unsupported_value.side} side)`;
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

function cm_nextChange(cm: CodeMirror.Editor) {
  cm.execCommand("goNextDiff");
  cm.scrollIntoView(null, 50);
}
function cm_prevChange(cm: CodeMirror.Editor) {
  cm.execCommand("goPrevDiff");
  cm.scrollIntoView(null, 50);
}
