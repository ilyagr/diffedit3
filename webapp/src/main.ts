import { html, render as lit_html_render } from "lit-html";

import CodeMirror, { Editor } from "codemirror";
import { MergeView } from "codemirror/addon/merge/merge";

import {
  MergeInput,
  get_merge_data,
  save,
  // command_line_args,
  exit_fatal_error,
  exit_success,
  TAURI_BACKEND,
  exit_user_abandoned_merge,
} from "./backend_interactions";

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

// TODO: Split off drawing one editor. Only draw a single div in a loop.
// Or not? Is it reasonable to render lit-html in an element that was just rendered in lit-html?
// If not, could have two functions.
// Or just don't use `lit` for creating the divs in a loop; leave a comment instead.
//
/// Renders the input inside the HTML element with id `unique_id`.
function render_input(unique_id: string, merge_input: MergeInput) {
  let templates = [];
  let k_uid = (k: string) => `${k}_${unique_id}`;
  for (let k in merge_input) {
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

  let target_element = document.getElementById(unique_id)!;
  target_element.innerHTML = "";
  lit_html_render(html`${templates}`, target_element);

  let merge_views: Record<string, MergeView> = {};
  for (let k in merge_input) {
    let collapseButtonEl = document.getElementById(`collapse_${k_uid(k)}`)!;
    let linewrapButtonEl = document.getElementById(`linewrap_${k_uid(k)}`)!;
    let prevChangeButtonEl = document.getElementById(`prevChange_${k_uid(k)}`)!;
    let nextChangeButtonEl = document.getElementById(`nextChange_${k_uid(k)}`)!;
    let detailsButtonEl = <HTMLDetailsElement>(
      document.getElementById(`details_${k_uid(k)}`)!
    );
    let cmEl = document.getElementById(`cm_${k_uid(k)}`)!;

    let config = {
      value: merge_input[k].edit ?? "",
      origLeft: merge_input[k].left ?? "", // Set to null for 2 panes
      orig: merge_input[k].right ?? "",
      lineNumbers: true,
      /* TODO: Toggling line wrapping breaks `collapseIdentical`. Need a
      settings system where the user can decide whether they want line wrapping,
      save, and reload. */
      lineWrapping: false,
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

// Error handling
function show_error_to_user(e: any) {
  console.log("Caught error, showing to user:", e);
  let dialogElt = <HTMLDialogElement>(
    document.getElementById("modal_dialog_with_message")!
  );
  let dialogContentsElt = document.getElementById(
    "message_of_modal_dialog_with_message"
  )!;
  lit_html_render(`${String(e)}`, dialogContentsElt);
  dialogElt.showModal();
  console.log("Done showing error to user.");
}

async function run_and_show_any_errors_to_user<T>(f: {
  (): Promise<T>;
  (): any;
}): Promise<T | undefined> {
  try {
    return await f();
  } catch (e) {
    show_error_to_user(e);
  }
}

import { listen } from "@tauri-apps/api/event";
window.addEventListener("DOMContentLoaded", async () => {
  let loading_elt = document.getElementById("loading_message")!;
  // TODO: Try the until directive?
  loading_elt.innerHTML = "";
  lit_html_render(
    html`
      <h2>Loading...</h2>
      <p>Getting the data we want to merge...</p>
    `,
    loading_elt
  );

  let input;
  try {
    input = await get_merge_data();
  } catch (e) {
    show_error_to_user(e);
    await exit_fatal_error();
    throw new Error("Internal error: this statement should be unreachable");
  }

  lit_html_render(
    html`
      <h2>Loading...</h2>
      <p>Rendering diffs...</p>
    `,
    loading_elt
  );

  let merge_views = render_input("lit", input);

  lit_html_render(html``, loading_elt);
  let save_or_tell_user = async () =>
    await run_and_show_any_errors_to_user(async () => {
      await save(merge_views.values());
    });

  let save_button = <HTMLButtonElement>document.getElementById("button_save")!;
  let save_and_quit_button = <HTMLButtonElement>(
    document.getElementById("button_save_and_quit")!
  );
  // TODO: Saving animation (disabled?) on the button. Inside `save_or_tell_user?`
  // Should also affect "Save and Quit" and the menu items with Tauri
  let save_and_quit_or_tell_user = async () =>
    await run_and_show_any_errors_to_user(async () => {
      await save(merge_views.values());
      // It's too late for the user to press the save buttons,
      // the server will be dead in a moment.
      save_button.disabled = true;
      save_and_quit_button.disabled = true;
      exit_success(); // Do not wait for the result
      // Make sure the exit command has time to get sent. For local server backend,
      // we still need to close the window manually.
      await new Promise((r) => setTimeout(r, 200));
      window.close();
    });
  let abandon_changes_and_quit = async () =>
    await run_and_show_any_errors_to_user(async () => {
      // It's too late for the user to press the save buttons,
      // the server will be dead in a moment.
      save_button.disabled = true;
      save_and_quit_button.disabled = true;
      exit_user_abandoned_merge(); // Do not wait for the result
      // Make sure the exit command has time to get sent. For local server backend,
      // we still need to close the window manually.
      await new Promise((r) => setTimeout(r, 200));
      window.close();
    });
  let revert = () => {
    window.location.reload();
    return false;
  };
  // TODO: Bind Ctrl- Or Cmd-S to save and Ctrl- Or Cmd-Q to to quit
  save_button.onclick = save_or_tell_user;
  save_and_quit_button.onclick = save_and_quit_or_tell_user;
  document.getElementById("button_abandon_changes_and_quit")!.onclick =
    abandon_changes_and_quit;
  document.getElementById("button_revert")!.onclick = revert;
  if (TAURI_BACKEND) {
    // Events from the app menu
    // Not sure whether I need to "unlisten"
    /* const unlisten = */ await listen("save", async (_event) =>
      save_or_tell_user()
    );
    await listen("save_and_quit", async (_event) =>
      save_and_quit_or_tell_user()
    );
    await listen("revert", async (_event) => revert());
    await listen("abandon_changes_and_quit", async (_event) =>
      exit_user_abandoned_merge()
    );
  } else {
    // Tauri takes care of this via the menu.
    document.addEventListener("keydown", async (e) => {
      // TODO: Only check for Meta on Mac, don't check for Meta elsewhere
      let CtrlOrCmd = e.metaKey || e.ctrlKey;
      if (e.key == "s" && CtrlOrCmd) {
        await save_or_tell_user();
        e.preventDefault();
        return false;
      }
      return true;
    });
  }

  // TODO: Some sort of the description of what we are comparing
  // The following is test code that's obsolete but does this, not prettily
  // let args: string[] = await command_line_args();
  // let one_arg_tmpl = (arg: string) => html`<code>${arg}</code>`;
  // lit_html_render(
  //   html`<p>Args: ${args.map(one_arg_tmpl)}</p>`,
  //   document.getElementById("args")!
  // );
});
