use std::path::{Path, PathBuf};

use walkdir::{DirEntry, WalkDir};

use crate::{DataInterface, DataReadError, DataSaveError, EntriesToCompare};

fn scan(root: &Path) -> impl Iterator<Item = Result<(DirEntry, String), DataReadError>> {
    // As an alternative to WalkDir, see
    // https://github.com/martinvonz/jj/blob/af8eb3fd74956effee00acf00011ff0413607213/lib/src/local_working_copy.rs#L849
    WalkDir::new(root)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
        .map(|e| Ok((e.clone(), std::fs::read_to_string(e.path())?)))
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum ThreeDirInput {
    // TODO: Separate FakeData
    FakeData,
    Dirs {
        left: PathBuf,
        right: PathBuf,
        edit: PathBuf,
    },
}

impl DataInterface for ThreeDirInput {
    // TODO: A more efficient `get_valid_entries` implementation

    fn scan(&self) -> Result<EntriesToCompare, DataReadError> {
        match self {
            Self::FakeData => Ok(fake_data()),
            Self::Dirs { left, right, edit } => scan_several([left, right, edit]),
        }
    }

    fn save_unchecked(
        &self,
        result: indexmap::IndexMap<String, String>,
    ) -> Result<(), DataSaveError> {
        let outdir = match self {
            Self::FakeData => {
                eprintln!("Can't save fake demo data. Here it is as TOML");
                eprintln!();
                eprintln!(
                    "{}",
                    toml::to_string(&result)
                        .unwrap_or_else(|err| format!("Failed to parse TOML: {err}"))
                );
                return Err(DataSaveError::CannotSaveFakeData);
            }
            Self::Dirs { edit, .. } => edit,
        };

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

impl TryInto<ThreeDirInput> for Cli {
    type Error = String;
    fn try_into(self) -> Result<ThreeDirInput, Self::Error> {
        if self.demo {
            Ok(ThreeDirInput::FakeData)
        } else {
            match self.dirs.as_slice() {
                [left, right, output] => Ok(ThreeDirInput::Dirs {
                    left: left.to_path_buf(),
                    right: right.to_path_buf(),
                    edit: output.to_path_buf(),
                }),
                [left, right] => Ok(ThreeDirInput::Dirs {
                    left: left.to_path_buf(),
                    right: right.to_path_buf(),
                    edit: right.to_path_buf(),
                }),
                _ => Err(format!(
                    "Must have 2 or 3 dirs to compare, got {} dirs instead",
                    self.dirs.len()
                )),
            }
        }
    }
}

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
fn scan_several(roots: [&PathBuf; 3]) -> Result<EntriesToCompare, DataReadError> {
    let mut result = EntriesToCompare::default();
    for (i, root) in roots.iter().enumerate() {
        for result_or_err in scan(root) {
            // TODO: Collect list of failed files
            let (file_entry, contents) = result_or_err?;
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
                .or_insert(Default::default())
                .as_mut();
            value[i] = Some(contents);
        }
    }
    Ok(result)
}
