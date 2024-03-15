//! Library version of `diffedit3`, including the webapp
//!
//! This is mainly intended for use with the `local_server` submodule by
//! <https://github.com/martinvonz/jj> at the moment, but could be used in other
//! apps or to make backends other than the `local_server``.
//!
//! See [the README](https://github.com/ilyagr/diffedit3/?tab=readme-ov-file)
//! for the description of `diffedit3-web` and `diffedit3-gui` binaries that use
//! this library in a stand-alone fashion. The latter binary is an example of a
//! different backend.

pub mod fs;
pub mod local_server;
pub mod types;

pub use fs::{Cli, ThreeDirInput};
pub use types::*;
