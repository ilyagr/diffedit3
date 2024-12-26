import { InvokeArgs, invoke as tauriInvoke } from "@tauri-apps/api/core";
import { exit as tauriExit } from "@tauri-apps/plugin-process";

export type FileEntry =
  | { type: "Missing" }
  | { type: "Text"; value: string }
  | { type: "Unsupported"; value: string };
export type SingleFileMergeInput = {
  left: FileEntry;
  right: FileEntry;
  edit: FileEntry;
};
export type MergeInput = Record<string, SingleFileMergeInput>;

export function to_text(file_entry: FileEntry): string | null {
  if (file_entry.type == "Text") {
    return file_entry.value;
  } else {
    return null;
  }
}

// Tauri interop

// https://github.com/tauri-apps/tauri/discussions/6119
// TODO: Can this be replaced with https://github.com/tauri-apps/tauri/discussions/6119#discussioncomment-11666454
// or something less brittle?
export const TAURI_BACKEND = "__TAURI_INTERNALS__" in globalThis;

async function backend_request(
  command_name: string,
  method: string,
  content?: Object | undefined,
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
  content?: Object | undefined,
) {
  let body = null,
    headers: Record<string, string> = {};
  if (content != null) {
    body = JSON.stringify(content);
    headers["Content-Type"] = "application/json";
  }
  const response = await fetch(`/api/${command_name}`, {
    method: method,
    body: body,
    headers: headers,
  });
  if (response.ok) {
    return await response.json();
  } else {
    let text = "";
    if (response.status < 500) {
      text = `Likely bug in the webapp: got response "${response.status} ${response.statusText}" for "${command_name}" request. Additional details, if any, follow. `;
    }
    // TODO: Modify message for 4xx or 3xx error codes
    throw text + (await response.text());
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
  const data: any = await backend_request("get_merge_data", "GET");
  for (const k in data) {
    data[k] = { left: data[k][0], right: data[k][1], edit: data[k][2] };
  }
  return data;
}

// TODO Maybe
// export async function set_up_tauri_menu() {}
