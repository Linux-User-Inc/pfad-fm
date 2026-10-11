use crate::error::{PfadError, Result};
use std::fs;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};

const FILE_NAME: &str = "selection";

/// PFAD_STATE_DIR overrides the default location, used by tests/CI.
pub(crate) fn default_dir() -> Result<PathBuf> {
    if let Some(dir) = std::env::var_os("PFAD_STATE_DIR") {
        return Ok(PathBuf::from(dir));
    }
    dirs::data_local_dir()
        .map(|d| d.join("pfad"))
        .ok_or_else(|| PfadError::Selection("no local data directory".into()))
}

/// Replaces the saved selection, an empty slice clears it.
pub(crate) fn save(dir: &Path, paths: &[PathBuf]) -> Result<()> {
    let mut data = String::new();
    for p in paths {
        let s = p.to_str().ok_or_else(|| PfadError::NonUtf8Path(p.clone()))?;
        data.push_str(s);
        data.push('\0');
    }

    fs::create_dir_all(dir)?;

    let tmp = dir.join(format!("{FILE_NAME}.{}.tmp", std::process::id()));
    fs::write(&tmp, data)?;
    fs::rename(&tmp, dir.join(FILE_NAME))?; // atomic replace
    Ok(())
}

/// Missing file = empty selection, emptiness is turned into an error by the caller.
pub(crate) fn load(dir: &Path) -> Result<Vec<PathBuf>> {
    let data = match fs::read_to_string(dir.join(FILE_NAME)) {
        Ok(d) => d,
        Err(e) if e.kind() == ErrorKind::NotFound => return Ok(Vec::new()),
        Err(e) => return Err(e.into()),
    };
    Ok(data.split('\0').filter(|s| !s.is_empty()).map(PathBuf::from).collect())
}
