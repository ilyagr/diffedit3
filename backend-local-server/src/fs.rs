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

fn scan(root: &Path) -> impl Iterator<Item = Result<(DirEntry, String), DataReadError>> {
    // As an alternative to WalkDir, see
    // https://github.com/martinvonz/jj/blob/af8eb3fd74956effee00acf00011ff0413607213/lib/src/local_working_copy.rs#L849
    WalkDir::new(root)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
        .map(|e| Ok((e.clone(), std::fs::read_to_string(e.path())?)))
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
                .or_insert([FileEntry::Missing, FileEntry::Missing, FileEntry::Missing])
                .as_mut();
            value[i] = FileEntry::Text(contents);
        }
    }
    Ok(result)
}
