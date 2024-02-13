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
//struct EntriesToCompare<P, const N: usize>(std::collections::BTreeMap<P,
// [Option<String>; N]>);
pub struct EntriesToCompare(std::collections::BTreeMap<PathBuf, [Option<String>; 3]>);

pub fn fake_data() -> EntriesToCompare {
    // let mut two_sides_map = btreemap! {
    //     "edited_file" => [
    //           Some("First\nThird\nFourth\nFourthAndAHalf\n\nFifth\nSixth\n----\
    // none two"),           Some("First\nSecond\nThird\nFifth\nSixth\n----\
    // none\n")     ],
    //     "deleted_file" => [Some("deleted"), None],
    //     "added file" => [None, Some("added")]
    // };
    let two_sides_map = vec![
        (
            "edited_file",
            [
                Some("First\nThird\nFourth\nFourthAndAHalf\n\nFifth\nSixth\n----\none two"),
                Some("First\nSecond\nThird\nFifth\nSixth\n----\none\n"),
            ],
        ),
        ("deleted_file", [Some("deleted"), None]),
        ("added file", [None, Some("added")]),
    ];
    let optstr = |opt: Option<&str>| opt.map(|s| s.to_string());
    EntriesToCompare(
        two_sides_map
            .into_iter()
            .map(|(key, [left, right])| {
                (
                    PathBuf::from(key),
                    [optstr(left), optstr(right), optstr(right)],
                )
            })
            .collect(),
    )
}

// pub fn scan_several<const N: usize>(roots: [&Path; N]) ->
// EntriesToCompare<PathBuf, N> {
// TODO: Change &PathBuf to &Path or something
pub fn scan_several(roots: [&PathBuf; 3]) -> EntriesToCompare {
    let mut result = EntriesToCompare::default();
    for (i, root) in roots.iter().enumerate() {
        for (file_entry, contents) in scan(root) {
            let value = result
                .0
                .entry(PathBuf::from(
                    file_entry.path().strip_prefix(root).expect("TODO:FIXME"),
                ))
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
        insta::assert_toml_snapshot!(scan_several([&path, &path, &path]), {
            r#"["./src/lib.rs"]"# => "(skipped because of amusing recursion)"
        },
        @r###"
        "./Cargo.toml" = [
            '''
        [package]
        name = "diff-tool-logic"
        version = "0.1.0"
        edition = "2021"

        [dependencies]
        itertools = "0.12.1"
        serde = { version = "1.0.196", features = ["serde_derive"] }
        walkdir = "2.4.0"

        [dev-dependencies]
        insta = { version = "1.34.0", features = ["redactions", "serde", "toml", "json"] }
        ''',
            '''
        [package]
        name = "diff-tool-logic"
        version = "0.1.0"
        edition = "2021"

        [dependencies]
        itertools = "0.12.1"
        serde = { version = "1.0.196", features = ["serde_derive"] }
        walkdir = "2.4.0"

        [dev-dependencies]
        insta = { version = "1.34.0", features = ["redactions", "serde", "toml", "json"] }
        ''',
            '''
        [package]
        name = "diff-tool-logic"
        version = "0.1.0"
        edition = "2021"

        [dependencies]
        itertools = "0.12.1"
        serde = { version = "1.0.196", features = ["serde_derive"] }
        walkdir = "2.4.0"

        [dev-dependencies]
        insta = { version = "1.34.0", features = ["redactions", "serde", "toml", "json"] }
        ''',
        ]
        "./src/lib.rs" = '(skipped because of amusing recursion)'
        "###);
    }
}
