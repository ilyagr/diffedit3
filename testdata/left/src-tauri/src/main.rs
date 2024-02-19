// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use indexmap::IndexMap;

#[tauri::command]
fn args() -> Vec<String> {
    std::env::args().collect()
}

#[tauri::command]
fn logoutput(result: IndexMap<String, String>) {
    for (name, contents) in result {
        let len = contents.len();
        println!("{name}: {len} bytes");
    }
    println!();
}

// TODO: Zoom. The `zoom` CSS property does not work with CodeMirror.
// See https://github.com/tauri-apps/tauri/issues/3310. Or just use a browser
// https://github.com/phcode-dev/phoenix-desktop/pull/162/files
//
// So far, the most promising approach is to change the `font-size` root
// CSS property
fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![args, logoutput])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
