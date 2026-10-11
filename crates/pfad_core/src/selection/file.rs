use crate::error::{PfadError, Result};
use std::fs;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};

const FILE_NAME: &str = "selection";

/// The directory the selection file lives in: `$PFAD_STATE_DIR` if set (used by
/// tests/CI), otherwise `pfad/` inside the OS local data directory.
pub(crate) fn default_dir() -> Result<PathBuf> {
    if let Some(dir) = std::env::var_os("PFAD_STATE_DIR") {
        return Ok(PathBuf::from(dir));
    }
    dirs::data_local_dir()
        .map(|d| d.join("pfad"))
        .ok_or_else(|| PfadError::Selection("no local data directory".into()))
}

/// Replaces the saved selection in `dir`; an empty slice clears it.
///
/// The directory is created if missing. The file is written to a temporary name
/// and renamed into place, so readers never see a half-written selection. Paths
/// that are not valid UTF-8 are rejected with [`PfadError::NonUtf8Path`] and the
/// previous selection is kept.
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

/// Reads the saved selection from `dir`.
///
/// A missing file counts as an empty selection; turning that into an error is
/// up to the caller.
pub(crate) fn load(dir: &Path) -> Result<Vec<PathBuf>> {
    let data = match fs::read_to_string(dir.join(FILE_NAME)) {
        Ok(d) => d,
        Err(e) if e.kind() == ErrorKind::NotFound => return Ok(Vec::new()),
        Err(e) => return Err(e.into()),
    };
    Ok(data.split('\0').filter(|s| !s.is_empty()).map(PathBuf::from).collect())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    fn paths(names: &[&str]) -> Vec<PathBuf> {
        names.iter().map(PathBuf::from).collect()
    }

    #[test]
    fn round_trip() {
        let dir = tempdir().unwrap();
        let sel = paths(&["/a/one.txt", "/b/two"]);

        save(dir.path(), &sel).unwrap();

        assert_eq!(load(dir.path()).unwrap(), sel);
    }

    #[test]
    fn save_replaces_previous_selection() {
        let dir = tempdir().unwrap();

        save(dir.path(), &paths(&["/old"])).unwrap();
        save(dir.path(), &paths(&["/new"])).unwrap();

        assert_eq!(load(dir.path()).unwrap(), paths(&["/new"]));
    }

    #[test]
    fn saving_empty_slice_clears_selection() {
        let dir = tempdir().unwrap();

        save(dir.path(), &paths(&["/a"])).unwrap();
        save(dir.path(), &[]).unwrap();

        assert!(load(dir.path()).unwrap().is_empty());
    }

    #[test]
    fn load_without_a_file_is_empty() {
        let dir = tempdir().unwrap();

        assert!(load(dir.path()).unwrap().is_empty());
    }

    #[test]
    fn spaces_unicode_and_newlines_survive() {
        let dir = tempdir().unwrap();
        let sel = paths(&["/a b/my file – ünïcödé 文件.txt", "/line\nbreak"]);

        save(dir.path(), &sel).unwrap();

        assert_eq!(load(dir.path()).unwrap(), sel);
    }

    #[test]
    fn save_creates_missing_directory() {
        let dir = tempdir().unwrap();
        let nested = dir.path().join("not").join("there").join("yet");

        save(&nested, &paths(&["/a"])).unwrap();

        assert_eq!(load(&nested).unwrap(), paths(&["/a"]));
    }

    #[test]
    fn save_leaves_no_temp_file_behind() {
        let dir = tempdir().unwrap();

        save(dir.path(), &paths(&["/a"])).unwrap();

        let names: Vec<_> = fs::read_dir(dir.path())
            .unwrap()
            .map(|e| e.unwrap().file_name())
            .collect();
        assert_eq!(names, vec![std::ffi::OsString::from(FILE_NAME)]);
    }

    #[cfg(unix)]
    fn non_utf8_path() -> PathBuf {
        use std::ffi::OsStr;
        use std::os::unix::ffi::OsStrExt;
        PathBuf::from(OsStr::from_bytes(b"/bad\xff\xfename"))
    }

    #[cfg(windows)]
    fn non_utf8_path() -> PathBuf {
        use std::ffi::OsString;
        use std::os::windows::ffi::OsStringExt;
        PathBuf::from(OsString::from_wide(&[0x62, 0x61, 0x64, 0xD800]))
    }

    #[test]
    fn non_utf8_path_is_rejected_and_old_selection_kept() {
        let dir = tempdir().unwrap();
        save(dir.path(), &paths(&["/keep"])).unwrap();

        let err = save(dir.path(), &[PathBuf::from("/ok"), non_utf8_path()]).unwrap_err();

        assert!(matches!(err, PfadError::NonUtf8Path(_)));
        assert_eq!(load(dir.path()).unwrap(), paths(&["/keep"]));
    }
}
