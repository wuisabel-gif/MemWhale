use super::*;

struct Temp(PathBuf);
impl Temp {
    fn new() -> Self {
        Self(staging(&std::env::temp_dir().join("memorywhale-test")).unwrap())
    }
    fn dir(&self) -> PathBuf {
        self.0.join("example")
    }
    fn stages(&self) -> Vec<PathBuf> {
        fs::read_dir(&self.0)
            .unwrap()
            .map(|e| e.unwrap().path())
            .filter(|p| {
                p.file_name()
                    .unwrap()
                    .to_string_lossy()
                    .starts_with(".memorywhale-skill-stage-")
            })
            .collect()
    }
}
impl Drop for Temp {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}
const TEXT: &str = "a skill\n";

#[test]
fn second_file_write_or_sync_failure_cleans_stage_and_allows_retry() {
    for fail_sync in [false, true] {
        let temp = Temp::new();
        let result = install_with(&temp.dir(), TEXT, |index, sync| {
            if index == 1 && sync == fail_sync {
                Err(io::Error::other("injected"))
            } else {
                Ok(())
            }
        });
        assert!(result.is_err());
        assert!(!temp.dir().exists());
        assert!(temp.stages().is_empty());
        install(&temp.dir(), TEXT).unwrap();
        assert_eq!(fs::read_to_string(temp.dir().join(SKILL)).unwrap(), TEXT);
    }
}

#[test]
fn publication_preserves_existing_empty_directory() {
    use std::os::unix::fs::MetadataExt;
    let temp = Temp::new();
    fs::create_dir(temp.dir()).unwrap();
    let inode = fs::metadata(temp.dir()).unwrap().ino();
    assert!(install(&temp.dir(), TEXT).is_err());
    assert_eq!(fs::metadata(temp.dir()).unwrap().ino(), inode);
    assert_eq!(fs::read_dir(temp.dir()).unwrap().count(), 0);
    assert!(temp.stages().is_empty());
}

#[test]
fn modified_target_is_staged_then_restored() {
    let temp = Temp::new();
    install(&temp.dir(), TEXT).unwrap();
    fs::write(temp.dir().join(SKILL), "user change").unwrap();
    assert!(remove(&temp.dir(), TEXT).is_err());
    assert_eq!(
        fs::read_to_string(temp.dir().join(SKILL)).unwrap(),
        "user change"
    );
    assert_eq!(
        fs::read_to_string(temp.dir().join(MARKER)).unwrap(),
        serde_json::to_string(TEXT).unwrap()
    );
    assert!(temp.stages().is_empty());
}

#[test]
fn destination_replacement_survives_failed_validation_and_restore() {
    let temp = Temp::new();
    install(&temp.dir(), TEXT).unwrap();
    fs::write(temp.dir().join(SKILL), "original user change").unwrap();
    let error = remove_with(&temp.dir(), TEXT, |stage| {
        assert!(!temp.dir().join(SKILL).exists());
        assert_eq!(
            fs::read_to_string(stage.join(SKILL)).unwrap(),
            "original user change"
        );
        fs::write(temp.dir().join(SKILL), "replacement").unwrap();
    })
    .unwrap_err();
    assert_eq!(
        fs::read_to_string(temp.dir().join(SKILL)).unwrap(),
        "replacement"
    );
    let stages = temp.stages();
    assert_eq!(stages.len(), 1);
    assert_eq!(
        fs::read_to_string(stages[0].join(SKILL)).unwrap(),
        "original user change"
    );
    assert!(error.contains(stages[0].to_str().unwrap()));
    assert!(!error.contains("original user change"));
    assert!(temp.dir().join(MARKER).exists());
}

#[test]
fn valid_removal_does_not_delete_post_staging_replacement() {
    let temp = Temp::new();
    install(&temp.dir(), TEXT).unwrap();
    let error = remove_with(&temp.dir(), TEXT, |_| {
        fs::write(temp.dir().join(SKILL), "replacement").unwrap();
    })
    .unwrap_err();
    assert!(error.contains("replacement remains untouched"));
    assert_eq!(
        fs::read_to_string(temp.dir().join(SKILL)).unwrap(),
        "replacement"
    );
    assert!(!temp.dir().join(MARKER).exists());
    assert!(temp.stages().is_empty());
}

#[test]
fn removal_preserves_unrelated_files_and_removes_only_empty_directory() {
    let temp = Temp::new();
    install(&temp.dir(), TEXT).unwrap();
    fs::write(temp.dir().join("notes"), "user notes").unwrap();
    remove(&temp.dir(), TEXT).unwrap();
    assert_eq!(
        fs::read_to_string(temp.dir().join("notes")).unwrap(),
        "user notes"
    );
    assert!(!temp.dir().join(SKILL).exists());
    assert!(!temp.dir().join(MARKER).exists());
    let other = temp.0.join("other");
    install(&other, TEXT).unwrap();
    remove(&other, TEXT).unwrap();
    assert!(!other.exists());
}

#[test]
fn failed_second_move_restores_first_entry() {
    let temp = Temp::new();
    install(&temp.dir(), TEXT).unwrap();
    fs::remove_file(temp.dir().join(MARKER)).unwrap();
    assert!(remove(&temp.dir(), TEXT).is_err());
    assert_eq!(fs::read_to_string(temp.dir().join(SKILL)).unwrap(), TEXT);
    assert!(temp.stages().is_empty());
}

#[test]
fn staged_validation_rejects_symlinks_directories_oversize_and_non_utf8() {
    for variant in 0..5 {
        let temp = Temp::new();
        install(&temp.dir(), TEXT).unwrap();
        let target = temp.dir().join(SKILL);
        fs::remove_file(&target).unwrap();
        match variant {
            0 => {
                let outside = temp.0.join("outside");
                fs::write(&outside, TEXT).unwrap();
                std::os::unix::fs::symlink(outside, &target).unwrap();
            }
            1 => fs::create_dir(&target).unwrap(),
            2 => fs::write(&target, vec![b'a'; LIMIT as usize + 1]).unwrap(),
            3 => fs::write(&target, [0xff]).unwrap(),
            _ => {
                fs::write(&target, TEXT).unwrap();
                fs::write(
                    temp.dir().join(MARKER),
                    vec![b' '; SNAPSHOT_LIMIT as usize + 1],
                )
                .unwrap();
            }
        }
        assert!(remove(&temp.dir(), TEXT).is_err());
        assert!(fs::symlink_metadata(&target).is_ok());
        assert!(temp.dir().join(MARKER).exists());
        assert!(temp.stages().is_empty());
    }
}

#[test]
fn staged_permissions_are_private() {
    use std::os::unix::fs::PermissionsExt;
    let temp = Temp::new();
    install(&temp.dir(), TEXT).unwrap();
    assert_eq!(
        fs::metadata(temp.dir()).unwrap().permissions().mode() & 0o777,
        0o700
    );
    for name in [SKILL, MARKER] {
        assert_eq!(
            fs::metadata(temp.dir().join(name))
                .unwrap()
                .permissions()
                .mode()
                & 0o777,
            0o600
        );
    }
}
