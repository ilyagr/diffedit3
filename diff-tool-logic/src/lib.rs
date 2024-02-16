use std::path::{Path, PathBuf};

use thiserror::Error;
use walkdir::{DirEntry, WalkDir};

#[derive(Error, Debug)]
pub enum DataSaveError {
    // TODO: Collect the list of what files couldn't be saved
    #[error("IO Error while saving {0}: {1}")]
    IOError(PathBuf, std::io::Error),
    #[error("Cannot save the demo fake data")]
    CannotSaveFakeData,
}

#[derive(Error, Debug)]
pub enum DataReadError {
    #[error("IO Error while reading: {0}")]
    IOError(#[from] std::io::Error),
}

impl serde::Serialize for DataSaveError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(self.to_string().as_ref())
    }
}

impl serde::Serialize for DataReadError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(self.to_string().as_ref())
    }
}

pub fn scan(root: &Path) -> impl Iterator<Item = Result<(DirEntry, String), DataReadError>> {
    // As an alternative to WalkDir, see
    // https://github.com/martinvonz/jj/blob/af8eb3fd74956effee00acf00011ff0413607213/lib/src/local_working_copy.rs#L849
    WalkDir::new(root)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
        .map(|e| Ok((e.clone(), std::fs::read_to_string(e.path())?)))
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum Input {
    FakeData,
    Dirs {
        left: PathBuf,
        right: PathBuf,
        edit: PathBuf,
    },
}

impl Input {
    pub fn scan(&self) -> Result<EntriesToCompare, DataReadError> {
        match self {
            Self::FakeData => Ok(fake_data()),
            Self::Dirs { left, right, edit } => scan_several([left, right, edit]),
        }
    }

    // TODO: Make more generic than IndexMap
    pub fn save(&self, result: indexmap::IndexMap<String, String>) -> Result<(), DataSaveError> {
        let outdir = match self {
            Self::FakeData => {
                // TOOO: Somewhat better error handling :)
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
            let relpath = dbg!(PathBuf::from(relpath));
            let path = outdir.join(relpath);
            std::fs::write(path.clone(), contents)
                .map_err(|io_err| DataSaveError::IOError(path, io_err))?;
        }
        Ok(())
    }
}

use clap::Parser;
#[derive(Parser)]
#[command(version, about, long_about = None)]
pub struct Cli {
    dirs: Vec<PathBuf>,
    /// Use demo fake data
    #[arg(long)]
    demo: bool,
}

impl TryInto<Input> for Cli {
    type Error = String;
    fn try_into(self) -> Result<Input, Self::Error> {
        if self.demo {
            Ok(Input::FakeData)
        } else {
            match self.dirs.as_slice() {
                [left, right, output] => Ok(Input::Dirs {
                    left: left.to_path_buf(),
                    right: right.to_path_buf(),
                    edit: output.to_path_buf(),
                }),
                [left, right] => Ok(Input::Dirs {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        insta::assert_yaml_snapshot!(Input::FakeData.scan().unwrap(), 
        @r###"
        ---
        added file:
          - ~
          - added
          - added
        deleted_file:
          - deleted
          - ~
          - ~
        edited_file:
          - "First\nThird\nFourth\nFourthAndAHalf\n\nFifth\nSixth\n----\none two"
          - "First\nSecond\nThird\nFifth\nSixth\n----\none\n"
          - "First\nSecond\nThird\nFifth\nSixth\n----\none\n"
        "###);
    }
}
