import { html, render as lit_html_render } from "lit-html";

import {
  get_merge_data,
  save,
  exit_fatal_error,
  exit_success,
  TAURI_BACKEND,
  exit_user_abandoned_merge,
} from "./backend_interactions";
import {render_input} from "./merge_state";

// Error handling
function show_error_to_user(e: any) {
  console.log("Caught error, showing to user:", e);
  const dialogElt = <HTMLDialogElement>(
    document.getElementById("modal_dialog_with_message")!
  );
  const dialogContentsElt = document.getElementById(
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
  const loading_elt = document.getElementById("loading_message")!;
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

  const merge_views = render_input("lit", input);

  lit_html_render(html``, loading_elt);
  const save_or_tell_user = async () =>
    await run_and_show_any_errors_to_user(async () => {
      await save(merge_views.values());
    });

  const save_button = <HTMLButtonElement>document.getElementById("button_save")!;
  const save_and_quit_button = <HTMLButtonElement>(
    document.getElementById("button_save_and_quit")!
  );
  // TODO: Saving animation (disabled?) on the button. Inside `save_or_tell_user?`
  // Should also affect "Save and Quit" and the menu items with Tauri
  const save_and_quit_or_tell_user = async () =>
    await run_and_show_any_errors_to_user(async () => {
      await save(merge_views.values());
      // It's too late for the user to press the save buttons,
      // the server will be dead in a moment.
      save_button.disabled = true;
      save_and_quit_button.disabled = true;
      exit_success(); // Do not wait for the result
      // Make sure the exit command has time to get sent. For local server backend,
      // we still need to close the window manually.
      await new Promise((r) => setTimeout(r, 100));
      window.close();
    });
  const abandon_changes_and_quit = async () =>
    await run_and_show_any_errors_to_user(async () => {
      // It's too late for the user to press the save buttons,
      // the server will be dead in a moment.
      save_button.disabled = true;
      save_and_quit_button.disabled = true;
      exit_user_abandoned_merge(); // Do not wait for the result
      // Make sure the exit command has time to get sent. For local server backend,
      // we still need to close the window manually.
      await new Promise((r) => setTimeout(r, 100));
      window.close();
    });
  const revert = () => {
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
      const CtrlOrCmd = e.metaKey || e.ctrlKey;
      if (e.key == "s" && CtrlOrCmd) {
        await save_or_tell_user();
        e.preventDefault();
        return false;
      }
      return true;
    });
  }

  // TODO: Some sort of the description of what we are comparing
});
