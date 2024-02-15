// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::path::PathBuf;

use clap::Parser;
use indexmap::IndexMap;

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    dirs: Vec<PathBuf>,
    /// Use demo fake data
    #[arg(long)]
    demo: bool,
}

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

#[tauri::command]
fn get_merge_data() -> diff_tool_logic::EntriesToCompare {
    let cli = Cli::parse();
    if cli.demo {
        diff_tool_logic::fake_data()
    } else {
        let dirs = match &cli.dirs.as_slice() {
            [left, right, output] => [left, right, output],
            [left, right] => [left, right, right],
            _ => panic!("Arguments should have been verified by now!"),
        };
        diff_tool_logic::scan_several(dirs)
    }
}

// TODO: Zoom. The `zoom` CSS property does not work with CodeMirror.
// See https://github.com/tauri-apps/tauri/issues/3310. Or just use a browser
// https://github.com/phcode-dev/phoenix-desktop/pull/162/files
//
// So far, the most promising approach is to change the `font-size` root
// CSS property
fn main() {
    let cli = Cli::parse();

    if !cli.demo && (cli.dirs.len() < 2 || cli.dirs.len() > 3) {
        todo!("ERROR");
    };

    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![args, logoutput, get_merge_data])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
