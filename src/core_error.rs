use std::io::Error;

use glob::{GlobError, PatternError};

#[derive(thiserror::Error, Debug)]
pub enum CoreError {
    #[error("pattern error {0}")]
    PatternError(#[from] PatternError),

    #[error("glob error {0}")]
    GlobError(#[from] GlobError),

    #[error("io error {0}")]
    IOError(#[from] Error),
}