use std::path::{Path, PathBuf};
use walkdir::{DirEntry, WalkDir};

pub fn scan(root: &Path) -> impl Iterator<Item = (DirEntry, String)> {
    // As an alternative to WalkDir, see
    // https://github.com/martinvonz/jj/blob/af8eb3fd74956effee00acf00011ff0413607213/lib/src/local_working_copy.rs#L849
    WalkDir::new(root)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
        .map(|e| {
            (
                e.clone(),
                std::fs::read_to_string(e.path())
                    .unwrap_or_else(|_| panic!("FIXME on {:?}", e.path())),
            )
        })
}

#[derive(Debug, Clone, PartialEq, Eq, Default, serde::Serialize, serde::Deserialize)]
//struct EntriesToCompare<P, const N: usize>(std::collections::BTreeMap<P, [Option<String>; N]>);
pub struct EntriesToCompare(std::collections::BTreeMap<PathBuf, [Option<String>; 3]>);

// pub fn scan_several<const N: usize>(roots: [&Path; N]) -> EntriesToCompare<PathBuf, N> {
pub fn scan_several(roots: [&Path; 3]) -> EntriesToCompare {
    let mut result = EntriesToCompare::default();
    for (i, root) in roots.iter().enumerate() {
        for (file_entry, contents) in scan(root) {
            let value = result
                .0
                .entry(PathBuf::from(file_entry.path()))
                .or_insert(Default::default())
                .as_mut();
            value[i] = Some(contents);
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use super::*;

    #[test]
    fn it_works() {
        let path = PathBuf::from_str("../src").unwrap();
        // dbg!(scan(&path).collect_vec());
        insta::assert_toml_snapshot!(scan_several([&path, &path, &path]), @r###"
        "../src/main.rs" = [
            '''
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
        ''',
            '''
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
        ''',
            '''
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
        ''',
        ]
        "###);
    }
}
