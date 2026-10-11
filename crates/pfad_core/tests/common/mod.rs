#![allow(dead_code)] // each test binary only uses part of this module

use pfad_core::PfadError;
use std::io::ErrorKind;
use std::sync::{Mutex, MutexGuard};
use tempfile::{TempDir, tempdir};

pub fn io_kind(err: PfadError) -> ErrorKind {
    match err {
        PfadError::Io(e) => e.kind(),
        other => panic!("expected PfadError::Io, got {other:?}"),
    }
}

// The selection store is found through a process-wide env var, so tests that
// copy or paste must not interleave.
static SERIAL: Mutex<()> = Mutex::new(());

/// Holds the lock and points `PFAD_STATE_DIR` at a fresh temp directory, so a
/// test never touches the real selection. Keep it alive for the whole test.
pub struct IsolatedState {
    _dir: TempDir,
    _lock: MutexGuard<'static, ()>,
}

pub fn isolated_state() -> IsolatedState {
    let lock = SERIAL.lock().unwrap_or_else(|e| e.into_inner());
    let dir = tempdir().unwrap();
    // Safe: SERIAL guarantees no other test thread reads or writes the env now.
    unsafe { std::env::set_var("PFAD_STATE_DIR", dir.path()) };
    IsolatedState { _dir: dir, _lock: lock }
}
