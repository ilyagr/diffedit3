use std::path::PathBuf;

use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq, Default, serde::Serialize, serde::Deserialize)]
//struct EntriesToCompare<P, const N: usize>(std::collections::BTreeMap<P,
// [Option<String>; N]>);
pub struct EntriesToCompare(pub std::collections::BTreeMap<PathBuf, [Option<String>; 3]>);

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
    // TODO: Use `&mut self` in save
    /// Do not use this method directly, as it does not check whether the
    /// requested paths are safe to save to.
    fn save_unchecked(
        &self,
        result: indexmap::IndexMap<String, String>,
    ) -> Result<(), DataSaveError>;

    /// Get a list of all the files we were originally asked to merge.
    ///
    /// The default implementation may be very inefficient.
    fn get_valid_entries(&self) -> Result<std::collections::HashSet<PathBuf>, DataReadError> {
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
    fn save(&self, result: indexmap::IndexMap<String, String>) -> Result<(), DataSaveError> {
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
