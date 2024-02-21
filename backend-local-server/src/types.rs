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
    fn scan(&self) -> Result<EntriesToCompare, DataReadError>;
    // TODO: Make `save` more generic than IndexMap
    fn save(&self, result: indexmap::IndexMap<String, String>) -> Result<(), DataSaveError>;
}
