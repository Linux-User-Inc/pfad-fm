
use crate::error::{PfadError, Result};
use std::ffi::OsStr;
use std::fs::{self, OpenOptions};
use std::io;
use std::path::{Path, PathBuf};
use std::fmt;

/// Whether [`remove_dir`] may delete a directory together with its contents.
pub enum Recursion {
    Yes,
    No,
}



/// One item of a directory listing, as returned by [`list_dir`].
#[derive(Debug)]
pub struct Entry {
    pub name: String,
    pub size: u64,
    pub is_dir: bool,
}
impl fmt::Display for Entry {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = self.convert_to_string();
        write!(f, "{} - {} bytes", name, self.size)
    }
}


impl Entry {
    /// The entry's name, with a trailing `/` added for directories.
    pub fn convert_to_string(&self) -> String {
        let mut name = self.name.clone();
        if self.is_dir {
            name += "/";
        }
        return name
    }
}


/// Creates a new empty file at `path`, creating any missing parent directories.
///
/// Fails with [`PfadError::AlreadyExists`] if something is already at `path`;
/// an existing file is never truncated.
pub fn create_file(path: impl AsRef<Path>) -> Result<()> {
    let path = path.as_ref();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    match OpenOptions::new().write(true).create_new(true).open(path) {
        Ok(_) => Ok(()),
        Err(e) if e.kind() == io::ErrorKind::AlreadyExists => {
            Err(PfadError::AlreadyExists(path.to_path_buf()))
        }
        Err(e) => Err(e.into()),
    }
}

/// Creates the directory at `path`, including any missing parent directories.
///
/// Fails with [`PfadError::AlreadyExists`] if `path` already exists, whether it
/// is a file or a directory.
pub fn create_dir(path: impl AsRef<Path>) -> Result<()> {
    let path = path.as_ref();
    if path.exists() {
        return Err(PfadError::AlreadyExists(path.to_path_buf()));
    }
    fs::create_dir_all(path)?;
    Ok(())
}

/// Permanently deletes the file at `path`. There is no trash or undo.
///
/// Fails with an I/O error if `path` is missing or is a directory
/// (use [`remove_dir`] for those).
pub fn remove_file(path: impl AsRef<Path>) -> Result<()> {
    fs::remove_file(path)?;
    Ok(())
}

/// Permanently deletes the directory at `path`. There is no trash or undo.
///
/// With [`Recursion::No`] only an empty directory is removed and anything else
/// fails, leaving the contents untouched. With [`Recursion::Yes`] the whole
/// tree is deleted. Fails with an I/O error if `path` is missing or is a file.
pub fn remove_dir(path: impl AsRef<Path>, recursion: Recursion) -> Result<()> {
    match recursion {
        Recursion::Yes => fs::remove_dir_all(path)?,
        Recursion::No =>  fs::remove_dir(path)?,
    }
    Ok(())
}

/// Renames the file or directory at `path` to `new_name`, keeping it in the same
/// parent directory. Returns the new path.
///
/// `new_name` must be a plain name without separators or `.`/`..`, otherwise
/// [`PfadError::InvalidName`] is returned. Never overwrites: if the target
/// exists, [`PfadError::AlreadyExists`] is returned.
pub fn rename(path: impl AsRef<Path>, new_name: impl AsRef<OsStr>) -> Result<PathBuf> {
    let path = path.as_ref();
    let new_name = new_name.as_ref();

    if Path::new(new_name).file_name() != Some(new_name) {
        return Err(PfadError::InvalidName(new_name.to_string_lossy().into_owned()));
    }

    let dest = path.with_file_name(new_name);
    if dest.exists() {
        return Err(PfadError::AlreadyExists(dest));
    }
    fs::rename(path, &dest)?;
    Ok(dest)
}

/// Lists the entries of the directory at `path`, directories first, then
/// sorted by name ignoring case.
///
/// Names are converted lossily to UTF-8. If any single entry cannot be read the
/// whole call fails.
pub fn list_dir(path: impl AsRef<Path>) -> Result<Vec<Entry>> {
    let mut entries = fs::read_dir(path)?.map(|res| -> Result<Entry> {
        let de = res?;
        let meta = de.metadata()?;
        Ok(Entry {
            name: de.file_name().to_string_lossy().into_owned(),
            is_dir: meta.is_dir(),
            size: meta.len(),
        })
    }).collect::<Result<Vec<_>>>()?;

    entries.sort_by(|a, b| {
        b.is_dir.cmp(&a.is_dir).then_with(|| a.name.to_lowercase().cmp(&b.name.to_lowercase()))
    });

    Ok(entries)
}
