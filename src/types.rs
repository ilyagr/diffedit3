use std::path::PathBuf;

use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(tag = "type", content = "value")]
pub enum FileEntry {
    Missing,
    // TODO: Track executable bit, other metadata perhaps
    Text(String),
    Unsupported(String),
}

/// TODO: Clean this up to make things more readable
const OUTPUT_INDEX: usize = 2;
#[derive(Debug, Clone, PartialEq, Eq, Default, serde::Serialize, serde::Deserialize)]
//struct EntriesToCompare<P, const N: usize>(std::collections::BTreeMap<P,
// [Option<String>; N]>);
pub struct EntriesToCompare(pub std::collections::BTreeMap<PathBuf, [FileEntry; 3]>);

#[derive(Error, Debug)]
pub enum DataSaveError {
    // TODO: Collect the list of what files couldn't be saved
    #[error("IO Error while saving {0}: {1}")]
    IOError(PathBuf, std::io::Error),
    #[error("Cannot save the demo fake data")]
    CannotSaveFakeData,
    #[error("Failed to retreive valid paths for saving: {0}")]
    ValidationIOError(#[from] DataReadError),
    #[error(
        "Security error: got request to save to a file that wasn't one of the files being merged: \
         '{0}'\nPerhaps this client is now connected to a different server than the one it was \
         started from?"
    )]
    ValidationFailError(String),
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

// TODO: What does 'static mean here? Can it be loosened?
pub trait DataInterface: Send + Sync + 'static {
    /// Return the content of either the original files to merge or the
    /// last-saved version.
    ///
    /// A `scan()` after a successful `save()` should return the saved results.
    fn scan(&self) -> Result<EntriesToCompare, DataReadError>;
    // TODO: Make `save` more generic than IndexMap
    /// Do not use this method directly, as it does not check whether the
    /// requested paths are safe to save to.
    fn save_unchecked(
        &mut self,
        result: indexmap::IndexMap<String, String>,
    ) -> Result<(), DataSaveError>;

    /// Get a list of all the files we were originally asked to merge.
    ///
    /// The default implementation may be very inefficient.
    fn get_valid_entries(&mut self) -> Result<std::collections::HashSet<PathBuf>, DataReadError> {
        let entries = self.scan()?;
        Ok(entries.0.keys().cloned().collect())
    }

    /// Save the result. First, check that each file being saved was one of the
    /// files we were comparing originally.
    ///
    /// This check helps with two potential problems when running a local
    /// server:
    /// - The frontend webapp could be connected to a different server than it
    ///   was started with.
    /// - A malicious frontend could try making the diff editor save to
    ///   `../../../../home/$USER/.bashrc`.
    fn save(&mut self, result: indexmap::IndexMap<String, String>) -> Result<(), DataSaveError> {
        let valid_entries = self.get_valid_entries()?;
        if let Some(unsafe_path) = result
            .keys()
            .find(|x| !valid_entries.contains::<PathBuf>(&x.into()))
        {
            // TODO: Have the server print some debug info, e.g. the list of
            // valid file names, to the terminal. It should not be returned to
            // the HTTP request, though.
            return Err(DataSaveError::ValidationFailError(unsafe_path.to_string()));
        }
        self.save_unchecked(result)
    }
}

// Dummy implementation for in-memory storage
// TODO: Make FakeData use this
// TODO: Create a separate type for the DataInterface, allow for callbacks
// and/or output to a Writer on scan or save.
impl DataInterface for EntriesToCompare {
    fn scan(&self) -> Result<EntriesToCompare, DataReadError> {
        Ok(self.clone())
    }

    fn save_unchecked(
        &mut self,
        result: indexmap::IndexMap<String, String>,
    ) -> Result<(), DataSaveError> {
        for (path, new_value) in result.into_iter() {
            self.0
                .get_mut(&PathBuf::from(path))
                .expect("At this point, `save()` should have verified that the path is valid")
                [OUTPUT_INDEX] = FileEntry::Text(new_value);
        }
        Ok(())
    }
}

pub struct FakeData;

impl DataInterface for FakeData {
    fn scan(&self) -> Result<EntriesToCompare, DataReadError> {
        // let mut two_sides_map = btreemap! {
        //     "edited_file" => [
        //           Some("First\nThird\nFourth\nFourthAndAHalf\n\nFifth\nSixth\n----\
        // none two"),           Some("First\nSecond\nThird\nFifth\nSixth\n----\
        // none\n")     ],
        //     "deleted_file" => [Some("deleted"), None],
        //     "added file" => [None, Some("added")]
        // };
        #[rustfmt::skip]
        let two_sides_map = vec![
            (
                "edited_file",
                [
                    FileEntry::Text(
                        "Long line, a long line, a quite long line. Long line, a long line, a \
                         quite long line. Long line, a long line, a quite long \
                         line.\nFirst\nThird\nFourth\nFourthAndAHalf\nSame\nSame\nSame\nSame\
                         \nSame\nSame\nSame\nSame\nSame\nSame\nSame\nSame\nSame\nSame\nSame\nSame\
                         \nSame\nSame\nSame\nSame\nSame\nFifth\nSixth\n----\none two"
                            .to_string(),
                    ),
                    FileEntry::Text(
                        "Long line, a long line, a quite long line. Long line, a long line, a \
                         quite long line. Something new. Long line, a long line, a quite long \
                         line.\nFirst\nSecond\nThird\nSame\nSame\nSame\nSame\nSame\nSame\nSame\
                         \nSame\nSame\nSame\nSame\nSame\nSame\nSame\nSame\nSame\nSame\nSame\nSame\
                         \nSame\nSame\nFifth\nSixth\n----\none\n"
                            .to_string(),
                    ),
                ],
            ),
            (
                "deleted_file",
                [FileEntry::Text("deleted".to_string()), FileEntry::Missing],
            ),
            (
                "added file",
                [FileEntry::Missing, FileEntry::Text("added".to_string())],
            ),
            (
                "unsupported-left",
                [
                    FileEntry::Unsupported("demo of an unsupported file".to_string()),
                    FileEntry::Text("text".to_string()),
                ],
            ),
            (
                "unsupported-right",
                [
                    FileEntry::Text("text".to_string()),
                    FileEntry::Unsupported("demo of an unsupported file".to_string()),
                ],
            ),
        ];
        Ok(EntriesToCompare(
            two_sides_map
                .into_iter()
                .map(|(key, [left, right])| (PathBuf::from(key), [left, right.clone(), right]))
                .collect(),
        ))
    }

    fn save_unchecked(
        &mut self,
        result: indexmap::IndexMap<String, String>,
    ) -> Result<(), DataSaveError> {
        eprintln!("Can't save fake demo data. Here it is as TOML");
        eprintln!();
        eprintln!(
            "{}",
            toml::to_string(&result).unwrap_or_else(|err| format!("Failed to parse TOML: {err}"))
        );
        Err(DataSaveError::CannotSaveFakeData)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fake_data() {
        insta::assert_yaml_snapshot!(FakeData.scan().unwrap(), 
        @r###"
        ---
        added file:
          - type: Missing
          - type: Text
            value: added
          - type: Text
            value: added
        deleted_file:
          - type: Text
            value: deleted
          - type: Missing
          - type: Missing
        edited_file:
          - type: Text
            value: "Long line, a long line, a quite long line. Long line, a long line, a quite long line. Long line, a long line, a quite long line.\nFirst\nThird\nFourth\nFourthAndAHalf\nSame\nSame\nSame\nSame\nSame\nSame\nSame\nSame\nSame\nSame\nSame\nSame\nSame\nSame\nSame\nSame\nSame\nSame\nSame\nSame\nSame\nFifth\nSixth\n----\none two"
          - type: Text
            value: "Long line, a long line, a quite long line. Long line, a long line, a quite long line. Something new. Long line, a long line, a quite long line.\nFirst\nSecond\nThird\nSame\nSame\nSame\nSame\nSame\nSame\nSame\nSame\nSame\nSame\nSame\nSame\nSame\nSame\nSame\nSame\nSame\nSame\nSame\nSame\nSame\nFifth\nSixth\n----\none\n"
          - type: Text
            value: "Long line, a long line, a quite long line. Long line, a long line, a quite long line. Something new. Long line, a long line, a quite long line.\nFirst\nSecond\nThird\nSame\nSame\nSame\nSame\nSame\nSame\nSame\nSame\nSame\nSame\nSame\nSame\nSame\nSame\nSame\nSame\nSame\nSame\nSame\nSame\nSame\nFifth\nSixth\n----\none\n"
        unsupported-left:
          - type: Unsupported
            value: demo of an unsupported file
          - type: Text
            value: text
          - type: Text
            value: text
        unsupported-right:
          - type: Text
            value: text
          - type: Unsupported
            value: demo of an unsupported file
          - type: Unsupported
            value: demo of an unsupported file
        "###);
    }
}
