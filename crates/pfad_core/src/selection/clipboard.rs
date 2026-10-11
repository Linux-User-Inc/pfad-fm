use crate::error::{PfadError, Result};
use clipboard_rs::{Clipboard, ClipboardContext};
use std::path::PathBuf;

fn context() -> Result<ClipboardContext> {
    ClipboardContext::new().map_err(|e| PfadError::Clipboard(e.to_string()))
}

/// Puts paths on the system clipboard as files. An empty slice clears it.
pub(crate) fn set(paths: &[PathBuf]) -> Result<()> {
    let mut path_strings = Vec::with_capacity(paths.len());
    for path in paths {
        let text = path.to_str().ok_or_else(|| PfadError::NonUtf8Path(path.clone()))?;
        path_strings.push(text.to_string());
    }

    let clipboard = context()?;
    let result = if path_strings.is_empty() {
        clipboard.clear()
    } else {
        clipboard.set_files(path_strings)
    };
    result.map_err(|e| PfadError::Clipboard(e.to_string()))
}

/// Reads the files currently on the system clipboard.
#[allow(dead_code)]
pub(crate) fn get() -> Result<Vec<PathBuf>> {
    let files = context()?
        .get_files()
        .map_err(|e| PfadError::Clipboard(e.to_string()))?;

    // Linux returns file://home/.., windows returns a plain path
    Ok(files
        .iter()
        .map(|file| PathBuf::from(file.strip_prefix("file://").unwrap_or(file)))
        .collect())
}
