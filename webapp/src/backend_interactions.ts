import { InvokeArgs, invoke as tauriInvoke } from "@tauri-apps/api/tauri";
import { exit as tauriExit } from "@tauri-apps/api/process";

type SingleFileMergeInput = {
  left: string | null;
  right: string | null;
  edit: string | null;
};
export type MergeInput = Record<string, SingleFileMergeInput>;

// Tauri interop

// https://github.com/tauri-apps/tauri/discussions/6119
export const TAURI_BACKEND = "__TAURI__" in globalThis;

export async function command_line_args(): Promise<string[]> {
  if (TAURI_BACKEND) {
    return await tauriInvoke("args");
  } else {
    return await ["unavailable"];
  }
}

export async function logoutput(result: InvokeArgs) {
  console.log(result);
  await tauriInvoke("logoutput", { result: result });
}

async function backend_request(
  command_name: string,
  method: string,
  content?: Object | undefined
) {
  if (TAURI_BACKEND) {
    let tauri_args = {};
    if (content != null) {
      tauri_args = { result: content };
    }
    return await tauriInvoke(command_name, tauri_args);
  } else {
    return await http_backend_request(command_name, method, content);
  }
}

async function http_backend_request(
  command_name: string,
  method: string,
  content?: Object | undefined
) {
  let body = null;
  if (content != null) {
    body = JSON.stringify(content);
  }
  let response = await fetch(`/api/${command_name}`, {
    method: method,
    body: body,
  });
  let data = await response.json();
  if (response.ok) {
    return data;
  } else {
    // TODO: Modify message for 4xx or 3xx error codes
    throw data;
  }
}

async function exit(code: number) {
  if (TAURI_BACKEND) {
    await tauriExit(code);
  } else {
    await http_backend_request("exit", "POST", code);
  }
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
  return await backend_request("save", "PUT", result);
}

export async function get_merge_data(): Promise<MergeInput> {
  let data: any = await backend_request("get_merge_data", "GET");
  for (let k in data) {
    data[k] = { left: data[k][0], right: data[k][1], edit: data[k][2] };
  }
  return data;
}

// TODO Maybe
// export async function set_up_tauri_menu() {}
