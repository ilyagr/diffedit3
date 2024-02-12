use itertools::Itertools;
use std::{
    fs::DirEntry,
    path::{Path, PathBuf},
};

pub fn scan(root: &Path) -> Vec<DirEntry> {
    root.read_dir()
        .unwrap()
        .map(|maybe_entry| maybe_entry.unwrap())
        .collect_vec()
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use super::*;

    #[test]
    fn it_works() {
        dbg!(scan(&PathBuf::from_str(".").unwrap()));
    }
}
