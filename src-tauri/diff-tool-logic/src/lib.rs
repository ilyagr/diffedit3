use std::path::Path;
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

#[cfg(test)]
mod tests {
    use itertools::Itertools;
    use std::path::PathBuf;
    use std::str::FromStr;

    use super::*;

    #[test]
    fn it_works() {
        dbg!(scan(&PathBuf::from_str(".").unwrap()).collect_vec());
    }
}
