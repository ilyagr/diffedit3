import { invoke } from "@tauri-apps/api/tauri";
// import "diff_match_patch" 
import CodeMirror from "codemirror";
import "codemirror/mode/meta";
import "codemirror/lib/codemirror.css";
import "codemirror/addon/merge/merge";

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

  let cmEl = document.getElementById("cm");
  if (cmEl == null) return;
  let panes = 2,
    highlight = true,
    connect = "align",
    collapse = false;
  let config: any = {
    value: "First\nSecond\nFourth\nFifth",
    origLeft: "First", //panes == 3 ? orig1 : null,
    orig: "Fifth",
    lineNumbers: true,
    mode: "text/html",
    highlightDifferences: highlight, // Not in typedef
    connect: connect,
    collapseIdentical: collapse,
  };
  let merge_view = CodeMirror.MergeView(cmEl, config);
});
