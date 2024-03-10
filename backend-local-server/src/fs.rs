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
