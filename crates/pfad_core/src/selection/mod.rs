mod clipboard;
mod file;

use crate::error::{PfadError, Result};
use crate::transfer;
use std::io;
use std::path::{Path, PathBuf};

/// Saves `paths` as the current selection. With `to_clipboard` they are also put
/// on the system clipboard, but `paste_files` only ever reads the saved selection.
/// An empty list clears the selection.
pub fn copy_files(paths: Vec<impl AsRef<Path>>, to_clipboard: bool) -> Result<()> {
    let mut absolute_paths: Vec<PathBuf> = Vec::new();
    for path in &paths {
        absolute_paths.push(std::path::absolute(path)?);
    }

    file::save(&file::default_dir()?, &absolute_paths)?;

    if to_clipboard {
        clipboard::set(&absolute_paths)?;
    }

    Ok(())
}

/// Copies the saved selection into `dest_dir` and returns the new paths.
///
/// Fails with [`PfadError::NothingToPaste`] if nothing was copied, and with an
/// I/O error if a copied item no longer exists; that check runs for every item
/// before anything is copied. Items are then copied in order and nothing is ever
/// overwritten, so a later failure (for example a name collision) can still
/// leave the earlier items copied.
pub fn paste_files(dest_dir: impl AsRef<Path>) -> Result<Vec<PathBuf>> {
    let dest_dir = std::path::absolute(dest_dir)?;

    let sources = file::load(&file::default_dir()?)?;
    if sources.is_empty() {
        return Err(PfadError::NothingToPaste);
    }

    for src in &sources {
        if !src.exists() {
            return Err(io::Error::new(
                io::ErrorKind::NotFound,
                format!("copied item no longer exists: {}", src.display()),
            )
            .into());
        }
    }

    let mut created = Vec::new();
    for src in &sources {
        if let Some(dest) = transfer::copy_into(src, &dest_dir)? {
            created.push(dest);
        }
    }

    Ok(created)
}
