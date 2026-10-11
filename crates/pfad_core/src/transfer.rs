use crate::error::{PfadError, Result};
use std::fs;
use std::io;
use std::path::{Path, PathBuf};


pub(crate) fn copy_into(src: &Path, dest_dir: &Path) -> Result<Option<PathBuf>> {
    let Some(name) = src.file_name() else { return Ok(None) };
    let dest = dest_dir.join(name);

    if dest.exists() {
        return Err(PfadError::AlreadyExists(dest));
    }

    if src.is_dir() {
        if dest_dir.starts_with(src) {
            return Err(PfadError::PasteIntoSelf(src.to_path_buf()));
        }
        copy_dir_all(src, &dest)?;
    } else {
        fs::copy(src, &dest)?;
    }

    Ok(Some(dest))
}

pub(crate) fn copy_dir_all(src: &Path, dest: &Path) -> io::Result<()> {
    fs::create_dir(dest)?;

    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let target = dest.join(entry.file_name());

        if entry.file_type()?.is_dir() {
            copy_dir_all(&entry.path(), &target)?;
        } else {
            fs::copy(entry.path(), &target)?;
        }
    }

    Ok(())
}
