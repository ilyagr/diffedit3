import { InvokeArgs, invoke } from "@tauri-apps/api/tauri";
import { exit } from "@tauri-apps/api/process";

// Tauri interop

// https://github.com/tauri-apps/tauri/discussions/6119
export const TAURI_BACKEND = "__TAURI__" in globalThis;

export async function command_line_args(): Promise<string[]> {
  if (TAURI_BACKEND) {
    return await invoke("args");
  } else {
    return await ["unavailable"];
  }
}

export async function logoutput(result: InvokeArgs) {
  console.log(result);
  await invoke("logoutput", { result: result });
}

export async function exit_success() {
  await exit(0);
}

export async function exit_user_abandoned_merge() {
  await exit(1);
}

export async function exit_fatal_error() {
  await exit(2);
}

export async function save(result: InvokeArgs) {
  console.log(result);
  await invoke("save", { result: result });
}

export async function get_merge_data() {
  let data: any;
  if (TAURI_BACKEND) {
    data = await invoke("get_merge_data");
  } else {
    let response = await fetch("/api/inputdata.json");
    console.log(
      response.status,
      response.statusText,
      ". Am I OK?",
      response.ok
    );
    data = await response.json();
  }
  for (let k in data) {
    data[k] = { left: data[k][0], right: data[k][1], edit: data[k][2] };
  }
  return data;
}

// TODO Maybe
// export async function set_up_tauri_menu() {}
