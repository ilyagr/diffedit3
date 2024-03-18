import { html, render as lit_html_render } from "lit-html";

import CodeMirror from "codemirror";
import { MergeView } from "codemirror/addon/merge/merge";

import toggle_rightside_icon from "../assets/icons/rightpane-icon.svg";
import pin_icon from "../assets/icons/pin.svg";
import pin_outline_icon from "../assets/icons/pin-outline.svg";

import {
  MergeInput,
  SingleFileMergeInput,
  to_text,
} from "./backend_interactions";
import { replaceElementByIdWithNewEmptyDiv, unreachable } from "./utils";

export class MergeState {
  protected merge_views: Record<string, MergeView>;
  protected dom_ids: Record<string, string>;
  protected initial_values: Record<string, SingleFileMergeInput>;

  protected constructor() {
    this.merge_views = {};
    this.dom_ids = {};
    this.initial_values = {};
  }

  public values(): Record<string, string> {
    const result: Record<string, string> = {};
    for (const k in this.merge_views) {
      // TODO: Treat deleted values properly. Currently, the server guesses
      // whether an empty string means an empty file or a missing file.
      result[k] = this.merge_views[k].editor().getValue();
    }
    return result;
  }

  public refreshAll() {
    for (const k in this.merge_views) {
      this.merge_views[k].editor().refresh();
    }
  }

  /// Renders the input inside the HTML element with id `unique_id`.
  public static renderInDomElement(unique_id: string, merge_input: MergeInput) {
    let templates = [];
    let k_uid = (k: string) => `${k}_${unique_id}`;

    for (const k in merge_input) {
      const error = to_error(merge_input[k]);
      if (error != null) {
        templates.push(html`
          <details id="details_${k_uid(k)}" class="merge-view">
            <summary>
              <code>${k}</code><span class="if-details-closed">: ${error}</span>
            </summary>
            ${error}
          </details>
        `);
      } else {
        templates.push(html`
          <details open id="details_${k_uid(k)}" class="merge-view">
            <!-- We will close this details element with javascript shortly. See below. -->
            <summary>
              <span id="pin_${k_uid(k)}" class="pin-span">
                <!-- TODO: This could be a toggleable checkbox as in
                  ---- https://developer.mozilla.org/en-US/docs/Web/CSS/:checked#toggling_elements_with_a_hidden_checkbox
                  --->
                <!-- TODO: Move style to CSS -->
                <!-- Try to move using relative position -->
                <img
                  src=${pin_outline_icon}
                  class="if-not-pinned"
                  style="height: 1em; margin-left: 0em; margin-right: 0.1em; margin-top: 0.1em;"
                  alt="Show only this file (pin)"
                  title="Show only this file (pin)"
                />
                <img
                  src=${pin_icon}
                  class="if-pinned"
                  style="height: 1em; margin-left: 0em; margin-right: 0.1em; margin-top: 0.1em;"
                  alt="Unpin"
                  title="Unpin"
                />
              </span>
              <code>${k}</code>
              <span class="if-details-open">
                <button
                  id="prevChange_${k_uid(k)}"
                  alt="Previous Change (Ctrl-↑, Super-↑)"
                  title="Previous Change (Ctrl-↑, Super-↑)"
                >
                  ▲
                  <!-- Previous Change. Alternatives: ⇧⇑▲
                  ---- https://stackoverflow.com/a/22156412/563359
                  --->
                </button>
                <button
                  id="nextChange_${k_uid(k)}"
                  alt="Next Change (Ctrl-↓, Super-↓)"
                  title="Next Change (Ctrl-↓, Super-↓)"
                >
                  ▼
                  <!-- Next Change. Alternatives:⇓⇩▼.
                  ---- or <div style="transform: scale(1, -1);">⇧</div>
                  ---- (For some reason, the down arrow ⇓ doesn't look the same as
                  ---- the up arrow ⇧, so the latter can be flipped)
                  --->
                </button>
                <button
                  id="rightside_${k_uid(k)}"
                  alt="Toggle visibility of the right pane"
                  title="Toggle visibility of the right pane"
                >
                  <!-- TODO: Less hacky way to vertically center the image -->
                  <img
                    src=${toggle_rightside_icon}
                    style="height: 1em; position: relative; top: 2px; "
                  />
                  <!-- 2 ⬄ 3 -->
                </button>
                <button
                  id="linewrap_${k_uid(k)}"
                  alt="Toggle wrapping of long lines"
                  title="Toggle wrapping of long lines"
                >
                  (Un)Wrap
                </button>
                <button
                  id="collapse_${k_uid(k)}"
                  alt="Toggle collapse of identical regions"
                  title="Toggle collapse of identical regions"
                >
                  (Un)Collapse
                </button>
                <button
                  id="align_${k_uid(k)}"
                  alt="Toggle insertion of blank lines to align changed regions."
                  title="Toggle insertion of blank lines to align changed regions."
                >
                  (Un)Align
                </button>
              </span>
            </summary>
            <div id="cm_${k_uid(k)}"></div>
          </details>
        `);
      }
    }

    let target_element = replaceElementByIdWithNewEmptyDiv(unique_id)!;
    // Rendering the template here defeats lit-htmls's optimizations and is not
    // at all in the spirit of lit-html. The original reason for this design is
    // to have CodeMirror rendered on an already shown DOM element, which is
    // hopefully the use-case CodeMirror 5 was most tested for. lit-html is used
    // mainly for its contextual escaping functionality as opposed to its
    // rendering opimizations.
    //
    // TODO: Consider improving this design and making it more modern.
    //    - We can't use lit-element because it's unlikely CodeMirror5 works
    //      well with shadow DOM.
    //    - We could create all the CodeMirror elements first and then return
    //      the resulting lit-html template. The caller can then render it. This
    //      could potentially trigger some CodeMirror bugs (or it might be
    //      fine). It might (or might not) also be slower since all the editor
    //      elements would be generated before any DOM is rendered. (Or maybe
    //      it's not slower?)
    lit_html_render(html`${templates}`, target_element);

    const merge_state = new MergeState();
    for (let k in merge_input) {
      if (to_error(merge_input[k]) != null) {
        continue;
      }
      merge_state.createCodeMirrorMergeWidget(
        k_uid(k),
        k,
        fillInDefaultSettings(merge_input[k]),
      );
    }

    return merge_state;
  }

  protected createCodeMirrorMergeWidget(
    unique_id: string,
    filename: string,
    merge_state: SingleMergeState,
  ) {
    const input = merge_state.input;
    // This method is tightly coupled with the DOM constructed in
    // `renderInDomElement`.
    const collapseButtonEl = document.getElementById(`collapse_${unique_id}`)!;
    const linewrapButtonEl = document.getElementById(`linewrap_${unique_id}`)!;
    const rightsideButtonEl = document.getElementById(
      `rightside_${unique_id}`,
    )!;
    const alignButtonEl = document.getElementById(`align_${unique_id}`)!;
    const prevChangeButtonEl = document.getElementById(
      `prevChange_${unique_id}`,
    )!;
    const nextChangeButtonEl = document.getElementById(
      `nextChange_${unique_id}`,
    )!;
    const pinButtonEl = document.getElementById(`pin_${unique_id}`)!;
    const detailsButtonEl = <HTMLDetailsElement>(
      document.getElementById(`details_${unique_id}`)!
    );
    const cmEl = document.getElementById(`cm_${unique_id}`)!;

    let rightSide = undefined;
    if (merge_state.showRightSide) {
      rightSide = to_text(input.right) ?? "";
    }
    const config = {
      value: to_text(input.edit) ?? "",
      origLeft: to_text(input.left) ?? "", // Set to null for 2 panes
      orig: rightSide,
      lineWrapping: merge_state.wrapLines,
      collapseIdentical:
        merge_state.collapseIdentical /* TODO: Could make the number of lines adjustable */,
      lineNumbers: true,
      mode: "text/plain",
      connect: merge_state.align ? "align" : undefined,
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
    if (merge_state.cursorPosition != null) {
      merge_view.editor().setSelection(merge_state.cursorPosition);
      merge_view.editor().scrollIntoView(null, 50);
    }

    collapseButtonEl.onclick = () =>
      this.recreateCodeMirrorFlippingOption(filename, "collapseIdentical");
    linewrapButtonEl.onclick = () =>
      this.recreateCodeMirrorFlippingOption(filename, "wrapLines");
    rightsideButtonEl.onclick = () =>
      this.recreateCodeMirrorFlippingOption(filename, "showRightSide");
    alignButtonEl.onclick = () =>
      this.recreateCodeMirrorFlippingOption(filename, "align");
    prevChangeButtonEl.onclick = () => cm_prevChange(merge_view.editor());
    nextChangeButtonEl.onclick = () => cm_nextChange(merge_view.editor());

    const parent_window = pinButtonEl.closest(".app-window")!;
    pinButtonEl.onclick = () => this.toggle_pinning(parent_window, unique_id);
    // Starting with details closed breaks CodeMirror, especially line numbers
    // in left and right merge view.
    detailsButtonEl.open = false;
    detailsButtonEl.ontoggle = () => {
      if (!detailsButtonEl.open) {
        // We just closed the details
        if (parent_window.classList.contains("pinned-mode")) {
          this.toggle_pinning(parent_window, unique_id);
        }
      } else {
        merge_view.editor().refresh();
      }
    };
    // console.log(detailsButtonEl);

    // TODO: Resizing. See https://codemirror.net/5/demo/merge.html
    this.merge_views[filename] = merge_view;
    this.dom_ids[filename] = unique_id;
    this.initial_values[filename] = input;
  }

  protected getSingleMergeState(filename: string): SingleMergeState {
    const merge_view = this.merge_views[filename];
    const editor = merge_view.editor();
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
      collapseIdentical: !!(editor as any).getOption(
        "collapseIdentical",
      ) /* TODO: Allow integer values? */,
      showRightSide: !!merge_view.rightOriginal(),
      align: (editor as any).getOption("connect") == "align",
      cursorPosition: editor.getCursor(),
    };
  }

  protected recreateCodeMirrorFlippingOption(
    filename: string,
    option: BooleandMergeStateOption,
  ) {
    const old_merge_view = this.merge_views[filename];
    if (old_merge_view == null) {
      console.warn(
        `Trying to toggle \`${option}\` option on a non-existent editor`,
        filename,
        this,
      );
      return;
    }
    let dom_id = this.dom_ids[filename];

    const current_state = this.getSingleMergeState(filename);
    replaceElementByIdWithNewEmptyDiv(`cm_${dom_id}`);
    this.createCodeMirrorMergeWidget(
      dom_id,
      filename,
      flip(current_state, option),
    );
    const detailsButtonEl = <HTMLDetailsElement>(
      document.getElementById(`details_${dom_id}`)!
    );
    // TODO: Perhaps one day we'll want to toggle this option when the details view is closed.
    detailsButtonEl.open = true;
    // TODO: Preserve cursor position
    // cm.scrollIntoView(null, 50); // Always happens automatically
  }

  protected toggle_pinning(parent_window: Element, unique_id: string) {
    parent_window.classList.toggle("pinned-mode");
    for (const merge_view of parent_window.getElementsByClassName(
      `merge-view` /* TODO: Should be this collections's merge views only */,
    )) {
      if (merge_view.id == `details_${unique_id}`) {
        merge_view.classList.toggle("pinned-mode-selected");
        merge_view.classList.add("pinned-mode-was-last-toggled"); // For animation purposes
        if (merge_view.classList.contains("pinned-mode-selected")) {
          (merge_view as HTMLDetailsElement).open = true;
        }
      } else {
        merge_view.classList.remove("pinned-mode-selected");
        merge_view.classList.remove("pinned-mode-was-last-toggled"); // For animation purposes
      }
    }
    this.refreshAll();
    return false;
  }
}

type SingleMergeState = {
  input: SingleFileMergeInput;
  wrapLines: boolean;
  collapseIdentical: boolean;
  showRightSide: boolean;
  align: boolean;
  // TODO: Also try to preserve the selection? Viewport position?
  cursorPosition?: CodeMirror.Position;
  // TODO: Track whether the entire merge pane is collapsed? This would be
  // useful if the options can be toggled with the pane closed, which is not the
  // case as of this writing.
};

type BooleandMergeStateOption =
  | "wrapLines"
  | "collapseIdentical"
  | "showRightSide"
  | "align";

function fillInDefaultSettings(input: SingleFileMergeInput): SingleMergeState {
  return {
    input: input,
    wrapLines: true,
    collapseIdentical: true,
    align: true,
    showRightSide: true,
  };
}

function flip(
  settings: SingleMergeState,
  boolean_option: BooleandMergeStateOption,
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
    unreachable();
  }
  return html`<b>error</b>: ${unsupported_value.file.value} (occurred on the
    ${unsupported_value.side} side)`;
}

// For margin > 150, the cursor is centered vertically for CodeMirror of default
// height (which is 300).
// TODO: We can add logic where if cursor is down 80% of the editor view on "Go
// to next diff", move it to 25% (and the opposite for go to previous diff)
const SCROLL_MARGIN = 200;
function cm_nextChange(cm: CodeMirror.Editor) {
  cm.execCommand("goNextDiff");
  cm.scrollIntoView(null, SCROLL_MARGIN);
}
function cm_prevChange(cm: CodeMirror.Editor) {
  cm.execCommand("goPrevDiff");
  cm.scrollIntoView(null, SCROLL_MARGIN);
}
