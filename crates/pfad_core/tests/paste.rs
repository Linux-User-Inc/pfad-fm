mod common;

use common::{io_kind, isolated_state};
use pfad_core::*;
use std::fs;
use std::io::ErrorKind;
use std::path::PathBuf;
use tempfile::tempdir;

#[test]
fn paste_single_file() {
    let _g = isolated_state();
    let src = tempdir().unwrap();
    let dest = tempdir().unwrap();
    let file = src.path().join("a.txt");
    fs::write(&file, "one").unwrap();

    copy_files(vec![&file], false).unwrap();
    let created = paste_files(dest.path(), false).unwrap();

    assert_eq!(created, vec![std::path::absolute(dest.path()).unwrap().join("a.txt")]);
    assert_eq!(fs::read_to_string(dest.path().join("a.txt")).unwrap(), "one");
    // The source is copied, not moved.
    assert!(file.exists());
}

#[test]
fn paste_directory_recursively() {
    let _g = isolated_state();
    let src = tempdir().unwrap();
    let dest = tempdir().unwrap();
    let sub = src.path().join("sub");
    fs::create_dir_all(sub.join("nested").join("deep")).unwrap();
    fs::write(sub.join("top.txt"), "top").unwrap();
    fs::write(sub.join("nested").join("mid.txt"), "mid").unwrap();
    fs::write(sub.join("nested").join("deep").join("low.txt"), "low").unwrap();

    copy_files(vec![&sub], false).unwrap();
    paste_files(dest.path(), false).unwrap();

    let out = dest.path().join("sub");
    assert_eq!(fs::read_to_string(out.join("top.txt")).unwrap(), "top");
    assert_eq!(fs::read_to_string(out.join("nested").join("mid.txt")).unwrap(), "mid");
    assert_eq!(
        fs::read_to_string(out.join("nested").join("deep").join("low.txt")).unwrap(),
        "low"
    );
    assert!(sub.join("top.txt").exists());
}

#[test]
fn paste_empty_directory() {
    let _g = isolated_state();
    let src = tempdir().unwrap();
    let dest = tempdir().unwrap();
    let empty = src.path().join("empty");
    fs::create_dir(&empty).unwrap();

    copy_files(vec![&empty], false).unwrap();
    paste_files(dest.path(), false).unwrap();

    assert!(dest.path().join("empty").is_dir());
}

#[test]
fn paste_mixed_files_and_dirs() {
    let _g = isolated_state();
    let src = tempdir().unwrap();
    let dest = tempdir().unwrap();
    let a = src.path().join("a.txt");
    let b = src.path().join("b.txt");
    let sub = src.path().join("sub");
    fs::write(&a, "a").unwrap();
    fs::write(&b, "b").unwrap();
    fs::create_dir(&sub).unwrap();
    fs::write(sub.join("inner.txt"), "inner").unwrap();

    copy_files(vec![a, b, sub], false).unwrap();
    let created = paste_files(dest.path(), false).unwrap();

    assert_eq!(created.len(), 3);
    assert_eq!(fs::read_to_string(dest.path().join("a.txt")).unwrap(), "a");
    assert_eq!(fs::read_to_string(dest.path().join("b.txt")).unwrap(), "b");
    assert_eq!(
        fs::read_to_string(dest.path().join("sub").join("inner.txt")).unwrap(),
        "inner"
    );
}

#[test]
fn paste_existing_file_is_an_error_and_not_overwritten() {
    let _g = isolated_state();
    let src = tempdir().unwrap();
    let dest = tempdir().unwrap();
    let file = src.path().join("a.txt");
    fs::write(&file, "new").unwrap();
    fs::write(dest.path().join("a.txt"), "old").unwrap();

    copy_files(vec![&file], false).unwrap();
    let err = paste_files(dest.path(), false).unwrap_err();

    assert!(matches!(err, PfadError::AlreadyExists(_)));
    assert_eq!(fs::read_to_string(dest.path().join("a.txt")).unwrap(), "old");
}

#[test]
fn paste_stops_at_first_collision_but_keeps_earlier_copies() {
    let _g = isolated_state();
    let src = tempdir().unwrap();
    let dest = tempdir().unwrap();
    let a = src.path().join("a.txt");
    let b = src.path().join("b.txt");
    fs::write(&a, "a").unwrap();
    fs::write(&b, "b").unwrap();
    fs::write(dest.path().join("b.txt"), "old").unwrap();

    copy_files(vec![&a, &b], false).unwrap();
    let err = paste_files(dest.path(), false).unwrap_err();

    assert!(matches!(err, PfadError::AlreadyExists(_)));
    assert!(dest.path().join("a.txt").exists());
    assert_eq!(fs::read_to_string(dest.path().join("b.txt")).unwrap(), "old");
}

#[test]
fn paste_dir_into_its_own_subdir_is_an_error() {
    let _g = isolated_state();
    let src = tempdir().unwrap();
    let sub = src.path().join("sub");
    let inner = sub.join("inner");
    fs::create_dir_all(&inner).unwrap();

    copy_files(vec![&sub], false).unwrap();
    let err = paste_files(&inner, false).unwrap_err();

    assert!(matches!(err, PfadError::PasteIntoSelf(_)));
    assert!(!inner.join("sub").exists());
}

#[test]
fn paste_after_clear_does_not_reuse_old_selection() {
    let _g = isolated_state();
    let src = tempdir().unwrap();
    let dest = tempdir().unwrap();
    let file = src.path().join("a.txt");
    fs::write(&file, "x").unwrap();

    copy_files(vec![&file], false).unwrap();
    copy_files(Vec::<PathBuf>::new(), false).unwrap();

    assert!(matches!(
        paste_files(dest.path(), false),
        Err(PfadError::NothingToPaste)
    ));
    assert_eq!(fs::read_dir(dest.path()).unwrap().count(), 0);
}

#[test]
fn paste_without_any_copy_is_nothing_to_paste() {
    let _g = isolated_state();
    let dest = tempdir().unwrap();

    assert!(matches!(
        paste_files(dest.path(), false),
        Err(PfadError::NothingToPaste)
    ));
}

#[test]
fn paste_keeps_the_selection_for_a_second_paste() {
    let _g = isolated_state();
    let src = tempdir().unwrap();
    let first = tempdir().unwrap();
    let second = tempdir().unwrap();
    let file = src.path().join("a.txt");
    fs::write(&file, "x").unwrap();

    copy_files(vec![&file], false).unwrap();
    paste_files(first.path(), false).unwrap();
    paste_files(second.path(), false).unwrap();

    assert!(first.path().join("a.txt").exists());
    assert!(second.path().join("a.txt").exists());
}

#[test]
fn paste_of_a_deleted_source_is_not_found_and_copies_nothing() {
    let _g = isolated_state();
    let src = tempdir().unwrap();
    let dest = tempdir().unwrap();
    let kept = src.path().join("kept.txt");
    let gone = src.path().join("gone.txt");
    fs::write(&kept, "x").unwrap();
    fs::write(&gone, "x").unwrap();

    copy_files(vec![&kept, &gone], false).unwrap();
    fs::remove_file(&gone).unwrap();
    let err = paste_files(dest.path(), false).unwrap_err();

    assert_eq!(io_kind(err), ErrorKind::NotFound);
    assert_eq!(fs::read_dir(dest.path()).unwrap().count(), 0);
}

#[test]
fn paste_uses_latest_copy() {
    let _g = isolated_state();
    let src = tempdir().unwrap();
    let dest = tempdir().unwrap();
    let a = src.path().join("a.txt");
    let b = src.path().join("b.txt");
    fs::write(&a, "a").unwrap();
    fs::write(&b, "b").unwrap();

    copy_files(vec![&a], false).unwrap();
    copy_files(vec![&b], false).unwrap();
    paste_files(dest.path(), false).unwrap();

    assert!(!dest.path().join("a.txt").exists());
    assert!(dest.path().join("b.txt").exists());
}

#[test]
fn paste_into_missing_destination_is_an_io_error() {
    let _g = isolated_state();
    let src = tempdir().unwrap();
    let dest = tempdir().unwrap();
    let file = src.path().join("a.txt");
    fs::write(&file, "x").unwrap();

    copy_files(vec![&file], false).unwrap();
    let err = paste_files(dest.path().join("nope"), false).unwrap_err();

    assert!(matches!(err, PfadError::Io(_)));
}

#[test]
fn paste_path_with_spaces_and_unicode() {
    let _g = isolated_state();
    let src = tempdir().unwrap();
    let dest = tempdir().unwrap();
    let name = "my file – ünïcödé 文件.txt";
    let file = src.path().join(name);
    fs::write(&file, "x").unwrap();

    copy_files(vec![&file], false).unwrap();
    paste_files(dest.path(), false).unwrap();

    assert!(dest.path().join(name).exists());
}
