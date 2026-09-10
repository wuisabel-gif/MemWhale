use super::*;

struct TestRoot(PathBuf);
impl TestRoot {
    fn new() -> Self {
        let mut random = [0u8; 16];
        getrandom::getrandom(&mut random).unwrap();
        let path = std::env::temp_dir().join(format!(
            "mw-skill-guard-{:032x}",
            u128::from_ne_bytes(random)
        ));
        fs::create_dir(&path).unwrap();
        Self(fs::canonicalize(path).unwrap())
    }
}
impl Drop for TestRoot {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

#[test]
fn cooperating_operations_cannot_remove_or_reinstall_under_an_active_lock() {
    let root = TestRoot::new();
    let path = root.0.join(".example.memorywhale-skill.lock");
    let guard = SkillLock::acquire(&path).unwrap();
    assert!(SkillLock::acquire(&path).is_err());
    let skill_dir = root.0.join("example");
    fs::create_dir(&skill_dir).unwrap();
    fs::remove_dir(&skill_dir).unwrap();
    assert!(
        path.exists(),
        "lock must live outside the removable directory"
    );
    assert!(SkillLock::acquire(&path).is_err());
    drop(guard);
    assert!(!path.exists());
    assert!(SkillLock::acquire(&path).is_ok());
}

#[test]
fn removal_rechecks_user_edits_after_initial_validation() {
    let root = TestRoot::new();
    let target = root.0.join("SKILL.md");
    let marker = root.0.join(MARKER);
    let original = "reviewed instructions";
    fs::write(&target, original).unwrap();
    fs::write(&marker, serde_json::to_string(original).unwrap()).unwrap();
    validate_owned(&target, &marker, original).unwrap();
    // Simulate an editor save between CLI preflight and the removal boundary.
    fs::write(&target, "new user instructions").unwrap();
    assert!(remove_owned(&target, &marker, original).is_err());
    assert_eq!(
        fs::read_to_string(&target).unwrap(),
        "new user instructions"
    );
    assert!(marker.exists());
}

#[test]
fn removal_rechecks_changed_ownership_and_leaves_both_files() {
    let root = TestRoot::new();
    let target = root.0.join("SKILL.md");
    let marker = root.0.join(MARKER);
    let original = "reviewed instructions";
    fs::write(&target, original).unwrap();
    fs::write(&marker, serde_json::to_string(original).unwrap()).unwrap();
    validate_owned(&target, &marker, original).unwrap();
    fs::write(
        &marker,
        serde_json::to_string("different owner snapshot").unwrap(),
    )
    .unwrap();
    assert!(remove_owned(&target, &marker, original).is_err());
    assert_eq!(fs::read_to_string(&target).unwrap(), original);
    assert!(marker.exists());
}
