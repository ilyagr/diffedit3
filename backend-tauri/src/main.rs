// Would prevent additional console window on Windows in release. We want the
// console window and waiting to exit until completion, though.

// #![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use clap::Parser;
use diffedit3::DataInterface;
use indexmap::IndexMap;
// Using parking_lot::Mutex for a timeout. We could alternatively use
// tokio::sync::Mutex, but the docs suggest only using it if absolutely
// neccessary.
use parking_lot::Mutex;
use tauri::menu::{Menu, MenuItemBuilder, Submenu};
// TODO: https://tauri.app/start/migrate/from-tauri-1/#migrate-to-menu-module

type DataMutex = Mutex<Box<dyn DataInterface>>;

#[tauri::command]
fn save(
    result: IndexMap<String, String>,
    state: tauri::State<DataMutex>,
) -> Result<(), diffedit3::DataSaveError> {
    // TODO: Add timeout like in diffedit3-web
    state.lock().save(result)
}

#[tauri::command]
fn get_merge_data(
    state: tauri::State<DataMutex>,
) -> Result<diffedit3::EntriesToCompare, diffedit3::DataReadError> {
    // TODO: Add timeout like in diffedit3-web
    state.lock().scan()
}

// TODO: Zoom. The `zoom` CSS property does not work with CodeMirror.
// See https://github.com/tauri-apps/tauri/issues/3310. Or just use a browser
// https://github.com/phcode-dev/phoenix-desktop/pull/162/files
//
// So far, the most promising approach is to change the `font-size` root
// CSS property
fn main() {
    let cli = diffedit3::Cli::parse();
    let input: Box<dyn diffedit3::DataInterface> =
        cli.into_data_interface().unwrap_or_else(|err| {
            eprintln!("Error: {err}");
            std::process::exit(2)
        });
    let input_mutex: DataMutex = Mutex::new(input);

    let abandon_changes_and_quit = CustomMenuItem::new(
        "abandon_changes_and_quit".to_string(),
        "Abandon Changes and Quit",
    );
    let revert = CustomMenuItem::new("revert".to_string(), "Revert to Last Save");
    let save_menu = CustomMenuItem::new("save".to_string(), "Save").accelerator("CmdOrControl+S");
    let save_and_quit = CustomMenuItem::new("save_and_quit".to_string(), "Save and Quit")
        .accelerator("CmdOrControl+Q");
    let submenu = Submenu::new(
        "File",
        Menu::new()
            .add_item(save_menu)
            .add_item(save_and_quit)
            .add_item(revert)
            .add_item(abandon_changes_and_quit),
    );
    // TODO: It'd be nice to keep Tauri's default menu and add a few items to it
    // instead of starting with a blank menu. Apparently, this is possible with
    // Tauri 2.0 (currently in beta), though the docs mention that only Submenus
    // can be added to the menu. See
    // https://github.com/tauri-apps/tauri/discussions/8853#discussioncomment-8483258
    let menu = Menu::new().add_submenu(submenu);

    tauri::Builder::default()
        .plugin(tauri_plugin_process::init())
        .menu(menu)
        .on_menu_event(|event| event.window().emit(event.menu_item_id(), ()).unwrap())
        .manage(input_mutex)
        .invoke_handler(tauri::generate_handler![get_merge_data, save])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
