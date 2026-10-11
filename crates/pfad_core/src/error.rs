
use std::{fmt, io, path::PathBuf};

#[derive(Debug)]
pub enum PfadError {
    Cli(String),
    Io(io::Error),
    AlreadyExists(PathBuf),
    InvalidName(String),
    Clipboard(String),
    PasteIntoSelf(PathBuf),
    Selection(String),
    NonUtf8Path(PathBuf),
    NothingToPaste,
}

impl fmt::Display for PfadError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PfadError::Io(e)  => write!(f, "I/O error: {e}"),
            PfadError::AlreadyExists(p) => write!(f, "already exists: {}", p.display()),
            PfadError::InvalidName(n) => write!(f, "invalid name: {n:?}"),
            PfadError::Cli(n) => write!(f, "CLI error: {n}"),
            PfadError::Clipboard(n) => write!(f, "clipboard error: {n}"),
            PfadError::PasteIntoSelf(p) => write!(f, "cannot paste a folder into itself: {}", p.display()),
            PfadError::Selection(n) => write!(f, "selection error: {n}"),
            PfadError::NonUtf8Path(p) => write!(f, "path is not valid UTF-8: {}", p.display()),
            PfadError::NothingToPaste => write!(f, "nothing to paste: copy something first"),
        }
    }
}

impl std::error::Error for PfadError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            PfadError::Io(e) => Some(e),
            _ => None,
        }
    }
}

impl From<io::Error> for PfadError {
    fn from(e: io::Error) -> Self {
        PfadError::Io(e)
    }
}

pub type Result<T> = std::result::Result<T, PfadError>;