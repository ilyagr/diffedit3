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
/*
use tauri::menu::{Menu, MenuEvent, MenuItem, Submenu};
use tauri::window::MenuType;
use tauri::Manager;
*/
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

    tauri::Builder::default()
        .plugin(tauri_plugin_process::init())
        /*
        .setup(|app| {
            let menu_items = vec![
                &MenuItem::new("save", "Save", MenuType::Item, true)
                    .with_accelerator("CmdOrControl+S")?
                    as &dyn tauri::menu::IsMenuItem<_>,
                &MenuItem::new("save_and_quit", "Save and Quit", MenuType::Item, true)
                    .with_accelerator("CmdOrControl+Q")?
                    as &dyn tauri::menu::IsMenuItem<_>,
                &MenuItem::new("revert", "Revert to Last Save", MenuType::Item, true)?
                    as &dyn tauri::menu::IsMenuItem<_>,
                &MenuItem::new(
                    "abandon_changes_and_quit",
                    "Abandon Changes and Quit",
                    MenuType::Item,
                    true,
                )? as &dyn tauri::menu::IsMenuItem<_>,
            ];
            let file_menu = Menu::with_items(app, &menu_items)?;
            let menu = Menu::with_items(
                app,
                &[&Submenu::new("File", file_menu)? as &dyn tauri::menu::IsMenuItem<_>],
            )?;
            app.set_menu(menu)?;
            Ok(())
        })
        .on_menu_event(|event| {
            event.window().emit(event.menu_item_id(), ()).unwrap();
        })
        */
        .manage(input_mutex)
        .invoke_handler(tauri::generate_handler![get_merge_data, save])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
