//! The file store, on a directory of the test's own.

use std::path::PathBuf;

use super::files::{app_name, delete, file_name, read, write};
use super::{MemoryStorage, StorageError, absolute, fake_storage, keep, kept, retire};

/// A fresh directory under the system temp dir, removed when dropped.
struct TempDir(PathBuf);

impl TempDir {
    fn new(test: &str) -> Self {
        let dir =
            std::env::temp_dir().join(format!("libero-storage-{}-{test}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        Self(dir)
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

#[test]
fn a_written_value_reads_back_and_overwrites() {
    let dir = TempDir::new("round-trip");
    assert_eq!(read(&dir.0, "theme"), Ok(None));
    write(&dir.0, "theme", "\"dark\"").unwrap();
    assert_eq!(read(&dir.0, "theme"), Ok(Some("\"dark\"".into())));
    write(&dir.0, "theme", "\"light\"").unwrap();
    assert_eq!(read(&dir.0, "theme"), Ok(Some("\"light\"".into())));
    // Only the value's file: no temporary file is left behind.
    assert_eq!(std::fs::read_dir(&dir.0).unwrap().count(), 1);
}

#[test]
fn delete_drops_the_value_and_ignores_a_missing_one() {
    let dir = TempDir::new("delete");
    write(&dir.0, "k", "1").unwrap();
    delete(&dir.0, "k").unwrap();
    assert_eq!(read(&dir.0, "k"), Ok(None));
    assert_eq!(delete(&dir.0, "k"), Ok(()));
}

#[test]
fn text_that_is_not_utf8_is_invalid() {
    let dir = TempDir::new("utf8");
    std::fs::create_dir_all(&dir.0).unwrap();
    std::fs::write(dir.0.join(file_name("k")), [0xff, 0xfe]).unwrap();
    assert_eq!(read(&dir.0, "k"), Err(StorageError::Invalid));
}

#[test]
fn keys_become_names_that_never_meet_or_leave_the_dir() {
    assert_eq!(file_name("player-volume_2"), "player-volume_2.json");
    assert_eq!(file_name("a/b"), "a%2Fb.json");
    assert_eq!(file_name(".."), "%2E%2E.json");
    assert_eq!(file_name("A"), "%41.json");
    assert_eq!(file_name("ü"), "%C3%BC.json");
    assert_eq!(file_name("con"), "%63on.json");
    assert_eq!(file_name(""), ".json");
    assert_ne!(file_name("A"), file_name("a"));
    assert_ne!(file_name("%41"), file_name("A"));
}

/// Libero's own settings stay raw text under their key, so values kept before still load.
#[test]
fn a_kept_setting_is_raw_text_under_its_key() {
    let local = MemoryStorage::leaked();
    let _fake = fake_storage(Some(local), None);
    keep("lsx-direction", Some("rtl"));
    assert_eq!(
        local.values.borrow().get("lsx-direction").cloned(),
        Some("rtl".into())
    );
    assert_eq!(kept("lsx-direction"), Some("rtl".into()));
    keep("lsx-direction", None);
    assert_eq!(kept("lsx-direction"), None);
}

#[test]
fn without_a_store_nothing_is_kept() {
    let local = MemoryStorage::leaked();
    local.refuse.set(Some(StorageError::Unavailable));
    let fake = fake_storage(Some(local), None);
    keep("lsx-color-scheme", Some("dark"));
    assert_eq!(kept("lsx-color-scheme"), None);
    drop(fake);
    let _none = fake_storage(None, None);
    keep("lsx-color-scheme", Some("dark"));
    assert_eq!(kept("lsx-color-scheme"), None);
}

#[test]
fn keys_that_differ_only_in_case_keep_their_own_files() {
    let dir = TempDir::new("case");
    for key in ["Key", "key", "a/b", ".."] {
        write(&dir.0, key, &format!("\"{key}\"")).unwrap();
    }
    for key in ["Key", "key", "a/b", ".."] {
        assert_eq!(read(&dir.0, key), Ok(Some(format!("\"{key}\""))));
    }
}

/// `localStorage` takes any key; a file name ends at 255 bytes.
#[test]
fn long_and_non_ascii_keys_round_trip() {
    let dir = TempDir::new("long");
    let shared = "x".repeat(300);
    let keys = [
        "猫".repeat(100),
        format!("{shared}-a"),
        format!("{shared}-b"),
        "Key/".repeat(80),
    ];
    for key in &keys {
        assert!(file_name(key).len() <= 255, "{key}");
        write(&dir.0, key, &format!("\"{key}\"")).unwrap();
    }
    for key in &keys {
        assert_eq!(read(&dir.0, key), Ok(Some(format!("\"{key}\""))));
    }
    assert_eq!(std::fs::read_dir(&dir.0).unwrap().count(), keys.len());
}

/// Every name that fit before keeps its file, so values stored then still load.
#[test]
fn names_that_fit_stay_as_they_were() {
    let plain = "k".repeat(250);
    assert_eq!(file_name(&plain), format!("{plain}.json"));
    let encoded = "ü".repeat(41);
    assert_eq!(file_name(&encoded), format!("{}.json", "%C3%BC".repeat(41)));
    assert!(file_name(&"k".repeat(251)).contains('~'));
}

#[test]
fn a_stale_temporary_file_is_swept_and_a_fresh_one_kept() {
    let dir = TempDir::new("sweep");
    std::fs::create_dir_all(&dir.0).unwrap();
    let stale = dir.0.join("theme.json.1-0.tmp");
    let fresh = dir.0.join("~1-1.tmp");
    let file = std::fs::File::create(&stale).unwrap();
    let old = std::time::SystemTime::now() - std::time::Duration::from_secs(2 * 24 * 60 * 60);
    file.set_modified(old).unwrap();
    std::fs::File::create(&fresh).unwrap();
    write(&dir.0, "theme", "1").unwrap();
    assert!(!stale.exists());
    assert!(fresh.exists());
}

#[cfg(unix)]
#[test]
fn the_store_is_private_to_the_user() {
    use std::os::unix::fs::PermissionsExt;
    let dir = TempDir::new("mode");
    write(&dir.0, "token", "1").unwrap();
    let mode =
        |path: &std::path::Path| std::fs::metadata(path).unwrap().permissions().mode() & 0o777;
    assert_eq!(mode(&dir.0), 0o700);
    assert_eq!(mode(&dir.0.join(file_name("token"))), 0o600);
}

#[test]
fn the_legacy_file_goes_only_after_a_good_write() {
    let dir = TempDir::new("legacy");
    std::fs::create_dir_all(&dir.0).unwrap();
    let legacy = dir.0.join("lsx-color-scheme");
    std::fs::write(&legacy, "dark").unwrap();
    retire(
        Some(dir.0.clone()),
        "lsx-color-scheme",
        Err(StorageError::Full),
    );
    assert!(legacy.exists());
    retire(Some(dir.0.clone()), "lsx-color-scheme", Ok(()));
    assert!(!legacy.exists());
}

#[test]
fn a_replaced_binary_keeps_its_app_directory() {
    let name = |path: &str| app_name(std::path::Path::new(path)).unwrap();
    assert_eq!(name("/usr/bin/notes"), "notes");
    assert_eq!(name("/usr/bin/notes (deleted)"), "notes");
}

#[test]
fn a_relative_storage_dir_is_fixed_at_the_call() {
    let dir = absolute("store".into());
    assert_eq!(dir, std::env::current_dir().unwrap().join("store"));
}
