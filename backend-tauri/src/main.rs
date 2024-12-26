// Would prevent additional console window on Windows in release. We want the
// console window and waiting to exit until completion, though.

// #![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use clap::Parser;
use diffedit3::DataInterface;
use indexmap::IndexMap;
use parking_lot::Mutex;
use tauri::menu::{MenuBuilder, MenuItemBuilder, Submenu};
use tauri::{Manager, State};
use tauri_plugin_process::init as process_init;

type DataMutex = Mutex<Box<dyn DataInterface>>;

#[tauri::command]
fn save(
    result: IndexMap<String, String>,
    state: State<DataMutex>,
) -> Result<(), diffedit3::DataSaveError> {
    state.lock().save(result)
}

#[tauri::command]
fn get_merge_data(
    state: State<DataMutex>,
) -> Result<diffedit3::EntriesToCompare, diffedit3::DataReadError> {
    state.lock().scan()
}

fn main() {
    let cli = diffedit3::Cli::parse();
    let input: Box<dyn diffedit3::DataInterface> =
        cli.into_data_interface().unwrap_or_else(|err| {
            eprintln!("Error: {err}");
            std::process::exit(2)
        });
    let input_mutex: DataMutex = Mutex::new(input);

    tauri::Builder::default()
        .plugin(process_init())
        .setup(|app| {
            let manager = app.handle();

            let abandon_changes_and_quit =
                MenuItemBuilder::with_id("abandon_changes_and_quit", "Abandon Changes and Quit")
                    .build(manager)?;

            let revert =
                MenuItemBuilder::with_id("revert", "Revert to Last Save").build(manager)?;

            let save_menu = MenuItemBuilder::with_id("save", "Save")
                .accelerator("CmdOrControl+S")
                .build(manager)?;

            let save_and_quit = MenuItemBuilder::with_id("save_and_quit", "Save and Quit")
                .accelerator("CmdOrControl+Q")
                .build(manager)?;

            // Build the File menu
            let mut file_menu = MenuBuilder::new(manager);
            file_menu = file_menu.item(&save_menu);
            file_menu = file_menu.item(&save_and_quit);
            file_menu = file_menu.item(&revert);
            file_menu = file_menu.item(&abandon_changes_and_quit);
            let file_menu = file_menu.build()?;

            // Create a Submenu from the File menu
            let file_submenu = Submenu::new(manager, "File", file_menu);

            // Build the top-level menu
            let mut menu_builder = MenuBuilder::new(manager);
            menu_builder = menu_builder.submenu(file_submenu);
            let menu = menu_builder.build()?;

            if let Some(main_window) = app.get_window("main") {
                main_window.set_menu(menu)?;
                main_window.on_menu_event(move |event| {
                    main_window.emit(event.menu_item_id(), ()).unwrap();
                });
            }

            Ok(())
        })
        .manage(input_mutex)
        .invoke_handler(tauri::generate_handler![get_merge_data, save])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
