use clipboard_rs::{Clipboard, ClipboardContext};
use crate::error::{PfadError, Result};
use crate::transfer;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{LazyLock, Mutex};

// black magic, thread safe though
static CLIPBOARD: LazyLock<Option<Mutex<ClipboardContext>>> =
    LazyLock::new(|| ClipboardContext::new().ok().map(Mutex::new));
// BIG IMPORTANT ! ! ! This is were copied paths go whe the system clipboard can't take them/does not exists.
static FALLBACK: Mutex<Option<Vec<PathBuf>>> = Mutex::new(None);
// Remember if the clipboard can't be used
static CLIPBOARD_DISABLED: AtomicBool = AtomicBool::new(false);

pub fn copy_files(paths: Vec<impl AsRef<Path>>) -> Result<()> {
    let mut absolute_paths: Vec<PathBuf> = Vec::new();
    for path in &paths {
        absolute_paths.push(std::path::absolute(path)?);
    }

    let Ok(mut fallback) = FALLBACK.lock() else {
        return Err(PfadError::Clipboard(String::from("fallback clipboard lock poisoned")))
    };

    if copy_to_system(&absolute_paths) {
        *fallback = None;
    } else {
        *fallback = Some(absolute_paths);
    }

    Ok(())
}
/// Internal function for copy_files()
fn copy_to_system(paths: &[PathBuf]) -> bool {
    if CLIPBOARD_DISABLED.load(Ordering::Relaxed) {
        return false;
    }

    let Some(clipboard) = CLIPBOARD.as_ref() else {
        return false;
    };
    let Ok(clipboard) = clipboard.lock() else {
        CLIPBOARD_DISABLED.store(true, Ordering::Relaxed);
        return false;
    };

    let mut path_strings: Vec<String> = Vec::new();
    for path in paths {
        let Some(text) = path.to_str() else {
            return false;
        };
        path_strings.push(text.to_string());
    }

    let ok = if path_strings.is_empty() {
        clipboard.clear().is_ok()
    } else {
        clipboard.set_files(path_strings).is_ok()
    };

    if !ok {
        CLIPBOARD_DISABLED.store(true, Ordering::Relaxed);
    }

    ok
}

pub fn paste_files(dest_dir: impl AsRef<Path>) -> Result<Vec<PathBuf>> {
    let dest_dir = std::path::absolute(dest_dir)?;
    let mut created = Vec::new();

    for src in selected_paths()? {
        if let Some(dest) = transfer::copy_into(&src, &dest_dir)? {
            created.push(dest);
        }
    }

    Ok(created)
}

fn selected_paths() -> Result<Vec<PathBuf>> {
    let Ok(fallback) = FALLBACK.lock() else {
        return Err(PfadError::Clipboard(String::from("fallback clipboard lock poisoned")));
    };

    if let Some(paths) = fallback.as_ref() {
        return Ok(paths.clone())
    }

    let Some(clipboard) = CLIPBOARD.as_ref() else {
        return Ok(Vec::new());
    };
    let Ok(clipboard) = clipboard.lock() else {
        return Ok(Vec::new());
    };

    let files = clipboard.get_files().unwrap_or_default();

    let mut paths = Vec::new();
    for file in files {
        // Linux returns file://home/.. while windows returns a plain path
        let text = file.strip_prefix("file://").unwrap_or(file.as_str());
        paths.push(PathBuf::from(text));
    }

    Ok(paths)
}
