use std::path::{Path, PathBuf};

use walkdir::{DirEntry, WalkDir};

use crate::{DataInterface, DataReadError, DataSaveError, EntriesToCompare, FakeData, FileEntry};

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ThreeDirInput {
    pub left: PathBuf,
    pub right: PathBuf,
    pub edit: PathBuf,
}

impl DataInterface for ThreeDirInput {
    // TODO: A more efficient `get_valid_entries` implementation

    fn scan(&self) -> Result<EntriesToCompare, DataReadError> {
        let Self { left, right, edit } = self;
        scan_several([left, right, edit])
    }

    fn save_unchecked(
        &mut self,
        result: indexmap::IndexMap<String, String>,
    ) -> Result<(), DataSaveError> {
        // TODO: Currently, the webapp does not distinguish between empty and
        // missing files. We should work around it by guessing: if either the
        // left or right side are missing, delete the file.
        // TODO: Preserve the executable bit
        let Self { edit: outdir, .. } = self;
        for (relpath, contents) in result.into_iter() {
            let relpath = PathBuf::from(relpath);
            let path = outdir.join(relpath);
            std::fs::write(path.clone(), contents)
                .map_err(|io_err| DataSaveError::IOError(path, io_err))?;
        }
        Ok(())
    }
}

use clap::Parser;
// TODO: Maybe try https://docs.rs/clap/latest/clap/_derive/index.html#adding-hand-implemented-subcommands-to-a-derived-application
// to make alternative parsers for demo and not demo. More likely, it's not
// worth the time. Just use a subcommand?
// TODO: Move the docstring to `backend_tauri`
/// Compare three directories, allowing the user to edit one of them
#[derive(Parser)]
#[command(version, about, long_about = None)]
pub struct Cli {
    /// Two or three directories: `LEFT RIGHT` or `LEFT RIGHT OUTPUT`
    ///
    /// If OUTPUT is not specified, the output goes to RIGHT.
    dirs: Vec<PathBuf>,
    /// Use fake data for a demo. No need to specify any DIRs
    #[arg(long, conflicts_with("dirs"))]
    demo: bool,
}

impl Cli {
    pub fn into_data_interface(self) -> Result<Box<dyn DataInterface>, String> {
        if self.demo {
            Ok(Box::new(FakeData))
        } else {
            match self.dirs.as_slice() {
                [left, right, output] => Ok(Box::new(ThreeDirInput {
                    left: left.to_path_buf(),
                    right: right.to_path_buf(),
                    edit: output.to_path_buf(),
                })),
                [left, right] => Ok(Box::new(ThreeDirInput {
                    left: left.to_path_buf(),
                    right: right.to_path_buf(),
                    edit: right.to_path_buf(),
                })),
                _ => Err(format!(
                    "Must have 2 or 3 dirs to compare, got {} dirs instead",
                    self.dirs.len()
                )),
            }
        }
    }
}

// TODO: Make configurable, figure out a reasonable default value. See jj's
// implementation of max_snapshot_size.
const MAX_FILE_LENGTH: usize = 200_000;

fn scan(root: &Path) -> impl Iterator<Item = (DirEntry, FileEntry)> {
    // As an alternative to WalkDir, see
    // https://github.com/martinvonz/jj/blob/af8eb3fd74956effee00acf00011ff0413607213/lib/src/local_working_copy.rs#L849
    WalkDir::new(root)
        .into_iter()
        .filter_map(|e| e.ok())
        // TODO: We currently treat symlinks or dirs as missing
        // files. This is not great when a file on one side
        // corresponds to a dir or a symlink on the other,
        // which should be a more prominent error.
        .filter(|e| e.file_type().is_file())
        .map(|e| -> (DirEntry, FileEntry) {
            (
                e.clone(),
                match std::fs::read_to_string(e.path()) {
                    Err(io_error) => FileEntry::Unsupported(format!("IO Error: {io_error}")),
                    // TODO: read no more than MAX_FILE_LENGTH
                    // TODO: Test
                    // TODO: Check executable byte
                    Ok(contents) if contents.len() > MAX_FILE_LENGTH => {
                        FileEntry::Unsupported(format!("File length exceeds {MAX_FILE_LENGTH}"))
                    }

                    Ok(contents) if contents.contains('\0') => {
                        FileEntry::Unsupported("A binary file (contains the NUL byte)".to_string())
                    }

                    Ok(contents) => FileEntry::Text(contents),
                },
            )
        })
}

// pub fn scan_several<const N: usize>(roots: [&Path; N]) ->
// EntriesToCompare<PathBuf, N> {
// TODO: Change &PathBuf to &Path or something
fn scan_several(roots: [&PathBuf; 3]) -> Result<EntriesToCompare, DataReadError> {
    let mut result = EntriesToCompare::default();
    for (i, root) in roots.iter().enumerate() {
        for (file_entry, contents) in scan(root) {
            let value = result
                .0
                .entry(PathBuf::from(
                    file_entry.path().strip_prefix(root).unwrap_or_else(|_| {
                        panic!(
                            "The path {:?} does not begin with {root:?}.",
                            file_entry.path(),
                        )
                    }),
                ))
                .or_insert([FileEntry::Missing, FileEntry::Missing, FileEntry::Missing])
                .as_mut();
            value[i] = contents;
        }
    }
    Ok(result)
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;
    use std::io::ErrorKind;

    use assert_matches::assert_matches;
    use indexmap::IndexMap;
    use indoc::indoc;
    use serde::Serialize;
    use tempdir::TempDir;

    use super::*;

    fn to_slash_string_lossy(path: &Path) -> String {
        path.to_string_lossy().replace('\\', "/")
    }

    fn showdir(path: &Path) -> impl Serialize {
        BTreeMap::from_iter(scan(path).map(|(dir_path, file_type)| {
            (
                to_slash_string_lossy(dir_path.path().strip_prefix(path).unwrap()),
                file_type,
            )
        }))
    }

    fn showscan(input: &ThreeDirInput) -> impl Serialize {
        let entries = input.scan().unwrap();
        BTreeMap::from_iter(
            entries
                .0
                .into_iter()
                .map(|(k, v)| (to_slash_string_lossy(&k), v)),
        )
    }

    fn left_right_edit_threedirinput(base: &Path) -> ThreeDirInput {
        ThreeDirInput {
            left: base.join("left").to_owned(),
            right: base.join("right").to_owned(),
            edit: base.join("edit").to_owned(),
        }
    }

    fn string_pair(first: &str, second: &str) -> (String, String) {
        (first.to_string(), second.to_string())
    }

    fn tmpdir_from_txtar(textarchive: &str) -> TempDir {
        let tmp_dir = TempDir::new("de3test").unwrap();
        txtar::from_str(textarchive)
            .materialize(tmp_dir.path())
            .unwrap();
        tmp_dir
    }

    #[test]
    fn common_case_save() {
        let tmp_dir = tmpdir_from_txtar(indoc! {"
        -- left/subdir/txt --
        Some text
        -- right/subdir/txt --
        Changed text
        -- edit/subdir/txt --
        Changed text for editing
        "});
        insta::assert_yaml_snapshot!(showdir(tmp_dir.path()), @r###"
        ---
        edit/subdir/txt:
          type: Text
          value: "Changed text for editing\n"
        left/subdir/txt:
          type: Text
          value: "Some text\n"
        right/subdir/txt:
          type: Text
          value: "Changed text\n"
        "###);
        // TODO: A different bug if edit/subdir/another_file is specified
        let mut input = left_right_edit_threedirinput(tmp_dir.path());
        insta::assert_yaml_snapshot!(showscan(&input), @r###"
        ---
        subdir/txt:
          - type: Text
            value: "Some text\n"
          - type: Text
            value: "Changed text\n"
          - type: Text
            value: "Changed text for editing\n"
        "###);
        let () = input
            .save(IndexMap::from([string_pair("subdir/txt", "Edited text")]))
            .unwrap();
        insta::assert_yaml_snapshot!(showscan(&input), @r###"
        ---
        subdir/txt:
          - type: Text
            value: "Some text\n"
          - type: Text
            value: "Changed text\n"
          - type: Text
            value: Edited text
        "###);

        // If the file exists on all sides, an empty save means an empty file.
        let () = input
            .save(IndexMap::from([string_pair("subdir/txt", "")]))
            .unwrap();
        insta::assert_yaml_snapshot!(showscan(&input), @r###"
        ---
        subdir/txt:
          - type: Text
            value: "Some text\n"
          - type: Text
            value: "Changed text\n"
          - type: Text
            value: ""
        "###);

        // Test a validation error
        let result = input.save(IndexMap::from([
            string_pair("another_txt", ""),
            string_pair("subdir/txt", "This text should not be written"),
        ]));
        insta::assert_debug_snapshot!(result, @r###"
        Err(
            ValidationFailError(
                "another_txt",
            ),
        )
        "###);
        // Should be the same as previous version
        insta::assert_yaml_snapshot!(showscan(&input), @r###"
        ---
        subdir/txt:
          - type: Text
            value: "Some text\n"
          - type: Text
            value: "Changed text\n"
          - type: Text
            value: ""
        "###);
    }

    #[test]
    fn deleted_lhs_file() {
        let tmp_dir = tmpdir_from_txtar(indoc! {"
        -- right/txt --
        Some text
        -- edit/txt --
        Some text for editing
        "});
        insta::assert_yaml_snapshot!(showdir(tmp_dir.path()), @r###"
        ---
        edit/txt:
          type: Text
          value: "Some text for editing\n"
        right/txt:
          type: Text
          value: "Some text\n"
        "###);
        let mut input = left_right_edit_threedirinput(tmp_dir.path());
        insta::assert_yaml_snapshot!(showscan(&input), @r###"
        ---
        txt:
          - type: Missing
          - type: Text
            value: "Some text\n"
          - type: Text
            value: "Some text for editing\n"
        "###);
        let () = input
            .save(IndexMap::from([string_pair("txt", "somevalue")]))
            .unwrap();
        insta::assert_yaml_snapshot!(showscan(&input), @r###"
        ---
        txt:
          - type: Missing
          - type: Text
            value: "Some text\n"
          - type: Text
            value: somevalue
        "###);
        // TODO: If the file is missing on LHS, an empty save should mean that the file
        // should be deleted.
        let () = input
            .save(IndexMap::from([string_pair("txt", "")]))
            .unwrap();
        insta::assert_yaml_snapshot!(showscan(&input), @r###"
        ---
        txt:
          - type: Missing
          - type: Text
            value: "Some text\n"
          - type: Text
            value: ""
        "###);
    }

    #[test]
    fn deleted_rhs_file() {
        let tmp_dir = tmpdir_from_txtar(indoc! {"
        -- left/txt --
        Some text
        "});
        insta::assert_yaml_snapshot!(showdir(tmp_dir.path()), @r###"
        ---
        left/txt:
          type: Text
          value: "Some text\n"
        "###);
        let mut input = left_right_edit_threedirinput(tmp_dir.path());
        insta::assert_yaml_snapshot!(showscan(&input), @r###"
        ---
        txt:
          - type: Text
            value: "Some text\n"
          - type: Missing
          - type: Missing
        "###);
        let result = input.save(IndexMap::from([string_pair("txt", "somevalue")]));
        // BUG: We fail to create `right/txt` because `right/` does not exist
        assert_matches!(result,
            Err(DataSaveError::IOError(path, err))
            if path.ends_with("txt") &&
               err.kind() == ErrorKind::NotFound
        );
        // TODO: If the file is missing on RHS, an empty save should mean that the file
        // should stay deleted. We should not even try to create it
        let result = input.save(IndexMap::from([string_pair("txt", "")]));
        // BUG: We fail to create `edit/txt` because `right/` does not exist
        assert_matches!(result,
            Err(DataSaveError::IOError(path, err))
            if path.ends_with("txt") &&
               err.kind() == ErrorKind::NotFound
        );
        insta::assert_yaml_snapshot!(showscan(&input), @r###"
        ---
        txt:
          - type: Text
            value: "Some text\n"
          - type: Missing
          - type: Missing
        "###);
    }

    #[test]
    fn deleted_rhs_file_with_dir() {
        let tmp_dir = tmpdir_from_txtar(indoc! {"
        -- left/txt --
        Some text
        -- edit/randomfile --
        This file exists solely to create the `edit/` dir
        "});
        insta::assert_yaml_snapshot!(showdir(tmp_dir.path()), @r###"
        ---
        edit/randomfile:
          type: Text
          value: "This file exists solely to create the `edit/` dir\n"
        left/txt:
          type: Text
          value: "Some text\n"
        "###);
        let mut input = left_right_edit_threedirinput(tmp_dir.path());
        insta::assert_yaml_snapshot!(showscan(&input), @r###"
        ---
        randomfile:
          - type: Missing
          - type: Missing
          - type: Text
            value: "This file exists solely to create the `edit/` dir\n"
        txt:
          - type: Text
            value: "Some text\n"
          - type: Missing
          - type: Missing
        "###);
        let () = input
            .save(IndexMap::from([string_pair("txt", "somevalue")]))
            .unwrap();
        insta::assert_yaml_snapshot!(showscan(&input), @r###"
        ---
        randomfile:
          - type: Missing
          - type: Missing
          - type: Text
            value: "This file exists solely to create the `edit/` dir\n"
        txt:
          - type: Text
            value: "Some text\n"
          - type: Missing
          - type: Text
            value: somevalue
        "###);
        // TODO: If the file is missing on RHS, an empty save should mean that the file
        // should be deleted.
        let () = input
            .save(IndexMap::from([string_pair("txt", "")]))
            .unwrap();
        insta::assert_yaml_snapshot!(showscan(&input), @r###"
        ---
        randomfile:
          - type: Missing
          - type: Missing
          - type: Text
            value: "This file exists solely to create the `edit/` dir\n"
        txt:
          - type: Text
            value: "Some text\n"
          - type: Missing
          - type: Text
            value: ""
        "###);
    }

    #[test]
    fn file_vs_dir() {
        let tmp_dir = tmpdir_from_txtar(indoc! {"
        -- left/test --
        Some text
        -- right/test/file --
        Dir file
        -- edit/test/file --
        Dir file (edit)
        "});
        insta::assert_yaml_snapshot!(showdir(tmp_dir.path()), @r###"
        ---
        edit/test/file:
          type: Text
          value: "Dir file (edit)\n"
        left/test:
          type: Text
          value: "Some text\n"
        right/test/file:
          type: Text
          value: "Dir file\n"
        "###);
        let mut input = left_right_edit_threedirinput(tmp_dir.path());
        insta::assert_yaml_snapshot!(showscan(&input), @r###"
        ---
        test:
          - type: Text
            value: "Some text\n"
          - type: Missing
          - type: Missing
        test/file:
          - type: Missing
          - type: Text
            value: "Dir file\n"
          - type: Text
            value: "Dir file (edit)\n"
        "###);
        let () = input
            .save(IndexMap::from([string_pair("test/file", "New value\n")]))
            .unwrap();
        insta::assert_yaml_snapshot!(showscan(&input), @r###"
        ---
        test:
          - type: Text
            value: "Some text\n"
          - type: Missing
          - type: Missing
        test/file:
          - type: Missing
          - type: Text
            value: "Dir file\n"
          - type: Text
            value: "New value\n"
        "###);
        // TODO: This should not cause an error; this is what will happen if the user
        // changes nothing and presses Save.
        let result = input.save(IndexMap::from([string_pair("test", "")]));

        // The error is of kind IsADirectory, but we can't check that in stable Rust.
        // (This also applies below) TODO: Could use `assert_debug_snapshot`,
        // but that requires some redacting of the file path.
        assert_matches!(dbg!(result),
            Err(DataSaveError::IOError(path, _))
            if path.ends_with("edit/test")
        );
        // It is OK to error out here, but it would be better to do soemthing more
        // intelligent. For example, the input could be marked as "Unsupported"
        let result = input.save(IndexMap::from([string_pair("test", "")]));
        assert_matches!(dbg!(result),
            Err(DataSaveError::IOError(path, _))
            if path.ends_with("edit/test")
        );
        insta::assert_yaml_snapshot!(showscan(&input), @r###"
        ---
        test:
          - type: Text
            value: "Some text\n"
          - type: Missing
          - type: Missing
        test/file:
          - type: Missing
          - type: Text
            value: "Dir file\n"
          - type: Text
            value: "New value\n"
        "###);
    }
}
