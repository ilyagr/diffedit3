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

#[derive(Debug, Clone, PartialEq, Eq, Default)]
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
        let path = PathBuf::from_str(".").unwrap();
        // dbg!(scan(&path).collect_vec());
        dbg!(scan_several([&path, &path, &path]));
    }
}
